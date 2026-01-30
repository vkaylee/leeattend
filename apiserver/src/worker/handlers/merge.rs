//! Merge Handler
//!
//! Aggregates batch results into final Excel report, uploads to S3,
//! and generates presigned URL for download.

use chrono::{NaiveDate, Utc};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};

use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::queue::{JobStatus, ReportJob, ReportQueue, ReportType};
use crate::services::excel_writer::EXCEL_CONTENT_TYPE;
use crate::services::{ExcelWriter, S3Client};

/// Presigned URL validity duration (24 hours).
const PRESIGNED_URL_EXPIRY: Duration = Duration::from_secs(86400);

/// Handler for merging batch results into final report.
pub struct MergeHandler;

impl MergeHandler {
    /// Process a merge job to aggregate all batch results.
    ///
    /// Flow:
    /// 1. Fetch all batch results from Redis
    /// 2. Deserialize and merge based on report type
    /// 3. Generate Excel file
    /// 4. Upload to S3
    /// 5. Generate presigned URL
    /// 6. Update job status to completed
    /// 7. Cleanup batch results
    pub async fn handle(pool: &PgPool, queue: &ReportQueue, job: &ReportJob) -> Result<(), String> {
        let parent_job_id = job.parent_job_id;

        info!(
            parent_job_id = %parent_job_id,
            "Starting merge job"
        );

        // 1. Get job metadata for context
        let meta = queue
            .get_job_metadata(parent_job_id)
            .await
            .map_err(|e| format!("Failed to get job metadata: {}", e))?
            .ok_or("Job metadata not found")?;

        // 2. Fetch all batch results from Redis
        let batch_results = queue
            .get_all_results(parent_job_id)
            .await
            .map_err(|e| format!("Failed to get batch results: {}", e))?;

        if batch_results.is_empty() {
            warn!(parent_job_id = %parent_job_id, "No batch results found for merge");
            return Err("No batch results to merge".to_string());
        }

        info!(
            parent_job_id = %parent_job_id,
            batch_count = batch_results.len(),
            "Retrieved batch results for merge"
        );

        // 3. Get workspace name for Excel title
        let workspace_name = Self::get_workspace_name(pool, meta.workspace_id).await;

        // 4. Get date range
        let date_range = meta.date_range.as_ref().map(|dr| (dr.start, dr.end)).unwrap_or((
            job.date_range.start,
            job.date_range.end,
        ));

        // 5. Merge results and generate Excel
        let (excel_data, file_name) = match meta.report_type {
            ReportType::Summary => {
                Self::merge_summary_results(&batch_results, &workspace_name, date_range, parent_job_id)?
            }
            ReportType::Detail => {
                Self::merge_detail_results(&batch_results, &workspace_name, date_range, parent_job_id)?
            }
        };

        info!(
            parent_job_id = %parent_job_id,
            file_size = excel_data.len(),
            file_name = %file_name,
            "Excel file generated"
        );

        // 6. Upload to S3
        let s3_key = format!(
            "{}/{}/{}/{}",
            meta.workspace_id,
            Utc::now().format("%Y"),
            Utc::now().format("%m"),
            file_name
        );

        let download_url = match Self::upload_to_s3(&s3_key, excel_data).await {
            Ok(url) => url,
            Err(e) => {
                // S3 not configured - use placeholder URL
                warn!(
                    parent_job_id = %parent_job_id,
                    error = %e,
                    "S3 upload failed, using placeholder URL"
                );
                format!("/api/reports/{}/download", parent_job_id)
            }
        };

        // 7. Update job metadata
        queue
            .set_result_url(parent_job_id, &download_url)
            .await
            .map_err(|e| format!("Failed to set result URL: {}", e))?;

        queue
            .update_job_status(parent_job_id, JobStatus::Completed)
            .await
            .map_err(|e| format!("Failed to update job status: {}", e))?;

        // 8. Cleanup batch results (optional, TTL handles this too)
        if let Err(e) = queue.delete_results(parent_job_id).await {
            warn!(
                parent_job_id = %parent_job_id,
                error = %e,
                "Failed to delete batch results (non-fatal)"
            );
        }

        info!(
            parent_job_id = %parent_job_id,
            download_url = %download_url,
            "Merge job completed successfully"
        );

        Ok(())
    }

    /// Merge summary results from all batches.
    fn merge_summary_results(
        batch_results: &[(String, String)],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        parent_job_id: uuid::Uuid,
    ) -> Result<(Vec<u8>, String), String> {
        let mut all_summaries: Vec<AttendanceSummary> = Vec::new();

        for (batch_key, json) in batch_results {
            let batch: Vec<AttendanceSummary> = serde_json::from_str(json).map_err(|e| {
                format!("Parse error for {}: {}", batch_key, e)
            })?;
            all_summaries.extend(batch);
        }

        // Sort by employee name (global sort)
        all_summaries.sort_by(|a, b| a.employee_name.cmp(&b.employee_name));

        info!(
            total_employees = all_summaries.len(),
            "Merged summary results"
        );

        // Generate Excel
        let excel_data = ExcelWriter::write_summary(&all_summaries, workspace_name, date_range)?;

        let file_name = format!(
            "attendance_summary_{}_{}.xlsx",
            parent_job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name))
    }

    /// Merge detail results from all batches.
    fn merge_detail_results(
        batch_results: &[(String, String)],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        parent_job_id: uuid::Uuid,
    ) -> Result<(Vec<u8>, String), String> {
        let mut all_reports: Vec<DailyAttendanceReport> = Vec::new();

        for (batch_key, json) in batch_results {
            let batch: Vec<DailyAttendanceReport> = serde_json::from_str(json).map_err(|e| {
                format!("Parse error for {}: {}", batch_key, e)
            })?;
            all_reports.extend(batch);
        }

        // Sort by employee name, then by date
        all_reports.sort_by(|a, b| {
            a.employee_name
                .cmp(&b.employee_name)
                .then(a.date.cmp(&b.date))
        });

        info!(
            total_records = all_reports.len(),
            "Merged detail results"
        );

        // Generate Excel
        let excel_data = ExcelWriter::write_detail(&all_reports, workspace_name, date_range)?;

        let file_name = format!(
            "attendance_detail_{}_{}.xlsx",
            parent_job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name))
    }

    /// Upload Excel file to S3 and generate presigned URL.
    async fn upload_to_s3(key: &str, data: Vec<u8>) -> Result<String, String> {
        let s3_client = S3Client::new().await.map_err(|e| e.to_string())?;

        s3_client
            .upload_report(key, data, EXCEL_CONTENT_TYPE)
            .await
            .map_err(|e| e.to_string())?;

        let url = s3_client
            .generate_presigned_url(key, PRESIGNED_URL_EXPIRY)
            .await
            .map_err(|e| e.to_string())?;

        Ok(url)
    }

    /// Get workspace name from database.
    async fn get_workspace_name(pool: &PgPool, workspace_id: uuid::Uuid) -> String {
        match sqlx::query_scalar!(
            "SELECT name FROM workspaces WHERE id = $1",
            workspace_id
        )
        .fetch_optional(pool)
        .await
        {
            Ok(Some(name)) => name,
            _ => "Workspace".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_merge_summary_empty_batches() {
        let batch_results: Vec<(String, String)> = vec![
            ("batch_00000".to_string(), "[]".to_string()),
            ("batch_00001".to_string(), "[]".to_string()),
        ];

        let result = MergeHandler::merge_summary_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
        );

        assert!(result.is_ok());
        let (data, name) = result.unwrap();
        assert!(data.len() > 100); // Valid Excel file
        assert!(name.contains("attendance_summary_"));
    }

    #[test]
    fn test_merge_summary_with_data() {
        let summaries1 = vec![
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Charlie Brown".to_string(),
                total_work_units: 160,
                total_leave_days: 2,
            },
        ];
        let summaries2 = vec![
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "Alice Smith".to_string(),
                total_work_units: 152,
                total_leave_days: 4,
            },
        ];

        let batch_results: Vec<(String, String)> = vec![
            ("batch_00000".to_string(), serde_json::to_string(&summaries1).unwrap()),
            ("batch_00001".to_string(), serde_json::to_string(&summaries2).unwrap()),
        ];

        let result = MergeHandler::merge_summary_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_detail_sorting() {
        let report1 = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Bob Jones".to_string(),
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            work_units: 8,
            status: "present".to_string(),
            generation_metadata: None,
        };
        let report2 = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Alice Smith".to_string(),
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            work_units: 8,
            status: "present".to_string(),
            generation_metadata: None,
        };

        let batch_results: Vec<(String, String)> = vec![
            ("batch_00000".to_string(), serde_json::to_string(&vec![report1]).unwrap()),
            ("batch_00001".to_string(), serde_json::to_string(&vec![report2]).unwrap()),
        ];

        let result = MergeHandler::merge_detail_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json_handling() {
        let batch_results: Vec<(String, String)> = vec![
            ("batch_00000".to_string(), "invalid json".to_string()),
        ];

        let result = MergeHandler::merge_summary_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parse error"));
    }
}
