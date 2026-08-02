use super::*;
use crate::cache::RosterInterval;
use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides};
use crate::models::{Holiday, PolicyRules, TimeEntry, WorkingCondition};
use chrono::{DateTime, NaiveDate};
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

// ── Scoring Context ─────────────────────────────────────────────────────

/// Immutable scoring context bundling shared references for attendance
/// scoring functions. Reduces parameter count across the scoring pipeline.
pub struct ScoringContext<'a> {
    tz: chrono_tz::Tz,
    working_conditions: &'a [WorkingCondition],
    holidays_map: &'a HashMap<NaiveDate, Holiday>,
    condition_type_catalog: &'a HashMap<String, crate::models::ConditionTypeDefinition>,
}

impl<'a> ScoringContext<'a> {
    pub fn new(
        tz: chrono_tz::Tz,
        working_conditions: &'a [WorkingCondition],
        holidays_map: &'a HashMap<NaiveDate, Holiday>,
        condition_type_catalog: &'a HashMap<String, crate::models::ConditionTypeDefinition>,
    ) -> Self {
        Self {
            tz,
            working_conditions,
            holidays_map,
            condition_type_catalog,
        }
    }

    pub fn tz(&self) -> chrono_tz::Tz {
        self.tz
    }

    pub fn holidays_map(&self) -> &HashMap<NaiveDate, Holiday> {
        self.holidays_map
    }
}

// ── Pure Scoring Helpers ─────────────────────────────────────────────────

/// Compute late arrival minutes (zero if on-time or within grace period).
///
/// Returns `0` on holidays — holiday work never counts as a violation.
pub(super) fn compute_late_minutes(
    punch_in: DateTime<chrono_tz::Tz>,
    shift_start: DateTime<chrono_tz::Tz>,
    grace_period_minutes: i32,
    is_holiday: bool,
) -> i64 {
    if is_holiday {
        return 0;
    }
    let threshold = shift_start + chrono::Duration::minutes(i64::from(grace_period_minutes));
    let diff = (punch_in - threshold).num_minutes();
    diff.max(0)
}

/// Compute early departure minutes (zero if not early).
///
/// Returns `0` on holidays — holiday work never counts as a violation.
pub(super) fn compute_early_minutes(
    punch_out: DateTime<chrono_tz::Tz>,
    shift_end: DateTime<chrono_tz::Tz>,
    is_holiday: bool,
) -> i64 {
    if is_holiday {
        return 0;
    }
    let diff = (shift_end - punch_out).num_minutes();
    diff.max(0)
}

/// Detect late/early violation from punch times and shift boundaries.
///
/// Combines [`compute_late_minutes`] and [`compute_early_minutes`] into a
/// boolean verdict. Returns `false` on holidays.
pub(super) fn detect_absence(
    punch_in: DateTime<chrono_tz::Tz>,
    punch_out: DateTime<chrono_tz::Tz>,
    shift_start: DateTime<chrono_tz::Tz>,
    shift_end: DateTime<chrono_tz::Tz>,
    grace_period_minutes: i32,
    is_holiday: bool,
) -> bool {
    compute_late_minutes(punch_in, shift_start, grace_period_minutes, is_holiday) > 0
        || compute_early_minutes(punch_out, shift_end, is_holiday) > 0
}

/// Resolve effective attendance rules with shift-level overrides (pure memory).
fn resolve_shift_policy(
    attendance_rules: &crate::models::AttendanceRules,
    working_conditions: &[WorkingCondition],
    date: NaiveDate,
    condition_type_catalog: &HashMap<String, crate::models::ConditionTypeDefinition>,
    interval: &RosterInterval,
    is_break_overtime: bool,
) -> PolicyRules {
    PolicyResolver::resolve_with_shift_overrides_pure(
        Some(attendance_rules.clone()),
        working_conditions.to_vec(),
        date,
        Some(condition_type_catalog),
        &ShiftPolicyOverrides::new(
            Some(interval.early_punch_in_grace_period_minutes),
            Some(interval.is_break_auto_deduct),
            Some(interval.allow_early_ot),
            attendance_rules.early_arrival_strategy,
            is_break_overtime,
        ),
    )
}

/// Resolve shift start/end `DateTime` from naive times, adjusting for
/// overnight shifts (end_time < start_time → end on next calendar day).
///
/// Delegates to `tz::resolve_shift_times` for unified DST handling
/// (ambiguous → first, gap → shift +1h, overnight → NaiveDate arithmetic).
pub(super) fn resolve_shift_window(
    tz: chrono_tz::Tz,
    date: NaiveDate,
    interval: &RosterInterval,
) -> Option<(DateTime<chrono_tz::Tz>, DateTime<chrono_tz::Tz>)> {
    let (start, end, _, _) = crate::utils::tz::resolve_shift_times(
        Some(interval.start_time),
        Some(interval.end_time),
        None,
        None,
        date,
        tz,
    );
    Some((start?, end?))
}

// ── AttendanceMatcher Scoring ───────────────────────────────────────────

impl AttendanceMatcher {
    /// Process a single time-entry matched to a shift interval.
    ///
    /// Resolves effective attendance rules (with shift overrides), checks for
    /// late/early violations. Returns `(std_u, ot_u, ot_pay_delta,
    /// holiday_premium_delta, is_violation)`.
    pub(super) fn process_matched_entry(
        scoring_ctx: &ScoringContext<'_>,
        entry: &TimeEntry,
        shift_start: DateTime<chrono_tz::Tz>,
        shift_end: DateTime<chrono_tz::Tz>,
        interval: &RosterInterval,
        current_date: NaiveDate,
        attendance_rules: &crate::models::AttendanceRules,
    ) -> (i32, i32, Decimal, Decimal, bool) {
        let punch_in = entry.punch_in_time().with_timezone(&scoring_ctx.tz());
        let punch_out = match entry.punch_out_time() {
            Some(po) => po.with_timezone(&scoring_ctx.tz()),
            None => return (0, 0, Decimal::ZERO, Decimal::ZERO, false),
        };

        let rules = resolve_shift_policy(
            attendance_rules,
            scoring_ctx.working_conditions,
            current_date,
            scoring_ctx.condition_type_catalog,
            interval,
            interval.break_is_overtime || entry.is_break_overtime(),
        );

        let is_holiday = scoring_ctx.holidays_map().contains_key(&current_date);
        let is_violation = detect_absence(
            punch_in,
            punch_out,
            shift_start,
            shift_end,
            rules.grace_period_minutes(),
            is_holiday,
        );

        (
            0,             // std_units — now from summaries (Phase 7)
            0,             // ot_units — now from summaries (Phase 7)
            Decimal::ZERO, // ot_pay — kept for OT pay chain (Formula Engine)
            Decimal::ZERO, // holiday_premium — now from summaries (Phase 7)
            is_violation,
        )
    }

    /// Process all entries for a single calendar day — filter work/break
    /// entries, iterate shifts, match entries to shifts, and delegate
    /// per-entry computation to [`process_matched_entry`].
    ///
    /// Returns `(std_units, ot_units, ot_pay, holiday_premium, violation_count)`.
    pub(super) fn process_daily_entries(
        scoring_ctx: &ScoringContext<'_>,
        current_date: NaiveDate,
        day_shifts: &[(Uuid, &RosterInterval)],
        entries: &[TimeEntry],
        attendance_rules: &crate::models::AttendanceRules,
    ) -> (i32, i32, Decimal, Decimal, i32) {
        let tz = scoring_ctx.tz();
        let work_entries: Vec<_> = entries
            .iter()
            .filter(|e| e.entry_type() == crate::services::pairing::types::EntryType::Work)
            .collect();

        let mut day_ot_pay = Decimal::ZERO;
        let mut day_holiday_premium = Decimal::ZERO;
        let mut day_violations = 0i32;

        for (shift_idx, (_shift_id, interval)) in day_shifts.iter().enumerate() {
            let Some((start, end)) = resolve_shift_window(tz, current_date, interval) else {
                continue;
            };

            let matching_entries = Self::match_entries_to_shift(
                tz,
                &work_entries,
                day_shifts,
                shift_idx,
                current_date,
                end,
            );

            for entry in matching_entries {
                let (_std_u, _ot_u, ot_pay_delta, premium, violation) = Self::process_matched_entry(
                    scoring_ctx,
                    entry,
                    start,
                    end,
                    interval,
                    current_date,
                    attendance_rules,
                );
                day_ot_pay += ot_pay_delta;
                day_holiday_premium += premium;
                if violation {
                    day_violations += 1;
                }
            }
        }

        (
            0, // day_std — now from summaries
            0, // day_ot — now from summaries
            day_ot_pay,
            day_holiday_premium,
            day_violations,
        )
    }

    /// Filter work entries that fall within a shift's time window.
    fn match_entries_to_shift<'a>(
        tz: chrono_tz::Tz,
        work_entries: &[&'a TimeEntry],
        day_shifts: &[(Uuid, &RosterInterval)],
        shift_idx: usize,
        current_date: NaiveDate,
        shift_end: DateTime<chrono_tz::Tz>,
    ) -> Vec<&'a TimeEntry> {
        work_entries
            .iter()
            .filter(|entry| {
                let punch_in = entry.punch_in_time().with_timezone(&tz);
                let prev_end = if shift_idx > 0 {
                    day_shifts.get(shift_idx - 1).and_then(|(_, prev_i)| {
                        // DST-safe: use naive_time_to_tz with calendar-day overnight detection
                        if prev_i.end_time < prev_i.start_time {
                            let next_date = current_date + chrono::Duration::days(1);
                            crate::utils::tz::naive_time_to_tz(prev_i.end_time, next_date, tz)
                        } else {
                            crate::utils::tz::naive_time_to_tz(prev_i.end_time, current_date, tz)
                        }
                    })
                } else {
                    None
                };
                let after_prev = prev_end.is_none_or(|pe| punch_in >= pe);
                let before_end = punch_in < shift_end;
                after_prev && before_end
            })
            .copied()
            .collect()
    }
}
