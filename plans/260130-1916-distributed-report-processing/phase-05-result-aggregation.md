# Phase 5: Result Aggregation

## Context Links
- [Plan Overview](./plan.md)
- [Phase 3: Generic Worker](./phase-03-generic-worker.md)
- [Phase 4: Application-Side Join](./phase-04-application-side-join.md)

## Overview
- **Priority**: P1 (Core)
- **Status**: pending
- **Effort**: 4h
- **Description**: Implement merge job that combines partial results from workers into final Excel file and uploads to S3.

## Key Insights

- Merge job triggered when all batch jobs complete
- Results are JSON fragments stored in Redis HASH
- Streaming aggregation prevents memory spike
- Excel generation uses streaming write (not full in-memory)
- S3 presigned URL returned for client download

## Requirements

### Functional
- F1: Collect all batch results from Redis HASH
- F2: Merge and sort results by employee name
- F3: Generate Excel file with proper formatting
- F4: Upload to S3 and generate presigned URL
- F5: Update job status to "completed" with file URL

### Non-Functional
- NF1: Merge job completes in < 60 seconds for 7,000 employees
- NF2: Memory usage < 100MB during merge
- NF3: Excel file size < 50MB for 7,000 employees x 30 days
- NF4: S3 URL valid for 24 hours

## Architecture

### Merge Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            MERGE JOB FLOW                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  1. Coordinator detects all batches complete                                 │
│     └─ Enqueues MergeResults job                                            │
│                                                                               │
│  2. Worker picks up merge job                                                │
│     └─ Routes to MergeHandler                                                │
│                                                                               │
│  3. MergeHandler executes:                                                   │
│     ┌─────────────────────────────────────────────────────────────────┐     │
│     │                                                                   │     │
│     │  a. HGETALL report:results:{parent_job_id}                       │     │
│     │     → Returns [("batch_00000", json), ("batch_00001", json), ...]│     │
│     │                                                                   │     │
│     │  b. Parse JSON fragments                                          │     │
│     │     → Vec<AttendanceSummary> or Vec<DailyAttendanceReport>       │     │
│     │                                                                   │     │
│     │  c. Sort by employee_name (global sort)                          │     │
│     │                                                                   │     │
│     │  d. Write to Excel (streaming)                                    │     │
│     │     → Use calamine/rust_xlsxwriter                               │     │
│     │                                                                   │     │
│     │  e. Upload to S3                                                  │     │
│     │     → PUT object with Content-Type: application/vnd.openxmlformats│     │
│     │                                                                   │     │
│     │  f. Generate presigned URL (24h expiry)                          │     │
│     │                                                                   │     │
│     │  g. Update job metadata with status=completed, result_url        │     │
│     │                                                                   │     │
│     │  h. Cleanup: DEL report:results:{parent_job_id}                  │     │
│     │                                                                   │     │
│     └─────────────────────────────────────────────────────────────────┘     │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### File Naming Convention

```
s3://leeattend-reports/{workspace_id}/{year}/{month}/
    attendance_summary_{job_id}_{timestamp}.xlsx
    attendance_detail_{job_id}_{timestamp}.xlsx
```

## Related Code Files

### Files to Create
- `apiserver/src/worker/handlers/merge.rs` - Merge handler
- `apiserver/src/services/excel_writer.rs` - Excel generation
- `apiserver/src/services/s3_client.rs` - S3 upload wrapper

### Files to Modify
- `apiserver/src/worker/handlers/mod.rs` - Export merge handler
- `apiserver/Cargo.toml` - Add Excel and S3 crates

## Implementation Steps

### Step 1: Add Dependencies

```toml
# Cargo.toml
rust_xlsxwriter = "0.79"  # Excel writing
aws-sdk-s3 = "1.0"        # S3 client
aws-config = "1.0"        # AWS configuration
```

### Step 2: Create S3 Client Wrapper

```rust
// services/s3_client.rs
use aws_sdk_s3::{Client, presigning::PresigningConfig};
use aws_config::BehaviorVersion;
use std::time::Duration;

pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new() -> Result<Self, String> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
        let client = Client::new(&config);
        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| "S3_BUCKET not set")?;

        Ok(Self { client, bucket })
    }

    pub async fn upload_report(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), String> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| format!("S3 upload failed: {}", e))?;

        Ok(())
    }

    pub async fn generate_presigned_url(
        &self,
        key: &str,
        expires_in: Duration,
    ) -> Result<String, String> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()
            .map_err(|e| format!("Presign config error: {}", e))?;

        let presigned = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| format!("Presign failed: {}", e))?;

        Ok(presigned.uri().to_string())
    }
}
```

### Step 3: Create Excel Writer

```rust
// services/excel_writer.rs
use crate::models::{AttendanceSummary, DailyAttendanceReport};
use rust_xlsxwriter::{Workbook, Format, FormatAlign};
use chrono::NaiveDate;

pub struct ExcelWriter;

impl ExcelWriter {
    /// Generate Excel for attendance summary report
    pub fn write_summary(
        summaries: &[AttendanceSummary],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // Header format
        let header_format = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(0x4472C4)
            .set_font_color(0xFFFFFF);

        // Write title
        worksheet.write_string(0, 0, &format!(
            "Attendance Summary Report - {}",
            workspace_name
        )).map_err(|e| e.to_string())?;

        worksheet.write_string(1, 0, &format!(
            "Period: {} to {}",
            date_range.0, date_range.1
        )).map_err(|e| e.to_string())?;

        // Headers (row 3)
        let headers = ["Employee Name", "Total Work Units", "Total Leave Days"];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(3, col as u16, header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        // Data rows
        for (idx, summary) in summaries.iter().enumerate() {
            let row = (idx + 4) as u32;
            worksheet.write_string(row, 0, &summary.employee_name)
                .map_err(|e| e.to_string())?;
            worksheet.write_number(row, 1, summary.total_work_units as f64)
                .map_err(|e| e.to_string())?;
            worksheet.write_number(row, 2, summary.total_leave_days as f64)
                .map_err(|e| e.to_string())?;
        }

        // Set column widths
        worksheet.set_column_width(0, 30).map_err(|e| e.to_string())?;
        worksheet.set_column_width(1, 18).map_err(|e| e.to_string())?;
        worksheet.set_column_width(2, 18).map_err(|e| e.to_string())?;

        // Write to buffer
        let buffer = workbook.save_to_buffer()
            .map_err(|e| format!("Excel write error: {}", e))?;

        Ok(buffer)
    }

    /// Generate Excel for daily attendance detail report
    pub fn write_detail(
        reports: &[DailyAttendanceReport],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_format = Format::new()
            .set_bold()
            .set_align(FormatAlign::Center)
            .set_background_color(0x4472C4)
            .set_font_color(0xFFFFFF);

        // Title
        worksheet.write_string(0, 0, &format!(
            "Attendance Detail Report - {}",
            workspace_name
        )).map_err(|e| e.to_string())?;

        // Headers
        let headers = [
            "Date", "Employee Name", "Scheduled Start", "Scheduled End",
            "Actual In", "Actual Out", "Work Units", "Status"
        ];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(3, col as u16, header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        // Data rows
        for (idx, report) in reports.iter().enumerate() {
            let row = (idx + 4) as u32;

            worksheet.write_string(row, 0, &report.date.to_string())
                .map_err(|e| e.to_string())?;
            worksheet.write_string(row, 1, &report.employee_name)
                .map_err(|e| e.to_string())?;
            worksheet.write_string(row, 2, &format_time(report.scheduled_start))
                .map_err(|e| e.to_string())?;
            worksheet.write_string(row, 3, &format_time(report.scheduled_end))
                .map_err(|e| e.to_string())?;
            worksheet.write_string(row, 4, &format_datetime(report.actual_punch_in))
                .map_err(|e| e.to_string())?;
            worksheet.write_string(row, 5, &format_datetime(report.actual_punch_out))
                .map_err(|e| e.to_string())?;
            worksheet.write_number(row, 6, report.work_units as f64)
                .map_err(|e| e.to_string())?;
            worksheet.write_string(row, 7, &report.status)
                .map_err(|e| e.to_string())?;
        }

        // Column widths
        worksheet.set_column_width(0, 12).map_err(|e| e.to_string())?;
        worksheet.set_column_width(1, 25).map_err(|e| e.to_string())?;
        for col in 2..8 {
            worksheet.set_column_width(col, 15).map_err(|e| e.to_string())?;
        }

        let buffer = workbook.save_to_buffer()
            .map_err(|e| format!("Excel write error: {}", e))?;

        Ok(buffer)
    }
}

fn format_time(time: Option<chrono::NaiveTime>) -> String {
    time.map(|t| t.format("%H:%M").to_string()).unwrap_or_default()
}

fn format_datetime(dt: Option<chrono::DateTime<chrono::Utc>>) -> String {
    dt.map(|t| t.format("%H:%M").to_string()).unwrap_or_default()
}
```

### Step 4: Implement Merge Handler

```rust
// worker/handlers/merge.rs
use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::queue::{ReportQueue, ReportJob, ReportType, JobStatus};
use crate::services::{ExcelWriter, S3Client};
use chrono::Utc;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(86400); // 24 hours

pub struct MergeHandler;

impl MergeHandler {
    pub async fn handle(
        pool: &PgPool,
        queue: &ReportQueue,
        job: &ReportJob,
    ) -> Result<(), String> {
        let parent_job_id = job.parent_job_id;

        tracing::info!(
            parent_job_id = %parent_job_id,
            "Starting merge job"
        );

        // 1. Get job metadata for context
        let meta = queue.get_job_metadata(parent_job_id).await
            .map_err(|e| format!("Failed to get metadata: {}", e))?
            .ok_or("Job metadata not found")?;

        // 2. Fetch all batch results from Redis
        let results = queue.get_all_results(parent_job_id).await
            .map_err(|e| format!("Failed to get results: {}", e))?;

        if results.is_empty() {
            return Err("No batch results found".to_string());
        }

        tracing::info!(
            parent_job_id = %parent_job_id,
            batch_count = results.len(),
            "Collected batch results"
        );

        // 3. Parse and merge based on report type
        let (excel_data, file_name) = match meta.report_type {
            ReportType::Summary => {
                Self::merge_summary_results(&results, &meta)?
            }
            ReportType::Detail => {
                Self::merge_detail_results(&results, &meta)?
            }
        };

        // 4. Upload to S3
        let s3_client = S3Client::new().await?;
        let s3_key = format!(
            "{}/{}/{}/{}",
            meta.workspace_id,
            Utc::now().format("%Y"),
            Utc::now().format("%m"),
            file_name
        );

        s3_client.upload_report(
            &s3_key,
            excel_data,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ).await?;

        tracing::info!(
            parent_job_id = %parent_job_id,
            s3_key = %s3_key,
            "Uploaded report to S3"
        );

        // 5. Generate presigned URL
        let download_url = s3_client
            .generate_presigned_url(&s3_key, PRESIGNED_URL_EXPIRY)
            .await?;

        // 6. Update job metadata with completion
        queue.set_result_url(parent_job_id, &download_url).await
            .map_err(|e| format!("Failed to set result URL: {}", e))?;

        queue.update_job_status(parent_job_id, JobStatus::Completed).await
            .map_err(|e| format!("Failed to update status: {}", e))?;

        // 7. Cleanup batch results (optional, TTL handles this)
        let _ = queue.delete_results(parent_job_id).await;

        tracing::info!(
            parent_job_id = %parent_job_id,
            "Merge job completed successfully"
        );

        Ok(())
    }

    fn merge_summary_results(
        results: &[(String, String)],
        meta: &JobMetadata,
    ) -> Result<(Vec<u8>, String), String> {
        // Parse all JSON fragments
        let mut all_summaries: Vec<AttendanceSummary> = Vec::new();

        for (batch_key, json) in results {
            let batch: Vec<AttendanceSummary> = serde_json::from_str(json)
                .map_err(|e| format!("Parse error for {}: {}", batch_key, e))?;
            all_summaries.extend(batch);
        }

        // Sort by employee name (global sort)
        all_summaries.sort_by(|a, b| a.employee_name.cmp(&b.employee_name));

        tracing::info!(
            total_employees = all_summaries.len(),
            "Merged summary results"
        );

        // Generate Excel
        let workspace_name = "Workspace"; // Could fetch from cache
        let date_range = (meta.date_range.start, meta.date_range.end);

        let excel_data = ExcelWriter::write_summary(
            &all_summaries,
            workspace_name,
            date_range,
        )?;

        let file_name = format!(
            "attendance_summary_{}_{}.xlsx",
            meta.job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name))
    }

    fn merge_detail_results(
        results: &[(String, String)],
        meta: &JobMetadata,
    ) -> Result<(Vec<u8>, String), String> {
        let mut all_reports: Vec<DailyAttendanceReport> = Vec::new();

        for (batch_key, json) in results {
            let batch: Vec<DailyAttendanceReport> = serde_json::from_str(json)
                .map_err(|e| format!("Parse error for {}: {}", batch_key, e))?;
            all_reports.extend(batch);
        }

        // Sort by employee name, then by date
        all_reports.sort_by(|a, b| {
            a.employee_name.cmp(&b.employee_name)
                .then(a.date.cmp(&b.date))
        });

        tracing::info!(
            total_records = all_reports.len(),
            "Merged detail results"
        );

        let workspace_name = "Workspace";
        let date_range = (meta.date_range.start, meta.date_range.end);

        let excel_data = ExcelWriter::write_detail(
            &all_reports,
            workspace_name,
            date_range,
        )?;

        let file_name = format!(
            "attendance_detail_{}_{}.xlsx",
            meta.job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name))
    }
}
```

### Step 5: Add Queue Operations for Merge

```rust
// queue/report_queue.rs (additions)
impl ReportQueue {
    /// Delete batch results after merge
    pub async fn delete_results(&self, parent_job_id: Uuid) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", RESULTS_PREFIX, parent_job_id);
        conn.del(&key).await
    }

    /// Store final result URL
    pub async fn set_result_url(&self, job_id: Uuid, url: &str) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("report:file:{}", job_id);
        conn.set_ex(&key, url, JOB_TTL_SECS as u64).await
    }

    /// Get result URL for completed job
    pub async fn get_result_url(&self, job_id: Uuid) -> Result<Option<String>, redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("report:file:{}", job_id);
        conn.get(&key).await
    }

    /// Update job status
    pub async fn update_job_status(&self, job_id: Uuid, status: JobStatus) -> Result<(), redis::RedisError> {
        // Get current metadata, update status, write back
        // (Could use Lua script for atomicity)
        let mut meta = self.get_job_metadata(job_id).await?
            .ok_or_else(|| redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Job not found"
            )))?;

        meta.status = status;
        if matches!(status, JobStatus::Completed | JobStatus::Failed) {
            meta.completed_at = Some(Utc::now());
        }

        self.update_job_metadata(&meta).await
    }

    pub async fn update_job_metadata(&self, meta: &JobMetadata) -> Result<(), redis::RedisError> {
        let mut conn = self.pool.conn();
        let key = format!("{}{}", META_PREFIX, meta.job_id);
        let json = serde_json::to_string(meta).unwrap();
        conn.set_ex(&key, json, JOB_TTL_SECS as u64).await
    }
}
```

## Todo List

- [ ] Add rust_xlsxwriter and aws-sdk-s3 dependencies
- [ ] Create `S3Client` wrapper with upload and presign
- [ ] Create `ExcelWriter` for summary and detail formats
- [ ] Implement `MergeHandler` for result aggregation
- [ ] Add queue operations for result URL and cleanup
- [ ] Configure S3 bucket and credentials
- [ ] Write unit tests for Excel generation
- [ ] Write integration tests with mock S3

## Success Criteria

- [ ] All batch results merged correctly
- [ ] Excel file properly formatted and readable
- [ ] S3 upload completes successfully
- [ ] Presigned URL valid and downloadable
- [ ] Job status updated to "completed"
- [ ] Batch results cleaned up after merge

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| S3 credentials misconfigured | Clear error message, environment validation |
| Large file memory pressure | Streaming write with rust_xlsxwriter |
| Merge job timeout | 300s timeout, consider chunked merge for very large |
| Incomplete batch results | Validate batch count before merge |

## Security Considerations

- S3 bucket should be private, access via presigned URLs only
- Presigned URLs expire after 24 hours
- Validate requester has access before generating download URL
- Log all S3 uploads for audit trail

## Next Steps

After this phase:
1. Phase 6: API Integration - Status endpoints and download
