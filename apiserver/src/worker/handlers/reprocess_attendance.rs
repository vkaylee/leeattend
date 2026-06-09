//! Bulk Attendance Reprocess Handler
//!
//! Re-evaluates attendance logs and generates time entries/shifts for a given
//! employee and date range. Updates JobMetadata status.

use chrono_tz::Tz;
use std::sync::Arc;
use tracing::{info, warn};

use crate::queue::{Job, JobStatus};

pub struct ReprocessAttendanceHandler;

impl ReprocessAttendanceHandler {
    pub async fn handle(
        queue: &Arc<dyn crate::repositories::QueueRepository>,
        job: &Job,
        deps: &crate::worker::WorkerDeps,
    ) -> Result<(), String> {
        let payload = job
            .reprocess_payload()
            .ok_or_else(|| "Missing reprocess payload".to_string())?;

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

        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(job.workspace_id);
        let employee = deps
            .employee_repo
            .get_employee_by_id(payload.employee_id, &auth_token)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Employee not found: {}", payload.employee_id))?;

        let active_employment_id = employee.active_employment_id().ok_or_else(|| {
            format!(
                "No active employment found for employee: {}",
                payload.employee_id
            )
        })?;

        // Initialize pairing service from shared deps
        let pairing_service = crate::services::pairing_service::PairingService::new(
            crate::services::pairing_service::PairingServiceArgs {
                time_entry_repo: Arc::clone(&deps.time_entry_repo),
                roster_repo: Arc::clone(&deps.roster_repo),
                policy_repo: Arc::clone(&deps.policy_repo),
                workspace_repo: Arc::clone(&deps.workspace_repo),
                employee_repo: Arc::clone(&deps.employee_repo),
                employment_repo: Arc::clone(&deps.employment_repo),
                holiday_repo: Arc::clone(&deps.holiday_repo),
                leave_request_repo: Arc::clone(&deps.leave_request_repo),
                daily_summary_repo: Arc::clone(&deps.daily_summary_repo),
                raw_log_repo: Arc::clone(&deps.raw_log_repo),
                shift_repo: Arc::clone(&deps.shift_repo),
            },
        );

        let mut current_date = payload.date_range.start;
        let end_date = payload.date_range.end;
        let mut success_count = 0;
        let mut error_count = 0;

        while current_date <= end_date {
            let tz: Tz = deps
                .employee_repo
                .get_employee_timezone_at_date(&auth_token, active_employment_id, current_date)
                .await
                .unwrap_or(chrono_tz::UTC);

            match pairing_service
                .reprocess_daily_attendance(
                    job.workspace_id,
                    active_employment_id,
                    current_date,
                    tz,
                )
                .await
            {
                Ok(()) => success_count += 1,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queue::{JobPayload, ReprocessPayload, TaskType};
    use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
    use crate::repositories::queue_repository::MockQueueRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::worker::test_utils::MockWorkerDepsBuilder;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_reprocess_attendance_handler_success() {
        let ws_id = Uuid::new_v4();
        let parent_job_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        let mut time_entry_repo = MockTimeEntryRepository::new();
        time_entry_repo
            .expect_replace_time_entries()
            .returning(|_, _, _, _, _| Ok(()));

        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();
        daily_summary_repo
            .expect_upsert_calculated_summary()
            .returning(|_, _, _, _, _, _, _, _| Ok(()));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));
        roster_repo
            .expect_get_workspace_schedules()
            .returning(|_, _, _, _, _| Box::pin(async { Ok(vec![]) }));

        let mut workspace_repo =
            crate::repositories::workspace_repository::MockWorkspaceRepository::new();
        workspace_repo.expect_get_workspace_by_id().returning(|_| {
            Box::pin(async {
                Ok(Some(crate::models::Workspace::new(
                    Uuid::new_v4(),
                    "test".to_string(),
                    "YYYY-MM-DD".to_string(),
                    "HH:mm".to_string(),
                    "UTC".to_string(),
                    "#ffffff".to_string(),
                    "en".to_string(),
                    None,
                    None,
                    "split".to_string(),
                    rust_decimal::Decimal::new(100, 2),
                    Utc::now(),
                    None,
                )))
            })
        });

        let mut builder = MockWorkerDepsBuilder::new();
        builder.workspace_repo = workspace_repo;
        builder.time_entry_repo = time_entry_repo;
        builder.daily_summary_repo = daily_summary_repo;

        let employment_id = Uuid::new_v4();
        let mut emp_repo = crate::repositories::employee::MockEmployeeRepository::new();
        emp_repo.expect_get_employee_by_id().returning(move |_, _| {
            Ok(Some(
                crate::models::Employee::new(
                    emp_id,
                    ws_id,
                    None,
                    "Test Employee".to_string(),
                    "EMP001".to_string(),
                    vec![],
                    Utc::now(),
                    Utc::now(),
                    None,
                )
                .with_employment_info(Some(employment_id), None, None),
            ))
        });
        emp_repo
            .expect_get_employee_timezone_fallback()
            .returning(|_, _| Ok(chrono_tz::UTC));
        emp_repo
            .expect_get_employee_timezone_at_date()
            .returning(|_, _, _| Ok(chrono_tz::UTC));
        builder.employee_repo = emp_repo;
        builder.roster_repo = roster_repo;

        let mut raw_log_repo =
            crate::repositories::raw_attendance_log_repository::MockRawAttendanceLogRepository::new(
            );
        raw_log_repo
            .expect_get_employment_logs_in_time_range()
            .returning(|_, _, _, _| Box::pin(async { Ok(vec![]) }));
        builder.raw_log_repo = raw_log_repo;

        let mut leave_repo =
            crate::repositories::leave_request_repository::MockLeaveRequestRepository::new();
        leave_repo
            .expect_get_approved_leave_dates()
            .returning(|_, _, _, _| Box::pin(async { Ok(std::collections::HashSet::new()) }));
        builder.leave_request_repo = leave_repo;

        let deps = builder.build();

        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id,
            batch_index: 0,
            task_type: TaskType::ReprocessAttendance,
            workspace_id: ws_id,
            payload: JobPayload::ReprocessAttendance(ReprocessPayload {
                employee_id: emp_id,
                date_range: crate::queue::DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                },
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
