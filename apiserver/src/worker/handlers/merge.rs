//! Merge Handler
//!
//! Aggregates batch results into final Excel report and streams directly to client via Pub/Sub.
//! Receives a [`Job`] with [`MergePayload`] containing date_range and config.

use chrono::{NaiveDate, Utc};
use tracing::{info, warn};

use crate::errors::{AppError, AppResult};
use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::queue::{Job, JobStatus, ReportType};
use crate::services::import_messages::ExcelReportMessages;
use crate::services::ExcelWriter;
use crate::repositories::{QueueRepository, PublisherRepository};
use std::sync::Arc;

/// Presigned URL validity duration - no longer used, kept for backward compatibility.
const _PRESIGNED_URL_EXPIRY: std::time::Duration = std::time::Duration::from_secs(86400);

/// Handler for merging batch results into final report.
/// Processes [`Job`] with [`JobPayload::MergeResults`].
pub struct MergeHandler;

impl MergeHandler {
    /// Process a merge job to aggregate all batch results.
    ///
    /// Flow:
    /// 1. Fetch all batch results from Redis
    /// 2. Deserialize and merge based on report type
    /// 3. Generate Excel file
    /// 4. Store Excel data in Redis for client download
    /// 5. Update job status to completed
    /// 6. Publish DONE signal to Pub/Sub
    /// 7. Cleanup batch results
    pub async fn handle(
        queue: &Arc<dyn QueueRepository>,
        publisher: &Arc<dyn PublisherRepository>,
        job: &Job,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
        match Self::handle_inner(queue, publisher, job, deps).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn handle_inner(
        queue: &Arc<dyn QueueRepository>,
        publisher: &Arc<dyn PublisherRepository>,
        job: &Job,
        deps: &crate::worker::WorkerDeps,
    ) -> AppResult<()> {
        let parent_job_id = job.parent_job_id;
        let t0 = std::time::Instant::now();

        info!(
            parent_job_id = %parent_job_id,
            "Starting merge job"
        );

        // 1. Get job metadata for context
        let meta = queue
            .get_job_metadata(parent_job_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get job metadata: {e}")))?
            .ok_or(AppError::NotFound("Job metadata not found".to_string()))?;

        // 2. Fetch all batch results from Redis
        let batch_results = queue
            .get_all_results(parent_job_id)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get batch results: {e}")))?;

        if batch_results.is_empty() {
            warn!(parent_job_id = %parent_job_id, "No batch results found for merge");
            return Err(AppError::Internal("No batch results to merge".to_string()));
        }

        info!(
            parent_job_id = %parent_job_id,
            batch_count = batch_results.len(),
            "Retrieved batch results for merge"
        );

        let t_fetch = t0.elapsed();

        // 3. Get workspace name, timezone, and language for Excel title/i18n
        let (workspace_name, workspace_tz, workspace_lang) =
            Self::get_workspace_info(deps.workspace_repo.as_ref(), meta.workspace_id).await;
        let excel_msgs = ExcelReportMessages::for_lang(&workspace_lang);

        // 4. Get date range
        let date_range = meta
            .date_range
            .as_ref()
            .map(|dr| (dr.start, dr.end))
            .unwrap_or_else(|| {
                // Fallback: try to get from merge payload
                job.merge_payload()
                    .map(|p| (p.date_range.start, p.date_range.end))
                    .unwrap_or_else(|| {
                        #[allow(clippy::expect_used)]
                        let d = chrono::NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid date");
                        (d, d)
                    })
            });

        // 5. Merge results and generate Excel
        let (excel_data, file_name, total_records) = match meta.report_type {
            Some(ReportType::Summary) => Self::merge_summary_results(
                &batch_results,
                &workspace_name,
                date_range,
                parent_job_id,
                &excel_msgs,
            )?,
            Some(ReportType::Detail) => {
                let work_unit_minutes = job
                    .merge_payload()
                    .map(|p| p.config.work_unit_minutes)
                    .unwrap_or(30);
                Self::merge_detail_results(
                    &batch_results,
                    &workspace_name,
                    date_range,
                    parent_job_id,
                    &workspace_tz,
                    work_unit_minutes,
                    &excel_msgs,
                )?
            }
            None => {
                return Err(AppError::Internal(
                    "No ReportType specified in metadata".to_string(),
                ));
            }
        };

        info!(
            parent_job_id = %parent_job_id,
            file_size = excel_data.len(),
            file_name = %file_name,
            "Excel file generated"
        );

        let t_merge_excel = t0.elapsed();

        // 6. Store Excel data in Redis for direct client download (TTL: 1 hour)
        queue
            .set_result_data(parent_job_id, &excel_data, &file_name)
            .await
            .map_err(|e| format!("Failed to store Excel data: {e}"))?;

        info!(
            parent_job_id = %parent_job_id,
            file_size = excel_data.len(),
            "Report data stored in Redis for client download"
        );

        let t_store = t0.elapsed();

        // Timing breakdown for merge phase
        tracing::info!(
            parent_job_id = %parent_job_id,
            fetch_ms = t_fetch.as_millis() as u64,
            merge_excel_ms = (t_merge_excel - t_fetch).as_millis() as u64,
            store_ms = (t_store - t_merge_excel).as_millis() as u64,
            total_ms = t_store.as_millis() as u64,
            total_records = total_records,
            file_size = excel_data.len(),
            "Merge phase timing breakdown"
        );

        // 7. Update job status to completed
        info!(parent_job_id = %parent_job_id, "Updating job status to completed");
        queue
            .update_job_status(parent_job_id, JobStatus::Completed)
            .await
            .map_err(|e| format!("Failed to update job status: {e}"))?;
        info!(parent_job_id = %parent_job_id, "Job status updated successfully");

        // 8. Publish DONE signal to Pub/Sub for streaming clients
        info!(parent_job_id = %parent_job_id, "Publishing DONE signal to clients");

        if let Err(e) = publisher
            .publish_done(parent_job_id, meta.total_batches, total_records)
            .await
        {
            // Non-fatal: streaming clients may have disconnected
            warn!(
                parent_job_id = %parent_job_id,
                error = %e,
                "Failed to publish DONE signal (non-fatal)"
            );
        } else {
            info!(
                parent_job_id = %parent_job_id,
                total_batches = meta.total_batches,
                total_records = total_records,
                "DONE signal published successfully"
            );
        }

        // 9. Cleanup batch results (optional, TTL handles this too)
        if let Err(e) = queue.delete_results(parent_job_id).await {
            warn!(
                parent_job_id = %parent_job_id,
                error = %e,
                "Failed to delete batch results (non-fatal)"
            );
        }

        info!(
            parent_job_id = %parent_job_id,
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
        msgs: &ExcelReportMessages,
    ) -> AppResult<(Vec<u8>, String, usize)> {
        let mut all_summaries: Vec<AttendanceSummary> = Vec::new();

        for (batch_key, json) in batch_results {
            let batch: Vec<AttendanceSummary> = serde_json::from_str(json)
                .map_err(|e| format!("Parse error for {batch_key}: {e}"))?;
            all_summaries.extend(batch);
        }

        // Sort by employee name (global sort)
        all_summaries.sort_by(|a, b| a.employee_name.cmp(&b.employee_name));

        info!(
            total_employees = all_summaries.len(),
            "Merged summary results"
        );

        // Generate Excel
        let excel_data =
            ExcelWriter::write_summary(&all_summaries, workspace_name, date_range, msgs)?;

        let total_records = all_summaries.len();

        let file_name = format!(
            "attendance_summary_{}_{}.xlsx",
            parent_job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name, total_records))
    }

    /// Merge detail results from all batches.
    fn merge_detail_results(
        batch_results: &[(String, String)],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        parent_job_id: uuid::Uuid,
        timezone: &str,
        work_unit_minutes: i32,
        msgs: &ExcelReportMessages,
    ) -> AppResult<(Vec<u8>, String, usize)> {
        let mut all_reports: Vec<DailyAttendanceReport> = Vec::new();

        for (batch_key, json) in batch_results {
            let batch: Vec<DailyAttendanceReport> = serde_json::from_str(json)
                .map_err(|e| format!("Parse error for {batch_key}: {e}"))?;
            all_reports.extend(batch);
        }

        // Sort by employee name, then by date
        all_reports.sort_by(|a, b| {
            a.employee_name
                .cmp(&b.employee_name)
                .then(a.date.cmp(&b.date))
        });

        info!(total_records = all_reports.len(), "Merged detail results");

        // Generate Excel
        let excel_data = ExcelWriter::write_detail(
            &all_reports,
            workspace_name,
            date_range,
            timezone,
            work_unit_minutes,
            msgs,
        )?;

        let total_records = all_reports.len();

        let file_name = format!(
            "attendance_detail_{}_{}.xlsx",
            parent_job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name, total_records))
    }

    /// Get workspace name, timezone, and default language from database.
    async fn get_workspace_info(
        workspace_repo: &dyn crate::repositories::WorkspaceRepository,
        workspace_id: uuid::Uuid,
    ) -> (String, String, String) {
        match workspace_repo.get_workspace_by_id(workspace_id).await {
            Ok(Some(workspace)) => (
                workspace.name,
                workspace.timezone,
                workspace.default_language,
            ),
            _ => ("Workspace".to_string(), "UTC".to_string(), "en".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn en_msgs() -> ExcelReportMessages {
        ExcelReportMessages::for_lang("en")
    }

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
            &en_msgs(),
        );

        assert!(result.is_ok());
        let (data, name, count) = result.unwrap();
        assert!(data.len() > 100); // Valid Excel file
        assert!(name.contains("attendance_summary_"));
        assert_eq!(count, 0);
    }

    #[test]
    fn test_merge_summary_with_data() {
        let summaries1 = vec![AttendanceSummary {
            employee_id: Uuid::new_v4(),
            employee_name: "Charlie Brown".to_string(),
            employee_id_number: "EMP003".to_string(),
            total_std_units: 160,
            total_ot_units: 0,
            total_leave_days: 2,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 160.0,
            computed_ot_hours: 0.0,
        }];
        let summaries2 = vec![AttendanceSummary {
            employee_id: Uuid::new_v4(),
            employee_name: "Alice Smith".to_string(),
            employee_id_number: "EMP001".to_string(),
            total_std_units: 152,
            total_ot_units: 0,
            total_leave_days: 4,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 152.0,
            computed_ot_hours: 0.0,
        }];

        let batch_results: Vec<(String, String)> = vec![
            (
                "batch_00000".to_string(),
                serde_json::to_string(&summaries1).unwrap(),
            ),
            (
                "batch_00001".to_string(),
                serde_json::to_string(&summaries2).unwrap(),
            ),
        ];

        let result = MergeHandler::merge_summary_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
            &en_msgs(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_merge_detail_sorting() {
        let report1 = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Bob Jones".to_string(),
            employee_id_number: "EMP002".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 8,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
        };
        let report2 = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Alice Smith".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 8,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
        };

        let batch_results: Vec<(String, String)> = vec![
            (
                "batch_00000".to_string(),
                serde_json::to_string(&vec![report1]).unwrap(),
            ),
            (
                "batch_00001".to_string(),
                serde_json::to_string(&vec![report2]).unwrap(),
            ),
        ];

        let result = MergeHandler::merge_detail_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
            "UTC",
            30,
            &en_msgs(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_json_handling() {
        let batch_results: Vec<(String, String)> =
            vec![("batch_00000".to_string(), "invalid json".to_string())];

        let result = MergeHandler::merge_summary_results(
            &batch_results,
            "Test Workspace",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
            &en_msgs(),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    /// Optimization #9: total_records must equal the actual number of deserialized records,
    /// not a fragile string match count.
    #[test]
    fn test_merge_summary_returns_accurate_total_records() {
        let s1 = vec![
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "A".to_string(),
                employee_id_number: "ID-A".to_string(),
                total_std_units: 160,
                total_ot_units: 0,
                total_leave_days: 0,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 160.0,
                computed_ot_hours: 0.0,
            },
            AttendanceSummary {
                employee_id: Uuid::new_v4(),
                employee_name: "B".to_string(),
                employee_id_number: "ID-B".to_string(),
                total_std_units: 120,
                total_ot_units: 0,
                total_leave_days: 3,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 120.0,
                computed_ot_hours: 0.0,
            },
        ];
        let s2 = vec![AttendanceSummary {
            employee_id: Uuid::new_v4(),
            employee_name: "C".to_string(),
            employee_id_number: "ID-C".to_string(),
            total_std_units: 80,
            total_ot_units: 0,
            total_leave_days: 1,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 80.0,
            computed_ot_hours: 0.0,
        }];

        let batch_results = vec![
            (
                "batch_00000".to_string(),
                serde_json::to_string(&s1).unwrap(),
            ),
            (
                "batch_00001".to_string(),
                serde_json::to_string(&s2).unwrap(),
            ),
        ];

        let (_, _, total_records) = MergeHandler::merge_summary_results(
            &batch_results,
            "W",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
            &en_msgs(),
        )
        .unwrap();

        assert_eq!(total_records, 3, "Should be 2 + 1 = 3 total records");
    }

    /// Verify detail merge also returns accurate total_records.
    #[test]
    fn test_merge_detail_returns_accurate_total_records() {
        let make_report = |name: &str, day: u32| DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: name.to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 8,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
        };

        let b1 = vec![
            make_report("Alice", 1),
            make_report("Alice", 2),
            make_report("Bob", 1),
        ];
        let b2 = vec![make_report("Charlie", 1)];

        let batch_results = vec![
            (
                "batch_00000".to_string(),
                serde_json::to_string(&b1).unwrap(),
            ),
            (
                "batch_00001".to_string(),
                serde_json::to_string(&b2).unwrap(),
            ),
        ];

        let (_, _, total_records) = MergeHandler::merge_detail_results(
            &batch_results,
            "W",
            (
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            ),
            Uuid::new_v4(),
            "UTC",
            30,
            &en_msgs(),
        )
        .unwrap();

        assert_eq!(total_records, 4, "Should be 3 + 1 = 4 total records");
    }

    #[test]
    fn test_attendance_summary_serialization_preserves_employee_id_number() {
        let summary = AttendanceSummary {
            employee_id: Uuid::new_v4(),
            employee_name: "Alice".to_string(),
            employee_id_number: "EMP-001".to_string(),
            total_std_units: 160,
            total_ot_units: 0,
            total_leave_days: 2,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 160.0,
            computed_ot_hours: 0.0,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("EMP-001"));
        let deserialized: AttendanceSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.employee_id_number, "EMP-001");
    }

    #[test]
    fn test_daily_attendance_report_serialization_preserves_employee_id_number() {
        let report = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employee_id: Uuid::new_v4(),
            employee_name: "Bob".to_string(),
            employee_id_number: "EMP-002".to_string(),
            shift_symbol: None,
            scheduled_start: None,
            scheduled_end: None,
            break_duration_minutes: None,
            break_start: None,
            break_end: None,
            is_break_auto_deduct: None,
            actual_punch_in: None,
            actual_punch_out: None,
            std_work_units: 8,
            ot_work_units: 0,
            ot_claim_status: None,
            unit_type: crate::models::enums::TimekeepingUnit::DAY,
            unit_coefficient_hours: 8.0,
            computed_std_hours: 8.0,
            computed_ot_hours: 0.0,
            status: crate::services::attendance_utils::AttendanceStatus::Present,
            is_holiday: false,
            is_unscheduled: false,
            holiday_name: None,
            generation_metadata: None,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("EMP-002"));
        let deserialized: DailyAttendanceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.employee_id_number, "EMP-002");
    }
}
