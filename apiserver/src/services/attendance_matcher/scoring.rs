use super::*;
use crate::cache::RosterInterval;
use crate::domain::policy::defaults::DEFAULT_OT_MULTIPLIER;
use crate::domain::policy::overtime::OvertimeEngine;
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides, StandardMonthConfig};
use crate::models::TimeEntry;
use crate::services::attendance_calculator::DailyCalcInput;
use crate::services::AttendanceCalculator;
use chrono::{NaiveDate, TimeZone};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

/// Compute OT pay for a single entry that generated overtime units.
///
/// Resolves the employee's OT policy (with caching), calculates the hourly
/// rate, then applies early-OT and late-OT (capped by `ot_ceiling`) using
/// `OvertimeEngine`. Falls back to `DEFAULT_OT_MULTIPLIER` when no policy
/// is assigned.
#[allow(clippy::too_many_arguments)]
fn compute_ot_pay_for_entry(
    ctx: &AttendanceContext,
    employee_id: Uuid,
    current_date: NaiveDate,
    punch_in: chrono::DateTime<chrono_tz::Tz>,
    punch_out: chrono::DateTime<chrono_tz::Tz>,
    shift_start: chrono::DateTime<chrono_tz::Tz>,
    shift_end: chrono::DateTime<chrono_tz::Tz>,
    ot_ceiling: Option<chrono::DateTime<chrono_tz::Tz>>,
    ot_units: i32,
    segment_ot_base: Decimal,
    std_config: &StandardMonthConfig,
    work_unit_minutes: i32,
    policy_cache: &mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
) -> Decimal {
    if let Some(policy_id) = ctx.overtime_data.resolve_policy_for_employee(
        employee_id,
        current_date,
        &ctx.all_dept_history,
    ) {
        let rules = if let Some(r) = policy_cache.get(&policy_id) {
            r.clone()
        } else {
            let r = ctx
                .overtime_data
                .policies
                .get(&policy_id)
                .map(|(_, rules)| rules.clone())
                .unwrap_or_default();
            policy_cache.insert(policy_id, r.clone());
            r
        };

        let hourly_rate = FinancialCalculator::calculate_hourly_rate(
            segment_ot_base,
            std_config.std_working_days(),
            std_config.hours_per_day(),
        );

        let mut total_ot_pay = Decimal::ZERO;

        // Early OT: punched in before shift start
        if punch_in < shift_start {
            let ot_end = std::cmp::min(punch_out, shift_start);
            let daily_splits = OvertimeEngine::split_interval_by_day(&punch_in, &ot_end, &ctx.tz);
            for (date, intervals) in daily_splits {
                let day_is_holiday = ctx.holidays_map.contains_key(&date);
                total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(
                    &rules,
                    date,
                    day_is_holiday,
                    &intervals,
                    hourly_rate,
                );
            }
        }

        // Late OT: punched out after shift end (capped by next shift or ot_ceiling)
        if punch_out > shift_end {
            let ot_start = std::cmp::max(punch_in, shift_end);
            let ot_end_for_pay = match ot_ceiling {
                Some(ceil) => std::cmp::min(punch_out, ceil),
                None => punch_out,
            };
            let daily_splits =
                OvertimeEngine::split_interval_by_day(&ot_start, &ot_end_for_pay, &ctx.tz);
            for (date, intervals) in daily_splits {
                let day_is_holiday = ctx.holidays_map.contains_key(&date);
                total_ot_pay += OvertimeEngine::calculate_overtime_pay_from_times(
                    &rules,
                    date,
                    day_is_holiday,
                    &intervals,
                    hourly_rate,
                );
            }
        }

        total_ot_pay
    } else {
        // No OT policy assigned — use default multiplier
        let hourly_rate = FinancialCalculator::calculate_hourly_rate(
            segment_ot_base,
            std_config.std_working_days(),
            std_config.hours_per_day(),
        );
        (Decimal::from(ot_units) * Decimal::from(work_unit_minutes) / Decimal::from(60))
            * hourly_rate
            * Decimal::from_f64(DEFAULT_OT_MULTIPLIER).unwrap_or_else(|| Decimal::new(15, 1))
    }
}

impl AttendanceMatcher {
    /// Process a single time-entry matched to a shift interval.
    ///
    /// Resolves effective attendance rules (with shift overrides), calculates
    /// daily standard + OT units, applies holiday work compensation, computes
    /// OT pay, and checks for late/early violations.
    ///
    /// Returns `(std_u, ot_u, ot_pay_delta, holiday_premium_delta, is_violation)`.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub(super) fn process_matched_entry(
        ctx: &AttendanceContext,
        employee_id: Uuid,
        entry: &TimeEntry,
        shift_start: chrono::DateTime<chrono_tz::Tz>,
        shift_end: chrono::DateTime<chrono_tz::Tz>,
        interval: &RosterInterval,
        current_date: NaiveDate,
        actual_break_minutes: i32,
        ot_ceiling: Option<chrono::DateTime<chrono_tz::Tz>>,
        attendance_rules: &crate::models::AttendanceRules,
        seg_units_per_day: Decimal,
        segment_total_base: Decimal,
        segment_ot_base: Decimal,
        std_config: &StandardMonthConfig,
        policy_cache: &mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
    ) -> (i32, i32, Decimal, Decimal, bool) {
        let punch_in = entry.punch_in_time().with_timezone(&ctx.tz);
        let punch_out_raw = entry.punch_out_time();
        let punch_out = match punch_out_raw {
            Some(po) => po.with_timezone(&ctx.tz),
            None => return (0, 0, Decimal::ZERO, Decimal::ZERO, false),
        };

        // Resolve effective attendance rules with shift-level overrides
        let rules = PolicyResolver::resolve_with_shift_overrides_pure(
            Some(attendance_rules.clone()),
            ctx.working_conditions.clone(),
            current_date,
            Some(&ctx.condition_type_catalog),
            &ShiftPolicyOverrides::new(
                Some(interval.early_punch_in_grace_period_minutes),
                Some(interval.is_break_auto_deduct),
                Some(interval.allow_early_ot),
                attendance_rules.early_arrival_strategy,
                interval.break_is_overtime || entry.is_break_overtime(),
            ),
        );

        // Construct break times as timezone-aware datetimes
        let break_start = interval.break_start.and_then(|t| {
            ctx.tz
                .from_local_datetime(&current_date.and_time(t))
                .single()
        });
        let break_end = interval.break_end.and_then(|t| {
            let mut dt = ctx
                .tz
                .from_local_datetime(&current_date.and_time(t))
                .single()?;
            if interval.break_end < interval.break_start {
                dt += chrono::Duration::days(1);
            }
            Some(dt)
        });

        let (std_u, ot_u) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in,
            punch_out,
            shift_start: Some(shift_start),
            shift_end: Some(shift_end),
            scheduled_break_minutes: interval.break_duration_minutes,
            actual_break_minutes,
            break_start,
            break_end,
            work_unit: attendance_rules.work_unit_minutes,
            rounding_policy: attendance_rules.rounding_policy,
            rules: rules.clone(),
            ot_ceiling,
            max_early_ot_minutes: interval.max_early_ot_minutes,
            segment_id: entry.segment_id(),
            ot_aggregation_mode: interval.ot_aggregation_mode,
        });

        // ── Holiday work compensation ──────────────────────────────
        let mut holiday_premium = Decimal::ZERO;
        let mut std_extra = 0i32;

        if let Some(holiday) = ctx.holidays_map.get(&current_date) {
            let daily_rate = FinancialCalculator::calculate_daily_rate(
                segment_total_base,
                std_config.std_working_days(),
            );
            if holiday.is_credit_on_work() {
                std_extra += seg_units_per_day.to_i32().unwrap_or(0);
            }
            let multiplier = holiday.ot_multiplier().unwrap_or_else(|| Decimal::from(3));
            if multiplier > Decimal::ONE {
                let worked_fraction = Decimal::from(std_u) / seg_units_per_day;
                holiday_premium += worked_fraction * daily_rate * (multiplier - Decimal::ONE);
            }
        }

        // ── OT pay ─────────────────────────────────────────────────
        let ot_pay_delta = if ot_u > 0 {
            compute_ot_pay_for_entry(
                ctx,
                employee_id,
                current_date,
                punch_in,
                punch_out,
                shift_start,
                shift_end,
                ot_ceiling,
                ot_u,
                segment_ot_base,
                std_config,
                attendance_rules.work_unit_minutes,
                policy_cache,
            )
        } else {
            Decimal::ZERO
        };

        // ── Violation detection ────────────────────────────────────
        let is_holiday = ctx.holidays_map.contains_key(&current_date);
        let is_violation = Self::check_late_early(
            punch_in,
            punch_out,
            shift_start,
            shift_end,
            rules.grace_period_minutes(),
            is_holiday,
        );

        (
            std_u + std_extra,
            ot_u,
            ot_pay_delta,
            holiday_premium,
            is_violation,
        )
    }

    /// Process all entries for a single calendar day — filter work/break
    /// entries, iterate shifts, match entries to shifts, and delegate
    /// per-entry computation to [`process_matched_entry`].
    ///
    /// Returns `(std_units, ot_units, ot_pay, holiday_premium, violation_count)`.
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub(super) fn process_daily_entries(
        ctx: &AttendanceContext,
        employee_id: Uuid,
        current_date: NaiveDate,
        day_shifts: &[(Uuid, &RosterInterval)],
        entries: &[TimeEntry],
        attendance_rules: &crate::models::AttendanceRules,
        seg_units_per_day: Decimal,
        segment_total_base: Decimal,
        segment_ot_base: Decimal,
        std_config: &StandardMonthConfig,
        policy_cache: &mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
    ) -> (i32, i32, Decimal, Decimal, i32) {
        let work_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type() == crate::services::pairing::types::EntryType::Work)
            .collect();
        let break_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type() == crate::services::pairing::types::EntryType::Break)
            .collect();
        let actual_break_minutes: i32 = break_entries
            .iter()
            .map(|e| e.duration_minutes().unwrap_or(0))
            .sum();

        let mut day_std = 0i32;
        let mut day_ot = 0i32;
        let mut day_ot_pay = Decimal::ZERO;
        let mut day_holiday_premium = Decimal::ZERO;
        let mut day_violations = 0i32;

        // Process each shift with OT ceiling = next shift's start
        for (shift_idx, (_shift_id, interval)) in day_shifts.iter().enumerate() {
            let ot_ceiling = day_shifts.get(shift_idx + 1).and_then(|(_, next_i)| {
                ctx.tz
                    .from_local_datetime(&current_date.and_time(next_i.start_time))
                    .single()
            });

            let start_dt = ctx
                .tz
                .from_local_datetime(&current_date.and_time(interval.start_time))
                .single();
            let end_dt = ctx
                .tz
                .from_local_datetime(&current_date.and_time(interval.end_time))
                .single();

            if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                if interval.end_time < interval.start_time {
                    end += chrono::Duration::days(1);
                }

                // Match work entries to this shift via time-window heuristic
                let matching_entries: Vec<_> = work_entries
                    .iter()
                    .filter(|entry| {
                        let punch_in = entry.punch_in_time().with_timezone(&ctx.tz);
                        let prev_end = if shift_idx > 0 {
                            day_shifts.get(shift_idx - 1).and_then(|(_, prev_i)| {
                                ctx.tz
                                    .from_local_datetime(&current_date.and_time(prev_i.end_time))
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
                    let (std_u, ot_u, ot_pay_delta, premium, violation) =
                        Self::process_matched_entry(
                            ctx,
                            employee_id,
                            entry,
                            start,
                            end,
                            interval,
                            current_date,
                            actual_break_minutes,
                            ot_ceiling,
                            attendance_rules,
                            seg_units_per_day,
                            segment_total_base,
                            segment_ot_base,
                            std_config,
                            policy_cache,
                        );
                    day_std += std_u;
                    day_ot += ot_u;
                    day_ot_pay += ot_pay_delta;
                    day_holiday_premium += premium;
                    if violation {
                        day_violations += 1;
                    }
                }
            }
        }

        // Fallback: entries with no matching shift (day_shifts is empty)
        if day_shifts.is_empty() {
            for entry in &work_entries {
                let punch_in = entry.punch_in_time().with_timezone(&ctx.tz);
                if let Some(punch_out_raw) = entry.punch_out_time() {
                    let punch_out = punch_out_raw.with_timezone(&ctx.tz);
                    day_std += AttendanceCalculator::calculate_units(
                        i32::try_from((punch_out - punch_in).num_minutes()).unwrap_or(0),
                        attendance_rules.work_unit_minutes,
                        attendance_rules.rounding_policy,
                    );
                }
            }
        }

        (
            day_std,
            day_ot,
            day_ot_pay,
            day_holiday_premium,
            day_violations,
        )
    }

    /// Auto-credit standard units for a paid holiday with no attendance.
    ///
    /// Returns unit count — half for half-day holidays, full otherwise.
    pub(super) fn credit_paid_holiday(
        holiday: &crate::models::Holiday,
        seg_units_per_day: Decimal,
    ) -> i32 {
        if holiday.holiday_type() == "half_day" {
            (seg_units_per_day / Decimal::from(2)).to_i32().unwrap_or(0)
        } else {
            seg_units_per_day.to_i32().unwrap_or(0)
        }
    }
}
