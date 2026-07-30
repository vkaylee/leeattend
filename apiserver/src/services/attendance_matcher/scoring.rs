use super::*;
use crate::cache::RosterInterval;
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides, StandardMonthConfig};
use crate::models::TimeEntry;
use chrono::{NaiveDate, TimeZone};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

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
        ctx: &AttendanceMatcherContext,
        _employee_id: Uuid,
        entry: &TimeEntry,
        shift_start: chrono::DateTime<chrono_tz::Tz>,
        shift_end: chrono::DateTime<chrono_tz::Tz>,
        interval: &RosterInterval,
        current_date: NaiveDate,
        _actual_break_minutes: i32,
        _ot_ceiling: Option<chrono::DateTime<chrono_tz::Tz>>,
        attendance_rules: &crate::models::AttendanceRules,
        _seg_units_per_day: Decimal,
        _segment_total_base: Decimal,
        _segment_ot_base: Decimal,
        _std_config: &StandardMonthConfig,
        _policy_cache: &mut HashMap<Uuid, Vec<crate::models::OvertimeRule>>,
    ) -> (i32, i32, Decimal, Decimal, bool) {
        let punch_in = entry.punch_in_time().with_timezone(&ctx.tz());
        let punch_out_raw = entry.punch_out_time();
        let punch_out = match punch_out_raw {
            Some(po) => po.with_timezone(&ctx.tz()),
            None => return (0, 0, Decimal::ZERO, Decimal::ZERO, false),
        };

        // Resolve effective attendance rules with shift-level overrides
        let rules = PolicyResolver::resolve_with_shift_overrides_pure(
            Some(attendance_rules.clone()),
            ctx.working_conditions().to_vec(),
            current_date,
            Some(ctx.condition_type_catalog()),
            &ShiftPolicyOverrides::new(
                Some(interval.early_punch_in_grace_period_minutes),
                Some(interval.is_break_auto_deduct),
                Some(interval.allow_early_ot),
                attendance_rules.early_arrival_strategy,
                interval.break_is_overtime || entry.is_break_overtime(),
            ),
        );

        // ── Units are now read from persisted attendance_daily_summaries ──
        // (Previously computed via AttendanceCalculator::calculate_daily_units here.
        //  Phase 7 of persist-workbuckets reads them from the summary table.)
        //
        // Holiday work premium and paid-holiday auto-credit are also handled
        // in calculate_payslip_with_context from summary data.

        // ── Violation detection ────────────────────────────────────
        let is_holiday = ctx.holidays_map().contains_key(&current_date);
        let is_violation = Self::check_late_early(
            punch_in,
            punch_out,
            shift_start,
            shift_end,
            rules.grace_period_minutes(),
            is_holiday,
        );

        (
            0,  // std_units — now from summaries (Phase 7)
            0,  // ot_units — now from summaries (Phase 7)
            Decimal::ZERO,  // ot_pay — kept for OT pay chain (Formula Engine)
            Decimal::ZERO,  // holiday_premium — now from summaries (Phase 7)
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
        ctx: &AttendanceMatcherContext,
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

        let mut day_ot_pay = Decimal::ZERO;
        let mut day_holiday_premium = Decimal::ZERO;
        let mut day_violations = 0i32;

        // Process each shift with OT ceiling = next shift's start
        for (shift_idx, (_shift_id, interval)) in day_shifts.iter().enumerate() {
            let ot_ceiling = day_shifts.get(shift_idx + 1).and_then(|(_, next_i)| {
                ctx.tz()
                    .from_local_datetime(&current_date.and_time(next_i.start_time))
                    .single()
            });

            let start_dt = ctx
                .tz()
                .from_local_datetime(&current_date.and_time(interval.start_time))
                .single();
            let end_dt = ctx
                .tz()
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
                        let punch_in = entry.punch_in_time().with_timezone(&ctx.tz());
                        let prev_end = if shift_idx > 0 {
                            day_shifts.get(shift_idx - 1).and_then(|(_, prev_i)| {
                                ctx.tz()
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
                    let (_std_u, _ot_u, ot_pay_delta, premium, violation) =
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
                    day_ot_pay += ot_pay_delta;
                    day_holiday_premium += premium;
                    if violation {
                        day_violations += 1;
                    }
                }
            }
        }

        (
            0,  // day_std — now from summaries
            0,  // day_ot — now from summaries
            day_ot_pay,
            day_holiday_premium,
            day_violations,
        )
    }
}
