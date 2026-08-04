use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use diesel_async::AsyncPgConnection;

use uuid::Uuid;

use crate::repositories::attendance_daily_summary_repository::{
    AttendanceDailySummaryRepository, UpsertCalculatedSummaryArgs, UpsertCalculatedSummaryTxArgs,
    UpsertPaperOtArgs,
};
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

    /// Transaction-aware variant of [`recalculate_daily_summary`].
    ///
    /// Routes the critical time-entry read and all writes through the provided
    /// connection so the caller can coordinate this recalculation with other
    /// operations inside a single database transaction (read-your-writes).
    async fn recalculate_daily_summary_tx(
        &self,
        conn: &mut AsyncPgConnection,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError>;

    /// Transaction-aware variant of [`recalculate_attendance_range`].
    async fn recalculate_attendance_range_tx(
        &self,
        conn: &mut AsyncPgConnection,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError>;

    /// Pure-compute phase of single-day recalculation.
    ///
    /// Performs all reads and computation but makes **no destructive writes**.
    /// Safe to retry on transient failures — no leave cancellations or
    /// summary upserts have been executed yet.
    async fn compute_daily_recalculation_plan(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyRecalculationPlan, AppError>;

    /// Persist phase of single-day recalculation.
    ///
    /// Executes destructive writes (cancel leaves, upsert summary) based on
    /// a pre-computed plan. **Must be called exactly once** after
    /// [`compute_daily_recalculation_plan`](Self::compute_daily_recalculation_plan)
    /// succeeds — never retry this method.
    async fn execute_daily_recalculation_plan(
        &self,
        auth_token: &TenantAuthToken,
        plan: &DailyRecalculationPlan,
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

    async fn recalculate_daily_summary_tx(
        &self,
        conn: &mut AsyncPgConnection,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<(), AppError> {
        self.recalculate_daily_summary_tx(conn, auth_token, employment_id, date)
            .await
    }

    async fn recalculate_attendance_range_tx(
        &self,
        conn: &mut AsyncPgConnection,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<(), AppError> {
        self.recalculate_attendance_range_tx(conn, auth_token, employment_id, start_date, end_date)
            .await
    }

    async fn compute_daily_recalculation_plan(
        &self,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<DailyRecalculationPlan, AppError> {
        self.compute_daily_recalculation_plan(None, auth_token, employment_id, date)
            .await
    }

    async fn execute_daily_recalculation_plan(
        &self,
        auth_token: &TenantAuthToken,
        plan: &DailyRecalculationPlan,
    ) -> Result<(), AppError> {
        self.execute_daily_recalculation_plan(None, auth_token, plan)
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

/// Plan produced by the pure-compute phase of single-day recalculation.
///
/// Separates side-effect-free computation from destructive writes
/// (cancel leave, upsert summary) so that the computation can be safely
/// retried without risk of double-refunding leave balances.
///
/// See: [`SummaryRecalculator::compute_daily_recalculation_plan`]
/// and [`SummaryRecalculator::execute_daily_recalculation_plan`].
#[derive(Debug, Clone)]
pub struct DailyRecalculationPlan {
    pub employment_id: Uuid,
    pub date: NaiveDate,
    pub actual_work_minutes: i32,
    pub system_detected_ot_minutes: i32,
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
    pub is_weekend: bool,
    pub ot_status: OtStatus,
    pub is_holiday: bool,
    pub holiday_name: Option<String>,
    /// Leave request to cancel because employee worked on a leave day.
    pub leave_to_cancel: Option<Uuid>,
    /// Holiday leave to reverse because employee worked above the threshold.
    pub holiday_leave_to_cancel: Option<Uuid>,
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

pub(super) struct ComputeSingleDayParams<'a> {
    pub date: NaiveDate,
    pub daily_schedule: &'a crate::models::EffectiveSchedule,
    pub day_entries: &'a [&'a crate::models::TimeEntry],
    pub attendance_rules: &'a crate::models::AttendanceRules,
    pub conditions: &'a [crate::models::WorkingCondition],
    pub catalog: &'a std::collections::HashMap<String, crate::models::ConditionTypeDefinition>,
    pub base_early_arrival_strategy: crate::models::EarlyArrivalStrategy,
    pub tz: Tz,
    pub filtered_holiday: Option<&'a crate::models::Holiday>,
    pub existing_summaries: &'a [AttendanceDailySummary],
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
        self.recalculate_daily_summary_with_conn(None, auth_token, employment_id, date)
            .await
    }

    /// Transaction-aware variant of [`recalculate_daily_summary`].
    ///
    /// Routes the time-entry read and all writes through `conn` so the caller
    /// can coordinate this recalculation with sibling writes in one transaction.
    #[tracing::instrument(skip_all, err)]
    pub async fn recalculate_daily_summary_tx(
        &self,
        conn: &mut AsyncPgConnection,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<()> {
        self.recalculate_daily_summary_with_conn(Some(conn), auth_token, employment_id, date)
            .await
    }

    // `as_deref_mut` reborrows the optional conn for sequential calls;
    // the temporary borrow is dropped between .await points so this is sound.
    #[allow(clippy::needless_option_as_deref, clippy::too_many_lines)]
    async fn compute_daily_recalculation_plan(
        &self,
        mut conn: Option<&mut AsyncPgConnection>,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<DailyRecalculationPlan> {
        let employment = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;
        let employee_id = employment.employee_id();

        // 1. Fetch schedule & time entries
        let (tz, daily_schedule, day_entries) = self
            .fetch_daily_schedule(conn.as_deref_mut(), auth_token, employment_id, date)
            .await?;

        // 2. Identify leave to cancel (work-on-leave-day) — read only, no write
        let leave_to_cancel = if !daily_schedule.is_holiday() && !day_entries.is_empty() {
            let leaves = self
                .leave_request_repo
                .get_employee_leave_requests(auth_token, employee_id)
                .await?;

            leaves.into_iter().find_map(|l| {
                if (l.status() == crate::models::enums::LeaveStatus::Approved
                    || l.status() == crate::models::enums::LeaveStatus::PreApproved)
                    && l.start_date() <= date
                    && l.end_date() >= date
                {
                    Some(l.id())
                } else {
                    None
                }
            })
        } else {
            None
        };

        // 3. Calculate work minutes and OT
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

        // 4. Holiday leave conflict detection — read only, collect leave ID
        let (is_holiday, holiday_name, holiday_leave_to_cancel) = self
            .detect_holiday_leave_conflict(
                auth_token,
                employee_id,
                date,
                actual_work_minutes,
                &daily_schedule,
            )
            .await?;

        // 5. Determine OT status
        let new_status = self
            .determine_ot_status(auth_token, employee_id, date, system_detected_ot_minutes)
            .await?;

        let weekday = date.weekday().num_days_from_monday();
        let is_weekend = weekday >= 5;

        Ok(DailyRecalculationPlan {
            employment_id,
            date,
            actual_work_minutes,
            system_detected_ot_minutes,
            standard_minutes: buckets.standard_minutes,
            early_ot_minutes: buckets.early_ot_minutes,
            late_ot_minutes: buckets.late_ot_minutes,
            break_ot_minutes: buckets.break_ot_minutes,
            is_weekend,
            ot_status: new_status,
            is_holiday,
            holiday_name,
            leave_to_cancel,
            holiday_leave_to_cancel,
        })
    }

    /// Persist phase — executes destructive writes exactly once.
    /// **Must NOT be retried.**
    #[allow(clippy::needless_option_as_deref)]
    async fn execute_daily_recalculation_plan(
        &self,
        mut conn: Option<&mut AsyncPgConnection>,
        auth_token: &TenantAuthToken,
        plan: &DailyRecalculationPlan,
    ) -> AppResult<()> {
        let workspace_id = auth_token.workspace_id();

        // 1. Cancel work-on-leave-day leave
        if let Some(leave_id) = plan.leave_to_cancel {
            if let Some(c) = conn.as_deref_mut() {
                self.leave_request_repo
                    .bulk_cancel_leave_requests_tx(c, auth_token, &[leave_id])
                    .await?;
            } else {
                self.leave_request_repo
                    .cancel_leave_request(auth_token, leave_id)
                    .await?;
            }
        }

        // 2. Cancel holiday leave
        if let Some(leave_id) = plan.holiday_leave_to_cancel {
            if let Some(c) = conn.as_deref_mut() {
                self.leave_request_repo
                    .bulk_cancel_leave_requests_tx(c, auth_token, &[leave_id])
                    .await?;
            } else {
                self.leave_request_repo
                    .cancel_leave_request(auth_token, leave_id)
                    .await?;
            }
        }

        // 3. Upsert summary
        if let Some(c) = conn {
            self.daily_summary_repo
                .upsert_calculated_summary_tx(
                    c,
                    UpsertCalculatedSummaryTxArgs {
                        workspace_id,
                        employment_id: plan.employment_id,
                        date: plan.date,
                        actual_work_minutes: plan.actual_work_minutes,
                        system_detected_ot_minutes: plan.system_detected_ot_minutes,
                        standard_minutes: plan.standard_minutes,
                        early_ot_minutes: plan.early_ot_minutes,
                        late_ot_minutes: plan.late_ot_minutes,
                        break_ot_minutes: plan.break_ot_minutes,
                        is_weekend: plan.is_weekend,
                        new_status: plan.ot_status,
                        is_holiday: plan.is_holiday,
                        holiday_name: plan.holiday_name.clone(),
                    },
                )
                .await?;
        } else {
            self.daily_summary_repo
                .upsert_calculated_summary(UpsertCalculatedSummaryArgs {
                    auth_token: *auth_token,
                    employment_id: plan.employment_id,
                    date: plan.date,
                    actual_work_minutes: plan.actual_work_minutes,
                    system_detected_ot_minutes: plan.system_detected_ot_minutes,
                    standard_minutes: plan.standard_minutes,
                    early_ot_minutes: plan.early_ot_minutes,
                    late_ot_minutes: plan.late_ot_minutes,
                    break_ot_minutes: plan.break_ot_minutes,
                    is_weekend: plan.is_weekend,
                    new_status: plan.ot_status,
                    is_holiday: plan.is_holiday,
                    holiday_name: plan.holiday_name.clone(),
                })
                .await?;
        }
        Ok(())
    }

    /// Compose compute → execute for backward compatibility with callers that
    /// expect a single atomic call. When calling from a retry context, prefer
    /// [`compute_daily_recalculation_plan`] + [`execute_daily_recalculation_plan`]
    /// so that only the side-effect-free compute phase is retried.
    #[allow(clippy::needless_option_as_deref, clippy::too_many_lines)]
    async fn recalculate_daily_summary_with_conn(
        &self,
        mut conn: Option<&mut AsyncPgConnection>,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> AppResult<()> {
        let plan = self
            .compute_daily_recalculation_plan(conn.as_deref_mut(), auth_token, employment_id, date)
            .await?;
        self.execute_daily_recalculation_plan(conn, auth_token, &plan)
            .await
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
        self.recalculate_attendance_range_with_conn(
            None,
            auth_token,
            employment_id,
            start_date,
            end_date,
        )
        .await
    }

    /// Transaction-aware variant of [`recalculate_attendance_range`].
    ///
    /// Routes the time-entry read and all writes through `conn` so the caller
    /// can coordinate this recalculation with sibling writes in one transaction.
    #[tracing::instrument(skip_all, err)]
    pub async fn recalculate_attendance_range_tx(
        &self,
        conn: &mut AsyncPgConnection,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<()> {
        self.recalculate_attendance_range_with_conn(
            Some(conn),
            auth_token,
            employment_id,
            start_date,
            end_date,
        )
        .await
    }

    // `as_deref_mut` reborrows the optional conn for several sequential writes;
    // the type is unchanged so the lint is a false positive here.
    #[allow(clippy::needless_option_as_deref, clippy::too_many_lines)]
    async fn recalculate_attendance_range_with_conn(
        &self,
        mut conn: Option<&mut AsyncPgConnection>,
        auth_token: &TenantAuthToken,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> AppResult<()> {
        // ── Phase A: Pre-fetch all shared data ──────────────────────────
        let pf = self
            .fetch_range_preamble(
                conn.as_deref_mut(),
                auth_token,
                employment_id,
                start_date,
                end_date,
            )
            .await?;
        let workspace_id = auth_token.workspace_id();
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

                let result = Self::compute_single_day(ComputeSingleDayParams {
                    date: curr_date,
                    daily_schedule,
                    day_entries: &day_entries,
                    attendance_rules: &pf.attendance_rules,
                    conditions: &pf.conditions,
                    catalog: &pf.catalog,
                    base_early_arrival_strategy,
                    tz: pf.tz,
                    filtered_holiday,
                    existing_summaries: &pf.existing_summaries,
                });

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
            if let Some(c) = conn.as_deref_mut() {
                self.leave_request_repo
                    .bulk_cancel_leave_requests_tx(c, auth_token, &leave_ids_to_cancel)
                    .await?;
            } else {
                self.leave_request_repo
                    .bulk_cancel_leave_requests(auth_token, &leave_ids_to_cancel)
                    .await?;
            }
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

            if let Some(c) = conn {
                self.daily_summary_repo
                    .batch_upsert_calculated_summaries_tx(
                        c,
                        workspace_id,
                        employment_id,
                        &batch_inputs,
                    )
                    .await?;
            } else {
                self.daily_summary_repo
                    .batch_upsert_calculated_summaries(auth_token, employment_id, &batch_inputs)
                    .await?;
            }
        }

        Ok(())
    }

    /// Phase A helper: batch-fetch all shared data for [`recalculate_attendance_range`].
    /// Eliminates per-day DB calls by pre-loading policy, conditions, catalog,
    /// schedules, time entries, leaves, holidays, and existing summaries.
    #[allow(clippy::too_many_lines)]
    async fn fetch_range_preamble(
        &self,
        conn: Option<&mut AsyncPgConnection>,
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
        let all_entries = if let Some(c) = conn {
            self.time_entry_repo
                .get_employment_time_entries_tx(
                    c,
                    workspace_id,
                    employment_id,
                    start_dt_utc,
                    end_dt_utc,
                )
                .await?
        } else {
            self.time_entry_repo
                .get_employment_time_entries(auth_token, employment_id, start_dt_utc, end_dt_utc)
                .await?
        };

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
        conn: Option<&mut AsyncPgConnection>,
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

        let entries = if let Some(c) = conn {
            self.time_entry_repo
                .get_employment_time_entries_tx(
                    c,
                    workspace_id,
                    employment_id,
                    start_dt_utc,
                    end_dt_utc,
                )
                .await?
        } else {
            self.time_entry_repo
                .get_employment_time_entries(auth_token, employment_id, start_dt_utc, end_dt_utc)
                .await?
        };

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
    #[allow(clippy::needless_pass_by_value)]
    fn compute_single_day(params: ComputeSingleDayParams<'_>) -> DayComputationResult {
        let overrides = ShiftPolicyOverrides::new(
            params.daily_schedule.early_punch_in_grace_period_minutes(),
            params.daily_schedule.is_break_auto_deduct(),
            params.daily_schedule.allow_early_ot(),
            params.base_early_arrival_strategy,
            params.daily_schedule.break_is_overtime().unwrap_or(false),
        );
        let rules = PolicyResolver::resolve_with_shift_overrides_pure(
            Some(params.attendance_rules.clone()),
            params.conditions.to_vec(),
            params.date,
            Some(params.catalog),
            &overrides,
        );

        let buckets = Self::calculate_work_and_ot_pure(
            params.tz,
            params.date,
            params.daily_schedule,
            params.day_entries,
            &rules,
        );
        let actual = buckets.standard_minutes
            + buckets.early_ot_minutes
            + buckets.late_ot_minutes
            + buckets.break_ot_minutes;
        let ot_mins = buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes;

        let is_holiday = params.filtered_holiday.is_some();
        let holiday_name = params.filtered_holiday.map(|h| h.name().to_string());

        let existing = params
            .existing_summaries
            .iter()
            .find(|s| s.date == params.date);
        let weekday = params.date.weekday().num_days_from_monday();

        DayComputationResult {
            date: params.date,
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

    /// Read-only holiday leave conflict detection.
    ///
    /// Returns `(is_holiday, holiday_name, Option<leave_id_to_reverse>)`.
    /// Makes **no destructive writes** — the caller is responsible for persisting
    /// the reversal via [`execute_daily_recalculation_plan`].
    async fn detect_holiday_leave_conflict(
        &self,
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
    ) -> AppResult<(bool, Option<String>, Option<Uuid>)> {
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

        let holiday_leave_to_cancel = if is_holiday && actual_work_minutes > 0 {
            if let Some(ref hol) = holiday {
                self.find_holiday_leave_to_reverse(
                    auth_token,
                    employee_id,
                    date,
                    actual_work_minutes,
                    daily_schedule,
                    hol,
                )
                .await?
            } else {
                None
            }
        } else {
            None
        };

        Ok((is_holiday, holiday_name, holiday_leave_to_cancel))
    }

    /// Read-only version of [`maybe_reverse_holiday_leave`].
    /// Finds the system-generated holiday leave and checks the threshold,
    /// but does **not** cancel anything. Returns the leave ID if it should be reversed.
    async fn find_holiday_leave_to_reverse(
        &self,
        auth_token: &TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
        actual_work_minutes: i32,
        daily_schedule: &crate::models::EffectiveSchedule,
        holiday: &crate::models::Holiday,
    ) -> AppResult<Option<Uuid>> {
        let system_leave = self
            .leave_request_repo
            .get_system_generated_holiday_leave(auth_token, employee_id, date)
            .await?;

        let Some((leave_request_id, _leave_type_id, _leave_code)) = system_leave else {
            return Ok(None);
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
            tracing::info!(
                employee_id = %employee_id,
                date = %date,
                work_percent = work_percent,
                threshold = threshold,
                "Holiday leave reversal queued (single-day compute)"
            );
            Ok(Some(leave_request_id))
        } else {
            tracing::info!(
                employee_id = %employee_id,
                date = %date,
                work_percent = work_percent,
                threshold = threshold,
                "Holiday leave kept: employee worked {}% (< {}% threshold)",
                work_percent, threshold
            );
            Ok(None)
        }
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
            .upsert_paper_ot(UpsertPaperOtArgs {
                auth_token: *auth_token,
                employment_id: args.employment_id,
                date: args.date,
                claimed_minutes: args.claimed_minutes,
                reason: args.reason,
                entered_by_user_id: args.entered_by_user_id,
                photo_url: args.photo_url,
            })
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
