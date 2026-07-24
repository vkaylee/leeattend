use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;
use std::sync::Arc;

use chrono::{NaiveDate, TimeZone};
use chrono_tz::Tz;

use uuid::Uuid;

use crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;

use crate::diesel_pool::TenantAuthToken;
use crate::domain::policy::defaults::DEFAULT_WORK_MINUTES_PER_DAY;

use crate::domain::policy::resolver::ShiftPolicyOverrides;
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::AttendancePolicyRepository;
use crate::repositories::EmployeeRepository;
use crate::repositories::{workspace_repository::WorkspaceRepository, TimeEntryRepository};
use crate::services::AttendanceCalculator;
use crate::services::PolicyResolverService;
use crate::utils::chrono_helpers::local_to_utc as local_midnight_to_utc;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SummaryRecalculator: Send + Sync {
    async fn recalculate_daily_summary(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError>;

    async fn recalculate_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError>;
}

#[async_trait::async_trait]
impl SummaryRecalculator for AttendanceSummaryService {
    async fn recalculate_daily_summary(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        self.recalculate_daily_summary(workspace_id, employment_id, date)
            .await
    }

    async fn recalculate_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError> {
        self.recalculate_attendance_range(workspace_id, employment_id, start_date, end_date)
            .await
    }
}

pub struct EnterPaperOtArgs {
    pub workspace_id: Uuid,
    pub employment_id: Uuid,
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
    pub condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub employee_repo: Arc<dyn EmployeeRepository>,
    pub employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
    pub leave_request_repo: Arc<dyn LeaveRequestRepository>,
    pub daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
}

pub struct AttendanceSummaryService {
    time_entry_repo: Arc<dyn TimeEntryRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    policy_repo: Arc<dyn AttendancePolicyRepository>,
    condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    employee_repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
    leave_request_repo: Arc<dyn LeaveRequestRepository>,
    daily_summary_repo: Arc<dyn AttendanceDailySummaryRepository>,
}

struct CalculateOtBucketsParams<'a> {
    tz: &'a Tz,
    date: NaiveDate,
    employee_id: Uuid,
    workspace_id: Uuid,
    daily_schedule: &'a crate::models::EffectiveSchedule,
    punch_in: chrono::DateTime<Tz>,
    punch_out: chrono::DateTime<Tz>,
    actual_break_minutes: i32,
}

impl AttendanceSummaryService {
    pub fn new(args: AttendanceSummaryServiceArgs) -> Self {
        Self {
            time_entry_repo: args.time_entry_repo,
            roster_repo: args.roster_repo,
            policy_repo: args.policy_repo,
            condition_type_repo: args.condition_type_repo,
            workspace_repo: args.workspace_repo,
            employee_repo: args.employee_repo,
            employment_repo: args.employment_repo,
            holiday_repo: args.holiday_repo,
            leave_request_repo: args.leave_request_repo,
            daily_summary_repo: args.daily_summary_repo,
        }
    }

    /// Recalculates and persists the daily summary for a specific employee and date.
    /// This should be called whenever time entries or schedules change.
    #[tracing::instrument(skip_all, err)]
    pub async fn recalculate_daily_summary(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<()> {
        let employment = self
            .employment_repo
            .get_employment_by_id(
                &TenantAuthToken::system_override(workspace_id),
                employment_id,
            )
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();

        // 1. Fetch schedule & time entries
        let (tz, daily_schedule, day_entries) = self
            .fetch_daily_schedule(workspace_id, employment_id, date)
            .await?;

        // Auto-cancel approved leave requests if the employee worked (day_entries is not empty) and it's not a holiday
        if !daily_schedule.is_holiday() && !day_entries.is_empty() {
            let leaves = self
                .leave_request_repo
                .get_employee_leave_requests(
                    &TenantAuthToken::system_override(workspace_id),
                    employee_id,
                )
                .await?;

            let leave_to_cancel = leaves.into_iter().find(|l| {
                (l.status() == crate::models::enums::LeaveStatus::Approved
                    || l.status() == crate::models::enums::LeaveStatus::PreApproved)
                    && l.start_date() <= date
                    && l.end_date() >= date
            });

            if let Some(req) = leave_to_cancel {
                // Cancel the leave request (which updates status and refunds balance)
                self.leave_request_repo
                    .cancel_leave_request(&TenantAuthToken::system_override(workspace_id), req.id())
                    .await?;
            }
        }

        // 2. Calculate work minutes and OT
        let (actual_work_minutes, system_detected_ot_minutes) = self
            .calculate_work_and_ot(
                &tz,
                date,
                employee_id,
                workspace_id,
                &daily_schedule,
                &day_entries,
            )
            .await?;

        // 3. Holiday leave conflict detection
        let (is_holiday, holiday_name) = self
            .handle_holiday_leave_conflict(
                workspace_id,
                employee_id,
                date,
                actual_work_minutes,
                &daily_schedule,
            )
            .await?;

        // 4. Determine OT status
        let new_status = self
            .determine_ot_status(workspace_id, employee_id, date, system_detected_ot_minutes)
            .await?;

        // 5. Persist
        self.daily_summary_repo
            .upsert_calculated_summary(
                &TenantAuthToken::system_override(workspace_id),
                employment_id,
                date,
                actual_work_minutes,
                system_detected_ot_minutes,
                new_status,
                is_holiday,
                holiday_name,
            )
            .await?;
        Ok(())
    }

    /// Recalculates and persists the daily summaries for a specific employee and date range in batch.
    /// Eliminates N+1 query loops.
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn recalculate_attendance_range(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<()> {
        let auth_token = TenantAuthToken::system_override(workspace_id);

        let employment = self
            .employment_repo
            .get_employment_by_id(&auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();
        let location_id = employment.primary_location_id();

        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound))?;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        // 1. Fetch effective schedules in range
        let (schedules, _) = self
            .roster_repo
            .get_workspace_schedules(
                None,
                &auth_token,
                start_date,
                end_date,
                Some(&vec![employment_id]),
                None,
                None,
            )
            .await?;

        let emp_sched = schedules.first().ok_or_else(|| {
            AppError::not_found_args(
                I18nKey::NotFound,
                vec![("details", "no_schedule_found_for_employee".to_string())],
            )
        })?;

        // 2. Fetch time entries in range
        let start_dt_utc =
            local_midnight_to_utc(&tz, start_date, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt_utc =
            local_midnight_to_utc(&tz, end_date, 23, 59, 59).map_err(AppError::Internal)?;

        let all_entries = self
            .time_entry_repo
            .get_employment_time_entries(&auth_token, employment_id, start_dt_utc, end_dt_utc)
            .await?;

        // 3. Fetch leaves in range (employee leaves)
        let leaves = self
            .leave_request_repo
            .get_employee_leave_requests(&auth_token, employee_id)
            .await?;

        // 4. Fetch holidays in range
        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(&auth_token, start_date, end_date)
            .await?;

        // 5. Fetch existing summaries in range to determine OT status
        let existing_summaries = self
            .daily_summary_repo
            .get_summaries(&auth_token, Some(employment_id), start_date, end_date)
            .await?;

        // 6. Iterate day-by-day in range
        let mut curr_date = start_date;
        while curr_date <= end_date {
            let daily_schedule = emp_sched.schedules().iter().find(|s| s.date() == curr_date);

            if let Some(daily_schedule) = daily_schedule {
                // Filter time entries for this date in local time
                let day_entries: Vec<_> = all_entries
                    .iter()
                    .filter(|e| e.punch_in_time().with_timezone(&tz).date_naive() == curr_date)
                    .cloned()
                    .collect();

                // 6a. Auto-cancel approved leaves if employee worked on a non-holiday
                if !daily_schedule.is_holiday() && !day_entries.is_empty() {
                    let leave_to_cancel = leaves.iter().find(|l| {
                        (l.status() == crate::models::enums::LeaveStatus::Approved
                            || l.status() == crate::models::enums::LeaveStatus::PreApproved)
                            && l.start_date() <= curr_date
                            && l.end_date() >= curr_date
                    });

                    if let Some(req) = leave_to_cancel {
                        self.leave_request_repo
                            .cancel_leave_request(&auth_token, req.id())
                            .await?;
                    }
                }

                // 6b. Calculate work minutes and OT
                let (actual_work_minutes, system_detected_ot_minutes) = self
                    .calculate_work_and_ot(
                        &tz,
                        curr_date,
                        employee_id,
                        workspace_id,
                        daily_schedule,
                        &day_entries,
                    )
                    .await?;

                // 6c. Holiday leave conflict check
                let holiday = holidays_map.get(&curr_date);
                let filtered_holiday = holiday
                    .filter(|&hol| hol.location_id().is_none() || hol.location_id() == location_id);

                let is_holiday = filtered_holiday.is_some();
                let holiday_name = filtered_holiday.map(|h| h.name().to_string());

                if is_holiday && actual_work_minutes > 0 {
                    if let Some(hol) = filtered_holiday {
                        if let Err(e) = self
                            .maybe_reverse_holiday_leave(
                                workspace_id,
                                employee_id,
                                curr_date,
                                actual_work_minutes,
                                daily_schedule,
                                hol,
                            )
                            .await
                        {
                            // CRITICAL: If holiday leave reversal fails, the employee
                            // is charged leave AND counted as working (double penalty).
                            // Surface this as an error so operators can investigate.
                            tracing::error!(
                                error = %e,
                                employee_id = %employee_id,
                                date = %curr_date,
                                holiday = %hol.name(),
                                "CRITICAL: Failed to reverse holiday leave — employee may be double-charged"
                            );
                        }
                    }
                }

                // 6d. Determine OT status
                let existing = existing_summaries.iter().find(|s| s.date == curr_date);
                let new_status = match existing {
                    Some(s) => {
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

                // 6e. Persist summary
                self.daily_summary_repo
                    .upsert_calculated_summary(
                        &auth_token,
                        employment_id,
                        curr_date,
                        actual_work_minutes,
                        system_detected_ot_minutes,
                        new_status,
                        is_holiday,
                        holiday_name,
                    )
                    .await?;
            }

            curr_date = curr_date.succ_opt().unwrap_or(curr_date);
        }

        Ok(())
    }

    /// Fetch workspace timezone, the employee's schedule for the date, and time entries.
    async fn fetch_daily_schedule(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<(
        Tz,
        crate::models::EffectiveSchedule,
        Vec<crate::models::TimeEntry>,
    )> {
        let workspace = self
            .workspace_repo
            .get_workspace_by_id(workspace_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::WorkspaceNotFound))?;

        let tz: Tz = crate::utils::tz::parse_workspace_tz(workspace.timezone());

        let (schedules, _) = self
            .roster_repo
            .get_workspace_schedules(
                None,
                &TenantAuthToken::system_override(workspace_id),
                date,
                date,
                Some(&vec![employment_id]),
                None,
                None,
            )
            .await?;

        let emp_sched = schedules.first().ok_or_else(|| {
            AppError::not_found_args(
                I18nKey::NotFound,
                vec![("details", "no_schedule_found_for_employee".to_string())],
            )
        })?;
        let daily_schedule = emp_sched
            .schedules()
            .iter()
            .find(|s| s.date() == date)
            .ok_or_else(|| {
                AppError::not_found_args(
                    I18nKey::NotFound,
                    vec![("details", "no_schedule_for_date".to_string())],
                )
            })?
            .clone();

        let start_dt_utc = local_midnight_to_utc(&tz, date, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt_utc =
            local_midnight_to_utc(&tz, date, 23, 59, 59).map_err(AppError::Internal)?;

        let entries = self
            .time_entry_repo
            .get_employment_time_entries(
                &TenantAuthToken::system_override(workspace_id),
                employment_id,
                start_dt_utc,
                end_dt_utc,
            )
            .await?;

        let day_entries: Vec<_> = entries
            .into_iter()
            .filter(|e| e.punch_in_time().with_timezone(&tz).date_naive() == date)
            .collect();

        Ok((tz, daily_schedule, day_entries))
    }

    /// Calculate actual work minutes and system-detected OT from time entries.
    async fn calculate_work_and_ot(
        &self,
        tz: &Tz,
        date: NaiveDate,
        employee_id: Uuid,
        workspace_id: Uuid,
        daily_schedule: &crate::models::EffectiveSchedule,
        day_entries: &[crate::models::TimeEntry],
    ) -> AppResult<(i32, i32)> {
        let actual_punch_in = day_entries
            .first()
            .map(|e| e.punch_in_time().with_timezone(tz));
        let actual_punch_out = day_entries
            .last()
            .and_then(|e| e.punch_out_time().map(|t| t.with_timezone(tz)));

        let (Some(punch_in), Some(punch_out)) = (actual_punch_in, actual_punch_out) else {
            return Ok((0, 0));
        };

        let actual_break_minutes: i32 = day_entries
            .iter()
            .filter(|e| e.entry_type() == "break")
            .map(|e| e.duration_minutes().unwrap_or(0))
            .sum();

        let raw_duration =
            i32::try_from(punch_out.signed_duration_since(punch_in).num_minutes()).unwrap_or(0);
        let actual_work_minutes = i32::max(0, raw_duration - actual_break_minutes);

        let system_detected_ot_minutes = self
            .calculate_ot_buckets(CalculateOtBucketsParams {
                tz,
                date,
                employee_id,
                workspace_id,
                daily_schedule,
                punch_in,
                punch_out,
                actual_break_minutes,
            })
            .await?;

        Ok((actual_work_minutes, system_detected_ot_minutes))
    }

    /// Internal: compute OT buckets from schedule + punches.
    async fn calculate_ot_buckets(&self, params: CalculateOtBucketsParams<'_>) -> AppResult<i32> {
        let CalculateOtBucketsParams {
            tz,
            date,
            employee_id,
            workspace_id,
            daily_schedule,
            punch_in,
            punch_out,
            actual_break_minutes,
        } = params;
        let (Some(sched_start_time), Some(sched_end_time)) =
            (daily_schedule.start_time(), daily_schedule.end_time())
        else {
            return Ok(0);
        };

        let start_dt = tz
            .from_local_datetime(&date.and_time(sched_start_time))
            .single();
        let end_dt = tz
            .from_local_datetime(&date.and_time(sched_end_time))
            .single();

        let (Some(start), Some(mut end)) = (start_dt, end_dt) else {
            return Ok(0);
        };

        if sched_end_time < sched_start_time {
            end += chrono::Duration::days(1);
        }

        let rules = PolicyResolverService::resolve_attendance_rules(
            &*self.policy_repo,
            &*self.employee_repo,
            &*self.condition_type_repo,
            employee_id,
            &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            date,
        )
        .await?;

        let policy_rules = PolicyResolverService::resolve_with_shift_overrides(
            &*self.policy_repo,
            &*self.employee_repo,
            &*self.condition_type_repo,
            employee_id,
            &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
            date,
            &ShiftPolicyOverrides::new(
                daily_schedule.early_punch_in_grace_period_minutes(),
                daily_schedule.is_break_auto_deduct(),
                daily_schedule.allow_early_ot(),
                rules.early_arrival_strategy(),
                daily_schedule.break_is_overtime().unwrap_or(false),
            ),
        )
        .await?;

        let break_start_dt = daily_schedule
            .break_start()
            .and_then(|t| tz.from_local_datetime(&date.and_time(t)).single());
        let break_end_dt = daily_schedule.break_end().and_then(|t| {
            let mut dt = tz.from_local_datetime(&date.and_time(t)).single()?;
            if daily_schedule.break_end() < daily_schedule.break_start() {
                dt += chrono::Duration::days(1);
            }
            Some(dt)
        });

        let buckets = AttendanceCalculator::calculate_buckets(
            &crate::services::attendance_calculator::DailyCalcInput {
                punch_in,
                punch_out,
                shift_start: Some(start),
                shift_end: Some(end),
                scheduled_break_minutes: daily_schedule.break_duration_minutes().unwrap_or(0),
                actual_break_minutes,
                break_start: break_start_dt,
                break_end: break_end_dt,
                work_unit: 0,
                rounding_policy: String::new(),
                rules: policy_rules,
                ot_ceiling: None,
                max_early_ot_minutes: daily_schedule.max_early_ot_minutes(),
                segment_id: None,
                ot_aggregation_mode:
                    crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            },
        );

        Ok(buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes)
    }

    /// Check if the date is a holiday and handle leave conflict (reversal if employee worked enough).
    async fn handle_holiday_leave_conflict(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
    ) -> AppResult<(bool, Option<String>)> {
        let active_employment = self
            .employment_repo
            .get_active_employment_on_date(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await?;
        let location_id = active_employment.and_then(|e| e.primary_location_id());

        let holiday = self
            .holiday_repo
            .is_holiday(
                &TenantAuthToken::system_override(workspace_id),
                date,
                location_id,
            )
            .await?;
        let is_holiday = holiday.is_some();
        let holiday_name = holiday.as_ref().map(|h| h.name().to_string());

        if is_holiday && actual_work_minutes > 0 {
            if let Some(ref hol) = holiday {
                if let Err(e) = self
                    .maybe_reverse_holiday_leave(
                        workspace_id,
                        employee_id,
                        date,
                        actual_work_minutes,
                        daily_schedule,
                        hol,
                    )
                    .await
                {
                    tracing::error!(
                        error = %e,
                        employee_id = %employee_id,
                        date = %date,
                        "CRITICAL: Failed to reverse holiday leave — employee may be double-charged"
                    );
                }
            }
        }

        Ok((is_holiday, holiday_name))
    }

    /// Reverse system-generated holiday leave if employee worked above the threshold.
    async fn maybe_reverse_holiday_leave(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
        holiday: &crate::models::Holiday,
    ) -> AppResult<()> {
        let system_leave = self
            .leave_request_repo
            .get_system_generated_holiday_leave(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await?;

        let Some((leave_request_id, _leave_type_id, _leave_code)) = system_leave else {
            return Ok(());
        };

        let shift_minutes = daily_schedule
            .start_time()
            .and_then(|st| {
                daily_schedule.end_time().map(|et: chrono::NaiveTime| {
                    let diff =
                        i32::try_from(et.signed_duration_since(st).num_minutes()).unwrap_or(0);
                    let break_mins = daily_schedule.break_duration_minutes().unwrap_or(0);
                    if diff > 0 {
                        diff - break_mins
                    } else {
                        diff + 1440 - break_mins
                    }
                })
            })
            .unwrap_or(DEFAULT_WORK_MINUTES_PER_DAY);

        let threshold = holiday.work_threshold_percent().unwrap_or(50);
        let work_percent = if shift_minutes > 0 {
            (actual_work_minutes * 100) / shift_minutes
        } else {
            100
        };

        if work_percent >= threshold {
            self.leave_request_repo
                .cancel_leave_request(
                    &TenantAuthToken::system_override(workspace_id),
                    leave_request_id,
                )
                .await?;

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
        Ok(())
    }

    /// Determine the OT status based on existing summary and detected OT.
    async fn determine_ot_status(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
        system_detected_ot_minutes: i32,
    ) -> AppResult<OtStatus> {
        let existing = self
            .daily_summary_repo
            .get_summary(
                &TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await?;

        let status = match existing {
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

        Ok(status)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn claim_ot(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> AppResult<()> {
        let summary_repo = self.daily_summary_repo.as_ref();

        let rows_affected = summary_repo
            .claim_ot(
                &TenantAuthToken::system_override(workspace_id),
                employment_id,
                date,
                claimed_minutes,
                reason,
            )
            .await?;

        if rows_affected == 0 {
            return Err(AppError::not_found(I18nKey::NotFound));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_ot(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus, // Approved or Rejected
    ) -> AppResult<()> {
        let summary_repo = self.daily_summary_repo.as_ref();

        let rows_affected = summary_repo
            .approve_ot(
                &TenantAuthToken::system_override(workspace_id),
                employment_id,
                date,
                approved_minutes,
                manager_note,
                status,
            )
            .await?;

        if rows_affected == 0 {
            return Err(AppError::not_found(I18nKey::NotFound));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn enter_paper_ot(
        &self,
        args: EnterPaperOtArgs,
    ) -> AppResult<AttendanceDailySummary> {
        let summary_repo = self.daily_summary_repo.as_ref();

        // 1. Check existing summary
        let existing = summary_repo
            .get_summary(
                &TenantAuthToken::system_override(args.workspace_id),
                args.employment_id,
                args.date,
            )
            .await?;

        // 2. Validate against system detected (if not allowing exceed)
        if !args.allow_exceed {
            if let Some(ref ex) = existing {
                if args.claimed_minutes > ex.system_detected_ot_minutes {
                    return Err(AppError::validation_args(
                        I18nKey::Validation,
                        vec![
                            (
                                "details",
                                "claimed_minutes_exceed_system_detected".to_string(),
                            ),
                            ("claimed", args.claimed_minutes.to_string()),
                            ("detected", ex.system_detected_ot_minutes.to_string()),
                        ],
                    ));
                }
            }
        }

        // 3. Upsert with paper form metadata
        let result = summary_repo
            .upsert_paper_ot(
                &TenantAuthToken::system_override(args.workspace_id),
                args.employment_id,
                args.date,
                args.claimed_minutes,
                args.reason,
                args.entered_by_user_id,
                args.photo_url,
            )
            .await?;

        Ok(result)
    }
}

#[async_trait::async_trait]
impl crate::services::approval_workflow::WorkflowCallbackHandler for AttendanceSummaryService {
    async fn on_workflow_completed(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        entity_id: Uuid,
        status: &str,
        _user_id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();

        let summary = self
            .daily_summary_repo
            .get_summary_by_id(auth_token, entity_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::AttendanceSummaryNotFound))?;

        let ot_status = match status {
            "approved" => crate::models::OtStatus::Approved,
            "rejected" => crate::models::OtStatus::Rejected,
            _ => crate::models::OtStatus::PendingManager,
        };

        self.approve_ot(
            workspace_id,
            summary.employment_id,
            summary.date,
            summary.claimed_ot_minutes,
            Some("Approved via workflow".to_string()),
            ot_status,
        )
        .await
    }
}

#[cfg(test)]
#[path = "attendance_summary_service_tests.rs"]
mod tests;
