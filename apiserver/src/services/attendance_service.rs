//! Single shared attendance computation — the canonical entry point for all
//! daily attendance calculations across the system.
//!
//! # Purpose
//! Phase 4 of the attendance technical-debt cleanup: instead of 5 independent
//! call sites each copy-pasting the "resolve → NaiveTime→DateTime convert →
//! build DailyCalcInput → call AttendanceCalculator" pipeline, this service
//! encapsulates that entire pipeline in one place.
//!
//! # Usage
//! ```ignore
//! let result = AttendanceService::compute_daily_attendance(ComputeDailyInput {
//!     daily: &schedule,
//!     day_entries: &entries,
//!     date,
//!     tz: &tz,
//!     rules: &policy_rules,
//!     work_unit_minutes: 30,
//!     rounding_policy: RoundingPolicy::Floor,
//!     max_early_ot_minutes: None,
//!     seat_id: None,
//! })>
//! // result.standard_minutes(), result, ot_minutes(), result.standard_units(), etc.
//! ```

use chrono::{NaiveDate, NaiveTime, TimeZone};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::models::{PolicyRules, RoundingPolicy, TimeEntry};
use crate::services::attendance_calculator::{
    DailyCalcInput, OtAggregationMode, SegmentCalcResult, WorkBuckets,
};
use crate::services::AttendanceCalculator;

// ── Result ────────────────────────────────────────────────────────────

/// Computation result for a single day's attendance.
///
/// Wraps the raw [`WorkBuckets`] produced by [`AttendanceCalculator`]
/// and provides convenience accessors for minutes and units.
#[derive(Debug)]
pub struct DailyAttendanceResult {
    /// Raw minute-level buckets from the attendance calculator.
    work_buckets: WorkBuckets,
}

impl DailyAttendanceResult {
    // ── Minutes accessors ─────────────────────────────────────────

    /// Total standard minutes (within shift boundaries, after deductions).
    #[must_use]
    pub fn standard_minutes(&self) -> i32 {
        self.work_buckets.standard_minutes
    }

    /// Sum of all overtime buckets in minutes.
    #[must_use]
    pub fn ot_minutes(&self) -> i32 {
        self.work_buckets.early_ot_minutes
            + self.work_buckets.late_ot_minutes
            + self.work_buckets.break_ot_minutes
    }

    /// Total actual work minutes as computed by the attendance engine.
    ///
    /// This is `standard_minutes + ot_minutes` — the single
    /// number that replaces the former `raw_duration - break` heuristic
    /// and aligns with payroll attendance computation.
    #[must_use]
    pub fn actual_work_minutes(&self) -> i32 {
        self.standard_minutes() + self.ot_minutes()
    }

    /// System-detected OT minutes — alias for `ot_minutes()`.
    ///
    /// Used by [`AttendanceDailySummary`](crate::models::AttendanceDailySummary)
    /// to populate `system_detected_ot_minutes`.
    #[must_use]
    pub fn system_detected_ot_minutes(&self) -> i32 {
        self.ot_minutes()
    }

    /// Reference to the underlying [`WorkBuckets`] for advanced use.
    #[must_use]
    pub fn work_buckets(&self) -> &WorkBuckets {
        &self.work_buckets
    }

    // ── Unit accessors ────────────────────────────────────────────

    /// Convert standard minutes to work units using the given unit size
    /// and rounding policy.
    #[must_use]
    pub fn standard_units(&self, work_unit_minutes: i32, rounding_policy: RoundingPolicy) -> i32 {
        AttendanceCalculator::calculate_units(
            self.standard_minutes(),
            work_unit_minutes,
            rounding_policy,
        )
    }

    /// Convert OT minutes to work units using the given unit size
    /// and rounding policy.
    #[must_use]
    pub fn ot_units(&self, work_unit_minutes: i32, rounding_policy: RoundingPolicy) -> i32 {
        AttendanceCalculator::calculate_units(self.ot_minutes(), work_unit_minutes, rounding_policy)
    }

    /// Returns `(std_units, ot_units)` — drop-in replacement for
    /// the former `calculate_daily_units` return value.
    #[must_use]
    pub fn into_units(self, work_unit_minutes: i32, rounding_policy: RoundingPolicy) -> (i32, i32) {
        (
            self.standard_units(work_unit_minutes, rounding_policy),
            self.ot_units(work_unit_minutes, rounding_policy),
        )
    }

    /// Returns the direct minute values: `(standard_minutes, ot_minutes)`.
    #[must_use]
    pub fn into_minutes(self) -> (i32, i32) {
        (self.standard_minutes(), self.ot_minutes())
    }
}

// ── Segment Result ────────────────────────────────────────────────────

/// Computation result for multi-segment daily attendance.
///
/// Wraps the raw [`SegmentCalcResult`] produced by
/// [`AttendanceCalculator::calculate_segment_units`] and provides
/// convenience accessors for per-segment and aggregate units.
#[derive(Debug)]
pub struct SegmentAttendanceResult {
    /// Raw segment-level calculation result from the attendance calculator.
    calc_result: SegmentCalcResult,
}

impl SegmentAttendanceResult {
    /// Reference to the per-segment buckets: `(segment_id, WorkBuckets)`.
    #[must_use]
    pub fn per_segment_buckets(&self) -> &[(Option<Uuid>, WorkBuckets)] {
        &self.calc_result.per_segment_buckets
    }

    /// Aggregate standard units across all segments.
    #[must_use]
    pub fn total_standard_units(&self) -> i32 {
        self.calc_result.total_standard_units
    }

    /// Aggregate OT units across all segments.
    #[must_use]
    pub fn total_ot_units(&self) -> i32 {
        self.calc_result.total_ot_units
    }

    /// Extract `(std_units, ot_units)` for a single segment by index.
    ///
    /// Uses the configured `work_unit_minutes` and `rounding_policy` to
    /// convert raw minutes → units via [`AttendanceCalculator::calculate_units`].
    ///
    /// Falls back to aggregate totals when the index is out of range
    /// (e.g. single-segment path where `per_segment_buckets` is empty).
    #[must_use]
    pub fn segment_units(
        &self,
        seg_idx: usize,
        work_unit_minutes: i32,
        rounding_policy: RoundingPolicy,
    ) -> (i32, i32) {
        self.calc_result
            .per_segment_buckets
            .get(seg_idx)
            .map(|(_, buckets)| {
                let std_units = AttendanceCalculator::calculate_units(
                    buckets.standard_minutes,
                    work_unit_minutes,
                    rounding_policy,
                );
                let ot_mins =
                    buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes;
                let ot_units = AttendanceCalculator::calculate_units(
                    ot_mins,
                    work_unit_minutes,
                    rounding_policy,
                );
                (std_units, ot_units)
            })
            .unwrap_or((
                self.calc_result.total_standard_units,
                self.calc_result.total_ot_units,
            ))
    }

    /// Reference to the underlying [`SegmentCalcResult`] for advanced use.
    #[must_use]
    pub fn calc_result(&self) -> &SegmentCalcResult {
        &self.calc_result
    }

    /// Returns `(total_std_units, total_ot_units)` — the aggregate across
    /// all segments, already converted to units by the calculator.
    #[must_use]
    pub fn into_total_units(self) -> (i32, i32) {
        (
            self.calc_result.total_standard_units,
            self.calc_result.total_ot_units,
        )
    }
}

// ── Input ─────────────────────────────────────────────────────────────

/// All inputs needed by [`AttendanceService::compute_daily_attendance`].
///
/// Split into a dedicated struct (rather than a long function signature)
/// to keep call sites readable and allow future additions without
/// breaking every call site.
#[derive(Debug)]
pub struct ComputeDailyInput<'a> {
    /// The effective schedule for this day (shift times, break info, etc.).
    pub daily: &'a crate::models::EffectiveSchedule,

    /// Time entries for the day, in punch-in chronological order.
    pub day_entries: &'a [&'a TimeEntry],

    /// The date being calculated.
    pub date: NaiveDate,

    /// Workspace timezone.
    pub tz: &'a Tz,

    /// Pre-resolved policy rules (grace period, break auto-deduct, etc.).
    pub rules: &'a PolicyRules,

    /// Size of one work unit in minutes (e.g. 30 → each unit = 30 min).
    pub work_unit_minutes: i32,

    /// Rounding mode for unit conversion.
    pub rounding_policy: RoundingPolicy,

    /// Cap on early OT minutes from the shift schedule.
    pub max_early_ot_minutes: Option<i32>,

    /// OT ceiling — next segment start, for split-shift OT capping.
    pub ot_ceiling: Option<chrono::DateTime<Tz>>,

    /// Links to a specific work segment (None for legacy / single-shift).
    pub segment_id: Option<Uuid>,

    /// OT aggregation mode for multi-segment days.
    pub ot_aggregation_mode: OtAggregationMode,
}

// ── Service ───────────────────────────────────────────────────────────

/// Stateless service that encapsulates the canonical daily attendance
/// computation pipeline.
///
/// # Pipeline
/// 1. Extract first/last punches and compute actual break minutes from entries
/// 2. Convert shift/break NaiveTime → timezone-aware DateTime
/// 3. Handle overnight shifts (end < start)
/// 4. Build [`DailyCalcInput`]
/// 5. Delegate to [`AttendanceCalculator::calculate_buckets`]
pub struct AttendanceService;

impl AttendanceService {
    /// Compute one day's attendance from raw punch data and resolved rules.
    ///
    /// Returns `None` when there are no valid punches (missing punch_in,
    /// missing punch_out, or zero entries).
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn compute_daily_attendance(input: ComputeDailyInput<'_>) -> Option<DailyAttendanceResult> {
        let ComputeDailyInput {
            daily,
            day_entries,
            date,
            tz,
            rules,
            work_unit_minutes,
            rounding_policy,
            max_early_ot_minutes,
            ot_ceiling,
            segment_id,
            ot_aggregation_mode,
        } = input;

        // ── 1. Extract punches ──────────────────────────────────────
        let punch_in = day_entries
            .first()
            .map(|e| e.punch_in_time().with_timezone(tz))?;

        let punch_out = day_entries
            .last()
            .and_then(|e| e.punch_out_time())?
            .with_timezone(tz);

        // ── 2. Compute break minutes from entries ────────────────────
        let actual_break_minutes: i32 = day_entries
            .iter()
            .filter(|e| e.entry_type() == crate::services::pairing::types::EntryType::Break)
            .map(|e| e.duration_minutes().unwrap_or(0))
            .sum();

        // ── 3. Convert shift times: NaiveTime → DateTime<Tz> ─────────
        let shift_start_dt = shift_naive_to_dt(daily.start_time(), date, *tz);
        let shift_end_dt = shift_naive_to_dt(daily.end_time(), date, *tz);

        // Handle overnight shift (end < start → end += 1 day)
        let (shift_start, shift_end) =
            if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
                if daily.end_time() < daily.start_time() {
                    end += chrono::Duration::days(1);
                }
                (Some(start), Some(end))
            } else {
                (None, None)
            };

        // ── 4. Convert break times: NaiveTime → DateTime<Tz> ─────────
        let break_start_dt = shift_naive_to_dt(daily.break_start(), date, *tz);
        let break_end_dt = shift_naive_to_dt(daily.break_end(), date, *tz);

        let (break_start, break_end) =
            if let (Some(bs), Some(mut be)) = (break_start_dt, break_end_dt) {
                if daily.break_end() < daily.break_start() {
                    be += chrono::Duration::days(1);
                }
                (Some(bs), Some(be))
            } else {
                (None, None)
            };

        // ── 5. Build input & call calculator ─────────────────────────
        let calc_input: DailyCalcInput<Tz> = DailyCalcInput {
            punch_in,
            punch_out,
            shift_start,
            shift_end,
            scheduled_break_minutes: daily.break_duration_minutes().unwrap_or(0),
            actual_break_minutes,
            break_start,
            break_end,
            work_unit: work_unit_minutes,
            rounding_policy,
            rules: rules.clone(),
            ot_ceiling,
            max_early_ot_minutes,
            segment_id,
            ot_aggregation_mode,
        };

        let work_buckets = AttendanceCalculator::calculate_buckets(&calc_input);

        Some(DailyAttendanceResult { work_buckets })
    }

    /// Compute multi-segment attendance from pre-built per-segment inputs.
    ///
    /// Routes to [`AttendanceCalculator::calculate_segment_units`] for
    /// multi-segment days, or [`AttendanceCalculator::calculate_daily_units`]
    /// for single-segment days (populating per-segment buckets in the latter
    /// case so downstream code can always index by segment position).
    ///
    /// The caller is responsible for building the inputs (punch matching,
    /// OT ceiling computation, per-segment policy overrides) — those are
    /// segment-specific concerns that live in the report service.
    #[must_use]
    pub fn compute_segment_attendance(inputs: &[DailyCalcInput<Tz>]) -> SegmentAttendanceResult {
        if inputs.is_empty() {
            return SegmentAttendanceResult {
                calc_result: SegmentCalcResult::default(),
            };
        }

        let calc_result = if inputs.len() > 1 {
            AttendanceCalculator::calculate_segment_units(inputs)
        } else {
            let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(&inputs[0]);
            let buckets = AttendanceCalculator::calculate_buckets(&inputs[0]);
            SegmentCalcResult {
                total_standard_units: std_units,
                total_ot_units: ot_units,
                per_segment_buckets: vec![(inputs[0].segment_id, buckets)],
            }
        };

        SegmentAttendanceResult { calc_result }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

/// Convert a `NaiveTime` to a timezone-aware `DateTime<Tz>` for the given date.
///
/// Returns `None` when the time is ambiguous (e.g., DST fall-back) or
/// invalid in the target timezone.
fn shift_naive_to_dt(
    naive_time_opt: Option<NaiveTime>,
    date: NaiveDate,
    tz: Tz,
) -> Option<chrono::DateTime<Tz>> {
    naive_time_opt.and_then(|t| tz.from_local_datetime(&date.and_time(t)).single())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, Timelike};

    #[test]
    fn shift_naive_to_dt_basic() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let t = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let result = shift_naive_to_dt(Some(t), date, tz);
        assert!(result.is_some());
        let dt = result.unwrap();
        assert_eq!(dt.hour(), 8);
        assert_eq!(dt.minute(), 0);
    }

    #[test]
    fn shift_naive_to_dt_none_returns_none() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert!(shift_naive_to_dt(None, date, tz).is_none());
    }
}
