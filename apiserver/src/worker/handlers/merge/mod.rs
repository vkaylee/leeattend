//! Merge Handler
//!
//! Aggregates batch results into final Excel report and streams directly to client via Pub/Sub.
//! Receives a [`Job`] with [`MergePayload`] containing date_range and config.

use tracing::{info, warn};

use crate::errors::{AppError, AppResult};
use crate::queue::{Job, JobStatus, ReportType};
use crate::repositories::{PublisherRepository, QueueRepository};
use crate::services::import_messages::ExcelReportMessages;
use crate::utils::i18n::I18nKey;
use std::sync::Arc;

mod summary;
mod detail;

#[cfg(test)]
mod tests;

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
