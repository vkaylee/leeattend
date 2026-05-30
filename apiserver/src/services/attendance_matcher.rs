//! AttendanceMatcher — Shift resolution, entry-to-shift matching,
//! daily unit calculation, and holiday credit logic.
//!
//! Extracted from `PayrollService::calculate_monthly_payslip` to satisfy SRP.

use crate::domain::policy::defaults::DEFAULT_OT_MULTIPLIER;
use crate::domain::policy::overtime::OvertimeEngine;
use crate::domain::policy::resolver::PolicyResolver;
use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use crate::repositories::roster::shift_resolution::resolve_effective_shift_from_data;
use crate::services::AttendanceCalculator;
use chrono::{NaiveDate, TimeZone};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

use super::financial_calculator::FinancialCalculator;
use super::payroll_service::PayrollContext;

/// Result of processing attendance for a single payroll segment.
pub struct SegmentAttendanceResult {
    pub std_units: i32,
    pub ot_units: i32,
    pub ot_pay: Decimal,
    pub holiday_work_premium: Decimal,
    pub late_early_count: i32,
}

use std::collections::HashMap;
use uuid::Uuid;

pub struct ProcessSegmentAttendanceArgs<'a> {
    pub ctx: &'a PayrollContext,
    pub seg_start: NaiveDate,
    pub seg_end: NaiveDate,
    pub segment_total_base: Decimal,
    pub segment_ot_base: Decimal,
    pub std_config: &'a crate::domain::policy::resolver::StandardMonthConfig,
    pub attendance_rules: &'a crate::models::AttendanceRules,
    pub seg_units_per_day: Decimal,
    pub policy_cache: &'a mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
    pub policy_repo: &'a dyn crate::repositories::AttendancePolicyRepository,
    pub employee_repo: &'a dyn crate::repositories::employee::EmployeeRepository,
    pub overtime_policy_repo: &'a dyn OvertimePolicyRepositoryTrait,
}

pub struct AttendanceMatcher;

impl AttendanceMatcher {
    /// Process daily attendance for a payroll segment date range.
    ///
    /// Iterates each calendar day in `[seg_start, seg_end]`, resolves shifts,
    /// matches time‑entries to shifts, calculates standard/OT units, holiday
    /// premiums, and OT pay.
    #[tracing::instrument(skip_all)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_segment_attendance(
        args: ProcessSegmentAttendanceArgs<'_>,
    ) -> SegmentAttendanceResult {
        let employee_id = args.ctx.employee_id;
        let workspace_id = args.ctx.workspace.id;

        let mut segment_std_units = 0i32;
        let mut segment_ot_units = 0i32;
        let mut segment_ot_pay = Decimal::ZERO;
        let mut holiday_work_premium = Decimal::ZERO;
        let mut late_early_count = 0i32;

        let mut current_date = args.seg_start;
        while current_date <= args.seg_end {
            // Resolve shift from pre-loaded roster data (in-memory, zero DB queries)
            let (shift_ids, _, _) = resolve_effective_shift_from_data(
                &args.ctx.roster_data,
                &args.ctx.payroll_roster_map,
                employee_id,
                &args.ctx.emp_id_number,
                args.ctx.emp_tags.as_ref(),
                current_date,
            );

            // Collect resolved shifts sorted by start_time for OT ceiling
            let mut day_shifts: Vec<_> = shift_ids
                .iter()
                .filter_map(|opt| opt.and_then(|id| args.ctx.roster_data.shifts.get(&id)))
                .collect();
            day_shifts.sort_by_key(|s| s.start_time);

            if let Some(entries) = args.ctx.daily_entries_map.get(&current_date) {
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
                        args.ctx
                            .tz
                            .from_local_datetime(&current_date.and_time(next_s.start_time))
                            .single()
                    });

                    let start_dt = args
                        .ctx
                        .tz
                        .from_local_datetime(&current_date.and_time(s.start_time))
                        .single();
                    let end_dt = args
                        .ctx
                        .tz
                        .from_local_datetime(&current_date.and_time(s.end_time))
                        .single();

                    if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                        if s.end_time < s.start_time {
                            end += chrono::Duration::days(1);
                        }

                        // Find the work entry that belongs to this shift
                        let matching_entries: Vec<_> = work_entries
                            .iter()
                            .filter(|entry| {
                                let punch_in = entry.punch_in_time.with_timezone(&args.ctx.tz);
                                let prev_end = if shift_idx > 0 {
                                    day_shifts.get(shift_idx - 1).and_then(|prev_s| {
                                        args.ctx
                                            .tz
                                            .from_local_datetime(
                                                &current_date.and_time(prev_s.end_time),
                                            )
                                            .single()
                                    })
                                } else {
                                    None
                                };
                                let after_prev = prev_end.is_none_or(|pe| punch_in >= pe);
                                let before_end = punch_in < end;
                                after_prev && before_end
                            })
                            .collect();

                        for entry in matching_entries {
                            let punch_in = entry.punch_in_time.with_timezone(&args.ctx.tz);
                            if let Some(punch_out_raw) = entry.punch_out_time {
                                let punch_out = punch_out_raw.with_timezone(&args.ctx.tz);

                                let rules = PolicyResolver::resolve_with_shift_overrides(
                                    args.policy_repo,
                                    args.employee_repo,
                                    employee_id,
                                    args.ctx.workspace.id,
                                    current_date,
                                    &crate::domain::policy::resolver::ShiftPolicyOverrides {
                                        grace_period_minutes: Some(
                                            s.early_punch_in_grace_period_minutes,
                                        ),
                                        is_break_auto_deduct: Some(s.is_break_auto_deduct),
                                        allow_early_ot: Some(s.allow_early_ot),
                                        early_arrival_strategy: args
                                            .attendance_rules
                                            .early_arrival_strategy,
                                        is_break_overtime: entry.is_break_overtime,
                                    },
                                )
                                .await;

                                let (std_u, ot_u) = AttendanceCalculator::calculate_daily_units(
                                    &crate::services::attendance_calculator::DailyCalcInput {
                                        punch_in,
                                        punch_out,
                                        shift_start: Some(start),
                                        shift_end: Some(end),
                                        scheduled_break_minutes: s.break_duration_minutes,
                                        actual_break_minutes,
                                        break_start: s.break_start.and_then(|t| {
                                            args.ctx
                                                .tz
                                                .from_local_datetime(&current_date.and_time(t))
                                                .single()
                                        }),
                                        break_end: s.break_end.and_then(|t| {
                                            let mut dt = args
                                                .ctx
                                                .tz
                                                .from_local_datetime(&current_date.and_time(t))
                                                .single()?;
                                            if s.break_end < s.break_start {
                                                dt += chrono::Duration::days(1);
                                            }
                                            Some(dt)
                                        }),
                                        work_unit: args.attendance_rules.work_unit_minutes,
                                        rounding_policy: args
                                            .attendance_rules
                                            .rounding_policy
                                            .clone(),
                                        rules: rules.clone(),
                                        ot_ceiling,
                                    },
                                );
                                segment_std_units += std_u;
                                segment_ot_units += ot_u;

                                // --- HOLIDAY WORK COMPENSATION ---
                                if let Some(holiday) = args.ctx.holidays_map.get(&current_date) {
                                    let daily_rate = FinancialCalculator::calculate_daily_rate(
                                        args.segment_total_base,
                                        args.std_config.std_working_days,
                                    );
                                    if holiday.is_credit_on_work {
                                        segment_std_units +=
                                            args.seg_units_per_day.to_i32().unwrap_or(0);
                                    }
                                    let multiplier =
                                        holiday.ot_multiplier.unwrap_or_else(|| Decimal::from(3));
                                    if multiplier > Decimal::ONE {
                                        let worked_fraction =
                                            Decimal::from(std_u) / args.seg_units_per_day;
                                        holiday_work_premium += worked_fraction
                                            * daily_rate
                                            * (multiplier - Decimal::ONE);
                                    }
                                }

                                if ot_u > 0 {
                                    if let Some(policy_id) = args
                                        .overtime_policy_repo
                                        .resolve_policy_for_employee(
                                            employee_id,
                                            workspace_id,
                                            current_date,
                                        )
                                        .await
                                        .unwrap_or(None)
                                    {
                                        let rules = if let Some(r) =
                                            args.policy_cache.get(&policy_id)
                                        {
                                            r.clone()
                                        } else {
                                            let r = match args
                                                .overtime_policy_repo
                                                .get_policy_with_rules(workspace_id, policy_id)
                                                .await
                                            {
                                                Ok(Some((_, rules))) => rules,
                                                Ok(None) => Vec::new(),
                                                Err(e) => {
                                                    tracing::warn!(policy_id = %policy_id, error = %e, "Failed to fetch OT policy rules, using empty rules");
                                                    Vec::new()
                                                }
                                            };
                                            args.policy_cache.insert(policy_id, r.clone());
                                            r
                                        };

                                        let hourly_rate =
                                            FinancialCalculator::calculate_hourly_rate(
                                                args.segment_ot_base,
                                                args.std_config.std_working_days,
                                                args.std_config.hours_per_day,
                                            );

                                        let mut total_ot_pay = Decimal::ZERO;
                                        if punch_in < start {
                                            let ot_end = std::cmp::min(punch_out, start);
                                            let daily_splits =
                                                OvertimeEngine::split_interval_by_day(
                                                    &punch_in,
                                                    &ot_end,
                                                    &args.ctx.tz,
                                                );
                                            for (date, intervals) in daily_splits {
                                                let day_is_holiday =
                                                    args.ctx.holidays_map.contains_key(&date);
                                                total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                            }
                                        }
                                        if punch_out > end {
                                            let ot_start = std::cmp::max(punch_in, end);
                                            let ot_end_for_pay = match ot_ceiling {
                                                Some(ceil) => std::cmp::min(punch_out, ceil),
                                                None => punch_out,
                                            };
                                            let daily_splits =
                                                OvertimeEngine::split_interval_by_day(
                                                    &ot_start,
                                                    &ot_end_for_pay,
                                                    &args.ctx.tz,
                                                );
                                            for (date, intervals) in daily_splits {
                                                let day_is_holiday =
                                                    args.ctx.holidays_map.contains_key(&date);
                                                total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, day_is_holiday, &intervals, hourly_rate);
                                            }
                                        }
                                        segment_ot_pay += total_ot_pay;
                                    } else {
                                        let hourly_rate =
                                            FinancialCalculator::calculate_hourly_rate(
                                                args.segment_ot_base,
                                                args.std_config.std_working_days,
                                                args.std_config.hours_per_day,
                                            );
                                        segment_ot_pay += (Decimal::from(ot_u)
                                            * Decimal::from(
                                                args.attendance_rules.work_unit_minutes,
                                            )
                                            / Decimal::from(60))
                                            * hourly_rate
                                            * Decimal::from_f64(DEFAULT_OT_MULTIPLIER)
                                                .unwrap_or_else(|| Decimal::new(15, 1));
                                    }
                                }

                                // Violation detection for IncentiveContext
                                if !args.ctx.holidays_map.contains_key(&current_date) {
                                    let is_late = punch_in
                                        > start
                                            + chrono::Duration::minutes(i64::from(
                                                rules.grace_period_minutes,
                                            ));
                                    let is_early = punch_out < end;
                                    if is_late || is_early {
                                        late_early_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback: entries with no matching shift (day_shifts is empty)
                if day_shifts.is_empty() {
                    for entry in &work_entries {
                        let punch_in = entry.punch_in_time.with_timezone(&args.ctx.tz);
                        if let Some(punch_out_raw) = entry.punch_out_time {
                            let punch_out = punch_out_raw.with_timezone(&args.ctx.tz);
                            segment_std_units += AttendanceCalculator::calculate_units(
                                (punch_out - punch_in).num_minutes() as i32,
                                args.attendance_rules.work_unit_minutes,
                                &args.attendance_rules.rounding_policy,
                            );
                        }
                    }
                }
            } else if let Some(holiday) = args.ctx.holidays_map.get(&current_date) {
                // Paid holiday with no attendance → auto-credit standard day
                if holiday.is_paid {
                    let credit_units = if holiday.holiday_type == "half_day" {
                        (args.seg_units_per_day / Decimal::from(2))
                            .to_i32()
                            .unwrap_or(0)
                    } else {
                        args.seg_units_per_day.to_i32().unwrap_or(0)
                    };
                    segment_std_units += credit_units;
                }
            }
            current_date = match current_date.succ_opt() {
                Some(d) => d,
                None => break,
            };
        }

        SegmentAttendanceResult {
            std_units: segment_std_units,
            ot_units: segment_ot_units,
            ot_pay: segment_ot_pay,
            holiday_work_premium,
            late_early_count,
        }
    }
}
