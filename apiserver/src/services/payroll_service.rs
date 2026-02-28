use axum::http::StatusCode;
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use futures_util::stream::{self, StreamExt};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

use crate::domain::policy::overtime::OvertimeEngine;
use crate::domain::policy::resolver::PolicyResolver;
use crate::models::{Payslip, PolicyRules};
use crate::repositories::overtime_policy_repository::OvertimePolicyRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::repositories::{
    department_repository, leave_request_repository, payroll_repository, roster_repository,
    shift_repository, time_entry_repository, SqlxWorkspaceRepository,
};
use crate::repositories::holiday_repository::HolidayRepository;
use crate::services::{
    payroll_evaluator::{IncentiveContext, PayrollEvaluator},
    AttendanceCalculator,
};

pub struct PayrollService;

impl PayrollService {
    pub async fn calculate_monthly_payslip(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Payslip, StatusCode> {
        // 0. Get workspace settings for Timezone & Rounding
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

        // 1. Define Period
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

        // 2. Fetch History Data for Segmentation
        let salary_history = payroll_repository::get_salary_components_history(
            pool,
            employee_id,
            start_date,
            end_date,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Fetch all dept history
        let all_dept_history =
            department_repository::get_employee_department_history(pool, employee_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4. Fetch Time Entries (All for the month)
        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let time_entries =
            time_entry_repository::get_employee_time_entries(pool, employee_id, start_dt, end_dt)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4b. Fetch holidays for the period (for OT holiday rule filtering)
        let holidays_map = HolidayRepository::get_holidays_in_range(
            pool, workspace_id, start_date, end_date,
        )
        .await
        .unwrap_or_default();

        // 5. Determine Segments
        let mut change_points = BTreeSet::new();
        change_points.insert(start_date);

        for (comp, _) in &salary_history {
            if comp.effective_date > start_date && comp.effective_date <= end_date {
                change_points.insert(comp.effective_date);
            }
        }
        for dept in &all_dept_history {
            if dept.effective_date > start_date && dept.effective_date <= end_date {
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
                end_date
            };
            segments.push((seg_start, seg_end));
        }

        // 6. Calculate Each Segment
        let mut total_base_pay = Decimal::ZERO;
        let mut total_std_units = 0;
        let mut total_ot_units = 0;
        let mut total_late_early_count = 0;
        let mut segment_snapshots = Vec::new();

        // Prepare time entries grouped by date for faster access
        let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
            HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time.with_timezone(&tz).date_naive();
            daily_entries_map.entry(local_date).or_default().push(entry);
        }

        let mut total_overtime_pay = Decimal::ZERO;
        let mut policy_cache: HashMap<Uuid, Vec<crate::models::OvertimeRule>> = HashMap::new();
        for (seg_start, seg_end) in segments {
            // A. Context Resolution
            let active_dept = all_dept_history
                .iter()
                .filter(|d| d.effective_date <= seg_start)
                .max_by_key(|d| d.effective_date);
            let dept_id = active_dept.map(|d| d.department_id);

            let mut segment_total_base = Decimal::ZERO;
            let mut segment_ot_base = Decimal::ZERO;
            let mut processed_def_ids = BTreeSet::new();
            for (esc, scd) in salary_history.iter().rev() {
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
                / Decimal::from(workspace.work_unit_minutes);

            // B. Attendance Calculation
            let mut segment_std_units = 0;
            let mut segment_ot_units = 0;
            let mut segment_ot_pay = Decimal::ZERO;

            let mut current_date = seg_start;
            while current_date <= seg_end {
                // Fetch shift and calculate
                let (shift_id, _) =
                    roster_repository::get_effective_shift(pool, employee_id, current_date)
                        .await
                        .unwrap_or((None, Vec::new()));

                if let Some(entries) = daily_entries_map.get(&current_date) {
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

                    for entry in work_entries {
                        let punch_in = entry.punch_in_time.with_timezone(&tz);
                        if let Some(punch_out_raw) = entry.punch_out_time {
                            let punch_out = punch_out_raw.with_timezone(&tz);
                            if let Some(sid) = shift_id {
                                if let Some(s) = shift_repository::get_shift_by_id(pool, sid)
                                    .await
                                    .unwrap_or(None)
                                {
                                    let start_dt = tz
                                        .from_local_datetime(&current_date.and_time(s.start_time))
                                        .single();
                                    let end_dt = tz
                                        .from_local_datetime(&current_date.and_time(s.end_time))
                                        .single();
                                    if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                                        if s.end_time < s.start_time {
                                            end += chrono::Duration::days(1);
                                        }

                                        let mut rules = PolicyResolver::resolve_attendance_rules(
                                            pool,
                                            employee_id,
                                            workspace.id,
                                            current_date,
                                        )
                                        .await
                                        .unwrap_or(PolicyRules {
                                            grace_period_minutes: s
                                                .early_punch_in_grace_period_minutes,
                                            authorized_early_leave_minutes: 0,
                                            is_early_leave_paid: false,
                                            early_arrival_strategy: workspace
                                                .early_arrival_strategy,
                                            is_break_auto_deduct: s.is_break_auto_deduct,
                                            is_break_overtime: entry.is_break_overtime,
                                            allow_early_ot: s.allow_early_ot,
                                        });

                                        // Override with shift data only if explicitly configured (non-zero)
                                        if s.early_punch_in_grace_period_minutes > 0 {
                                            rules.grace_period_minutes =
                                                s.early_punch_in_grace_period_minutes;
                                        }
                                        rules.is_break_auto_deduct = s.is_break_auto_deduct;
                                        rules.is_break_overtime = entry.is_break_overtime;
                                        rules.allow_early_ot = s.allow_early_ot;

                                        let (std_u, ot_u) =
                                            AttendanceCalculator::calculate_daily_units(
                                                punch_in,
                                                punch_out,
                                                Some(start),
                                                Some(end),
                                                s.break_duration_minutes,
                                                actual_break_minutes,
                                                workspace.work_unit_minutes,
                                                &workspace.rounding_policy,
                                                &rules,
                                            );
                                        segment_std_units += std_u;
                                        segment_ot_units += ot_u;

                                        if ot_u > 0 {
                                            if let Some(policy_id) = OvertimePolicyRepository::resolve_policy_for_employee(pool, employee_id, workspace_id, current_date).await.unwrap_or(None) {
                                                let rules = if let Some(r) = policy_cache.get(&policy_id) {
                                                    r.clone()
                                                } else {
                                                    let r = OvertimePolicyRepository::get_policy_with_rules(pool, policy_id).await.unwrap_or(None).map(|(_, rules)| rules).unwrap_or_default();
                                                    policy_cache.insert(policy_id, r.clone());
                                                    r
                                                };

                                                let hourly_rate = if std_config.std_working_days.is_zero() || std_config.hours_per_day.is_zero() {
                                                    Decimal::ZERO
                                                } else {
                                                    (segment_ot_base / std_config.std_working_days) / std_config.hours_per_day
                                                };

                                                // Handle overnight OT properly using split_interval_by_day
                                                let mut total_ot_pay = Decimal::ZERO;
                                                if punch_in < start {
                                                    let ot_end = std::cmp::min(punch_out, start);
                                                    let daily_splits = OvertimeEngine::split_interval_by_day(punch_in, ot_end, &tz);
                                                    for (date, intervals) in daily_splits {
                                                        let day_is_holiday = holidays_map.contains_key(&date);
                                                        total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                                    }
                                                }
                                                if punch_out > end {
                                                    let ot_start = std::cmp::max(punch_in, end);
                                                    let daily_splits = OvertimeEngine::split_interval_by_day(ot_start, punch_out, &tz);
                                                    for (date, intervals) in daily_splits {
                                                        let day_is_holiday = holidays_map.contains_key(&date);
                                                        total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                                    }
                                                }
                                                segment_ot_pay += total_ot_pay;
                                            } else {
                                                let hourly_rate = if std_config.std_working_days.is_zero() || std_config.hours_per_day.is_zero() { Decimal::ZERO } else { (segment_ot_base / std_config.std_working_days) / std_config.hours_per_day };
                                                segment_ot_pay += (Decimal::from(ot_u) * Decimal::from(workspace.work_unit_minutes) / Decimal::from(60)) * hourly_rate * Decimal::from_f64(1.5).unwrap_or(Decimal::new(15, 1));
                                            }
                                        }

                                        // Violation detection for IncentiveContext
                                        // Skip holidays — employees should not be penalized
                                        // for arriving "late" on a holiday
                                        if !holidays_map.contains_key(&current_date) {
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
                            } else {
                                segment_std_units += AttendanceCalculator::calculate_units(
                                    (punch_out - punch_in).num_minutes() as i32,
                                    workspace.work_unit_minutes,
                                    &workspace.rounding_policy,
                                );
                            }
                        }
                    }
                }
                current_date = match current_date.succ_opt() {
                    Some(d) => d,
                    None => break,
                };
            }

            // C. Financials
            let worked_days_dec = Decimal::from(segment_std_units) / seg_units_per_day;
            let daily_rate = if std_config.std_working_days.is_zero() {
                Decimal::ZERO
            } else {
                segment_total_base / std_config.std_working_days
            };
            let segment_amount = daily_rate * worked_days_dec;

            total_base_pay += segment_amount;
            total_std_units += segment_std_units;
            total_ot_units += segment_ot_units;
            total_overtime_pay += segment_ot_pay;

            segment_snapshots.push((
                seg_start,
                seg_end,
                dept_id,
                segment_total_base,
                std_config.std_working_days,
                std_config.hours_per_day,
                worked_days_dec,
                worked_days_dec,
                segment_amount,
                std_config.source,
            ));
        }

        // 6b. Enforce monthly OT cap from PayrollSettings
        let payroll_settings = crate::repositories::payroll_repository::get_payroll_settings_at(
            pool, workspace_id, end_date,
        )
        .await
        .unwrap_or(None);
        if let Some(ref ps) = payroll_settings {
            if let Some(max_monthly_hours) = ps.max_ot_hours_per_month {
                let max_ot_minutes = max_monthly_hours * Decimal::from(60);
                let max_ot_units_dec = max_ot_minutes / Decimal::from(workspace.work_unit_minutes);
                let max_ot_units = max_ot_units_dec.to_i32().unwrap_or(i32::MAX);
                if total_ot_units > max_ot_units {
                    // Proportionally reduce OT pay
                    if total_ot_units > 0 {
                        let ratio = Decimal::from(max_ot_units) / Decimal::from(total_ot_units);
                        total_overtime_pay = (total_overtime_pay * ratio)
                            .round_dp(0);
                    }
                    total_ot_units = max_ot_units;
                    tracing::info!(
                        employee_id = %employee_id,
                        "Monthly OT capped at {} units (max {} hours)",
                        max_ot_units,
                        max_monthly_hours
                    );
                }
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
            start_date,
            end_date,
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
                let daily_rate = if last_snap.4.is_zero() {
                    Decimal::ZERO
                } else {
                    last_snap.3 / last_snap.4
                };
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
            start_date,
            end_date,
            employee_id
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if unpaid_holiday_days > 0 {
            if let Some(last_snap) = segment_snapshots.last() {
                let daily_rate = if last_snap.4.is_zero() {
                    Decimal::ZERO
                } else {
                    last_snap.3 / last_snap.4
                };
                deductions_amount += daily_rate * Decimal::from(unpaid_holiday_days);
            }
        }

        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id,
            period_start: start_date,
            period_end: end_date,
            base_pay_amount: total_base_pay,
            overtime_pay_amount: total_overtime_pay,
            deductions_amount,
            net_pay_amount: total_base_pay + total_overtime_pay + total_incentive_amount
                - deductions_amount,
            standard_units: total_std_units,
            overtime_units: total_ot_units,
            status: "draft".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_payslip = payroll_repository::create_payslip(pool, &payslip)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for (start, end, dept, base, std, hours, actual, paid, amount, source) in segment_snapshots
        {
            payroll_repository::create_payslip_segment(
                pool,
                created_payslip.id,
                start,
                end,
                dept,
                base,
                std,
                hours,
                actual,
                paid,
                amount,
                Some(source),
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        for res in incentive_results {
            let _ =
                payroll_repository::create_payslip_incentive(pool, created_payslip.id, &res).await;
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

/// Convert local date/time to UTC DateTime, handling DST edge cases
fn local_to_utc(tz: &Tz, date: NaiveDate, h: u32, m: u32, s: u32) -> Result<DateTime<Utc>, String> {
    let naive = date
        .and_hms_opt(h, m, s)
        .ok_or_else(|| format!("Invalid time: {h}:{m}:{s}"))?;
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(dt, _) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::None => {
            // DST gap - use next valid time
            let shifted = naive + chrono::Duration::hours(1);
            tz.from_local_datetime(&shifted)
                .single()
                .map(|dt| dt.with_timezone(&Utc))
                .ok_or_else(|| "Cannot resolve DST gap".to_string())
        }
    }
}
