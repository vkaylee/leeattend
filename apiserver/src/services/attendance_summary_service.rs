use std::sync::Arc;

use chrono::{NaiveDate, TimeZone};
use chrono_tz::Tz;

use uuid::Uuid;

use crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;

use crate::domain::policy::defaults::DEFAULT_WORK_MINUTES_PER_DAY;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::AttendancePolicyRepository;
use crate::repositories::EmployeeRepository;
use crate::repositories::{workspace_repository::WorkspaceRepository, TimeEntryRepository};
use crate::services::AttendanceCalculator;

pub struct EnterPaperOtArgs {
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String,
    pub entered_by_user_id: Uuid,
    pub photo_url: Option<String>,
    pub allow_exceed: bool,
}

pub struct AttendanceSummaryServiceArgs {
    pub time_entry_repo: Arc<dyn TimeEntryRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub policy_repo: Arc<dyn AttendancePolicyRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
    pub leave_request_repo: Arc<dyn LeaveRequestRepository>,
    pub daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
}

pub struct AttendanceSummaryService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    policy_repo: Arc<dyn AttendancePolicyRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
}

impl AttendanceSummaryService {
    pub fn new(args: AttendanceSummaryServiceArgs) -> Self {
        Self {
            time_entry_repo: args.time_entry_repo,
            roster_repo: args.roster_repo,
            policy_repo: args.policy_repo,
            workspace_repo: args.workspace_repo,
            employee_repo: args.employee_repo,
            holiday_repo: args.holiday_repo,
            leave_request_repo: args.leave_request_repo,
            daily_summary_repo: args.daily_summary_repo,
        }
    }

    /// Recalculates and persists the daily summary for a specific employee and date.
    /// This should be called whenever time entries or schedules change.
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn recalculate_daily_summary(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), String> {
        // 1. Get Workspace Settings
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("Workspace not found")?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(&workspace.timezone);

        // 2. Get Effective Schedule for this specific date
        let schedules = self
            .roster_repo
            .get_workspace_schedules(None, workspace_id, date, date, Some(&vec![employee_id]))
            .await
            .map_err(|e| e.to_string())?;

        let emp_sched = schedules.first().ok_or("No schedule found for employee")?;
        let daily_schedule = emp_sched
            .schedules
            .iter()
            .find(|s| s.date == date)
            .ok_or("No schedule for date")?;

        // 3. Get Time Entries for the date (in UTC range)
        let start_dt_utc = local_midnight_to_utc(&tz, date, 0, 0, 0)?;
        let end_dt_utc = local_midnight_to_utc(&tz, date, 23, 59, 59)?;

        let entries = self
            .time_entry_repo
            .get_employee_time_entries(workspace_id, employee_id, start_dt_utc, end_dt_utc)
            .await
            .map_err(|e| e.to_string())?;

        // Filter entries that actually belong to this date in local time
        let day_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.punch_in_time.with_timezone(&tz).date_naive() == date)
            .collect();

        // 4. Calculate Stats
        let mut actual_work_minutes = 0;
        let mut system_detected_ot_minutes = 0;

        let actual_punch_in = day_entries
            .first()
            .map(|e| e.punch_in_time.with_timezone(&tz));
        let actual_punch_out = day_entries
            .last()
            .and_then(|e| e.punch_out_time.map(|t| t.with_timezone(&tz)));

        if let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) {
            let actual_break_minutes: i32 = day_entries
                .iter()
                .filter(|e| e.entry_type == "break")
                .map(|e| e.duration_minutes.unwrap_or(0))
                .sum();

            // Calculate raw duration for "actual_work_minutes"
            let raw_duration = punch_out.signed_duration_since(punch_in).num_minutes() as i32;
            actual_work_minutes = i32::max(0, raw_duration - actual_break_minutes);

            // Calculate OT
            if let (Some(sched_start_time), Some(sched_end_time)) =
                (daily_schedule.start_time, daily_schedule.end_time)
            {
                let start_dt = tz
                    .from_local_datetime(&date.and_time(sched_start_time))
                    .single();
                let end_dt = tz
                    .from_local_datetime(&date.and_time(sched_end_time))
                    .single();

                if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                    if sched_end_time < sched_start_time {
                        end += chrono::Duration::days(1);
                    }

                    let rules =
                        crate::domain::policy::resolver::PolicyResolver::resolve_attendance_rules(
                            &*self.policy_repo,
                            &*self.employee_repo,
                            employee_id,
                            workspace_id,
                            date,
                        )
                        .await
                        .map_err(|e| e.to_string())?;

                    // Resolve Policy Rules
                    let policy_rules = PolicyResolver::resolve_with_shift_overrides(
                        &*self.policy_repo,
                        &*self.employee_repo,
                        employee_id,
                        workspace_id,
                        date,
                        &crate::domain::policy::resolver::ShiftPolicyOverrides {
                            grace_period_minutes: daily_schedule
                                .early_punch_in_grace_period_minutes,
                            is_break_auto_deduct: daily_schedule.is_break_auto_deduct,
                            allow_early_ot: daily_schedule.allow_early_ot,
                            early_arrival_strategy: rules.early_arrival_strategy,
                            is_break_overtime: false,
                        },
                    )
                    .await;

                    let buckets = AttendanceCalculator::calculate_buckets(
                        &crate::services::attendance_calculator::DailyCalcInput {
                            punch_in,
                            punch_out,
                            shift_start: Some(start),
                            shift_end: Some(end),
                            scheduled_break_minutes: daily_schedule
                                .break_duration_minutes
                                .unwrap_or(0),
                            actual_break_minutes,
                            work_unit: 0,
                            rounding_policy: String::new(),
                            rules: policy_rules,
                            ot_ceiling: None,
                        },
                    );

                    // System Detected OT = Early OT + Late OT
                    system_detected_ot_minutes = buckets.early_ot_minutes + buckets.late_ot_minutes;
                }
            }
        }

        // 4b. Check if this date is a holiday
        let holiday = self
            .holiday_repo
            .is_holiday(workspace_id, date)
            .await
            .unwrap_or(None);
        let is_holiday = holiday.is_some();
        let holiday_name = holiday.as_ref().map(|h| h.name.clone());

        // 4c. Holiday leave conflict detection
        if is_holiday && actual_work_minutes > 0 {
            if let Some(ref hol) = holiday {
                let system_leave = self
                    .leave_request_repo
                    .get_system_generated_holiday_leave(workspace_id, employee_id, date)
                    .await
                    .unwrap_or(None);

                if let Some((leave_request_id, _leave_type_id, _leave_code)) = system_leave {
                    let shift_minutes = daily_schedule
                        .start_time
                        .and_then(|st| {
                            daily_schedule.end_time.map(|et| {
                                let diff = et.signed_duration_since(st).num_minutes() as i32;
                                let break_mins = daily_schedule.break_duration_minutes.unwrap_or(0);
                                if diff > 0 {
                                    diff - break_mins
                                } else {
                                    diff + 1440 - break_mins
                                }
                            })
                        })
                        .unwrap_or(DEFAULT_WORK_MINUTES_PER_DAY);

                    let threshold = hol.work_threshold_percent.unwrap_or(50);
                    let work_percent = if shift_minutes > 0 {
                        (actual_work_minutes * 100) / shift_minutes
                    } else {
                        100
                    };

                    if work_percent >= threshold {
                        let _ = self
                            .leave_request_repo
                            .cancel_leave_request(workspace_id, leave_request_id)
                            .await;

                        tracing::info!(
                            employee_id = %employee_id,
                            date = %date,
                            work_percent = work_percent,
                            threshold = threshold,
                            "Holiday leave reversed: employee worked {}% (>= {}% threshold)",
                            work_percent, threshold
                        );
                    } else {
                        tracing::info!(
                            employee_id = %employee_id,
                            date = %date,
                            work_percent = work_percent,
                            threshold = threshold,
                            "Holiday leave kept: employee worked {}% (< {}% threshold)",
                            work_percent, threshold
                        );
                    }
                }
            }
        }

        let summary_repo = self.daily_summary_repo.as_ref();

        let existing = summary_repo
            .get_summary(workspace_id, employee_id, date)
            .await
            .map_err(|e| e.to_string())?;

        let new_status = match existing {
            Some(ref s) => {
                if s.ot_status == OtStatus::None && system_detected_ot_minutes > 0 {
                    OtStatus::PendingEmployee
                } else {
                    s.ot_status
                }
            }
            None => {
                if system_detected_ot_minutes > 0 {
                    OtStatus::PendingEmployee
                } else {
                    OtStatus::None
                }
            }
        };

        summary_repo
            .upsert_calculated_summary(
                workspace_id,
                employee_id,
                date,
                actual_work_minutes,
                system_detected_ot_minutes,
                new_status,
                is_holiday,
                holiday_name,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn claim_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> Result<(), String> {
        let summary_repo = self.daily_summary_repo.as_ref();

        let rows_affected = summary_repo
            .claim_ot(workspace_id, employee_id, date, claimed_minutes, reason)
            .await
            .map_err(|e| e.to_string())?;

        if rows_affected == 0 {
            return Err("Record not found".to_string());
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_ot(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus, // Approved or Rejected
    ) -> Result<(), String> {
        let summary_repo = self.daily_summary_repo.as_ref();

        let rows_affected = summary_repo
            .approve_ot(
                workspace_id,
                employee_id,
                date,
                approved_minutes,
                manager_note,
                status,
            )
            .await
            .map_err(|e| e.to_string())?;

        if rows_affected == 0 {
            return Err("Record not found".to_string());
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn enter_paper_ot(
        &self,
        args: EnterPaperOtArgs,
    ) -> Result<AttendanceDailySummary, String> {
        let summary_repo = self.daily_summary_repo.as_ref();

        // 1. Check existing summary
        let existing = summary_repo
            .get_summary(args.workspace_id, args.employee_id, args.date)
            .await
            .map_err(|e| e.to_string())?;

        // 2. Validate against system detected (if not allowing exceed)
        if !args.allow_exceed {
            if let Some(ref ex) = existing {
                if args.claimed_minutes > ex.system_detected_ot_minutes {
                    return Err(format!(
                        "Claimed {} minutes exceeds system detected {} minutes",
                        args.claimed_minutes, ex.system_detected_ot_minutes
                    ));
                }
            }
        }

        // 3. Upsert with paper form metadata
        let result = summary_repo
            .upsert_paper_ot(
                args.workspace_id,
                args.employee_id,
                args.date,
                args.claimed_minutes,
                args.reason,
                args.entered_by_user_id,
                args.photo_url,
            )
            .await
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}

use crate::utils::chrono_helpers::local_to_utc as local_midnight_to_utc;

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use crate::errors::AppError;

    use crate::repositories::time_entry_repository::MockTimeEntryRepository;
    use crate::repositories::roster::MockRosterRepository;
    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::workspace_repository::MockWorkspaceRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::holiday_repository::MockHolidayRepository;
    use crate::repositories::leave_request_repository::MockLeaveRequestRepository;

    // --- Mock Daily Summary Repo (manual since it needs custom state for these tests) ---
    struct MockDailySummaryRepo {
        claim_called: Mutex<bool>,
        approve_called: Mutex<bool>,
        paper_called: Mutex<bool>,
        should_fail_exceed: bool,
    }

    fn create_dummy_summary(sys_ot: i32) -> AttendanceDailySummary {
        AttendanceDailySummary {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            actual_work_minutes: 0,
            system_detected_ot_minutes: sys_ot,
            claimed_ot_minutes: 0,
            approved_ot_minutes: 0,
            ot_status: OtStatus::None,
            ot_reason: None,
            manager_note: None,
            claim_entered_by_user_id: None,
            claim_entered_at: None,
            paper_form_photo_url: None,
            entered_via: None,
            is_holiday: false,
            holiday_name: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[async_trait]
    impl AttendanceDailySummaryRepository for MockDailySummaryRepo {
        async fn get_summary(&self, _w: Uuid, _e: Uuid, _d: NaiveDate) -> Result<Option<AttendanceDailySummary>, AppError> {
            if self.should_fail_exceed {
                Ok(Some(create_dummy_summary(60)))
            } else {
                Ok(None)
            }
        }
        async fn upsert_calculated_summary(&self, _w: Uuid, _e: Uuid, _d: NaiveDate, _a: i32, _s: i32, _n: OtStatus, _h: bool, _hn: Option<String>) -> Result<(), AppError> { unimplemented!() }
        async fn claim_ot(&self, _w: Uuid, _e: Uuid, _d: NaiveDate, _c: i32, _r: Option<String>) -> Result<usize, AppError> {
            *self.claim_called.lock().unwrap() = true;
            Ok(1)
        }
        async fn approve_ot(&self, _w: Uuid, _e: Uuid, _d: NaiveDate, _a: i32, _m: Option<String>, _s: OtStatus) -> Result<usize, AppError> {
            *self.approve_called.lock().unwrap() = true;
            Ok(1)
        }
        async fn upsert_paper_ot(&self, _w: Uuid, _e: Uuid, _d: NaiveDate, _c: i32, _r: String, _user: Uuid, _p: Option<String>) -> Result<AttendanceDailySummary, AppError> {
            *self.paper_called.lock().unwrap() = true;
            Ok(create_dummy_summary(0))
        }
        async fn get_batch_ot_statuses(&self, _w: Uuid, _e: &[Uuid], _s: NaiveDate, _en: NaiveDate) -> Result<std::collections::HashMap<(Uuid, NaiveDate), String>, AppError> { unimplemented!() }
        async fn get_summaries(&self, _w: Uuid, _e: Option<Uuid>, _s: NaiveDate, _en: NaiveDate) -> Result<Vec<AttendanceDailySummary>, AppError> { unimplemented!() }
    }

    fn make_service(mock_repo: Arc<MockDailySummaryRepo>) -> AttendanceSummaryService {
        AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
            roster_repo: Arc::new(MockRosterRepository::new()),
            policy_repo: Arc::new(MockAttendancePolicyRepository::new()),
            workspace_repo: Arc::new(MockWorkspaceRepository::new()),
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            holiday_repo: Arc::new(MockHolidayRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            daily_summary_repo: mock_repo,
        })
    }

    #[tokio::test]
    async fn test_claim_ot() {
        let repo = Arc::new(MockDailySummaryRepo {
            claim_called: Mutex::new(false),
            approve_called: Mutex::new(false),
            paper_called: Mutex::new(false),
            should_fail_exceed: false,
        });
        let svc = make_service(Arc::clone(&repo));

        svc.claim_ot(Uuid::new_v4(), Uuid::new_v4(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 60, None).await.unwrap();
        assert!(*repo.claim_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_approve_ot() {
        let repo = Arc::new(MockDailySummaryRepo {
            claim_called: Mutex::new(false),
            approve_called: Mutex::new(false),
            paper_called: Mutex::new(false),
            should_fail_exceed: false,
        });
        let svc = make_service(Arc::clone(&repo));

        svc.approve_ot(Uuid::new_v4(), Uuid::new_v4(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 60, None, OtStatus::Approved).await.unwrap();
        assert!(*repo.approve_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_enter_paper_ot() {
        let repo = Arc::new(MockDailySummaryRepo {
            claim_called: Mutex::new(false),
            approve_called: Mutex::new(false),
            paper_called: Mutex::new(false),
            should_fail_exceed: false,
        });
        let svc = make_service(Arc::clone(&repo));

        let args = EnterPaperOtArgs {
            workspace_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            claimed_minutes: 60,
            reason: "OT".to_string(),
            entered_by_user_id: Uuid::new_v4(),
            photo_url: None,
            allow_exceed: true,
        };

        svc.enter_paper_ot(args).await.unwrap();
        assert!(*repo.paper_called.lock().unwrap());
    }

    #[tokio::test]
    async fn test_enter_paper_ot_exceed_fails() {
        let repo = Arc::new(MockDailySummaryRepo {
            claim_called: Mutex::new(false),
            approve_called: Mutex::new(false),
            paper_called: Mutex::new(false),
            should_fail_exceed: true,
        });
        let svc = make_service(Arc::clone(&repo));

        let args = EnterPaperOtArgs {
            workspace_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            claimed_minutes: 120, // greater than 60
            reason: "OT".to_string(),
            entered_by_user_id: Uuid::new_v4(),
            photo_url: None,
            allow_exceed: false,
        };

        let result = svc.enter_paper_ot(args).await;
        assert!(result.is_err());
        assert!(!*repo.paper_called.lock().unwrap());
    }
}
