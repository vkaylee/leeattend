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
    #[allow(clippy::too_many_lines)]
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
        // reassignment), otherwise resolve the employment(s) covering the date
        // range. Terminated employments are valid if their date range covers
        // the reprocess range.
        let active_employment_id = if let Some(id) = payload.employment_id {
            // Validate workspace ownership for pinned employment
            let employment = deps
                .employment_repo
                .get_employment_by_id(&auth_token, id)
                .await
                .map_err(|e| format!("Failed to lookup pinned employment: {e}"))?
                .ok_or_else(|| format!("Pinned employment not found: {id}"))?;
            if employment.workspace_id() != job.workspace_id {
                return Err(format!(
                    "Pinned employment {id} does not belong to workspace {}",
                    job.workspace_id
                ));
            }
            id
        } else {
            // Resolve employment(s) overlapping the date range.
            let employments = deps
                .employment_repo
                .list_employments_effective_in_range(
                    &auth_token,
                    payload.employee_id,
                    payload.date_range.start,
                    payload.date_range.end,
                )
                .await
                .map_err(|e| format!("Failed to resolve employment by date range: {e}"))?;

            if employments.is_empty() {
                return Err(format!(
                    "No employment covers date range {:?}–{:?} for employee {}",
                    payload.date_range.start, payload.date_range.end, payload.employee_id
                ));
            }
            if employments.len() > 1 {
                return Err(format!(
                    "Date range {:?}–{:?} spans {} employments for employee {}. Specify employment_id.",
                    payload.date_range.start,
                    payload.date_range.end,
                    employments.len(),
                    payload.employee_id
                ));
            }
            employments[0].id()
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
        /// `employment_id` is `None`: handler must resolve employment by date range.
        /// `list_employments_effective_in_range` is expected to be called.
        ViaDateRange,
        /// `employment_id` is `Some`: handler must validate it via
        /// `get_employment_by_id` and never call employee lookup.
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
        resolution: &EmploymentResolution,
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
            .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(()));
        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));
        daily_summary_repo
            .expect_get_summaries()
            .returning(|_, _, _, _| Ok(vec![]));
        daily_summary_repo
            .expect_batch_upsert_calculated_summaries()
            .returning(|_, _, _| Ok(()));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| {
                Box::pin(async {
                    Ok((
                        None,
                        vec![],
                        std::sync::Arc::new(crate::cache::RosterData::default()),
                    ))
                })
            });

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
        match *resolution {
            EmploymentResolution::ViaDateRange => {
                // Date-range path: do NOT call employee lookup; use employment repo instead.
                emp_repo.expect_get_employee_by_id().never();
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
            .expect_resolve_employment_on_date()
            .returning(|_, _, _| Ok(crate::models::employment::EmploymentResolution::NotFound));
        employment_repo
            .expect_list_employments_effective_in_range()
            .returning(move |_, _, _, _| {
                Ok(vec![crate::models::employment::Employment::new(
                    employment_id,
                    ws_id,
                    emp_id,
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
                )])
            });
        employment_repo
            .expect_get_employment_by_id()
            .returning(move |_, employment_id| {
                Ok(Some(crate::models::employment::Employment::new(
                    employment_id,
                    ws_id,
                    emp_id,
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

        // Mock get_employee_effective_policy - return None to use fallback rules
        builder
            .policy_repo
            .expect_get_employee_effective_policy()
            .returning(|_, _, _| Ok(None));

        // Mock get_working_conditions_at - return empty vec (no special conditions)
        builder
            .employee_repo
            .expect_get_working_conditions_at()
            .returning(|_, _, _| Ok(vec![]));

        // Mock get_working_conditions_batch_by_employments - called by
        // AttendanceSummaryService::fetch_range_preamble via
        // PolicyResolverService::resolve_attendance_rules_batch_by_employment.
        builder
            .employee_repo
            .expect_get_working_conditions_batch_by_employments()
            .returning(|_, _, _, _| Ok(std::collections::HashMap::new()));

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
            &EmploymentResolution::ViaDateRange,
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

        let deps = build_deps(emp_id, employment_id, ws_id, &EmploymentResolution::Pinned);
        let job = reprocess_job(ws_id, emp_id, Some(employment_id));

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(result.is_ok(), "Handler failed: {:?}", result.err());
    }

    /// When `employment_id` is `None` and the employee has a terminated
    /// employment whose date range covers the reprocess range, the handler
    /// must resolve it via `list_employments_effective_in_range` and succeed.
    #[tokio::test]
    async fn test_reprocess_resolves_terminated_employment_by_date_range() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        let deps = build_deps(
            emp_id,
            employment_id,
            ws_id,
            &EmploymentResolution::ViaDateRange,
        );
        let job = reprocess_job(ws_id, emp_id, None);

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(
            result.is_ok(),
            "Handler should resolve terminated employment by date range, got: {:?}",
            result.err()
        );
    }

    /// When `employment_id` is `None` and no employment covers the range,
    /// the handler must return an error (not fall back to active_employment_id).
    #[tokio::test]
    async fn test_reprocess_fails_when_no_employment_covers_range() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let _employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        // Build deps where employment_repo returns empty list for date range
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
            .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(()));
        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));
        daily_summary_repo
            .expect_get_summaries()
            .returning(|_, _, _, _| Ok(vec![]));
        daily_summary_repo
            .expect_batch_upsert_calculated_summaries()
            .returning(|_, _, _| Ok(()));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| {
                Box::pin(async {
                    Ok((
                        None,
                        vec![],
                        std::sync::Arc::new(crate::cache::RosterData::default()),
                    ))
                })
            });
        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| Box::pin(async { Ok((vec![], 0)) }));

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
        emp_repo.expect_get_employee_by_id().never();
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

        // Employment repo: return empty for date-range lookup
        let mut employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        employment_repo
            .expect_get_active_employment_on_date()
            .returning(|_, _, _| Ok(None));
        employment_repo
            .expect_resolve_employment_on_date()
            .returning(|_, _, _| Ok(crate::models::employment::EmploymentResolution::NotFound));
        employment_repo
            .expect_list_employments_effective_in_range()
            .returning(|_, _, _, _| Ok(vec![])); // EMPTY
        employment_repo
            .expect_get_employment_by_id()
            .returning(|_, _| Ok(None));
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

        let deps = builder.build();
        let job = reprocess_job(ws_id, emp_id, None);

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(
            result.is_err(),
            "Handler should fail when no employment covers the range"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("No employment covers date range"),
            "Error message should mention missing employment, got: {err_msg}"
        );
    }

    /// When `employment_id` is `None` and multiple employments overlap the
    /// range (rehire scenario), the handler must return an error asking for
    /// explicit employment_id.
    #[tokio::test]
    async fn test_reprocess_fails_when_multiple_employments_overlap_range() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_a = Uuid::new_v4();
        let employment_b = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

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
            .returning(|_, _, _, _, _, _, _, _, _, _, _, _, _| Ok(()));
        daily_summary_repo
            .expect_get_summary()
            .returning(|_, _, _| Ok(None));
        daily_summary_repo
            .expect_get_summaries()
            .returning(|_, _, _, _| Ok(vec![]));
        daily_summary_repo
            .expect_batch_upsert_calculated_summaries()
            .returning(|_, _, _| Ok(()));

        let mut roster_repo = MockRosterRepository::new();
        roster_repo
            .expect_get_effective_shift()
            .returning(|_, _, _| {
                Box::pin(async {
                    Ok((
                        None,
                        vec![],
                        std::sync::Arc::new(crate::cache::RosterData::default()),
                    ))
                })
            });
        roster_repo
            .expect_get_workspace_schedules()
            .returning(move |_, _, _, _, _, _, _| Box::pin(async { Ok((vec![], 0)) }));

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
        emp_repo.expect_get_employee_by_id().never();
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

        // Employment repo: return TWO employments for date-range lookup
        let mut employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        employment_repo
            .expect_get_active_employment_on_date()
            .returning(|_, _, _| Ok(None));
        employment_repo
            .expect_resolve_employment_on_date()
            .returning(|_, _, _| Ok(crate::models::employment::EmploymentResolution::NotFound));
        employment_repo
            .expect_list_employments_effective_in_range()
            .returning(move |_, _, _, _| {
                Ok(vec![
                    crate::models::employment::Employment::new(
                        employment_a,
                        ws_id,
                        emp_id,
                        uuid::Uuid::new_v4(),
                        None,
                        None,
                        "terminated".to_string(),
                        chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                        Some(chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
                        None,
                        "full_time".to_string(),
                        None,
                        chrono::Utc::now(),
                        chrono::Utc::now(),
                        None,
                        None,
                    ),
                    crate::models::employment::Employment::new(
                        employment_b,
                        ws_id,
                        emp_id,
                        uuid::Uuid::new_v4(),
                        None,
                        None,
                        "active".to_string(),
                        chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                        None,
                        None,
                        "full_time".to_string(),
                        None,
                        chrono::Utc::now(),
                        chrono::Utc::now(),
                        None,
                        None,
                    ),
                ])
            });
        employment_repo
            .expect_get_employment_by_id()
            .returning(|_, _| Ok(None));
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

        let deps = builder.build();
        let job = reprocess_job(ws_id, emp_id, None);

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(
            result.is_err(),
            "Handler should fail when multiple employments overlap the range"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("spans 2 employments"),
            "Error message should mention ambiguous employments, got: {err_msg}"
        );
    }

    /// Single-day reprocess (start == end) should succeed and count as 1 day.
    #[tokio::test]
    async fn test_reprocess_single_day_range() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        let deps = build_deps(
            emp_id,
            employment_id,
            ws_id,
            &EmploymentResolution::ViaDateRange,
        );

        let mut job = reprocess_job(ws_id, emp_id, None);
        // Override date range to a single day (start == end)
        if let JobPayload::ReprocessAttendance(ref mut payload) = job.payload {
            let single_day = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
            payload.date_range.start = single_day;
            payload.date_range.end = single_day;
        }

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(
            result.is_ok(),
            "Single-day reprocess should succeed, got: {:?}",
            result.err()
        );
    }

    /// When the job has no reprocess payload, the handler should return an error.
    #[tokio::test]
    async fn test_reprocess_missing_payload_returns_error() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        let deps = build_deps(
            emp_id,
            employment_id,
            ws_id,
            &EmploymentResolution::ViaDateRange,
        );

        // Create a job with the wrong payload type (not ReprocessAttendance)
        let job = Job {
            job_id: Uuid::new_v4(),
            parent_job_id: Uuid::new_v4(),
            batch_index: 0,
            task_type: TaskType::ReprocessAttendance,
            workspace_id: ws_id,
            payload: JobPayload::FinalizeImport(crate::queue::FinalizeImportPayload {
                import_job_id: Uuid::new_v4(),
            }),
            priority: crate::queue::JobPriority::Normal,
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
            trace_context: None,
        };

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(
            result.is_err(),
            "Missing reprocess payload should return error"
        );
        assert!(
            result.unwrap_err().contains("Missing reprocess payload"),
            "Error should mention missing payload"
        );
    }

    /// When pinned employment belongs to a different workspace, handler should error.
    #[tokio::test]
    async fn test_reprocess_pinned_employment_wrong_workspace() {
        let ws_id = Uuid::new_v4();
        let other_ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        let mut queue = MockQueueRepository::new();
        queue.expect_get_job_metadata().returning(|_| Ok(None));

        // Build deps with a different workspace for the employment
        let mut builder = MockWorkerDepsBuilder::new();

        let mut employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        employment_repo
            .expect_get_employment_by_id()
            .returning(move |_, eid| {
                Ok(Some(crate::models::employment::Employment::new(
                    eid,
                    other_ws_id, // Different workspace!
                    emp_id,
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

        let deps = builder.build();
        let job = reprocess_job(ws_id, emp_id, Some(employment_id));

        let queue_arc: Arc<dyn crate::repositories::QueueRepository> = Arc::new(queue);
        let result = ReprocessAttendanceHandler::handle(&queue_arc, &job, &deps).await;

        assert!(
            result.is_err(),
            "Pinned employment with wrong workspace should return error"
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("does not belong to workspace"),
            "Error should mention workspace mismatch, got: {err_msg}"
        );
    }
}
