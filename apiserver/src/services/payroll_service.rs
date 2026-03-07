use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use futures_util::stream::{self, StreamExt};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::policy::defaults::DEFAULT_OT_MULTIPLIER;
use crate::domain::policy::overtime::OvertimeEngine;
use crate::domain::policy::resolver::PolicyResolver;
use crate::models::Payslip;
use crate::repositories::overtime_policy_repository::OvertimePolicyRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{
    department_repository, leave_request_repository, payroll_repository, roster_repository,
    time_entry_repository, SqlxWorkspaceRepository,
};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::services::{
    payroll_evaluator::{IncentiveContext, PayrollEvaluator},
    AttendanceCalculator,
};

pub struct PayrollService;

// ---------------------------------------------------------------------------
// Internal data structs for PayrollService decomposition
// ---------------------------------------------------------------------------

/// All pre-fetched data required for payroll calculation.
/// Constructed once by `fetch_context()` and passed to subsequent phases.
struct PayrollContext {
    workspace: crate::models::Workspace,
    tz: Tz,
    start_date: NaiveDate,
    end_date: NaiveDate,
    salary_history: Vec<(crate::models::EmployeeSalaryComponent, crate::models::SalaryComponentDefinition)>,
    all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>>,
    holidays_map: HashMap<NaiveDate, crate::models::Holiday>,
    roster_data: Arc<crate::cache::RosterData>,
    emp_id_number: String,
    emp_tags: Option<Vec<String>>,
    payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>>,
}

/// Running totals accumulated during segment calculation.
#[allow(dead_code)]
struct SegmentCalcResult {
    total_base_pay: Decimal,
    total_std_units: i32,
    total_ot_units: i32,
    total_overtime_pay: Decimal,
    total_holiday_work_premium: Decimal,
    total_late_early_count: i32,
    segment_snapshots: Vec<SegmentSnapshot>,
}

/// Named record for a payroll segment snapshot.
///
/// Replaces the 10-element tuple that was previously used for
/// `segment_snapshots`, giving each field a meaningful name.
#[derive(Debug, Clone)]
struct SegmentSnapshot {
    seg_start: NaiveDate,
    seg_end: NaiveDate,
    department_id: Option<Uuid>,
    total_base: Decimal,
    std_working_days: Decimal,
    hours_per_day: Decimal,
    actual_worked_days: Decimal,
    paid_days: Decimal,
    amount: Decimal,
    source: String,
}

impl PayrollService {
    /// Phase 1: Fetch all data needed for payroll calculation.
    async fn fetch_context(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<PayrollContext, StatusCode> {
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Error fetching workspace: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or(StatusCode::BAD_REQUEST)?;
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(StatusCode::BAD_REQUEST)?;
        let end_date = next_month_date
            .pred_opt()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let salary_history = payroll_repository::get_salary_components_history(
            pool, employee_id, start_date, end_date,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let all_dept_history =
            department_repository::get_employee_department_history(pool, employee_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let time_entries =
            time_entry_repository::get_employee_time_entries(pool, employee_id, start_dt, end_dt)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let holidays_map = HolidayRepository::get_holidays_in_range(
            pool, workspace_id, start_date, end_date,
        )
        .await
        .unwrap_or_default();

        let roster_data = crate::cache::build_roster_data(pool, workspace_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to build roster data: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let emp_row = sqlx::query(
            "SELECT employee_id_number, tags FROM employees WHERE id = $1",
        )
        .bind(employee_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

        let emp_id_number: String = {
            use sqlx::Row;
            emp_row.get("employee_id_number")
        };
        let emp_tags: Option<Vec<String>> = {
            use sqlx::Row;
            emp_row.get("tags")
        };

        let rosters = roster_repository::get_employee_rosters(pool, employee_id, start_date, end_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let payroll_roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = rosters
            .into_iter()
            .map(|r| ((r.employee_id, r.date), r.shift_id))
            .collect();

        let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
            HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time.with_timezone(&tz).date_naive();
            daily_entries_map.entry(local_date).or_default().push(entry);
        }

        Ok(PayrollContext {
            workspace,
            tz,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map,
            holidays_map,
            roster_data,
            emp_id_number,
            emp_tags,
            payroll_roster_map,
        })
    }

    /// Phase 2: Determine salary/dept change segments.
    fn build_segments(ctx: &PayrollContext) -> Vec<(NaiveDate, NaiveDate)> {
        let mut change_points = BTreeSet::new();
        change_points.insert(ctx.start_date);

        for (comp, _) in &ctx.salary_history {
            if comp.effective_date > ctx.start_date && comp.effective_date <= ctx.end_date {
                change_points.insert(comp.effective_date);
            }
        }
        for dept in &ctx.all_dept_history {
            if dept.effective_date > ctx.start_date && dept.effective_date <= ctx.end_date {
                change_points.insert(dept.effective_date);
            }
        }

        let sorted_dates: Vec<NaiveDate> = change_points.into_iter().collect();
        let mut segments = Vec::new();

        for i in 0..sorted_dates.len() {
            let seg_start = sorted_dates[i];
            let seg_end = if i + 1 < sorted_dates.len() {
                sorted_dates[i + 1].pred_opt().unwrap_or(seg_start)
            } else {
                ctx.end_date
            };
            segments.push((seg_start, seg_end));
        }
        segments
    }

    /// Phase 2b: Financial rate abstractions to enforce DRY
    fn calculate_daily_rate(total_base: Decimal, std_working_days: Decimal) -> Decimal {
        if std_working_days.is_zero() {
            Decimal::ZERO
        } else {
            total_base / std_working_days
        }
    }

    fn calculate_hourly_rate(ot_base: Decimal, std_working_days: Decimal, hours_per_day: Decimal) -> Decimal {
        if std_working_days.is_zero() || hours_per_day.is_zero() {
            Decimal::ZERO
        } else {
            (ot_base / std_working_days) / hours_per_day
        }
    }

    /// Phase 4: Apply monthly OT cap, proportionally reducing OT pay.
    fn apply_ot_cap(
        total_ot_units: i32,
        total_overtime_pay: Decimal,
        max_monthly_hours: Decimal,
        work_unit_minutes: i32,
    ) -> (i32, Decimal) {
        let max_ot_minutes = max_monthly_hours * Decimal::from(60);
        let max_ot_units_dec = max_ot_minutes / Decimal::from(work_unit_minutes);
        let max_ot_units = max_ot_units_dec.to_i32().unwrap_or(i32::MAX);

        if total_ot_units > max_ot_units {
            let adjusted_pay = if total_ot_units > 0 {
                let ratio = Decimal::from(max_ot_units) / Decimal::from(total_ot_units);
                (total_overtime_pay * ratio).round_dp(0)
            } else {
                total_overtime_pay
            };
            (max_ot_units, adjusted_pay)
        } else {
            (total_ot_units, total_overtime_pay)
        }
    }

    pub async fn calculate_monthly_payslip(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Payslip, StatusCode> {
        // Phase 1: Fetch all context data
        let ctx = Self::fetch_context(pool, employee_id, workspace_id, year, month).await?;

        // Phase 2: Build segments
        let segments = Self::build_segments(&ctx);
        // Phase 3: Calculate each segment
        let mut total_base_pay = Decimal::ZERO;
        let mut total_std_units = 0;
        let mut total_ot_units = 0;
        let mut total_late_early_count = 0;
        let mut segment_snapshots = Vec::new();
        let mut total_overtime_pay = Decimal::ZERO;
        let mut total_holiday_work_premium = Decimal::ZERO;
        let mut policy_cache: HashMap<Uuid, Vec<crate::models::OvertimeRule>> = HashMap::new();

        for (seg_start, seg_end) in segments {
            // A. Context Resolution
            let active_dept = ctx.all_dept_history
                .iter()
                .filter(|d| d.effective_date <= seg_start)
                .max_by_key(|d| d.effective_date);
            let dept_id = active_dept.map(|d| d.department_id);

            let mut segment_total_base = Decimal::ZERO;
            let mut segment_ot_base = Decimal::ZERO;
            let mut processed_def_ids = BTreeSet::new();
            for (esc, scd) in ctx.salary_history.iter().rev() {
                if !processed_def_ids.contains(&scd.id) && esc.effective_date <= seg_start {
                    segment_total_base += esc.amount;
                    if scd.is_base_for_ot {
                        segment_ot_base += esc.amount;
                    }
                    processed_def_ids.insert(scd.id);
                }
            }

            // Resolve Standard Month Config
            let std_config = PolicyResolver::resolve_standard_month(
                pool,
                employee_id,
                dept_id,
                workspace_id,
                seg_start,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let seg_units_per_day = (std_config.hours_per_day * Decimal::from(60))
                / Decimal::from(ctx.workspace.work_unit_minutes);

            // B. Attendance Calculation
            let mut segment_std_units = 0;
            let mut segment_ot_units = 0;
            let mut segment_ot_pay = Decimal::ZERO;

            let mut current_date = seg_start;
            while current_date <= seg_end {
                // Resolve shift from pre-loaded roster data (in-memory, zero DB queries)
                let (shift_ids, _, _) = roster_repository::resolve_effective_shift_from_data(
                    &ctx.roster_data,
                    &ctx.payroll_roster_map,
                    employee_id,
                    &ctx.emp_id_number,
                    ctx.emp_tags.as_ref(),
                    current_date,
                );

                // Collect resolved shifts sorted by start_time for OT ceiling
                let mut day_shifts: Vec<_> = shift_ids
                    .iter()
                    .filter_map(|opt| opt.and_then(|id| ctx.roster_data.shifts.get(&id)))
                    .collect();
                day_shifts.sort_by_key(|s| s.start_time);

                if let Some(entries) = ctx.daily_entries_map.get(&current_date) {
                    let work_entries: Vec<_> = entries
                        .iter()
                        .filter(|e| e.entry_type == "work" || e.entry_type.is_empty())
                        .collect();
                    let break_entries: Vec<_> =
                        entries.iter().filter(|e| e.entry_type == "break").collect();
                    let actual_break_minutes: i32 = break_entries
                        .iter()
                        .map(|e| e.duration_minutes.unwrap_or(0))
                        .sum();

                    // Process each shift with OT ceiling = next shift's start
                    for (shift_idx, s) in day_shifts.iter().enumerate() {
                        // OT ceiling: cap late OT at next shift's start time
                        let ot_ceiling = day_shifts.get(shift_idx + 1).and_then(|next_s| {
                            ctx.tz.from_local_datetime(&current_date.and_time(next_s.start_time))
                                .single()
                        });

                        let start_dt = ctx.tz
                            .from_local_datetime(&current_date.and_time(s.start_time))
                            .single();
                        let end_dt = ctx.tz
                            .from_local_datetime(&current_date.and_time(s.end_time))
                            .single();

                        if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                            if s.end_time < s.start_time {
                                end += chrono::Duration::days(1);
                            }

                            // Find the work entry that belongs to this shift
                            let matching_entries: Vec<_> = work_entries.iter().filter(|entry| {
                                let punch_in = entry.punch_in_time.with_timezone(&ctx.tz);
                                // Entry belongs to this shift if punch_in is before shift end
                                // and after previous shift's end (or before shift start with grace)
                                let prev_end = if shift_idx > 0 {
                                    day_shifts.get(shift_idx - 1).and_then(|prev_s| {
                                        ctx.tz.from_local_datetime(&current_date.and_time(prev_s.end_time)).single()
                                    })
                                } else {
                                    None
                                };
                                let after_prev = prev_end.is_none_or(|pe| punch_in >= pe);
                                let before_end = punch_in < end;
                                after_prev && before_end
                            }).collect();

                            for entry in matching_entries {
                                let punch_in = entry.punch_in_time.with_timezone(&ctx.tz);
                                if let Some(punch_out_raw) = entry.punch_out_time {
                                    let punch_out = punch_out_raw.with_timezone(&ctx.tz);

                                    let rules = PolicyResolver::resolve_with_shift_overrides(
                                        pool,
                                        employee_id,
                                        ctx.workspace.id,
                                        current_date,
                                        &crate::domain::policy::resolver::ShiftPolicyOverrides {
                                            grace_period_minutes: Some(s.early_punch_in_grace_period_minutes),
                                            is_break_auto_deduct: Some(s.is_break_auto_deduct),
                                            allow_early_ot: Some(s.allow_early_ot),
                                            early_arrival_strategy: ctx.workspace.early_arrival_strategy,
                                            is_break_overtime: entry.is_break_overtime,
                                        },
                                    )
                                    .await;

                                    let (std_u, ot_u) =
                                        AttendanceCalculator::calculate_daily_units(
                                            punch_in,
                                            punch_out,
                                            Some(start),
                                            Some(end),
                                            s.break_duration_minutes,
                                            actual_break_minutes,
                                            ctx.workspace.work_unit_minutes,
                                            &ctx.workspace.rounding_policy,
                                            &rules,
                                            ot_ceiling,
                                        );
                                    segment_std_units += std_u;
                                    segment_ot_units += ot_u;

                                    // --- HOLIDAY WORK COMPENSATION (configurable per-holiday) ---
                                     if let Some(holiday) = ctx.holidays_map.get(&current_date) {
                                        let daily_rate = Self::calculate_daily_rate(segment_total_base, std_config.std_working_days);
                                        if holiday.is_credit_on_work {
                                            segment_std_units += seg_units_per_day.to_i32().unwrap_or(0);
                                        }
                                        let multiplier = holiday.ot_multiplier.unwrap_or(Decimal::from(3));
                                        if multiplier > Decimal::ONE {
                                            let worked_fraction = Decimal::from(std_u) / seg_units_per_day;
                                            total_holiday_work_premium += worked_fraction * daily_rate * (multiplier - Decimal::ONE);
                                        }
                                    }

                                    if ot_u > 0 {
                                        if let Some(policy_id) = OvertimePolicyRepository::resolve_policy_for_employee(pool, employee_id, workspace_id, current_date).await.unwrap_or(None) {
                                            let rules = if let Some(r) = policy_cache.get(&policy_id) {
                                                r.clone()
                                            } else {
                                                let r = OvertimePolicyRepository::get_policy_with_rules(pool, workspace_id, policy_id).await.unwrap_or(None).map(|(_, rules)| rules).unwrap_or_default();
                                                policy_cache.insert(policy_id, r.clone());
                                                r
                                            };

                                            let hourly_rate = Self::calculate_hourly_rate(segment_ot_base, std_config.std_working_days, std_config.hours_per_day);

                                            let mut total_ot_pay = Decimal::ZERO;
                                            if punch_in < start {
                                                let ot_end = std::cmp::min(punch_out, start);
                                                let daily_splits = OvertimeEngine::split_interval_by_day(punch_in, ot_end, &ctx.tz);
                                                for (date, intervals) in daily_splits {
                                                    let day_is_holiday = ctx.holidays_map.contains_key(&date);
                                                    total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                                }
                                            }
                                            if punch_out > end {
                                                let ot_start = std::cmp::max(punch_in, end);
                                                // Cap OT interval at ot_ceiling too
                                                let ot_end_for_pay = match ot_ceiling {
                                                    Some(ceil) => std::cmp::min(punch_out, ceil),
                                                    None => punch_out,
                                                };
                                                let daily_splits = OvertimeEngine::split_interval_by_day(ot_start, ot_end_for_pay, &ctx.tz);
                                                for (date, intervals) in daily_splits {
                                                    let day_is_holiday = ctx.holidays_map.contains_key(&date);
                                                    total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                                }
                                            }
                                            segment_ot_pay += total_ot_pay;
                                        } else {
                                            let hourly_rate = Self::calculate_hourly_rate(segment_ot_base, std_config.std_working_days, std_config.hours_per_day);
                                            segment_ot_pay += (Decimal::from(ot_u) * Decimal::from(ctx.workspace.work_unit_minutes) / Decimal::from(60)) * hourly_rate * Decimal::from_f64(DEFAULT_OT_MULTIPLIER).unwrap_or(Decimal::new(15, 1));
                                        }
                                    }

                                    // Violation detection for IncentiveContext
                                    if !ctx.holidays_map.contains_key(&current_date) {
                                        let is_late = punch_in
                                            > start
                                                + chrono::Duration::minutes(
                                                    rules.grace_period_minutes as i64,
                                                );
                                        let is_early = punch_out < end;
                                        if is_late || is_early {
                                            total_late_early_count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Fallback: entries with no matching shift (day_shifts is empty)
                    if day_shifts.is_empty() {
                        for entry in &work_entries {
                            let punch_in = entry.punch_in_time.with_timezone(&ctx.tz);
                            if let Some(punch_out_raw) = entry.punch_out_time {
                                let punch_out = punch_out_raw.with_timezone(&ctx.tz);
                                segment_std_units += AttendanceCalculator::calculate_units(
                                    (punch_out - punch_in).num_minutes() as i32,
                                    ctx.workspace.work_unit_minutes,
                                    &ctx.workspace.rounding_policy,
                                );
                            }
                        }
                    }
                } else if let Some(holiday) = ctx.holidays_map.get(&current_date) {
                    // Paid holiday with no attendance → auto-credit standard day
                    if holiday.is_paid {
                        let credit_units = if holiday.holiday_type == "half_day" {
                            // Half-day holiday: credit half a standard day
                            (seg_units_per_day / Decimal::from(2))
                                .to_i32()
                                .unwrap_or(0)
                        } else {
                            seg_units_per_day.to_i32().unwrap_or(0)
                        };
                        segment_std_units += credit_units;
                    }
                }
                current_date = match current_date.succ_opt() {
                    Some(d) => d,
                    None => break,
                };
            }

            // C. Financials
            let worked_days_dec = Decimal::from(segment_std_units) / seg_units_per_day;
            let daily_rate = Self::calculate_daily_rate(segment_total_base, std_config.std_working_days);
            let segment_amount = daily_rate * worked_days_dec;

            total_base_pay += segment_amount;
            total_std_units += segment_std_units;
            total_ot_units += segment_ot_units;
            total_overtime_pay += segment_ot_pay;

            segment_snapshots.push(SegmentSnapshot {
                seg_start,
                seg_end,
                department_id: dept_id,
                total_base: segment_total_base,
                std_working_days: std_config.std_working_days,
                hours_per_day: std_config.hours_per_day,
                actual_worked_days: worked_days_dec,
                paid_days: worked_days_dec,
                amount: segment_amount,
                source: std_config.source,
            });
        }

        // 6b. Enforce monthly OT cap from PayrollSettings
        let payroll_settings = crate::repositories::payroll_repository::get_payroll_settings_at(
            pool, workspace_id, ctx.end_date,
        )
        .await
        .unwrap_or(None);
        if let Some(ref ps) = payroll_settings {
            if let Some(max_monthly_hours) = ps.max_ot_hours_per_month {
                let (capped_units, capped_pay) = Self::apply_ot_cap(
                    total_ot_units,
                    total_overtime_pay,
                    max_monthly_hours,
                    ctx.workspace.work_unit_minutes,
                );
                if capped_units < total_ot_units {
                    tracing::info!(
                        employee_id = %employee_id,
                        "Monthly OT capped at {} units (max {} hours)",
                        capped_units,
                        max_monthly_hours
                    );
                }
                total_ot_units = capped_units;
                total_overtime_pay = capped_pay;
            }
        }

        // 7. Incentives Calculation
        let active_policies = payroll_repository::get_active_incentive_policies(pool, workspace_id)
            .await
            .unwrap_or_default();
        let mut incentive_results = Vec::new();
        let mut total_incentive_amount = Decimal::ZERO;

        // Fetch leave breakdown once for the entire period
        let leave_breakdown = leave_request_repository::get_approved_leave_breakdown(
            pool,
            employee_id,
            ctx.start_date,
            ctx.end_date,
            workspace_id,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch leave breakdown for employee {}: {}",
                employee_id,
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        if !active_policies.is_empty() {
            let context = IncentiveContext {
                total_work_units: total_std_units,
                total_leave_days: leave_breakdown.total_days,
                unpaid_leave_days: leave_breakdown.unpaid_days,
                sick_leave_days: leave_breakdown.sick_days,
                late_early_count: total_late_early_count,
            };

            for policy in active_policies {
                let tiers = payroll_repository::get_policy_tiers(pool, policy.id)
                    .await
                    .unwrap_or_default();
                let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &context);
                if !res.amount.is_zero() {
                    total_incentive_amount += res.amount;
                    incentive_results.push(res);
                }
            }
        }

        // 8. Deductions Calculation (Unpaid Leave)
        // For simplicity, we calculate deduction based on the LATEST segment's daily rate
        let mut deductions_amount = Decimal::ZERO;
        if leave_breakdown.unpaid_days > 0.0 {
            if let Some(last_snap) = segment_snapshots.last() {
                let daily_rate =
                    Self::calculate_daily_rate(last_snap.total_base, last_snap.std_working_days);
                deductions_amount = daily_rate
                    * Decimal::from_f64(leave_breakdown.unpaid_days).unwrap_or(Decimal::ZERO);
            }
        }

        // 8b. Deductions for unpaid holidays (is_paid=false) where employee has NO leave request
        // These are days the employee didn't work AND had no leave deducted → unpaid day
        let unpaid_holiday_days: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!"
               FROM holidays h
               WHERE h.workspace_id = $1
                 AND h.is_paid = false
                 AND h.date >= $2
                 AND h.date <= $3
                 AND NOT EXISTS (
                     SELECT 1 FROM leave_requests lr
                     WHERE lr.employee_id = $4
                       AND lr.status = 'approved'
                       AND lr.start_date <= h.date
                       AND lr.end_date >= h.date
                 )"#,
            workspace_id,
            ctx.start_date,
            ctx.end_date,
            employee_id
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if unpaid_holiday_days > 0 {
            if let Some(last_snap) = segment_snapshots.last() {
                let daily_rate =
                    Self::calculate_daily_rate(last_snap.total_base, last_snap.std_working_days);
                deductions_amount += daily_rate * Decimal::from(unpaid_holiday_days);
            }
        }

        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id,
            period_start: ctx.start_date,
            period_end: ctx.end_date,
            base_pay_amount: total_base_pay,
            overtime_pay_amount: total_overtime_pay,
            deductions_amount,
            holiday_work_premium_amount: total_holiday_work_premium,
            net_pay_amount: total_base_pay + total_overtime_pay + total_incentive_amount
                + total_holiday_work_premium - deductions_amount,
            standard_units: total_std_units,
            overtime_units: total_ot_units,
            status: "draft".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Self::persist_calculated_payslip(pool, payslip, segment_snapshots, incentive_results).await
    }

    /// Helper extracted to move Database operations out of the calculation flow
    #[allow(clippy::too_many_arguments)]
    async fn persist_calculated_payslip(
        pool: &PgPool,
        payslip: Payslip,
        segment_snapshots: Vec<SegmentSnapshot>,
        incentive_results: Vec<crate::models::IncentiveEvaluationResult>,
    ) -> Result<Payslip, StatusCode> {
        let created_payslip = payroll_repository::create_payslip(pool, &payslip)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for snap in segment_snapshots {
            payroll_repository::create_payslip_segment(
                pool,
                created_payslip.id,
                snap.seg_start,
                snap.seg_end,
                snap.department_id,
                snap.total_base,
                snap.std_working_days,
                snap.hours_per_day,
                snap.actual_worked_days,
                snap.paid_days,
                snap.amount,
                Some(snap.source),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        for res in incentive_results {
            let _ = payroll_repository::create_payslip_incentive(pool, created_payslip.id, &res).await;
        }

        Ok(created_payslip)
    }

    pub async fn calculate_batch_payslips(
        pool: &PgPool,
        employee_ids: Vec<Uuid>,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> crate::models::BatchCalculatePayrollResponse {
        const MAX_CONCURRENT_PAYROLL: usize = 10;

        let results: Vec<_> = stream::iter(employee_ids)
            .map(|emp_id| {
                let pool_clone = pool.clone();
                async move {
                    match Self::calculate_monthly_payslip(
                        &pool_clone,
                        emp_id,
                        workspace_id,
                        year,
                        month,
                    )
                    .await
                    {
                        Ok(payslip) => crate::models::BatchCalculateResult {
                            employee_id: emp_id,
                            status: "success".to_string(),
                            payslip: Some(payslip),
                            error_message: None,
                        },
                        Err(status) => crate::models::BatchCalculateResult {
                            employee_id: emp_id,
                            status: "error".to_string(),
                            payslip: None,
                            error_message: Some(format!("HTTP Error: {status}")),
                        },
                    }
                }
            })
            .buffer_unordered(MAX_CONCURRENT_PAYROLL)
            .collect()
            .await;

        crate::models::BatchCalculatePayrollResponse { results }
    }
}

use crate::utils::chrono_helpers::local_to_utc;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).expect("valid date")
    }

    fn make_salary_entry(effective_date: NaiveDate) -> (crate::models::EmployeeSalaryComponent, crate::models::SalaryComponentDefinition) {
        (
            crate::models::EmployeeSalaryComponent {
                id: Uuid::new_v4(),
                employee_id: Uuid::new_v4(),
                component_definition_id: Uuid::new_v4(),
                amount: Decimal::from(1000),
                effective_date,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            crate::models::SalaryComponentDefinition {
                id: Uuid::new_v4(),
                workspace_id: Uuid::new_v4(),
                name: "Base".into(),
                code: "BASE".into(),
                is_base_for_ot: true,
                is_taxable: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        )
    }

    fn make_dept_entry(effective_date: NaiveDate) -> crate::models::EmployeeDepartmentHistory {
        crate::models::EmployeeDepartmentHistory {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            department_id: Uuid::new_v4(),
            effective_date,
            notes: None,
            created_at: Utc::now(),
        }
    }

    fn make_ctx(
        start_date: NaiveDate,
        end_date: NaiveDate,
        salary_history: Vec<(crate::models::EmployeeSalaryComponent, crate::models::SalaryComponentDefinition)>,
        all_dept_history: Vec<crate::models::EmployeeDepartmentHistory>,
    ) -> PayrollContext {
        PayrollContext {
            workspace: crate::models::Workspace {
                id: Uuid::new_v4(),
                name: "Test".into(),
                work_unit_minutes: 30,
                rounding_policy: "floor".into(),
                date_format: "YYYY-MM-DD".into(),
                time_format: "HH:mm".into(),
                timezone: "UTC".into(),
                theme_primary_color: "#000".into(),
                default_language: "en".into(),
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Allow,
                holiday_overnight_strategy: "split".into(),
                default_holiday_ot_multiplier: Decimal::from(2),
                created_at: Utc::now(),
            },
            tz: chrono_tz::UTC,
            start_date,
            end_date,
            salary_history,
            all_dept_history,
            daily_entries_map: HashMap::new(),
            holidays_map: HashMap::new(),
            roster_data: Arc::new(crate::cache::RosterData {
                shifts: HashMap::new(),
                fixed_assignments: HashMap::new(),
                pattern_assignments: HashMap::new(),
                dept_patterns: HashMap::new(),
                dept_fixed_shifts: HashMap::new(),
                active_rules: Vec::new(),
                rule_patterns: HashMap::new(),
                dept_history: HashMap::new(),
            }),
            emp_id_number: "EMP001".into(),
            emp_tags: None,
            payroll_roster_map: HashMap::new(),
        }
    }

    #[test]
    fn test_build_segments_no_changes() {
        let ctx = make_ctx(d(2026, 3, 1), d(2026, 3, 31), vec![], vec![]);
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_salary_change_mid_month() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 16))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 15)));
        assert_eq!(segments[1], (d(2026, 3, 16), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_dept_change_mid_month() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![],
            vec![make_dept_entry(d(2026, 3, 10))],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 9)));
        assert_eq!(segments[1], (d(2026, 3, 10), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_multiple_changes() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 10))],
            vec![make_dept_entry(d(2026, 3, 20))],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 9)));
        assert_eq!(segments[1], (d(2026, 3, 10), d(2026, 3, 19)));
        assert_eq!(segments[2], (d(2026, 3, 20), d(2026, 3, 31)));
    }

    #[test]
    fn test_build_segments_change_on_first_day_no_extra_segment() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 1))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1); // change on start_date doesn't create extra segment
    }

    #[test]
    fn test_build_segments_change_after_end_ignored() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 4, 5))],
            vec![],
        );
        let segments = PayrollService::build_segments(&ctx);
        assert_eq!(segments.len(), 1); // change outside range skipped
    }

    #[test]
    fn test_build_segments_same_day_salary_and_dept() {
        let ctx = make_ctx(
            d(2026, 3, 1), d(2026, 3, 31),
            vec![make_salary_entry(d(2026, 3, 15))],
            vec![make_dept_entry(d(2026, 3, 15))],
        );
        let segments = PayrollService::build_segments(&ctx);
        // Same date → deduplicated by BTreeSet
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], (d(2026, 3, 1), d(2026, 3, 14)));
        assert_eq!(segments[1], (d(2026, 3, 15), d(2026, 3, 31)));
    }

    // --- Tests for calculate_daily_rate ---

    #[test]
    fn test_daily_rate_normal() {
        // Base salary 26_000 / 26 working days = 1_000 per day
        let rate = PayrollService::calculate_daily_rate(
            Decimal::from(26_000),
            Decimal::from(26),
        );
        assert_eq!(rate, Decimal::from(1_000));
    }

    #[test]
    fn test_daily_rate_zero_working_days_returns_zero() {
        // Edge case: zero working days should not panic/divide-by-zero
        let rate = PayrollService::calculate_daily_rate(
            Decimal::from(30_000),
            Decimal::ZERO,
        );
        assert_eq!(rate, Decimal::ZERO);
    }

    #[test]
    fn test_daily_rate_zero_base_returns_zero() {
        let rate = PayrollService::calculate_daily_rate(
            Decimal::ZERO,
            Decimal::from(26),
        );
        assert_eq!(rate, Decimal::ZERO);
    }

    // --- Tests for apply_ot_cap ---

    #[test]
    fn test_ot_cap_no_cap_needed() {
        // 10 OT units, cap allows 20 hours (= 40 units at 30-min work_unit)
        let (units, pay) = PayrollService::apply_ot_cap(
            10,                   // total_ot_units
            Decimal::from(5_000), // total_overtime_pay
            Decimal::from(20),    // max_monthly_hours = 20h
            30,                   // work_unit_minutes = 30 min
        );
        // 20h * 60 = 1200 min / 30 = 40 max units; 10 < 40 → no cap
        assert_eq!(units, 10);
        assert_eq!(pay, Decimal::from(5_000));
    }

    #[test]
    fn test_ot_cap_applied() {
        // 50 OT units exceed cap of 40 → should be reduced
        let (units, pay) = PayrollService::apply_ot_cap(
            50,                    // total_ot_units
            Decimal::from(10_000), // total_overtime_pay
            Decimal::from(20),     // max_monthly_hours = 20h
            30,                    // work_unit_minutes = 30 min
        );
        // Max units = 20*60/30 = 40
        assert_eq!(units, 40);
        // Pay proportional: 10_000 * (40/50) = 8_000
        assert_eq!(pay, Decimal::from(8_000));
    }

    #[test]
    fn test_ot_cap_exact_limit() {
        let (units, pay) = PayrollService::apply_ot_cap(
            40,                   // exactly at limit
            Decimal::from(8_000),
            Decimal::from(20),    // 20h * 60 / 30 = 40
            30,
        );
        assert_eq!(units, 40); // no reduction
        assert_eq!(pay, Decimal::from(8_000));
    }

    #[test]
    fn test_ot_cap_zero_units() {
        let (units, pay) = PayrollService::apply_ot_cap(
            0,
            Decimal::ZERO,
            Decimal::from(20),
            30,
        );
        assert_eq!(units, 0);
        assert_eq!(pay, Decimal::ZERO);
    }

    // --- Tests for SegmentSnapshot struct ---

    #[test]
    fn test_segment_snapshot_construction() {
        let snap = SegmentSnapshot {
            seg_start: d(2026, 3, 1),
            seg_end: d(2026, 3, 15),
            department_id: Some(Uuid::new_v4()),
            total_base: Decimal::from(13_000),
            std_working_days: Decimal::from(26),
            hours_per_day: Decimal::from(8),
            actual_worked_days: Decimal::from(11),
            paid_days: Decimal::from(11),
            amount: Decimal::from(5_500),
            source: "workspace".to_string(),
        };

        assert_eq!(snap.seg_start, d(2026, 3, 1));
        assert_eq!(snap.seg_end, d(2026, 3, 15));
        assert!(snap.department_id.is_some());
        assert_eq!(snap.source, "workspace");
    }

    #[test]
    fn test_segment_snapshot_daily_rate_via_helper() {
        let snap = SegmentSnapshot {
            seg_start: d(2026, 3, 1),
            seg_end: d(2026, 3, 31),
            department_id: None,
            total_base: Decimal::from(26_000),
            std_working_days: Decimal::from(26),
            hours_per_day: Decimal::from(8),
            actual_worked_days: Decimal::from(22),
            paid_days: Decimal::from(22),
            amount: Decimal::from(22_000),
            source: "system".to_string(),
        };

        let daily_rate = PayrollService::calculate_daily_rate(
            snap.total_base,
            snap.std_working_days,
        );
        assert_eq!(daily_rate, Decimal::from(1_000));
    }
}
