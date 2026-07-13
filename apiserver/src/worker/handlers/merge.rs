//! Merge Handler
//!
//! Aggregates batch results into final Excel report and streams directly to client via Pub/Sub.
//! Receives a [`Job`] with [`MergePayload`] containing date_range and config.

use chrono::{NaiveDate, Utc};
use tracing::{info, warn};

use crate::errors::{AppError, AppResult};
use crate::models::{AttendanceSummary, DailyAttendanceReport};
use crate::queue::{Job, JobStatus, ReportType};
use crate::repositories::{PublisherRepository, QueueRepository};
use crate::services::import_messages::ExcelReportMessages;
use crate::services::ExcelWriter;
use crate::utils::i18n::I18nKey;
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
        let parent_job_id = job.parent_job_id;
        match Self::handle_inner(queue, publisher, job, deps).await {
            Ok(()) => {
                let auth_token =
                    crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id);
                if let Err(e) = deps
                    .outbox_repo
                    .update_job_status(&auth_token, parent_job_id, "completed", None)
                    .await
                {
                    tracing::error!(
                        parent_job_id = %parent_job_id,
                        error = ?e,
                        "Failed to update outbox job status to completed"
                    );
                }
                Ok(())
            }
            Err(e) => {
                let err_str = e.to_string();
                let auth_token =
                    crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id);
                if let Err(err) = deps
                    .outbox_repo
                    .update_job_status(&auth_token, parent_job_id, "failed", Some(err_str.clone()))
                    .await
                {
                    tracing::error!(
                        parent_job_id = %parent_job_id,
                        error = ?err,
                        "Failed to update outbox job status to failed"
                    );
                }
                Err(err_str)
            }
        }
    }

    #[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
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
            .ok_or_else(|| AppError::not_found(I18nKey::JobNotFound))?;

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
        let date_range = meta.date_range.as_ref().map_or_else(
            || {
                // Fallback: try to get from merge payload
                job.merge_payload().map_or_else(
                    || {
                        let d = chrono::NaiveDate::from_ymd_opt(2000, 1, 1).unwrap_or_default();
                        (d, d)
                    },
                    |p| (p.date_range.start, p.date_range.end),
                )
            },
            |dr| (dr.start, dr.end),
        );

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(meta.workspace_id);

        // 5. Merge results and generate Excel
        let (excel_data, file_name, total_records) = match meta.report_type {
            Some(ReportType::Summary) => {
                Self::merge_summary_results(
                    &batch_results,
                    &workspace_name,
                    date_range,
                    parent_job_id,
                    &excel_msgs,
                    deps,
                    &auth_token,
                )
                .await?
            }
            Some(ReportType::Detail) => {
                let work_unit_minutes = job
                    .merge_payload()
                    .map_or(30, |p| p.config.work_unit_minutes);
                Self::merge_detail_results(
                    &batch_results,
                    &workspace_name,
                    date_range,
                    parent_job_id,
                    &workspace_tz,
                    work_unit_minutes,
                    &excel_msgs,
                    deps,
                    &auth_token,
                )
                .await?
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
            merge_excel_ms = t_merge_excel.checked_sub(t_fetch).unwrap_or_default().as_millis() as u64,
            store_ms = t_store.checked_sub(t_merge_excel).unwrap_or_default().as_millis() as u64,
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
    async fn merge_summary_results(
        batch_results: &[(String, String)],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        parent_job_id: uuid::Uuid,
        msgs: &ExcelReportMessages,
        deps: &crate::worker::WorkerDeps,
        auth_token: &crate::diesel_pool::TenantAuthToken,
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

        // Fetch custom field definitions and values
        let all_definitions = deps
            .custom_field_repo
            .list_definitions(auth_token, None)
            .await?;
        let exportable_definitions: Vec<_> = all_definitions
            .into_iter()
            .filter(crate::models::CustomFieldDefinition::is_exportable)
            .collect();

        let mut cf_headers = Vec::new();
        let mut cf_values_map = std::collections::HashMap::new();

        if !exportable_definitions.is_empty() && !all_summaries.is_empty() {
            let mut entity_ids = Vec::new();
            for s in &all_summaries {
                entity_ids.push(s.employee_id);
                entity_ids.push(s.employment_id);
            }

            let values_map = deps
                .custom_field_repo
                .get_values_for_entities(auth_token, &entity_ids)
                .await?;

            cf_headers = exportable_definitions
                .iter()
                .map(|d| d.field_label().to_string())
                .collect();

            for summary in &all_summaries {
                let mut row_values = Vec::new();
                for def in &exportable_definitions {
                    let lookup_id = match def.entity_type() {
                        "employee" => Some(summary.employee_id),
                        "employment" => Some(summary.employment_id),
                        _ => None,
                    };

                    let mut found_val_str = String::new();
                    if let Some(id) = lookup_id {
                        if let Some(pairs) = values_map.get(&id) {
                            if let Some((val, _)) = pairs.iter().find(|(_, d)| d.id() == def.id()) {
                                found_val_str = match def.field_type() {
                                    "text" | "select" | "url" | "email" | "phone" => {
                                        val.value_text().unwrap_or("").to_string()
                                    }
                                    "number" => val
                                        .value_number()
                                        .map(|n| n.to_string())
                                        .unwrap_or_default(),
                                    "date" => {
                                        val.value_date().map(|d| d.to_string()).unwrap_or_default()
                                    }
                                    "boolean" => {
                                        val.value_bool().map(|b| b.to_string()).unwrap_or_default()
                                    }
                                    "multiselect" => val
                                        .value_json()
                                        .map(ToString::to_string)
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                };
                            }
                        }
                    }
                    row_values.push(found_val_str);
                }
                cf_values_map.insert(summary.employee_id, row_values);
            }
        }

        let total_records = all_summaries.len();
        let ws_name = workspace_name.to_string();
        let messages = *msgs;

        // Generate Excel in thread pool
        let excel_data = tokio::task::spawn_blocking(move || {
            ExcelWriter::write_summary(
                &all_summaries,
                &ws_name,
                date_range,
                &messages,
                &cf_headers,
                &cf_values_map,
            )
        })
        .await
        .map_err(|e| AppError::Internal(format!("Excel generation join error: {e}")))??;

        let file_name = format!(
            "attendance_summary_{}_{}.xlsx",
            parent_job_id,
            Utc::now().format("%Y%m%d_%H%M%S")
        );

        Ok((excel_data, file_name, total_records))
    }

    /// Merge detail results from all batches.
    #[allow(clippy::too_many_lines)]
    async fn merge_detail_results(
        batch_results: &[(String, String)],
        workspace_name: &str,
        date_range: (NaiveDate, NaiveDate),
        parent_job_id: uuid::Uuid,
        timezone: &str,
        work_unit_minutes: i32,
        msgs: &ExcelReportMessages,
        deps: &crate::worker::WorkerDeps,
        auth_token: &crate::diesel_pool::TenantAuthToken,
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

        // Fetch custom field definitions and values
        let all_definitions = deps
            .custom_field_repo
            .list_definitions(auth_token, None)
            .await?;
        let exportable_definitions: Vec<_> = all_definitions
            .into_iter()
            .filter(crate::models::CustomFieldDefinition::is_exportable)
            .collect();

        let mut cf_headers = Vec::new();
        let mut cf_values_map = std::collections::HashMap::new();

        if !exportable_definitions.is_empty() && !all_reports.is_empty() {
            let mut entity_ids = Vec::new();
            for r in &all_reports {
                entity_ids.push(r.employee_id);
                entity_ids.push(r.employment_id);
            }

            let values_map = deps
                .custom_field_repo
                .get_values_for_entities(auth_token, &entity_ids)
                .await?;

            cf_headers = exportable_definitions
                .iter()
                .map(|d| d.field_label().to_string())
                .collect();

            for report in &all_reports {
                let mut row_values = Vec::new();
                for def in &exportable_definitions {
                    let lookup_id = match def.entity_type() {
                        "employee" => Some(report.employee_id),
                        "employment" => Some(report.employment_id),
                        _ => None,
                    };

                    let mut found_val_str = String::new();
                    if let Some(id) = lookup_id {
                        if let Some(pairs) = values_map.get(&id) {
                            if let Some((val, _)) = pairs.iter().find(|(_, d)| d.id() == def.id()) {
                                found_val_str = match def.field_type() {
                                    "text" | "select" | "url" | "email" | "phone" => {
                                        val.value_text().unwrap_or("").to_string()
                                    }
                                    "number" => val
                                        .value_number()
                                        .map(|n| n.to_string())
                                        .unwrap_or_default(),
                                    "date" => {
                                        val.value_date().map(|d| d.to_string()).unwrap_or_default()
                                    }
                                    "boolean" => {
                                        val.value_bool().map(|b| b.to_string()).unwrap_or_default()
                                    }
                                    "multiselect" => val
                                        .value_json()
                                        .map(ToString::to_string)
                                        .unwrap_or_default(),
                                    _ => String::new(),
                                };
                            }
                        }
                    }
                    row_values.push(found_val_str);
                }
                cf_values_map.insert(report.employee_id, row_values);
            }
        }

        let total_records = all_reports.len();
        let ws_name = workspace_name.to_string();
        let tz = timezone.to_string();
        let messages = *msgs;

        // Generate Excel in thread pool
        let excel_data = tokio::task::spawn_blocking(move || {
            ExcelWriter::write_detail(
                &all_reports,
                &ws_name,
                date_range,
                &tz,
                work_unit_minutes,
                &messages,
                &cf_headers,
                &cf_values_map,
            )
        })
        .await
        .map_err(|e| AppError::Internal(format!("Excel generation join error: {e}")))??;

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
                workspace.name().to_string(),
                workspace.timezone().to_string(),
                workspace.default_language().to_string(),
            ),
            _ => ("Workspace".to_string(), "UTC".to_string(), "en".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::TenantAuthToken;
    use uuid::Uuid;

    fn en_msgs() -> ExcelReportMessages {
        ExcelReportMessages::for_lang("en")
    }

    #[tokio::test]
    async fn test_merge_summary_empty_batches() {
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        let deps = MockWorkerDepsBuilder::new().build();
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

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
            &deps,
            &auth_token,
        )
        .await;

        assert!(result.is_ok());
        let (data, name, count) = result.unwrap();
        assert!(data.len() > 100); // Valid Excel file
        assert!(name.contains("attendance_summary_"));
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_merge_summary_with_data() {
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        let deps = MockWorkerDepsBuilder::new().build();
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

        let summaries1 = vec![AttendanceSummary {
            employment_id: Uuid::new_v4(),
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
            employer_id: None,
            employer_name: None,
        }];
        let summaries2 = vec![AttendanceSummary {
            employment_id: Uuid::new_v4(),
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
            employer_id: None,
            employer_name: None,
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
            &deps,
            &auth_token,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_merge_detail_sorting() {
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        let deps = MockWorkerDepsBuilder::new().build();
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

        let report1 = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Bob Jones".to_string(),
            employee_id_number: "EMP002".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            grace_period_minutes: None,
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
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
        };
        let report2 = DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Alice Smith".to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            grace_period_minutes: None,
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
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
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
            &deps,
            &auth_token,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_json_handling() {
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        let deps = MockWorkerDepsBuilder::new().build();
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

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
            &deps,
            &auth_token,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    /// Optimization #9: total_records must equal the actual number of deserialized records,
    /// not a fragile string match count.
    #[tokio::test]
    async fn test_merge_summary_returns_accurate_total_records() {
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        let deps = MockWorkerDepsBuilder::new().build();
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

        let s1 = vec![
            AttendanceSummary {
                employment_id: Uuid::new_v4(),
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
                employer_id: None,
                employer_name: None,
            },
            AttendanceSummary {
                employment_id: Uuid::new_v4(),
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
                employer_id: None,
                employer_name: None,
            },
        ];
        let s2 = vec![AttendanceSummary {
            employment_id: Uuid::new_v4(),
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
            employer_id: None,
            employer_name: None,
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
            &deps,
            &auth_token,
        )
        .await
        .unwrap();

        assert_eq!(total_records, 3, "Should be 2 + 1 = 3 total records");
    }

    /// Verify detail merge also returns accurate total_records.
    #[tokio::test]
    async fn test_merge_detail_returns_accurate_total_records() {
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        let deps = MockWorkerDepsBuilder::new().build();
        let auth_token = TenantAuthToken::system_override(Uuid::new_v4());

        let make_report = |name: &str, day: u32| DailyAttendanceReport {
            date: NaiveDate::from_ymd_opt(2026, 1, day).unwrap(),
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: name.to_string(),
            employee_id_number: "EMP001".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            grace_period_minutes: None,
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
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
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
            &deps,
            &auth_token,
        )
        .await
        .unwrap();

        assert_eq!(total_records, 4, "Should be 3 + 1 = 4 total records");
    }

    #[test]
    fn test_attendance_summary_serialization_preserves_employee_id_number() {
        let summary = AttendanceSummary {
            employment_id: Uuid::new_v4(),
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
            employer_id: None,
            employer_name: None,
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
            employment_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            employee_name: "Bob".to_string(),
            employee_id_number: "EMP-002".to_string(),
            shift_symbol: None,
            shift_name: None,
            scheduled_start: None,
            scheduled_end: None,
            grace_period_minutes: None,
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
            is_leave_paid: None,
            employer_name: None,
            timezone: None,
            segment_id: None,
            shift_id: None,
            interval_id: None,
            interval_name: None,
            segment_type: None,
            sort_order: None,
            segment_count: None,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("EMP-002"));
        let deserialized: DailyAttendanceReport = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.employee_id_number, "EMP-002");
    }

    #[tokio::test]
    async fn test_merge_handler_handle_inner_with_mocks() {
        use crate::queue::{
            DateRange, Job, JobMetadata, JobPayload, JobPriority, JobStatus, MergePayload,
            ReportConfig, ReportType, TaskType,
        };
        use crate::repositories::publisher_repository::MockPublisherRepository;
        use crate::repositories::queue_repository::MockQueueRepository;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use std::sync::Arc;

        let ws_id = Uuid::new_v4();
        let parent_job_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(move |_| {
            Ok(Some(JobMetadata {
                job_id: parent_job_id,
                requester_id: Uuid::new_v4(),
                total_batches: 2,
                completed_batches: 2,
                failed_batches: 0,
                status: JobStatus::Completed,
                workspace_id: ws_id,
                report_type: Some(ReportType::Summary),
                date_range: Some(DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                }),
                created_at: Utc::now(),
                completed_at: Some(Utc::now()),
                error_message: None,
                task_type: None,
                payload: None,
            }))
        });

        queue.expect_get_all_results().returning(move |_| {
            let summary = AttendanceSummary {
                employment_id: Uuid::new_v4(),
                employee_id: Uuid::new_v4(),
                employee_name: "Mocked Emp".to_string(),
                employee_id_number: "MOCK-001".to_string(),
                total_std_units: 8,
                total_ot_units: 0,
                total_leave_days: 0,
                unit_type: crate::models::enums::TimekeepingUnit::DAY,
                unit_coefficient_hours: 8.0,
                computed_std_hours: 8.0,
                computed_ot_hours: 0.0,
                employer_id: None,
                employer_name: None,
            };
            let json = serde_json::to_string(&vec![summary]).unwrap();
            Ok(vec![
                ("batch_0".to_string(), json.clone()),
                ("batch_1".to_string(), json),
            ])
        });

        queue.expect_set_result_data().returning(|_, _, _| Ok(()));
        queue.expect_update_job_status().returning(|_, _| Ok(()));
        queue.expect_delete_results().returning(|_| Ok(()));

        let mut publisher = MockPublisherRepository::new();
        publisher.expect_publish_done().returning(|_, _, _| Ok(()));

        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                Box::pin(async move {
                    let ws = crate::models::Workspace::new(
                        ws_id,
                        "Test Workspace".to_string(),
                        "YYYY-MM-DD".to_string(),
                        "HH:mm".to_string(),
                        "UTC".to_string(),
                        "#000000".to_string(),
                        "en".to_string(),
                        None,
                        None,
                        "split".to_string(),
                        rust_decimal::Decimal::new(15, 1),
                        Utc::now(),
                        None,
                    );
                    Ok(Some(ws))
                })
            });

        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id,
            batch_index: 0,
            task_type: TaskType::MergeResults,
            workspace_id: ws_id,
            payload: JobPayload::MergeResults(MergePayload {
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                },
            }),
            priority: JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
            trace_context: None,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

        let result = MergeHandler::handle_inner(&queue_arc, &pub_arc, &job, &deps).await;

        assert!(
            result.is_ok(),
            "MergeHandler::handle_inner failed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_merge_handler_handle_inner_detail_with_mocks() {
        use crate::queue::{
            DateRange, Job, JobMetadata, JobPayload, JobPriority, JobStatus, MergePayload,
            ReportConfig, ReportType, TaskType,
        };
        use crate::repositories::publisher_repository::MockPublisherRepository;
        use crate::repositories::queue_repository::MockQueueRepository;
        use crate::worker::test_utils::MockWorkerDepsBuilder;
        use std::sync::Arc;

        let ws_id = Uuid::new_v4();
        let parent_job_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(move |_| {
            Ok(Some(JobMetadata {
                job_id: parent_job_id,
                requester_id: Uuid::new_v4(),
                total_batches: 2,
                completed_batches: 2,
                failed_batches: 0,
                status: JobStatus::Completed,
                workspace_id: ws_id,
                report_type: Some(ReportType::Detail),
                date_range: Some(DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                }),
                created_at: Utc::now(),
                completed_at: Some(Utc::now()),
                error_message: None,
                task_type: None,
                payload: None,
            }))
        });

        queue.expect_get_all_results().returning(move |_| {
            let report = DailyAttendanceReport {
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                employment_id: Uuid::new_v4(),
                employee_id: Uuid::new_v4(),
                employee_name: "Mocked Emp".to_string(),
                employee_id_number: "MOCK-001".to_string(),
                shift_symbol: Some("HC".to_string()),
                shift_name: Some("Day Shift".to_string()),
                scheduled_start: None,
                scheduled_end: None,
                grace_period_minutes: None,
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
                is_leave_paid: None,
                employer_name: None,
                timezone: None,
                segment_id: None,
                shift_id: None,
                interval_id: None,
                interval_name: None,
                segment_type: None,
                sort_order: None,
                segment_count: None,
            };
            let json = serde_json::to_string(&vec![report]).unwrap();
            Ok(vec![
                ("batch_0".to_string(), json.clone()),
                ("batch_1".to_string(), json),
            ])
        });

        queue.expect_set_result_data().returning(|_, _, _| Ok(()));
        queue.expect_update_job_status().returning(|_, _| Ok(()));
        queue.expect_delete_results().returning(|_| Ok(()));

        let mut publisher = MockPublisherRepository::new();
        publisher.expect_publish_done().returning(|_, _, _| Ok(()));

        let mut builder = MockWorkerDepsBuilder::new();
        builder
            .workspace_repo
            .expect_get_workspace_by_id()
            .returning(move |_| {
                Box::pin(async move {
                    let ws = crate::models::Workspace::new(
                        ws_id,
                        "Test Workspace".to_string(),
                        "YYYY-MM-DD".to_string(),
                        "HH:mm".to_string(),
                        "UTC".to_string(),
                        "#000000".to_string(),
                        "en".to_string(),
                        None,
                        None,
                        "split".to_string(),
                        rust_decimal::Decimal::new(15, 1),
                        Utc::now(),
                        None,
                    );
                    Ok(Some(ws))
                })
            });

        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id,
            batch_index: 0,
            task_type: TaskType::MergeResults,
            workspace_id: ws_id,
            payload: JobPayload::MergeResults(MergePayload {
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Detail,
                    timezone: "UTC".to_string(),
                    work_unit_minutes: 30,
                    rounding_policy: "floor".to_string(),
                },
            }),
            priority: JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
            trace_context: None,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let pub_arc: Arc<dyn crate::repositories::PublisherRepository> = Arc::new(publisher);

        let result = MergeHandler::handle_inner(&queue_arc, &pub_arc, &job, &deps).await;

        assert!(
            result.is_ok(),
            "MergeHandler::handle_inner detail failed: {:?}",
            result.err()
        );
    }
}
