use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;

use uuid::Uuid;

use crate::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;

use crate::diesel_pool::TenantAuthToken;
use crate::domain::policy::defaults::DEFAULT_WORK_MINUTES_PER_DAY;

use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::models::{AttendanceDailySummary, OtStatus};
use crate::repositories::attendance_daily_summary_repository::BatchSummaryInput;
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::AttendancePolicyRepository;
use crate::repositories::EmployeeRepository;
use crate::repositories::{workspace_repository::WorkspaceRepository, TimeEntryRepository};
use crate::services::attendance_calculator::WorkBuckets;
use crate::services::PolicyResolverService;
use crate::utils::chrono_helpers::local_to_utc as local_midnight_to_utc;

#[cfg_attr(test, mockall::automock)]
#[async_trait::async_trait]
pub trait SummaryRecalculator: Send + Sync {
    async fn recalculate_daily_summary(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError>;

    async fn recalculate_attendance_range(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError>;
}

#[async_trait::async_trait]
impl SummaryRecalculator for AttendanceSummaryService {
    async fn recalculate_daily_summary(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        self.recalculate_daily_summary(auth_token, employment_id, date)
            .await
    }

    async fn recalculate_attendance_range(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError> {
        self.recalculate_attendance_range(auth_token, employment_id, start_date, end_date)
            .await
    }
}

pub struct EnterPaperOtArgs {
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: String,
    pub entered_by_user_id: Uuid,
    pub photo_url: Option<String>,
    pub allow_exceed: bool,
}

/// Pre-computed result for a single day's attendance, ready for batch persistence.
///
/// Produced by the pure-compute phase of [`AttendanceSummaryService::recalculate_attendance_range`]
/// so that all DB writes can be batched into a single transaction after the loop.
pub(crate) struct DayComputationResult {
    date: NaiveDate,
    actual_work_minutes: i32,
    system_detected_ot_minutes: i32,
    standard_minutes: i32,
    early_ot_minutes: i32,
    late_ot_minutes: i32,
    break_ot_minutes: i32,
    is_weekend: bool,
    ot_status: OtStatus,
    is_holiday: bool,
    holiday_name: Option<String>,
}

/// All data pre-fetched by Phase A of [`AttendanceSummaryService::recalculate_attendance_range`].
/// Enables the pure-compute loop (Phase B) to run with zero DB calls.
struct RangePrefetchData {
    employment: crate::models::Employment,
    tz: Tz,
    emp_sched: crate::models::EmployeeSchedule,
    all_entries: Vec<crate::models::TimeEntry>,
    leaves: Vec<crate::models::LeaveRequest>,
    holidays_map: std::collections::HashMap<NaiveDate, crate::models::Holiday>,
    existing_summaries: Vec<AttendanceDailySummary>,
    attendance_rules: crate::models::AttendanceRules,
    conditions: Vec<crate::models::WorkingCondition>,
    catalog: std::collections::HashMap<String, crate::models::ConditionTypeDefinition>,
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
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<()> {
        let _workspace_id = auth_token.workspace_id();
        let employment = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();

        // 1. Fetch schedule & time entries
        let (tz, daily_schedule, day_entries) = self
            .fetch_daily_schedule(auth_token, employment_id, date)
            .await?;

        // Auto-cancel approved leave requests if the employee worked (day_entries is not empty) and it's not a holiday
        if !daily_schedule.is_holiday() && !day_entries.is_empty() {
            let leaves = self
                .leave_request_repo
                .get_employee_leave_requests(auth_token, employee_id)
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
                    .cancel_leave_request(auth_token, req.id())
                    .await?;
            }
        }

        // 2. Calculate work minutes and OT
        let buckets = self
            .calculate_work_and_ot(
                &tz,
                date,
                employee_id,
                auth_token,
                &daily_schedule,
                &day_entries,
            )
            .await?;
        let actual_work_minutes = buckets.standard_minutes
            + buckets.early_ot_minutes
            + buckets.late_ot_minutes
            + buckets.break_ot_minutes;
        let system_detected_ot_minutes =
            buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes;

        // 3. Holiday leave conflict detection
        let (is_holiday, holiday_name) = self
            .handle_holiday_leave_conflict(
                auth_token,
                employee_id,
                date,
                actual_work_minutes,
                &daily_schedule,
            )
            .await?;

        // 4. Determine OT status
        let new_status = self
            .determine_ot_status(auth_token, employee_id, date, system_detected_ot_minutes)
            .await?;

        let weekday = date.weekday().num_days_from_monday(); // 0=Mon, 6=Sun
        let is_weekend = weekday >= 5; // Default: Sat+Sun — configurable per workspace later

        // 5. Persist
        self.daily_summary_repo
            .upsert_calculated_summary(
                auth_token,
                employment_id,
                date,
                actual_work_minutes,
                system_detected_ot_minutes,
                buckets.standard_minutes,
                buckets.early_ot_minutes,
                buckets.late_ot_minutes,
                buckets.break_ot_minutes,
                is_weekend,
                new_status,
                is_holiday,
                holiday_name,
            )
            .await?;
        Ok(())
    }

    /// Recalculates and persists the daily summaries for a specific employee and date range.
    ///
    /// Uses a **pre-fetch → pure-compute → batch-write** pipeline to eliminate N+1 queries:
    /// - Phase A: batch-fetch all shared data (policy, conditions, catalog, schedules, entries, etc.)
    /// - Phase B: iterate days with zero DB calls (pure in-memory computation)
    /// - Phase C: batch-write all summaries in one transaction + bulk cancel leaves
    ///
    /// Result: ~7 DB calls regardless of range size (was ~240-300 for a 30-day range).
    #[tracing::instrument(skip_all, err)]
    pub async fn recalculate_attendance_range(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<()> {
        // ── Phase A: Pre-fetch all shared data ──────────────────────────
        let pf = self
            .fetch_range_preamble(auth_token, employment_id, start_date, end_date)
            .await?;
        let employee_id = pf.employment.employee_id();
        let location_id = pf.employment.primary_location_id();
        let base_early_arrival_strategy = pf.attendance_rules.early_arrival_strategy;

        // ── Phase B: Pure computation loop (ZERO DB calls) ──────────────
        let mut day_results: Vec<DayComputationResult> = Vec::new();
        let mut leave_ids_to_cancel: Vec<Uuid> = Vec::new();

        let mut curr_date = start_date;
        while curr_date <= end_date {
            if let Some(daily_schedule) = pf
                .emp_sched
                .schedules()
                .iter()
                .find(|s| s.date() == curr_date)
            {
                let day_entries: Vec<&crate::models::TimeEntry> = pf
                    .all_entries
                    .iter()
                    .filter(|e| e.punch_in_time().with_timezone(&pf.tz).date_naive() == curr_date)
                    .collect();

                // Collect leave cancellations (in-memory)
                if !daily_schedule.is_holiday() && !day_entries.is_empty() {
                    if let Some(req) = pf.leaves.iter().find(|l| {
                        (l.status() == crate::models::enums::LeaveStatus::Approved
                            || l.status() == crate::models::enums::LeaveStatus::PreApproved)
                            && l.start_date() <= curr_date
                            && l.end_date() >= curr_date
                    }) {
                        leave_ids_to_cancel.push(req.id());
                    }
                }

                // Compute day result (pure)
                let holiday = pf.holidays_map.get(&curr_date);
                let filtered_holiday = holiday
                    .filter(|&hol| hol.location_id().is_none() || hol.location_id() == location_id);

                let result = Self::compute_single_day(
                    curr_date,
                    daily_schedule,
                    &day_entries,
                    &pf.attendance_rules,
                    &pf.conditions,
                    &pf.catalog,
                    base_early_arrival_strategy,
                    pf.tz,
                    filtered_holiday,
                    &pf.existing_summaries,
                );

                // Collect holiday reversal IDs
                if let Some(hol) = filtered_holiday {
                    if result.is_holiday && result.actual_work_minutes > 0 {
                        if let Some(lid) = Self::should_reverse_holiday_leave(
                            &pf.leaves,
                            employee_id,
                            curr_date,
                            result.actual_work_minutes,
                            daily_schedule,
                            hol,
                        ) {
                            leave_ids_to_cancel.push(lid);
                        }
                    }
                }

                day_results.push(result);
            }

            curr_date = curr_date.succ_opt().unwrap_or(curr_date);
        }

        // ── Phase C: Batch writes ───────────────────────────────────────
        if !leave_ids_to_cancel.is_empty() {
            self.leave_request_repo
                .bulk_cancel_leave_requests(auth_token, &leave_ids_to_cancel)
                .await?;
        }

        if !day_results.is_empty() {
            let batch_inputs: Vec<BatchSummaryInput> = day_results
                .iter()
                .map(|r| BatchSummaryInput {
                    date: r.date,
                    actual_work_minutes: r.actual_work_minutes,
                    system_detected_ot_minutes: r.system_detected_ot_minutes,
                    standard_minutes: r.standard_minutes,
                    early_ot_minutes: r.early_ot_minutes,
                    late_ot_minutes: r.late_ot_minutes,
                    break_ot_minutes: r.break_ot_minutes,
                    is_weekend: r.is_weekend,
                    ot_status: r.ot_status,
                    is_holiday: r.is_holiday,
                    holiday_name: r.holiday_name.clone(),
                })
                .collect();

            self.daily_summary_repo
                .batch_upsert_calculated_summaries(auth_token, employment_id, &batch_inputs)
                .await?;
        }

        Ok(())
    }

    /// Phase A helper: batch-fetch all shared data for [`recalculate_attendance_range`].
    /// Eliminates per-day DB calls by pre-loading policy, conditions, catalog,
    /// schedules, time entries, leaves, holidays, and existing summaries.
    async fn fetch_range_preamble(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<RangePrefetchData> {
        let workspace_id = auth_token.workspace_id();

        let employment = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();

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
                auth_token,
                start_date,
                end_date,
                Some(&vec![employment_id]),
                None,
                None,
            )
            .await?;
        let emp_sched = schedules.into_iter().next().ok_or_else(|| {
            AppError::not_found_args(
                I18nKey::NotFound,
                vec![("details", "no_schedule_found_for_employee".to_string())],
            )
        })?;

        let start_dt_utc =
            local_midnight_to_utc(&tz, start_date, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt_utc =
            local_midnight_to_utc(&tz, end_date, 23, 59, 59).map_err(AppError::Internal)?;
        let all_entries = self
            .time_entry_repo
            .get_employment_time_entries(auth_token, employment_id, start_dt_utc, end_dt_utc)
            .await?;

        let leaves = self
            .leave_request_repo
            .get_employee_leave_requests(auth_token, employee_id)
            .await?;

        let holidays_map = self
            .holiday_repo
            .get_holidays_in_range(auth_token, start_date, end_date)
            .await?;

        let existing_summaries = self
            .daily_summary_repo
            .get_summaries(auth_token, Some(employment_id), start_date, end_date)
            .await?;

        let attendance_rules = PolicyResolverService::resolve_attendance_policy(
            &*self.policy_repo,
            employee_id,
            auth_token,
            start_date,
            tz,
        )
        .await?;

        let conditions_map = PolicyResolverService::resolve_attendance_rules_batch_by_employment(
            &*self.employee_repo,
            &[employment_id],
            auth_token,
            start_date,
            end_date,
        )
        .await?;
        let conditions = conditions_map
            .get(&employment_id)
            .cloned()
            .unwrap_or_default();

        let catalog = PolicyResolverService::load_condition_type_catalog(
            &*self.condition_type_repo,
            auth_token,
        )
        .await?;

        Ok(RangePrefetchData {
            employment,
            tz,
            emp_sched,
            all_entries,
            leaves,
            holidays_map,
            existing_summaries,
            attendance_rules,
            conditions,
            catalog,
        })
    }

    /// Fetch workspace timezone, the employee's schedule for the date, and time entries.
    async fn fetch_daily_schedule(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<(
        Tz,
        crate::models::EffectiveSchedule,
        Vec<crate::models::TimeEntry>,
    )> {
        let workspace_id = auth_token.workspace_id();
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
                auth_token,
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
            .get_employment_time_entries(auth_token, employment_id, start_dt_utc, end_dt_utc)
            .await?;

        let day_entries: Vec<_> = entries
            .into_iter()
            .filter(|e| e.punch_in_time().with_timezone(&tz).date_naive() == date)
            .collect();

        Ok((tz, daily_schedule, day_entries))
    }

    /// Calculate actual work minutes and system-detected OT from time entries.
    ///
    /// Delegates to [`AttendanceService`] — the single shared path for all
    /// attendance computation — instead of the former manual heuristic
    /// (`raw_duration - break_minutes`).
    async fn calculate_work_and_ot(
        &self,
        tz: &Tz,
        date: NaiveDate,
        employee_id: Uuid,
        auth_token: &TenantAuthToken,
        daily_schedule: &crate::models::EffectiveSchedule,
        day_entries: &[crate::models::TimeEntry],
    ) -> AppResult<WorkBuckets> {
        // Resolve policy rules (was previously in the now-removed
        // calculate_ot_buckets helper).
        let base_rules = PolicyResolverService::resolve_attendance_rules(
            &*self.policy_repo,
            &*self.employee_repo,
            &*self.condition_type_repo,
            employee_id,
            auth_token,
            date,
            *tz,
        )
        .await?;

        let early_arrival_strategy = base_rules.early_arrival_strategy();
        let rules = PolicyResolver::apply_shift_overrides(
            base_rules,
            &ShiftPolicyOverrides::new(
                daily_schedule.early_punch_in_grace_period_minutes(),
                daily_schedule.is_break_auto_deduct(),
                daily_schedule.allow_early_ot(),
                early_arrival_strategy,
                daily_schedule.break_is_overtime().unwrap_or(false),
            ),
        );

        // Convert &[TimeEntry] to &[&TimeEntry] for the shared service.
        let entry_refs: Vec<&crate::models::TimeEntry> = day_entries.iter().collect();

        let result = crate::services::AttendanceService::compute_daily_attendance(
            crate::services::ComputeDailyInput {
                daily: daily_schedule,
                day_entries: &entry_refs,
                date,
                tz,
                rules: &rules,
                work_unit_minutes: 0, // minutes mode: raw minutes, no unit conversion
                rounding_policy: crate::models::RoundingPolicy::default(),
                max_early_ot_minutes: daily_schedule.max_early_ot_minutes(),
                ot_ceiling: None,
                segment_id: None,
                ot_aggregation_mode:
                    crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            },
        );

        match result {
            Some(r) => Ok(r.work_buckets().clone()),
            None => Ok(WorkBuckets::default()),
        }
    }

    /// Pure variant of [`calculate_work_and_ot`] — takes pre-resolved [`PolicyRules`]
    /// and makes zero DB calls. Used by the batch path in
    /// [`recalculate_attendance_range`].
    fn calculate_work_and_ot_pure(
        tz: Tz,
        date: NaiveDate,
        daily_schedule: &crate::models::EffectiveSchedule,
        day_entries: &[&crate::models::TimeEntry],
        rules: &crate::models::PolicyRules,
    ) -> WorkBuckets {
        let result = crate::services::AttendanceService::compute_daily_attendance(
            crate::services::ComputeDailyInput {
                daily: daily_schedule,
                day_entries,
                date,
                tz: &tz,
                rules,
                work_unit_minutes: 0,
                rounding_policy: crate::models::RoundingPolicy::default(),
                max_early_ot_minutes: daily_schedule.max_early_ot_minutes(),
                ot_ceiling: None,
                segment_id: None,
                ot_aggregation_mode:
                    crate::services::attendance_calculator::OtAggregationMode::Aggregate,
            },
        );

        match result {
            Some(r) => r.work_buckets().clone(),
            None => WorkBuckets::default(),
        }
    }

    /// Pure holiday-leave-reversal check: returns `Some(leave_request_id)` if the
    /// system-generated holiday leave should be cancelled (employee worked above threshold).
    /// Makes zero DB calls — filters from the pre-fetched `leaves` slice.
    fn should_reverse_holiday_leave(
        leaves: &[crate::models::LeaveRequest],
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
        holiday: &crate::models::Holiday,
    ) -> Option<Uuid> {
        // Find system-generated leave for this date (requested_by_user_id is None
        // is the DB filter; we approximate by matching on status + date overlap).
        // The definitive filter is the DB query, but for the batch path we collect
        // candidates and let the bulk_cancel handle idempotent cancellation.
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

        if work_percent < threshold {
            return None;
        }

        // Find the system-generated leave for this date.
        // System-generated leaves have no requested_by_user_id; we identify them
        // by matching the date overlap and approved status from the pre-fetched leaves.
        let system_leave = leaves.iter().find(|l| {
            (l.status() == crate::models::enums::LeaveStatus::Approved
                || l.status() == crate::models::enums::LeaveStatus::PreApproved)
                && l.start_date() <= date
                && l.end_date() >= date
        });

        system_leave.map(|l| {
            tracing::info!(
                employee_id = %employee_id,
                date = %date,
                work_percent = work_percent,
                threshold = threshold,
                "Holiday leave reversal queued (batch)"
            );
            l.id()
        })
    }

    /// Pure OT status determination — no DB calls. Uses pre-fetched existing summary.
    fn determine_ot_status_pure(
        existing: Option<&AttendanceDailySummary>,
        system_detected_ot_minutes: i32,
    ) -> OtStatus {
        match existing {
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
        }
    }

    /// Pure per-day computation: builds a [`DayComputationResult`] with zero DB calls.
    /// Called inside the batch loop of [`recalculate_attendance_range`].
    #[allow(clippy::too_many_arguments)]
    fn compute_single_day(
        date: NaiveDate,
        daily_schedule: &crate::models::EffectiveSchedule,
        day_entries: &[&crate::models::TimeEntry],
        attendance_rules: &crate::models::AttendanceRules,
        conditions: &[crate::models::WorkingCondition],
        catalog: &std::collections::HashMap<String, crate::models::ConditionTypeDefinition>,
        base_early_arrival_strategy: crate::models::EarlyArrivalStrategy,
        tz: Tz,
        filtered_holiday: Option<&crate::models::Holiday>,
        existing_summaries: &[AttendanceDailySummary],
    ) -> DayComputationResult {
        let overrides = ShiftPolicyOverrides::new(
            daily_schedule.early_punch_in_grace_period_minutes(),
            daily_schedule.is_break_auto_deduct(),
            daily_schedule.allow_early_ot(),
            base_early_arrival_strategy,
            daily_schedule.break_is_overtime().unwrap_or(false),
        );
        let rules = PolicyResolver::resolve_with_shift_overrides_pure(
            Some(attendance_rules.clone()),
            conditions.to_vec(),
            date,
            Some(catalog),
            &overrides,
        );

        let buckets =
            Self::calculate_work_and_ot_pure(tz, date, daily_schedule, day_entries, &rules);
        let actual = buckets.standard_minutes
            + buckets.early_ot_minutes
            + buckets.late_ot_minutes
            + buckets.break_ot_minutes;
        let ot_mins = buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes;

        let is_holiday = filtered_holiday.is_some();
        let holiday_name = filtered_holiday.map(|h| h.name().to_string());

        let existing = existing_summaries.iter().find(|s| s.date == date);
        let weekday = date.weekday().num_days_from_monday();

        DayComputationResult {
            date,
            actual_work_minutes: actual,
            system_detected_ot_minutes: ot_mins,
            standard_minutes: buckets.standard_minutes,
            early_ot_minutes: buckets.early_ot_minutes,
            late_ot_minutes: buckets.late_ot_minutes,
            break_ot_minutes: buckets.break_ot_minutes,
            is_weekend: weekday >= 5,
            ot_status: Self::determine_ot_status_pure(existing, ot_mins),
            is_holiday,
            holiday_name,
        }
    }

    /// Check if the date is a holiday and handle leave conflict (reversal if employee worked enough).
    async fn handle_holiday_leave_conflict(
        &self,
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
    ) -> AppResult<(bool, Option<String>)> {
        let active_employment = self
            .employment_repo
            .get_active_employment_on_date(auth_token, employee_id, date)
            .await?;
        let location_id = active_employment.and_then(|e| e.primary_location_id());

        let holiday = self
            .holiday_repo
            .is_holiday(auth_token, date, location_id)
            .await?;
        let is_holiday = holiday.is_some();
        let holiday_name = holiday.as_ref().map(|h| h.name().to_string());

        if is_holiday && actual_work_minutes > 0 {
            if let Some(ref hol) = holiday {
                if let Err(e) = self
                    .maybe_reverse_holiday_leave(
                        auth_token,
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
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
        holiday: &crate::models::Holiday,
    ) -> AppResult<()> {
        let system_leave = self
            .leave_request_repo
            .get_system_generated_holiday_leave(auth_token, employee_id, date)
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
                .cancel_leave_request(auth_token, leave_request_id)
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
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
        system_detected_ot_minutes: i32,
    ) -> AppResult<OtStatus> {
        let existing = self
            .daily_summary_repo
            .get_summary(auth_token, employee_id, date)
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
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        claimed_minutes: i32,
        reason: Option<String>,
    ) -> AppResult<()> {
        let summary_repo = self.daily_summary_repo.as_ref();

        let rows_affected = summary_repo
            .claim_ot(auth_token, employment_id, date, claimed_minutes, reason)
            .await?;

        if rows_affected == 0 {
            return Err(AppError::not_found(I18nKey::NotFound));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn approve_ot(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
        approved_minutes: i32,
        manager_note: Option<String>,
        status: OtStatus, // Approved or Rejected
    ) -> AppResult<()> {
        let summary_repo = self.daily_summary_repo.as_ref();

        let rows_affected = summary_repo
            .approve_ot(
                auth_token,
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
        auth_token: &TenantAuthToken,
        args: EnterPaperOtArgs,
    ) -> AppResult<AttendanceDailySummary> {
        let summary_repo = self.daily_summary_repo.as_ref();

        // 1. Check existing summary
        let existing = summary_repo
            .get_summary(auth_token, args.employment_id, args.date)
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
                auth_token,
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
        let _workspace_id = auth_token.workspace_id();

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
            auth_token,
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
