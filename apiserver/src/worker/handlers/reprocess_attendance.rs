//! Bulk Attendance Reprocess Handler
//!
//! Re-evaluates attendance logs and generates time entries/shifts for a given
//! employee and date range. Updates JobMetadata status.

use chrono_tz::Tz;
use std::sync::Arc;
use tracing::{info, warn};

use crate::queue::{Job, JobQueue, JobStatus, RedisPool};

pub struct ReprocessAttendanceHandler;

impl ReprocessAttendanceHandler {
    pub async fn handle(
        diesel: &crate::diesel_pool::DieselPools,
        job: &Job,
        redis: &RedisPool,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .reprocess_payload()
            .ok_or_else(|| "Missing reprocess payload".to_string())?;

        let queue = JobQueue::new(redis.clone());

        // Update metadata -> Processing
        if let Ok(Some(mut meta)) = queue.get_job_metadata(job.parent_job_id).await {
            // Only update to Processing if it's Pending, to avoid overwriting Completed/Failed
            if meta.status == JobStatus::Pending {
                meta.status = JobStatus::Processing;
                let _ = queue.update_job_metadata(&meta).await;
            }
        }

        info!(
            job_id = %job.job_id,
            employee_id = %payload.employee_id,
            start_date = %payload.date_range.start,
            end_date = %payload.date_range.end,
            "Starting bulk attendance reprocess"
        );

        let tz_str: String = deps
            .workspace_repo
            .get_workspace_by_id(job.workspace_id)
            .await
            .map(|w| w.map(|w| w.timezone).unwrap_or_else(|| "UTC".to_string()))
            .unwrap_or_else(|_| "UTC".to_string());

        let tz: Tz = tz_str.parse().unwrap_or(chrono_tz::UTC);

        // Initialize pairing service from shared deps
        let pairing_service = crate::services::pairing_service::PairingService::new(
            crate::services::pairing_service::PairingServiceArgs {
                time_entry_repo: Arc::clone(&deps.time_entry_repo),
                roster_repo: Arc::clone(&deps.roster_repo),
                policy_repo: Arc::clone(&deps.policy_repo),
                workspace_repo: Arc::new(crate::repositories::DieselWorkspaceRepository::new(
                    diesel.clone(),
                )),
                employee_repo: Arc::new(crate::repositories::DieselEmployeeRepository::new(
                    diesel.clone(),
                )),
                holiday_repo: Arc::new(crate::repositories::DieselHolidayRepository::new(
                    diesel.clone(),
                )),
                leave_request_repo: Arc::new(
                    crate::repositories::DieselLeaveRequestRepository::new(diesel.clone()),
                ),
                daily_summary_repo: Arc::new(
                    crate::repositories::DieselAttendanceDailySummaryRepository::new(
                        diesel.clone(),
                    ),
                ),
                raw_log_repo: Arc::new(crate::repositories::DieselRawAttendanceLogRepository::new(
                    diesel.clone(),
                )),
                shift_repo: Arc::new(crate::repositories::DieselShiftRepository::new(
                    diesel.clone(),
                )),
            },
        );

        let mut current_date = payload.date_range.start;
        let end_date = payload.date_range.end;
        let mut success_count = 0;
        let mut error_count = 0;

        while current_date <= end_date {
            match pairing_service
                .reprocess_daily_attendance(job.workspace_id, payload.employee_id, current_date, tz)
                .await
            {
                Ok(_) => success_count += 1,
                Err(e) => {
                    warn!(
                        employee_id = %payload.employee_id,
                        date = %current_date,
                        error = %e,
                        "Failed to reprocess attendance for date"
                    );
                    error_count += 1;
                }
            }
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }

        info!(
            job_id = %job.job_id,
            employee_id = %payload.employee_id,
            success_count,
            error_count,
            "Finished bulk attendance reprocess for employee"
        );

        if error_count > 0 && success_count == 0 {
            Err(format!("Reprocess failed for all {error_count} days"))
        } else {
            Ok(())
        }
    }
}
