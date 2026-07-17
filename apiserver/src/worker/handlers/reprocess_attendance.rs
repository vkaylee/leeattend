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

        // Prefer the exact employment when the caller pinned one (e.g. shift
        // reassignment), otherwise fall back to the employee's active employment.
        let active_employment_id = match payload.employment_id {
            Some(id) => id,
            None => {
                let employee = deps
                    .employee_repo
                    .get_employee_by_id(payload.employee_id, &auth_token)
                    .await
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| format!("Employee not found: {}", payload.employee_id))?;

                employee.active_employment_id().ok_or_else(|| {
                    format!(
                        "No active employment found for employee: {}",
                        payload.employee_id
                    )
                })?
            }
        };

        // Initialize pairing service from shared deps
        let pairing_service = deps.pairing_service();

        let mut success_count = 0;
        let mut error_count = 0;

        let tz: Tz = deps
            .employee_repo
            .get_employee_timezone_fallback(&auth_token, active_employment_id)
            .await
            .unwrap_or(chrono_tz::UTC);

        match pairing_service
            .reprocess_attendance_range(
                job.workspace_id,
                active_employment_id,
                payload.date_range.start,
                payload.date_range.end,
                tz,
            )
            .await
        {
            Ok(()) => {
                success_count = (payload.date_range.end - payload.date_range.start).num_days() + 1;
            }
            Err(e) => {
                warn!(
                    employee_id = %payload.employee_id,
                    error = %e,
                    "Failed to reprocess attendance range"
                );
                error_count = (payload.date_range.end - payload.date_range.start).num_days() + 1;
            }
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

    /// How the handler is expected to resolve the employment to reprocess.
    enum EmploymentResolution {
        /// `employment_id` is `None`: handler must look up the employee and use
        /// its active employment. `get_employee_by_id` is expected to be called.
        ViaActiveEmployee,
        /// `employment_id` is `Some`: handler must use it directly and never call
        /// `get_employee_by_id`.
        Pinned,
    }

    /// Build a fully-mocked [`WorkerDeps`] whose pairing pipeline succeeds with no
    /// logs (so `reprocess_attendance_range` runs but produces empty results).
    ///
    /// `resolution` controls the employee-repo expectation so a test can assert the
    /// handler took the pinned-employment path without touching the employee lookup.
    fn build_deps(
        emp_id: Uuid,
        employment_id: Uuid,
        ws_id: Uuid,
        resolution: EmploymentResolution,
    ) -> crate::worker::WorkerDeps {
        let mut time_entry_repo = MockTimeEntryRepository::new();
        time_entry_repo
            .expect_replace_time_entries()
            .returning(|_, _, _, _, _| Ok(()));
        time_entry_repo
            .expect_get_employment_time_entries()
            .returning(|_, _, _, _| Ok(vec![]));

        let mut daily_summary_repo = MockAttendanceDailySummaryRepository::new();
        daily_summary_repo
            .expect_upsert_calculated_summary()
            .returning(|_, _, _, _, _, _, _, _| Ok(()));
        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));
        daily_summary_repo
            .expect_get_summaries()
            .returning(|_, _, _, _| Ok(vec![]));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| Box::pin(async { Ok((None, vec![])) }));

        let sched_emp_id = emp_id;
        let sched_employment_id = employment_id;
        roster_repo.expect_get_workspace_schedules().returning(
            move |_, _, start_date, end_date, _, _, _| {
                let empty_schedule = |date| {
                    crate::models::EffectiveSchedule::new(
                        date,
                        None,
                        "none".to_string(),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        vec![],
                        false,
                        false,
                        None,
                    )
                };
                let schedules = vec![empty_schedule(start_date), empty_schedule(end_date)];
                Box::pin(async move {
                    Ok((
                        vec![crate::models::EmployeeSchedule::new(
                            sched_employment_id,
                            sched_emp_id,
                            "EMP001".to_string(),
                            "Test Employee".to_string(),
                            schedules,
                        )],
                        1,
                    ))
                })
            },
        );

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

        let mut emp_repo = crate::repositories::employee::MockEmployeeRepository::new();
        match resolution {
            EmploymentResolution::ViaActiveEmployee => {
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
                        .with_employment_info(
                            Some(employment_id),
                            None,
                            None,
                        ),
                    ))
                });
            }
            EmploymentResolution::Pinned => {
                // Pinned path must not consult the employee lookup at all.
                emp_repo.expect_get_employee_by_id().never();
            }
        }
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
        leave_repo
            .expect_get_employee_leave_requests()
            .returning(|_, _| Box::pin(async { Ok(vec![]) }));
        builder.leave_request_repo = leave_repo;

        let mut employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        employment_repo
            .expect_get_active_employment_on_date()
            .returning(|_, _, _| Ok(None));
        employment_repo
            .expect_get_employment_by_id()
            .returning(|_, employment_id| {
                Ok(Some(crate::models::employment::Employment::new(
                    employment_id,
                    uuid::Uuid::new_v4(),
                    employment_id,
                    uuid::Uuid::new_v4(),
                    None,
                    None,
                    "active".to_string(),
                    chrono::NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                    None,
                    None,
                    "full_time".to_string(),
                    None,
                    chrono::Utc::now(),
                    chrono::Utc::now(),
                    None,
                    None,
                )))
            });
        builder.employment_repo = employment_repo;

        let mut holiday_repo =
            crate::repositories::holiday_repository::MockHolidayRepository::new();
        holiday_repo
            .expect_is_holiday()
            .returning(|_, _, _| Box::pin(async { Ok(None) }));
        holiday_repo
            .expect_get_holidays_in_range()
            .returning(|_, _, _| Box::pin(async { Ok(std::collections::HashMap::new()) }));
        builder.holiday_repo = holiday_repo;

        builder.build()
    }

    fn reprocess_job(ws_id: Uuid, employee_id: Uuid, employment_id: Option<Uuid>) -> Job {
        Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: TaskType::ReprocessAttendance,
            workspace_id: ws_id,
            payload: JobPayload::ReprocessAttendance(ReprocessPayload {
                employee_id,
                employment_id,
                date_range: crate::queue::DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
                },
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
            trace_context: None,
        }
    }

    #[tokio::test]
    async fn test_reprocess_attendance_handler_success() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        let deps = build_deps(
            emp_id,
            employment_id,
            ws_id,
            EmploymentResolution::ViaActiveEmployee,
        );
        let job = reprocess_job(ws_id, emp_id, None);

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }

    /// When `employment_id` is set (shift reassignment), the handler reprocesses
    /// that exact employment and never resolves the employee's active one — the
    /// `.never()` expectation on `get_employee_by_id` enforces this.
    #[tokio::test]
    async fn test_reprocess_attendance_handler_uses_pinned_employment() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        let deps = build_deps(emp_id, employment_id, ws_id, EmploymentResolution::Pinned);
        let job = reprocess_job(ws_id, emp_id, Some(employment_id));

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }
}
