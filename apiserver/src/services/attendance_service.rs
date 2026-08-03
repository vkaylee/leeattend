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

#[cfg(test)]
use chrono::NaiveTime;
use chrono::{DateTime, NaiveDate};
use chrono_tz::Tz;
use tracing::error;
use uuid::Uuid;

use crate::models::{PolicyRules, RoundingPolicy, TimeEntry};
use crate::services::attendance_calculator::{
    DailyCalcInput, OtAggregationMode, SegmentCalcResult, WorkBuckets,
};
use crate::services::pairing::types::EntryType;
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

        // ── 3–4. Convert shift & break times: NaiveTime → DateTime<Tz> ──
        let (shift_start, shift_end, break_start, break_end) =
            crate::utils::tz::resolve_shift_times(
                daily.start_time(),
                daily.end_time(),
                daily.break_start(),
                daily.break_end(),
                date,
                *tz,
            );

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
        let input_groups: Vec<&[DailyCalcInput<Tz>]> = inputs
            .iter()
            .map(|input| std::slice::from_ref(input))
            .collect();

        Self::compute_segment_attendance_sessions(&input_groups)
    }

    /// Compute segment attendance from the actual punch sessions belonging to
    /// each segment. A segment can contain several work/overtime entries; each
    /// entry is calculated independently so gaps between sessions are never
    /// counted as worked time.
    #[must_use]
    pub fn compute_segment_attendance_sessions(
        input_groups: &[&[DailyCalcInput<Tz>]],
    ) -> SegmentAttendanceResult {
        let Some(first_input) = input_groups.iter().flat_map(|group| group.iter()).next() else {
            return SegmentAttendanceResult {
                calc_result: SegmentCalcResult::default(),
            };
        };

        let per_segment_buckets: Vec<(Option<Uuid>, WorkBuckets)> = input_groups
            .iter()
            .map(|group| {
                let buckets = Self::aggregate_session_buckets(group);
                let segment_id = group.first().and_then(|input| input.segment_id);
                (segment_id, buckets)
            })
            .collect();

        let total_standard_minutes = per_segment_buckets
            .iter()
            .map(|(_, buckets)| buckets.standard_minutes)
            .sum();
        let total_ot_minutes = per_segment_buckets
            .iter()
            .map(|(_, buckets)| {
                buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes
            })
            .sum();

        let (total_standard_units, total_ot_units) = match first_input.ot_aggregation_mode {
            OtAggregationMode::Aggregate => (
                AttendanceCalculator::calculate_units(
                    total_standard_minutes,
                    first_input.work_unit,
                    first_input.rounding_policy,
                ),
                AttendanceCalculator::calculate_units(
                    total_ot_minutes,
                    first_input.work_unit,
                    first_input.rounding_policy,
                ),
            ),
            OtAggregationMode::PerSegment => {
                per_segment_buckets
                    .iter()
                    .fold((0, 0), |(std_units, ot_units), (_, buckets)| {
                        (
                            std_units
                                + AttendanceCalculator::calculate_units(
                                    buckets.standard_minutes,
                                    first_input.work_unit,
                                    first_input.rounding_policy,
                                ),
                            ot_units
                                + AttendanceCalculator::calculate_units(
                                    buckets.early_ot_minutes
                                        + buckets.late_ot_minutes
                                        + buckets.break_ot_minutes,
                                    first_input.work_unit,
                                    first_input.rounding_policy,
                                ),
                        )
                    })
            }
        };

        SegmentAttendanceResult {
            calc_result: SegmentCalcResult {
                total_standard_units,
                total_ot_units,
                per_segment_buckets,
            },
        }
    }

    fn aggregate_session_buckets(inputs: &[DailyCalcInput<Tz>]) -> WorkBuckets {
        let mut total = WorkBuckets::default();

        for input in inputs {
            // The early-OT ceiling belongs to the segment, not to each punch
            // session. Calculate sessions uncapped, then apply it once below.
            let mut uncapped_input = input.clone();
            uncapped_input.max_early_ot_minutes = None;
            let buckets = AttendanceCalculator::calculate_buckets(&uncapped_input);
            total.standard_minutes += buckets.standard_minutes;
            total.early_ot_minutes += buckets.early_ot_minutes;
            total.late_ot_minutes += buckets.late_ot_minutes;
            total.break_ot_minutes += buckets.break_ot_minutes;
        }

        if let Some(max_early_ot_minutes) =
            inputs.first().and_then(|input| input.max_early_ot_minutes)
        {
            total.early_ot_minutes = total.early_ot_minutes.min(max_early_ot_minutes);
        }

        total
    }

    /// Compute attendance from time entries by splitting them into individual
    /// work sessions. Break entries are filtered out and gaps between sessions
    /// are never counted as worked time.
    ///
    /// This is the shared implementation used by the daily report path, the
    /// segment report path, and the Excel export worker.
    #[must_use]
    pub fn compute_session_attendance(
        day_entries: &[&TimeEntry],
        tz: Tz,
        shift_start: Option<DateTime<Tz>>,
        shift_end: Option<DateTime<Tz>>,
        break_start: Option<DateTime<Tz>>,
        break_end: Option<DateTime<Tz>>,
        scheduled_break_minutes: i32,
        work_unit_minutes: i32,
        rounding_policy: RoundingPolicy,
        rules: &PolicyRules,
        ot_ceiling: Option<DateTime<Tz>>,
        max_early_ot_minutes: Option<i32>,
        segment_id: Option<Uuid>,
        ot_aggregation_mode: OtAggregationMode,
    ) -> SegmentAttendanceResult {
        let work_entries: Vec<&TimeEntry> = day_entries
            .iter()
            .filter(|e| e.entry_type() != EntryType::Break && e.punch_out_time().is_some())
            .copied()
            .collect();

        let break_entries: Vec<&TimeEntry> = day_entries
            .iter()
            .filter(|e| e.entry_type() == EntryType::Break)
            .copied()
            .collect();

        if work_entries.is_empty() {
            return SegmentAttendanceResult {
                calc_result: SegmentCalcResult::default(),
            };
        }

        let break_overlap = |session: &TimeEntry| -> i32 {
            let Some(session_out) = session.punch_out_time() else {
                return 0;
            };
            let session_in = session.punch_in_time();

            break_entries
                .iter()
                .filter_map(|break_entry| {
                    let break_out = break_entry.punch_out_time()?;
                    let overlap_start = std::cmp::max(session_in, break_entry.punch_in_time());
                    let overlap_end = std::cmp::min(session_out, break_out);
                    (overlap_end > overlap_start).then(|| {
                        i32::try_from(
                            overlap_end
                                .signed_duration_since(overlap_start)
                                .num_minutes(),
                        )
                        .unwrap_or(0)
                    })
                })
                .sum()
        };

        let session_inputs: Vec<DailyCalcInput<Tz>> = work_entries
            .iter()
            .map(|entry| DailyCalcInput {
                punch_in: entry.punch_in_time().with_timezone(&tz),
                punch_out: entry
                    .punch_out_time()
                    .unwrap_or_else(|| {
                        error!(
                            entry_id = %entry.id(),
                            "punch_out_time is None despite pre-filter — data inconsistency"
                        );
                        entry.punch_in_time()
                    })
                    .with_timezone(&tz),
                shift_start,
                shift_end,
                scheduled_break_minutes: if work_entries.len() == 1 {
                    scheduled_break_minutes
                } else {
                    0
                },
                actual_break_minutes: break_overlap(entry),
                break_start,
                break_end,
                work_unit: work_unit_minutes,
                rounding_policy,
                rules: rules.clone(),
                ot_ceiling,
                max_early_ot_minutes,
                segment_id,
                ot_aggregation_mode,
            })
            .collect();

        let input_groups: Vec<&[DailyCalcInput<Tz>]> = vec![session_inputs.as_slice()];
        Self::compute_segment_attendance_sessions(&input_groups)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

/// Convert a `NaiveTime` to a timezone-aware `DateTime<Tz>` for the given date.
///
/// Returns `None` when the time is ambiguous (e.g., DST fall-back) or
/// invalid in the target timezone.
#[cfg(test)]
fn shift_naive_to_dt(
    naive_time_opt: Option<NaiveTime>,
    date: NaiveDate,
    tz: Tz,
) -> Option<chrono::DateTime<Tz>> {
    naive_time_opt.and_then(|t| crate::utils::tz::naive_time_to_tz(t, date, tz))
}

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use super::*;
    use crate::models::EarlyArrivalStrategy;
    use crate::models::{EffectiveSchedule, PolicyRules, RoundingPolicy, TimeEntry};
    use crate::services::attendance_calculator::{
        DailyCalcInput, OtAggregationMode, SegmentCalcResult, WorkBuckets,
    };
    use crate::services::pairing::types::EntryType;
    use chrono::{NaiveTime, TimeZone, Timelike, Utc};
    use uuid::Uuid;

    // =========================================================================
    // Test helpers
    // =========================================================================

    const TEST_DATE: NaiveDate = match NaiveDate::from_ymd_opt(2026, 1, 1) {
        Some(d) => d,
        None => panic!("invalid test date"),
    };

    fn dt(h: u32, m: u32) -> chrono::DateTime<Utc> {
        Utc.from_local_datetime(&TEST_DATE.and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap()))
            .single()
            .expect("non-ambiguous UTC time")
    }

    fn dt_tz(tz: Tz, h: u32, m: u32) -> chrono::DateTime<Tz> {
        tz.from_local_datetime(&TEST_DATE.and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap()))
            .single()
            .expect("non-ambiguous time")
    }

    fn dt_utc_d2(h: u32, m: u32) -> chrono::DateTime<Utc> {
        let d2 = TEST_DATE.succ_opt().unwrap();
        Utc.from_local_datetime(&d2.and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap()))
            .single()
            .expect("non-ambiguous UTC time")
    }

    fn tz_utc() -> Tz {
        chrono_tz::UTC
    }

    fn tz_hcm() -> Tz {
        "Asia/Ho_Chi_Minh".parse().unwrap()
    }

    fn mock_rules() -> PolicyRules {
        PolicyRules::new(
            0,                           // grace_period_minutes
            0,                           // authorized_early_leave_minutes
            false,                       // is_early_leave_paid
            EarlyArrivalStrategy::Allow, // early_arrival_strategy
            false,                       // is_break_auto_deduct
            false,                       // is_break_overtime
            false,                       // allow_early_ot
        )
    }

    fn mock_work_entry(
        punch_in: chrono::DateTime<Utc>,
        punch_out: chrono::DateTime<Utc>,
        duration: i32,
    ) -> TimeEntry {
        TimeEntry::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            punch_in,
            Some(punch_out),
            Some(duration),
            false,
            EntryType::Work,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    fn mock_break_entry(
        punch_in: chrono::DateTime<Utc>,
        punch_out: chrono::DateTime<Utc>,
        duration: i32,
    ) -> TimeEntry {
        TimeEntry::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            None,
            punch_in,
            Some(punch_out),
            Some(duration),
            false,
            EntryType::Break,
            None,
            None,
            Utc::now(),
            Utc::now(),
        )
    }

    /// Build a schedule with a scheduled break duration (no fixed break frame).
    fn mock_schedule_with_scheduled_break(
        start_h: u32,
        start_m: u32,
        end_h: u32,
        end_m: u32,
        break_minutes: i32,
    ) -> EffectiveSchedule {
        EffectiveSchedule::new(
            TEST_DATE,
            Some(Uuid::new_v4()),
            "test".to_string(),
            Some("Test Shift".to_string()),
            Some("T".to_string()),
            NaiveTime::from_hms_opt(start_h, start_m, 0),
            NaiveTime::from_hms_opt(end_h, end_m, 0),
            Some(break_minutes),
            None,   // break_start
            None,   // break_end
            None,   // is_break_auto_deduct
            None,   // early_punch_in_grace_period_minutes
            None,   // allow_early_ot
            None,   // break_is_overtime
            None,   // max_early_ot_minutes
            vec![], // traces
            false,  // is_holiday
            false,  // is_holiday_paid
            None,   // holiday_name
        )
    }
    /// Build an `EffectiveSchedule` with sensible defaults for test use.
    /// Defaults: date=TEST_DATE, source="test", shift 08:00–17:00, no break,
    /// no auto-deduct, no OT flags, no holiday.
    fn mock_schedule(start_h: u32, start_m: u32, end_h: u32, end_m: u32) -> EffectiveSchedule {
        EffectiveSchedule::new(
            TEST_DATE,
            Some(Uuid::new_v4()),
            "test".to_string(),
            Some("Test Shift".to_string()),
            Some("T".to_string()),
            NaiveTime::from_hms_opt(start_h, start_m, 0),
            NaiveTime::from_hms_opt(end_h, end_m, 0),
            None,   // break_duration_minutes
            None,   // break_start
            None,   // break_end
            None,   // is_break_auto_deduct
            None,   // early_punch_in_grace_period_minutes
            None,   // allow_early_ot
            None,   // break_is_overtime
            None,   // max_early_ot_minutes
            vec![], // traces
            false,  // is_holiday
            false,  // is_holiday_paid
            None,   // holiday_name
        )
    }

    /// Build an `EffectiveSchedule` with a fixed break window.
    fn mock_schedule_with_break(
        start_h: u32,
        start_m: u32,
        end_h: u32,
        end_m: u32,
        break_start_h: u32,
        break_start_m: u32,
        break_end_h: u32,
        break_end_m: u32,
        break_dur: i32,
    ) -> EffectiveSchedule {
        EffectiveSchedule::new(
            TEST_DATE,
            Some(Uuid::new_v4()),
            "test".to_string(),
            Some("Test Shift".to_string()),
            Some("T".to_string()),
            NaiveTime::from_hms_opt(start_h, start_m, 0),
            NaiveTime::from_hms_opt(end_h, end_m, 0),
            Some(break_dur),
            NaiveTime::from_hms_opt(break_start_h, break_start_m, 0),
            NaiveTime::from_hms_opt(break_end_h, break_end_m, 0),
            None,   // is_break_auto_deduct
            None,   // early_punch_in_grace_period_minutes
            None,   // allow_early_ot
            None,   // break_is_overtime
            None,   // max_early_ot_minutes
            vec![], // traces
            false,  // is_holiday
            false,  // is_holiday_paid
            None,   // holiday_name
        )
    }

    /// Build an overnight `EffectiveSchedule` (end < start).
    fn mock_overnight_schedule(
        start_h: u32,
        start_m: u32,
        end_h: u32,
        end_m: u32,
    ) -> EffectiveSchedule {
        EffectiveSchedule::new(
            TEST_DATE,
            Some(Uuid::new_v4()),
            "test".to_string(),
            Some("Overnight Shift".to_string()),
            Some("N".to_string()),
            NaiveTime::from_hms_opt(start_h, start_m, 0),
            NaiveTime::from_hms_opt(end_h, end_m, 0),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![],
            false,
            false,
            None,
        )
    }

    /// Build a no-shift `EffectiveSchedule` (day off / holiday).
    fn mock_no_shift_schedule() -> EffectiveSchedule {
        EffectiveSchedule::new(
            TEST_DATE,
            None, // shift_id = None → no shift
            "none".to_string(),
            None,
            None,
            None, // start_time
            None, // end_time
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            vec![],
            false,
            false,
            None,
        )
    }

    // =========================================================================
    // ComputeDailyInputBuilder
    // =========================================================================

    /// Test builder for `ComputeDailyInput`. Owns the data so references
    /// remain valid during the `compute()` call.
    struct ComputeDailyInputBuilder {
        schedule: EffectiveSchedule,
        entries: Vec<TimeEntry>,
        date: NaiveDate,
        tz: Tz,
        rules: PolicyRules,
        work_unit_minutes: i32,
        rounding_policy: RoundingPolicy,
        max_early_ot_minutes: Option<i32>,
        ot_ceiling: Option<chrono::DateTime<Tz>>,
        segment_id: Option<Uuid>,
        ot_aggregation_mode: OtAggregationMode,
    }

    impl ComputeDailyInputBuilder {
        fn new() -> Self {
            Self {
                schedule: mock_schedule(8, 0, 17, 0),
                entries: Vec::new(),
                date: TEST_DATE,
                tz: tz_utc(),
                rules: mock_rules(),
                work_unit_minutes: 30,
                rounding_policy: RoundingPolicy::Floor,
                max_early_ot_minutes: None,
                ot_ceiling: None,
                segment_id: None,
                ot_aggregation_mode: OtAggregationMode::Aggregate,
            }
        }

        fn schedule(mut self, s: EffectiveSchedule) -> Self {
            self.schedule = s;
            self
        }

        fn schedule_mut(mut self, f: impl FnOnce(&mut EffectiveSchedule)) -> Self {
            f(&mut self.schedule);
            self
        }

        fn entries(mut self, entries: Vec<TimeEntry>) -> Self {
            self.entries = entries;
            self
        }

        fn work_entry(
            mut self,
            punch_in: chrono::DateTime<Utc>,
            punch_out: chrono::DateTime<Utc>,
        ) -> Self {
            let dur = (punch_out - punch_in).num_minutes() as i32;
            self.entries.push(mock_work_entry(punch_in, punch_out, dur));
            self
        }

        fn break_entry(
            mut self,
            punch_in: chrono::DateTime<Utc>,
            punch_out: chrono::DateTime<Utc>,
            dur: i32,
        ) -> Self {
            self.entries
                .push(mock_break_entry(punch_in, punch_out, dur));
            self
        }

        fn tz(mut self, tz: Tz) -> Self {
            self.tz = tz;
            self
        }

        fn rules_mut(mut self, f: impl FnOnce(&mut PolicyRules)) -> Self {
            f(&mut self.rules);
            self
        }

        fn work_unit(mut self, minutes: i32) -> Self {
            self.work_unit_minutes = minutes;
            self
        }

        fn rounding(mut self, policy: RoundingPolicy) -> Self {
            self.rounding_policy = policy;
            self
        }

        fn max_early_ot(mut self, minutes: i32) -> Self {
            self.max_early_ot_minutes = Some(minutes);
            self
        }

        fn ot_ceiling_at(mut self, h: u32, m: u32) -> Self {
            self.ot_ceiling = Some(dt_tz(self.tz, h, m));
            self
        }

        fn segment_id(mut self, id: Uuid) -> Self {
            self.segment_id = Some(id);
            self
        }

        fn ot_aggregation_mode(mut self, mode: OtAggregationMode) -> Self {
            self.ot_aggregation_mode = mode;
            self
        }

        fn date(mut self, date: NaiveDate) -> Self {
            self.date = date;
            self
        }

        /// Build `ComputeDailyInput` and call `compute_daily_attendance`.
        fn compute(&self) -> Option<DailyAttendanceResult> {
            let entry_refs: Vec<&TimeEntry> = self.entries.iter().collect();
            let input = ComputeDailyInput {
                daily: &self.schedule,
                day_entries: &entry_refs,
                date: self.date,
                tz: &self.tz,
                rules: &self.rules,
                work_unit_minutes: self.work_unit_minutes,
                rounding_policy: self.rounding_policy,
                max_early_ot_minutes: self.max_early_ot_minutes,
                ot_ceiling: self.ot_ceiling,
                segment_id: self.segment_id,
                ot_aggregation_mode: self.ot_aggregation_mode,
            };
            AttendanceService::compute_daily_attendance(input)
        }
    }

    // =========================================================================
    // Helper: build a DailyCalcInput for segment_attendance tests
    // =========================================================================

    fn make_calc_input(
        punch_in: chrono::DateTime<Tz>,
        punch_out: chrono::DateTime<Tz>,
        shift_start: chrono::DateTime<Tz>,
        shift_end: chrono::DateTime<Tz>,
        work_unit: i32,
        rules: &PolicyRules,
        ot_ceiling: Option<chrono::DateTime<Tz>>,
        segment_id: Option<Uuid>,
    ) -> DailyCalcInput<Tz> {
        DailyCalcInput {
            punch_in,
            punch_out,
            shift_start: Some(shift_start),
            shift_end: Some(shift_end),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            work_unit,
            rounding_policy: RoundingPolicy::Floor,
            rules: rules.clone(),
            ot_ceiling,
            max_early_ot_minutes: None,
            segment_id,
            ot_aggregation_mode: OtAggregationMode::Aggregate,
        }
    }

    // =========================================================================
    // Existing helper tests (kept)
    // =========================================================================

    #[test]
    fn shift_naive_to_dt_basic() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        let t = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let result = shift_naive_to_dt(Some(t), date, tz);
        assert!(result.is_some());
        let dt_val = result.unwrap();
        assert_eq!(dt_val.hour(), 8);
        assert_eq!(dt_val.minute(), 0);
    }

    #[test]
    fn shift_naive_to_dt_none_returns_none() {
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert!(shift_naive_to_dt(None, date, tz).is_none());
    }

    #[test]
    fn shift_naive_to_dt_dst_gap_resolves() {
        // America/New_York DST starts 2026-03-08 at 02:00 → 03:00
        // shift_naive_to_dt delegates to naive_time_to_tz which delegates to local_to_tz
        let tz: Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
        let t = NaiveTime::from_hms_opt(2, 30, 0).unwrap();
        let result = shift_naive_to_dt(Some(t), date, tz);
        assert!(
            result.is_some(),
            "DST gap time should resolve via shift_naive_to_dt"
        );
    }

    #[test]
    fn shift_naive_to_dt_dst_ambiguous_resolves() {
        // America/New_York DST ends 2026-11-01 at 02:00 → 01:00
        // 01:30 exists twice — should pick first (EDT)
        let tz: Tz = "America/New_York".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let t = NaiveTime::from_hms_opt(1, 30, 0).unwrap();
        let result = shift_naive_to_dt(Some(t), date, tz);
        assert!(
            result.is_some(),
            "DST ambiguous time should resolve via shift_naive_to_dt"
        );
        let dt = result.unwrap();
        // First occurrence: EDT (UTC-4) → 01:30 + 4 = 05:30 UTC
        assert_eq!(dt.with_timezone(&Utc).hour(), 5);
        assert_eq!(dt.with_timezone(&Utc).minute(), 30);
    }

    #[test]
    fn shift_naive_to_dt_overnight_manual_check() {
        // Verify the pattern used in compute_daily_attendance:
        // shift_naive_to_dt does NOT handle overnight — caller does it manually
        let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let start_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let end_time = NaiveTime::from_hms_opt(6, 0, 0).unwrap();

        let start_dt = shift_naive_to_dt(Some(start_time), date, tz).unwrap();
        let mut end_dt = shift_naive_to_dt(Some(end_time), date, tz).unwrap();

        // Manual overnight check (same pattern as compute_daily_attendance)
        if end_time < start_time {
            end_dt += chrono::Duration::days(1);
        }

        assert_eq!(start_dt.date_naive(), date);
        assert_eq!(
            end_dt.date_naive(),
            NaiveDate::from_ymd_opt(2026, 6, 16).unwrap()
        );
        // Duration should be 8 hours
        let duration = end_dt.signed_duration_since(start_dt);
        assert_eq!(duration.num_hours(), 8);
    }

    #[test]
    fn shift_naive_to_dt_various_timezones() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let t = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

        // HCM (UTC+7): 09:00 local = 02:00 UTC
        let tz_hcm: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
        let dt_hcm = shift_naive_to_dt(Some(t), date, tz_hcm).unwrap();
        assert_eq!(dt_hcm.with_timezone(&Utc).hour(), 2);

        // Tokyo (UTC+9): 09:00 local = 00:00 UTC
        let tz_tokyo: Tz = "Asia/Tokyo".parse().unwrap();
        let dt_tokyo = shift_naive_to_dt(Some(t), date, tz_tokyo).unwrap();
        assert_eq!(dt_tokyo.with_timezone(&Utc).hour(), 0);

        // London (UTC+1 BST in summer): 09:00 local = 08:00 UTC
        let tz_london: Tz = "Europe/London".parse().unwrap();
        let dt_london = shift_naive_to_dt(Some(t), date, tz_london).unwrap();
        assert_eq!(dt_london.with_timezone(&Utc).hour(), 8);
    }

    // =========================================================================
    // Module B: DailyAttendanceResult accessor tests
    // =========================================================================

    mod daily_result_accessors {
        use super::*;

        fn make_result(std: i32, early: i32, late: i32, brk: i32) -> DailyAttendanceResult {
            DailyAttendanceResult {
                work_buckets: WorkBuckets {
                    standard_minutes: std,
                    early_ot_minutes: early,
                    late_ot_minutes: late,
                    break_ot_minutes: brk,
                },
            }
        }

        #[test]
        fn standard_minutes_returns_correct_value() {
            let r = make_result(500, 30, 20, 10);
            assert_eq!(r.standard_minutes(), 500);
        }

        #[test]
        fn ot_minutes_sums_all_ot_buckets() {
            let r = make_result(500, 30, 20, 10);
            assert_eq!(r.ot_minutes(), 60);
        }

        #[test]
        fn actual_work_minutes_sums_std_and_ot() {
            let r = make_result(500, 30, 20, 10);
            assert_eq!(r.actual_work_minutes(), 560);
        }

        #[test]
        fn system_detected_ot_equals_ot_minutes() {
            let r = make_result(500, 30, 20, 10);
            assert_eq!(r.system_detected_ot_minutes(), 60);
        }

        #[test]
        fn work_buckets_returns_reference() {
            let r = make_result(500, 30, 20, 10);
            assert_eq!(r.work_buckets().standard_minutes, 500);
            assert_eq!(r.work_buckets().early_ot_minutes, 30);
        }

        #[test]
        fn standard_units_floor() {
            let r = make_result(500, 0, 0, 0);
            assert_eq!(r.standard_units(30, RoundingPolicy::Floor), 16);
        }

        #[test]
        fn standard_units_ceil() {
            let r = make_result(500, 0, 0, 0);
            assert_eq!(r.standard_units(30, RoundingPolicy::Ceil), 17);
        }

        #[test]
        fn standard_units_round() {
            let r = make_result(495, 0, 0, 0);
            // 495/30 = 16.5 → rounds to 17
            assert_eq!(r.standard_units(30, RoundingPolicy::Round), 17);
        }

        #[test]
        fn standard_units_none_truncates() {
            let r = make_result(500, 0, 0, 0);
            assert_eq!(r.standard_units(30, RoundingPolicy::None), 16);
        }

        #[test]
        fn ot_units_with_rounding() {
            let r = make_result(0, 30, 30, 15);
            assert_eq!(r.ot_units(30, RoundingPolicy::Ceil), 3);
        }

        #[test]
        fn into_units_returns_tuple() {
            let r = make_result(500, 0, 75, 0);
            let (std, ot) = r.into_units(30, RoundingPolicy::Floor);
            assert_eq!(std, 16);
            assert_eq!(ot, 2);
        }

        #[test]
        fn into_minutes_returns_tuple() {
            let r = make_result(500, 0, 75, 0);
            let (std, ot) = r.into_minutes();
            assert_eq!(std, 500);
            assert_eq!(ot, 75);
        }

        #[test]
        fn all_zero_buckets() {
            let r = make_result(0, 0, 0, 0);
            assert_eq!(r.standard_minutes(), 0);
            assert_eq!(r.ot_minutes(), 0);
            assert_eq!(r.actual_work_minutes(), 0);
            assert_eq!(r.standard_units(30, RoundingPolicy::Floor), 0);
        }
    }

    // =========================================================================
    // Module D: SegmentAttendanceResult accessor tests
    // =========================================================================

    mod segment_result_accessors {
        use super::*;

        fn make_calc(
            total_std: i32,
            total_ot: i32,
            buckets: Vec<(Option<Uuid>, WorkBuckets)>,
        ) -> SegmentCalcResult {
            SegmentCalcResult {
                total_standard_units: total_std,
                total_ot_units: total_ot,
                per_segment_buckets: buckets,
            }
        }

        fn make_result(
            total_std: i32,
            total_ot: i32,
            buckets: Vec<(Option<Uuid>, WorkBuckets)>,
        ) -> SegmentAttendanceResult {
            SegmentAttendanceResult {
                calc_result: make_calc(total_std, total_ot, buckets),
            }
        }

        fn make_buckets(std: i32, early: i32, late: i32, brk: i32) -> WorkBuckets {
            WorkBuckets {
                standard_minutes: std,
                early_ot_minutes: early,
                late_ot_minutes: late,
                break_ot_minutes: brk,
            }
        }

        #[test]
        fn total_standard_units() {
            let r = make_result(20, 5, vec![]);
            assert_eq!(r.total_standard_units(), 20);
        }

        #[test]
        fn total_ot_units() {
            let r = make_result(20, 5, vec![]);
            assert_eq!(r.total_ot_units(), 5);
        }

        #[test]
        fn segment_units_valid_index() {
            let id0 = Uuid::new_v4();
            let id1 = Uuid::new_v4();
            let r = make_result(
                30,
                8,
                vec![
                    (Some(id0), make_buckets(480, 0, 0, 0)),  // 16 std, 0 ot
                    (Some(id1), make_buckets(240, 60, 0, 0)), // 8 std, 2 ot
                ],
            );

            let (s0, o0) = r.segment_units(0, 30, RoundingPolicy::Floor);
            assert_eq!(s0, 16);
            assert_eq!(o0, 0);

            let (s1, o1) = r.segment_units(1, 30, RoundingPolicy::Floor);
            assert_eq!(s1, 8);
            assert_eq!(o1, 2);
        }

        #[test]
        fn segment_units_out_of_range_falls_back_to_totals() {
            let r = make_result(
                20,
                5,
                vec![(Some(Uuid::new_v4()), make_buckets(100, 0, 0, 0))],
            );
            let (s, o) = r.segment_units(99, 30, RoundingPolicy::Floor);
            assert_eq!(s, 20);
            assert_eq!(o, 5);
        }

        #[test]
        fn segment_units_empty_vec_falls_back() {
            let r = make_result(10, 2, vec![]);
            let (s, o) = r.segment_units(0, 30, RoundingPolicy::Floor);
            assert_eq!(s, 10);
            assert_eq!(o, 2);
        }

        #[test]
        fn into_total_units_consumes() {
            let r = make_result(20, 5, vec![]);
            assert_eq!(r.into_total_units(), (20, 5));
        }

        #[test]
        fn calc_result_returns_reference() {
            let r = make_result(20, 5, vec![]);
            assert_eq!(r.calc_result().total_standard_units, 20);
        }

        #[test]
        fn per_segment_buckets_empty_by_default() {
            let r = SegmentAttendanceResult {
                calc_result: SegmentCalcResult::default(),
            };
            assert!(r.per_segment_buckets().is_empty());
            assert_eq!(r.total_standard_units(), 0);
        }
    }

    // =========================================================================
    // Module A: compute_daily_attendance pipeline tests
    // =========================================================================

    mod pipeline {
        use super::*;

        mod basic_work {
            use super::*;

            #[test]
            fn single_work_entry_within_bounds() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
                assert_eq!(r.ot_minutes(), 0);
                assert_eq!(r.actual_work_minutes(), 540);
            }

            #[test]
            fn single_work_entry_partial_shift() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(9, 0), dt(16, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 420);
                assert_eq!(r.ot_minutes(), 0);
            }

            #[test]
            fn exact_shift_boundaries() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
                assert_eq!(r.ot_minutes(), 0);
            }
        }

        mod break_entries {
            use super::*;

            #[test]
            fn two_punch_no_break_entries() {
                // Pipeline uses first punch_in and last punch_out only.
                // Without auto-deduct or Break entries, no break deduction applies.
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(8, 0, 17, 0))
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ])
                    .compute()
                    .unwrap();
                // actual_break_minutes = 0 (no Break entries)
                // shift has no auto-deduct → standard = 540 (9h * 60)
                assert_eq!(r.standard_minutes(), 540);
            }

            #[test]
            fn four_punch_with_break_entry() {
                let r = ComputeDailyInputBuilder::new()
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        mock_break_entry(dt(12, 0), dt(13, 0), 60),
                        mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ])
                    .compute()
                    .unwrap();
                // actual_break=60 passed to calculator; no auto-deduct → no additional deduction
                assert_eq!(r.standard_minutes(), 480);
            }

            #[test]
            fn break_entry_duration_summed() {
                let r = ComputeDailyInputBuilder::new()
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        mock_break_entry(dt(12, 0), dt(12, 30), 30),
                        mock_break_entry(dt(12, 30), dt(13, 0), 30),
                        mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ])
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 480);
            }

            #[test]
            fn break_entry_zero_duration() {
                // Break entry with duration_minutes=None should contribute 0
                let entry = TimeEntry::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    None,
                    None,
                    dt(12, 0),
                    Some(dt(13, 0)),
                    None, // duration = None → unwrap_or(0)
                    false,
                    EntryType::Break,
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                );
                let r = ComputeDailyInputBuilder::new()
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        entry,
                        mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ])
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                // duration=None → break contribution=0, auto-deduct finds 0 actual break
                // and falls back to scheduled_break=0 → no deduction → 540
                assert_eq!(r.standard_minutes(), 540);
            }
        }

        mod overnight_shift {
            use super::*;

            #[test]
            fn end_less_than_start_adds_one_day() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_overnight_schedule(22, 0, 6, 0))
                    .work_entry(dt(22, 0), dt_utc_d2(6, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 480);
                assert_eq!(r.work_buckets().late_ot_minutes, 0);
            }

            #[test]
            fn overnight_with_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_overnight_schedule(22, 0, 6, 0))
                    .work_entry(dt(21, 0), dt_utc_d2(6, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 480);
                assert_eq!(r.work_buckets().early_ot_minutes, 60);
            }

            #[test]
            fn overnight_punch_in_respected() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_overnight_schedule(22, 0, 6, 0))
                    .work_entry(dt(22, 0), dt_utc_d2(6, 0))
                    .compute()
                    .unwrap();
                // 8 hours = 480 minutes standard
                assert_eq!(r.standard_minutes(), 480);
            }
        }

        mod missing_punches {
            use super::*;

            #[test]
            fn empty_entries_returns_none() {
                let r = ComputeDailyInputBuilder::new().compute();
                assert!(r.is_none());
            }

            #[test]
            fn missing_punch_out_returns_none() {
                let entry = TimeEntry::new(
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    Uuid::new_v4(),
                    None,
                    None,
                    dt(8, 0),
                    None, // no punch_out
                    None,
                    false,
                    EntryType::Work,
                    None,
                    None,
                    Utc::now(),
                    Utc::now(),
                );
                let r = ComputeDailyInputBuilder::new()
                    .entries(vec![entry])
                    .compute();
                assert!(r.is_none());
            }

            #[test]
            fn single_entry_with_punch_out_returns_some() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(12, 0))
                    .compute();
                assert!(r.is_some());
            }
        }

        mod no_shift {
            use super::*;

            #[test]
            fn all_standard_minutes() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_no_shift_schedule())
                    .work_entry(dt(8, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
                assert_eq!(r.ot_minutes(), 0);
            }

            #[test]
            fn short_session() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_no_shift_schedule())
                    .work_entry(dt(9, 0), dt(9, 15))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 15);
            }

            #[test]
            fn with_break_entries() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_no_shift_schedule())
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        mock_break_entry(dt(12, 0), dt(13, 0), 60),
                        mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ])
                    .compute()
                    .unwrap();
                // No shift → all standard (break minutes still computed from entries)
                assert_eq!(r.standard_minutes(), 540);
            }
        }

        mod early_ot {
            use super::*;

            #[test]
            fn early_arrival_ot_when_enabled() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(6, 0), dt(17, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
                assert_eq!(r.work_buckets().early_ot_minutes, 120);
            }

            #[test]
            fn early_arrival_blocked_when_disabled() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(6, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 0);
                assert_eq!(r.standard_minutes(), 540);
            }

            #[test]
            fn early_ot_at_exact_shift_start_is_zero() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(17, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 0);
            }
        }

        mod late_ot {
            use super::*;

            #[test]
            fn uncapped_late_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(20, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
                assert_eq!(r.work_buckets().late_ot_minutes, 180);
            }

            #[test]
            fn exact_shift_end_zero_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().late_ot_minutes, 0);
            }

            #[test]
            fn one_minute_over() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(17, 1))
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().late_ot_minutes, 1);
            }
        }

        mod break_overtime {
            use super::*;

            #[test]
            fn fixed_worked_through_break() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_break(8, 0, 17, 0, 12, 0, 13, 0, 60))
                    .work_entry(dt(8, 0), dt(17, 0))
                    .rules_mut(|r| r.set_is_break_overtime(true))
                    .compute()
                    .unwrap();
                // No actual break taken → break_ot earned
                assert_eq!(r.standard_minutes(), 480);
                assert_eq!(r.work_buckets().break_ot_minutes, 60);
            }

            #[test]
            fn fixed_break_taken() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_break(8, 0, 17, 0, 12, 0, 13, 0, 60))
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        mock_break_entry(dt(12, 0), dt(13, 0), 60),
                        mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ])
                    .rules_mut(|r| r.set_is_break_overtime(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 480);
                assert_eq!(r.work_buckets().break_ot_minutes, 0);
            }

            #[test]
            fn flexible_break_worked() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_scheduled_break(8, 0, 17, 0, 60))
                    .rules_mut(|r| {
                        r.set_is_break_overtime(true);
                        r.set_is_break_auto_deduct(true);
                    })
                    .work_entry(dt(8, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                // Flexible mode: no frame → scheduled_break used as overlap
                assert_eq!(r.standard_minutes(), 480);
                assert_eq!(r.work_buckets().break_ot_minutes, 60);
            }
        }

        mod ot_ceiling_tests {
            use super::*;

            #[test]
            fn caps_late_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(8, 0, 12, 0))
                    .work_entry(dt(8, 0), dt(13, 30))
                    .ot_ceiling_at(13, 0)
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 240);
                // Late OT: punch_out=13:30, shift_end=12:00, ceiling=13:00
                // effective_punch_out = min(13:30, 13:00) = 13:00
                // late = 13:00 - 12:00 = 60
                assert_eq!(r.work_buckets().late_ot_minutes, 60);
            }

            #[test]
            fn ceiling_exact_boundary() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(8, 0, 12, 0))
                    .work_entry(dt(8, 0), dt(13, 0))
                    .ot_ceiling_at(13, 0)
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 240);
                assert_eq!(r.work_buckets().late_ot_minutes, 60);
            }

            #[test]
            fn ceiling_1min_before_punch_out() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(8, 0, 12, 0))
                    .work_entry(dt(8, 0), dt(12, 59))
                    .ot_ceiling_at(13, 0)
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().late_ot_minutes, 59);
            }

            #[test]
            fn no_ceiling_unlimited_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(8, 0, 12, 0))
                    .work_entry(dt(8, 0), dt(13, 30))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 240);
                assert_eq!(r.work_buckets().late_ot_minutes, 90);
            }
        }

        mod max_early_ot_cap {
            use super::*;

            #[test]
            fn caps_early_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(5, 0), dt(17, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .max_early_ot(120)
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 120);
            }

            #[test]
            fn cap_zero() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(5, 0), dt(17, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .max_early_ot(0)
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 0);
            }

            #[test]
            fn cap_higher_than_actual_not_limiting() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(5, 0), dt(17, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .max_early_ot(240)
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 180);
            }

            #[test]
            fn no_cap_unlimited() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(5, 0), dt(17, 0))
                    .rules_mut(|r| r.set_allow_early_ot(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 180);
            }
        }

        mod early_leave {
            use super::*;

            #[test]
            fn extends_standard_when_paid() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(16, 30))
                    .rules_mut(|r| {
                        r.set_authorized_early_leave_minutes(60);
                        r.set_is_early_leave_paid(true);
                    })
                    .compute()
                    .unwrap();
                // Exit 30min early, paid up to 60 → full shift
                assert_eq!(r.standard_minutes(), 540);
            }

            #[test]
            fn not_paid_when_flag_disabled() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(16, 30))
                    .rules_mut(|r| {
                        r.set_authorized_early_leave_minutes(60);
                        r.set_is_early_leave_paid(false);
                    })
                    .compute()
                    .unwrap();
                // Exit 30min early, not paid → 510 standard (8.5h)
                assert_eq!(r.standard_minutes(), 510);
            }

            #[test]
            fn exceeds_allowance() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(15, 0))
                    .rules_mut(|r| {
                        r.set_authorized_early_leave_minutes(60);
                        r.set_is_early_leave_paid(true);
                    })
                    .compute()
                    .unwrap();
                // Exit 120min early, only 60min covered → 420 (7h)
                assert_eq!(r.standard_minutes(), 420);
            }
        }

        mod grace_period {
            use super::*;

            #[test]
            fn within_grace_window_no_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(7, 30, 17, 0))
                    .work_entry(dt(7, 15), dt(17, 0))
                    .rules_mut(|r| {
                        r.set_grace_period_minutes(30);
                        r.set_allow_early_ot(true);
                    })
                    .compute()
                    .unwrap();
                // 7:15 is within 30-min grace before 7:30 → no early OT
                assert_eq!(r.work_buckets().early_ot_minutes, 0);
            }

            #[test]
            fn outside_grace_window() {
                // Punch at 6:59, grace threshold is 7:00, Allow strategy
                // 6:59 < 7:00 (grace threshold) → effective_ot_start = 6:59
                // early_ot = 7:30 - 6:59 = 31 minutes
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule(7, 30, 17, 0))
                    .work_entry(dt(6, 59), dt(17, 0))
                    .rules_mut(|r| {
                        r.set_grace_period_minutes(30);
                        r.set_allow_early_ot(true);
                    })
                    .compute()
                    .unwrap();
                assert_eq!(r.work_buckets().early_ot_minutes, 31);
            }

            #[test]
            fn clamp_strategy_caps_at_grace() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(6, 45), dt(17, 0))
                    .rules_mut(|r| {
                        r.set_grace_period_minutes(30);
                        r.set_allow_early_ot(true);
                        r.set_early_arrival_strategy(EarlyArrivalStrategy::Clamp);
                    })
                    .compute()
                    .unwrap();
                // Clamp: effective_ot_start = max(punch_in, grace_threshold)
                // grace_threshold = 8:00 - 30min = 7:30
                // effective_ot_start = max(6:45, 7:30) = 7:30
                // early_ot = shift_start(8:00) - 7:30 = 30
                assert_eq!(r.work_buckets().early_ot_minutes, 30);
            }

            #[test]
            fn block_strategy_within_grace_no_ot() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(7, 45), dt(17, 0))
                    .rules_mut(|r| {
                        r.set_grace_period_minutes(30);
                        r.set_allow_early_ot(true);
                        r.set_early_arrival_strategy(EarlyArrivalStrategy::Block);
                    })
                    .compute()
                    .unwrap();
                // Block: within grace → effective_ot_start = shift_start, so early_ot = 0
                assert_eq!(r.work_buckets().early_ot_minutes, 0);
            }

            #[test]
            fn block_strategy_out_of_grace() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(6, 45), dt(17, 0))
                    .rules_mut(|r| {
                        r.set_grace_period_minutes(30);
                        r.set_allow_early_ot(true);
                        r.set_early_arrival_strategy(EarlyArrivalStrategy::Block);
                    })
                    .compute()
                    .unwrap();
                // Block: out of grace → effective_ot_start = punch_in(6:45)
                // early_ot = shift_start(8:00) - 6:45 = 75
                assert_eq!(r.work_buckets().early_ot_minutes, 75);
            }
        }

        mod frame_intersection {
            use super::*;

            #[test]
            fn full_overlap() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_break(8, 0, 17, 0, 11, 30, 12, 30, 60))
                    .work_entry(dt(8, 0), dt(17, 0))
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                // Full overlap: 60min break fully overlaps with work → deduct 60
                assert_eq!(r.standard_minutes(), 480);
            }

            #[test]
            fn partial_overlap_leave_during_break() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_break(8, 0, 17, 0, 11, 30, 12, 30, 60))
                    .work_entry(dt(8, 0), dt(12, 0))
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                // Punch 8:00-12:00, break 11:30-12:30
                // Overlap: 11:30-12:00 = 30min
                // Standard: 4h - 30min = 210
                assert_eq!(r.standard_minutes(), 210);
            }

            #[test]
            fn no_overlap_leave_before_break() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_break(8, 0, 17, 0, 11, 30, 12, 30, 60))
                    .work_entry(dt(8, 0), dt(11, 0))
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 180);
            }

            #[test]
            fn arrive_during_break() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_break(8, 0, 17, 0, 11, 30, 12, 30, 60))
                    .work_entry(dt(12, 0), dt(17, 0))
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                // Punch 12:00-17:00, break 11:30-12:30 → overlap 12:00-12:30 = 30
                assert_eq!(r.standard_minutes(), 270);
            }
        }

        mod auto_deduct {
            use super::*;

            #[test]
            fn deducts_scheduled_when_no_actual() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_scheduled_break(8, 0, 17, 0, 60))
                    .work_entry(dt(8, 0), dt(17, 0))
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                // 540 - 60 = 480
                assert_eq!(r.standard_minutes(), 480);
            }

            #[test]
            fn actual_break_takes_precedence() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_scheduled_break(8, 0, 17, 0, 60))
                    .entries(vec![
                        mock_work_entry(dt(8, 0), dt(12, 0), 240),
                        mock_break_entry(dt(12, 0), dt(12, 45), 45),
                        mock_work_entry(dt(12, 45), dt(17, 0), 255),
                    ])
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                // actual_break=45, so deduction=45 (not the scheduled 60)
                assert_eq!(r.standard_minutes(), 495);
            }

            #[test]
            fn disabled_no_deduction() {
                let r = ComputeDailyInputBuilder::new()
                    .schedule(mock_schedule_with_scheduled_break(8, 0, 17, 0, 60))
                    .work_entry(dt(8, 0), dt(17, 0))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
            }

            #[test]
            fn zero_scheduled_no_deduction() {
                let r = ComputeDailyInputBuilder::new()
                    .work_entry(dt(8, 0), dt(17, 0))
                    .rules_mut(|r| r.set_is_break_auto_deduct(true))
                    .compute()
                    .unwrap();
                assert_eq!(r.standard_minutes(), 540);
            }
        }
    }

    // =========================================================================
    // Module C: compute_segment_attendance tests
    // =========================================================================

    mod segment_attendance {
        use super::*;

        mod empty {
            use super::*;

            #[test]
            fn empty_inputs_returns_default() {
                let r = AttendanceService::compute_segment_attendance(&[]);
                assert_eq!(r.total_standard_units(), 0);
                assert_eq!(r.total_ot_units(), 0);
                assert!(r.per_segment_buckets().is_empty());
            }
        }

        #[test]
        fn multiple_sessions_do_not_count_the_gap_as_work() {
            let tz = tz_utc();
            let rules = mock_rules();

            let mut morning = make_calc_input(
                dt_tz(tz, 8, 5),
                dt_tz(tz, 12, 0),
                dt_tz(tz, 8, 0),
                dt_tz(tz, 17, 0),
                30,
                &rules,
                None,
                None,
            );
            let mut afternoon = make_calc_input(
                dt_tz(tz, 13, 0),
                dt_tz(tz, 17, 0),
                dt_tz(tz, 8, 0),
                dt_tz(tz, 17, 0),
                30,
                &rules,
                None,
                None,
            );
            morning.rounding_policy = RoundingPolicy::Round;
            afternoon.rounding_policy = RoundingPolicy::Round;

            let sessions = vec![morning, afternoon];
            let groups = vec![sessions.as_slice()];
            let result = AttendanceService::compute_segment_attendance_sessions(&groups);

            // 08:05-12:00 + 13:00-17:00 = 475 minutes. The one-hour gap
            // must not be counted; rounding 475 minutes to 30-minute units
            // yields the expected eight hours (16 units).
            assert_eq!(result.total_standard_units(), 16);
            assert_eq!(result.total_ot_units(), 0);
            assert_eq!(result.per_segment_buckets()[0].1.standard_minutes, 475);
        }

        // ── compute_session_attendance (shared function) tests ──────────

        #[test]
        fn shared_multi_session_excludes_gap() {
            let tz = tz_utc();
            let rules = mock_rules();

            let morning = mock_work_entry(
                dt_tz(tz, 9, 0).with_timezone(&Utc),
                dt_tz(tz, 12, 0).with_timezone(&Utc),
                180,
            );
            let lunch = mock_break_entry(
                dt_tz(tz, 12, 0).with_timezone(&Utc),
                dt_tz(tz, 13, 0).with_timezone(&Utc),
                60,
            );
            let afternoon = mock_work_entry(
                dt_tz(tz, 13, 0).with_timezone(&Utc),
                dt_tz(tz, 17, 0).with_timezone(&Utc),
                240,
            );

            let entries: Vec<&TimeEntry> = vec![&morning, &lunch, &afternoon];
            let shift_start = Some(dt_tz(tz, 9, 0));
            let shift_end = Some(dt_tz(tz, 17, 0));

            let result = AttendanceService::compute_session_attendance(
                &entries,
                tz,
                shift_start,
                shift_end,
                None, // break_start
                None, // break_end
                60,   // scheduled_break_minutes (ignored for multi-session)
                30,
                RoundingPolicy::Round,
                &rules,
                None, // ot_ceiling
                None, // max_early_ot_minutes
                None, // segment_id
                OtAggregationMode::Aggregate,
            );

            // 09:00-12:00 + 13:00-17:00 = 420 minutes
            // 420 / 30 = 14 units = 7h
            assert_eq!(result.total_standard_units(), 14);
            assert_eq!(result.total_ot_units(), 0);
            assert_eq!(result.per_segment_buckets()[0].1.standard_minutes, 420);
        }

        #[test]
        fn shared_single_session_still_works() {
            let tz = tz_utc();
            let rules = mock_rules();

            let entry = mock_work_entry(
                dt_tz(tz, 9, 0).with_timezone(&Utc),
                dt_tz(tz, 17, 0).with_timezone(&Utc),
                480,
            );
            let entries: Vec<&TimeEntry> = vec![&entry];
            let shift_start = Some(dt_tz(tz, 9, 0));
            let shift_end = Some(dt_tz(tz, 17, 0));

            let result = AttendanceService::compute_session_attendance(
                &entries,
                tz,
                shift_start,
                shift_end,
                None,
                None,
                0,
                30,
                RoundingPolicy::Floor,
                &rules,
                None,
                None,
                None,
                OtAggregationMode::Aggregate,
            );

            // 09:00-17:00 = 480 min → 16 units with Floor rounding
            assert_eq!(result.total_standard_units(), 16);
            assert_eq!(result.total_ot_units(), 0);
        }

        #[test]
        fn shared_empty_entries_returns_zero() {
            let tz = tz_utc();
            let rules = mock_rules();
            let entries: Vec<&TimeEntry> = vec![];

            let result = AttendanceService::compute_session_attendance(
                &entries,
                tz,
                Some(dt_tz(tz, 9, 0)),
                Some(dt_tz(tz, 17, 0)),
                None,
                None,
                0,
                30,
                RoundingPolicy::Floor,
                &rules,
                None,
                None,
                None,
                OtAggregationMode::Aggregate,
            );

            assert_eq!(result.total_standard_units(), 0);
            assert_eq!(result.total_ot_units(), 0);
        }

        mod single_segment {
            use super::*;

            #[test]
            fn routes_to_daily_units() {
                let tz = tz_utc();
                let rules = mock_rules();
                let input = make_calc_input(
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 17, 0),
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 17, 0),
                    30,
                    &rules,
                    None,
                    None,
                );
                let r = AttendanceService::compute_segment_attendance(&[input]);
                assert_eq!(r.total_standard_units(), 18);
                assert_eq!(r.total_ot_units(), 0);
                assert_eq!(r.per_segment_buckets().len(), 1);
            }

            #[test]
            fn populates_per_segment_with_id() {
                let tz = tz_utc();
                let seg_id = Uuid::new_v4();
                let rules = mock_rules();
                let input = make_calc_input(
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 17, 0),
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 17, 0),
                    30,
                    &rules,
                    None,
                    Some(seg_id),
                );
                let r = AttendanceService::compute_segment_attendance(&[input]);
                assert_eq!(r.per_segment_buckets()[0].0, Some(seg_id));
            }

            #[test]
            fn with_ot() {
                let tz = tz_utc();
                let mut rules = mock_rules();
                rules.set_allow_early_ot(true);
                let input = make_calc_input(
                    dt_tz(tz, 7, 0),
                    dt_tz(tz, 17, 0),
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 17, 0),
                    30,
                    &rules,
                    None,
                    None,
                );
                let r = AttendanceService::compute_segment_attendance(&[input]);
                assert_eq!(r.total_standard_units(), 18);
                assert_eq!(r.total_ot_units(), 2);
            }
        }

        mod multi_segment {
            use super::*;

            #[test]
            fn two_segments_aggregate() {
                let tz = tz_utc();
                let rules = mock_rules();
                let input1 = make_calc_input(
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 12, 0),
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 12, 0),
                    30,
                    &rules,
                    None,
                    None,
                );
                let input2 = make_calc_input(
                    dt_tz(tz, 13, 0),
                    dt_tz(tz, 17, 0),
                    dt_tz(tz, 13, 0),
                    dt_tz(tz, 17, 0),
                    30,
                    &rules,
                    None,
                    None,
                );
                let r = AttendanceService::compute_segment_attendance(&[input1, input2]);
                assert_eq!(r.total_standard_units(), 16); // 480/30
                assert_eq!(r.per_segment_buckets().len(), 2);
            }

            #[test]
            fn two_segments_aggregate_with_ot() {
                let tz = tz_utc();
                let rules = mock_rules();
                let input1 = make_calc_input(
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 12, 0),
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 12, 0),
                    30,
                    &rules,
                    None,
                    None,
                );
                // Seg2 ends at 19:00 → 60min late OT
                let input2 = make_calc_input(
                    dt_tz(tz, 13, 0),
                    dt_tz(tz, 19, 0),
                    dt_tz(tz, 13, 0),
                    dt_tz(tz, 17, 0),
                    30,
                    &rules,
                    None,
                    None,
                );
                let r = AttendanceService::compute_segment_attendance(&[input1, input2]);
                // 240 + 360 = 600 std mins → 600/30 = 20. No wait, seg2 has shift_end=17:00 but punch_out=19:00
                // So seg2: standard = 240, late_ot = 120
                // Aggregate: std = 240 + 240 = 480/30 = 16, ot = 120/30 = 4
                // Wait, actually seg2 punch_out=19:00, shift_end=17:00 → late_ot=120
                // But standard is capped at shift_end, so seg2_std = 240, seg2_late_ot = 120
                // Total: std = 240 + 240 = 480, ot = 120
                // But with Aggregate mode, the total OT minutes (120) is aggregated, and 120/30 = 4
                // Hmm, Aggregate sums all standard and all OT minutes across segments, then applies unit conversion
                assert_eq!(r.total_standard_units(), 16);
                assert_eq!(r.total_ot_units(), 4);
            }

            #[test]
            fn per_segment_buckets_match_ids() {
                let tz = tz_utc();
                let rules = mock_rules();
                let seg0 = Uuid::new_v4();
                let seg1 = Uuid::new_v4();
                let input1 = make_calc_input(
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 12, 0),
                    dt_tz(tz, 8, 0),
                    dt_tz(tz, 12, 0),
                    30,
                    &rules,
                    None,
                    Some(seg0),
                );
                let input2 = make_calc_input(
                    dt_tz(tz, 13, 0),
                    dt_tz(tz, 17, 0),
                    dt_tz(tz, 13, 0),
                    dt_tz(tz, 17, 0),
                    30,
                    &rules,
                    None,
                    Some(seg1),
                );
                let r = AttendanceService::compute_segment_attendance(&[input1, input2]);
                assert_eq!(r.per_segment_buckets()[0].0, Some(seg0));
                assert_eq!(r.per_segment_buckets()[1].0, Some(seg1));
            }
        }
    }

    // =========================================================================
    // Module E: Edge cases
    // =========================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn timezone_hcm_gives_same_standard_minutes() {
            let tz = tz_hcm();
            // Build TimeEntry with HCM-local times (08:00 HCM → UTC, 17:00 HCM → UTC)
            let hcm_8am = tz
                .from_local_datetime(&TEST_DATE.and_hms_opt(8, 0, 0).unwrap())
                .single()
                .expect("valid HCM time")
                .with_timezone(&Utc);
            let hcm_5pm = tz
                .from_local_datetime(&TEST_DATE.and_hms_opt(17, 0, 0).unwrap())
                .single()
                .expect("valid HCM time")
                .with_timezone(&Utc);
            let dur = (hcm_5pm - hcm_8am).num_minutes() as i32;

            let schedule = EffectiveSchedule::new(
                TEST_DATE,
                Some(Uuid::new_v4()),
                "test".to_string(),
                Some("Test".to_string()),
                None,
                NaiveTime::from_hms_opt(8, 0, 0),
                NaiveTime::from_hms_opt(17, 0, 0),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                vec![],
                false,
                false,
                None,
            );
            let entry = TimeEntry::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                None,
                None,
                hcm_8am,
                Some(hcm_5pm),
                Some(dur),
                false,
                EntryType::Work,
                None,
                None,
                Utc::now(),
                Utc::now(),
            );
            let rules = mock_rules();

            let entry_refs: Vec<&TimeEntry> = vec![&entry];
            let input = ComputeDailyInput {
                daily: &schedule,
                day_entries: &entry_refs,
                date: TEST_DATE,
                tz: &tz,
                rules: &rules,
                work_unit_minutes: 30,
                rounding_policy: RoundingPolicy::Floor,
                max_early_ot_minutes: None,
                ot_ceiling: None,
                segment_id: None,
                ot_aggregation_mode: OtAggregationMode::Aggregate,
            };
            let r = AttendanceService::compute_daily_attendance(input).unwrap();
            // 9 hours (08:00-17:00 HCM) = 540 minutes standard
            assert_eq!(r.standard_minutes(), 540);
        }

        #[test]
        fn shift_start_none_but_end_some_becomes_none_shift() {
            // If start_time is None, shift_naive_to_dt returns None,
            // and the if-let (Some(start), Some(end)) fails → (None, None)
            let schedule = EffectiveSchedule::new(
                TEST_DATE,
                Some(Uuid::new_v4()),
                "test".to_string(),
                None,
                None,
                None,                              // start_time = None
                NaiveTime::from_hms_opt(17, 0, 0), // end_time = Some
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                vec![],
                false,
                false,
                None,
            );
            let builder = ComputeDailyInputBuilder::new()
                .schedule(schedule)
                .work_entry(dt(8, 0), dt(17, 0));
            // Should NOT panic — pipeline handles partial shift gracefully
            let r = builder.compute().unwrap();
            // No shift → all standard
            assert_eq!(r.standard_minutes(), 540);
        }

        #[test]
        fn break_start_some_end_none_becomes_none_break() {
            let schedule = EffectiveSchedule::new(
                TEST_DATE,
                Some(Uuid::new_v4()),
                "test".to_string(),
                None,
                None,
                NaiveTime::from_hms_opt(8, 0, 0),
                NaiveTime::from_hms_opt(17, 0, 0),
                Some(60),
                NaiveTime::from_hms_opt(12, 0, 0), // break_start = Some
                None,                              // break_end = None
                Some(true),
                None,
                None,
                None,
                None,
                vec![],
                false,
                false,
                None,
            );
            let builder = ComputeDailyInputBuilder::new()
                .schedule(schedule)
                .rules_mut(|r| r.set_is_break_auto_deduct(true))
                .work_entry(dt(8, 0), dt(17, 0));
            // if-let (Some(bs), Some(be)) fails → break_start/end both None
            let r = builder.compute().unwrap();
            assert_eq!(r.standard_minutes(), 480); // flat deduction of 60
        }

        #[test]
        fn overtime_entries_not_summed_as_breaks() {
            let ot_entry = TimeEntry::new(
                Uuid::new_v4(),
                Uuid::new_v4(),
                Uuid::new_v4(),
                None,
                None,
                dt(17, 0),
                Some(dt(18, 0)),
                Some(60),
                false,
                EntryType::Overtime,
                None,
                None,
                Utc::now(),
                Utc::now(),
            );
            let r = ComputeDailyInputBuilder::new()
                .entries(vec![
                    mock_work_entry(dt(8, 0), dt(12, 0), 240),
                    mock_break_entry(dt(12, 0), dt(13, 0), 60),
                    mock_work_entry(dt(13, 0), dt(17, 0), 240),
                    ot_entry,
                ])
                .compute()
                .unwrap();
            // Overtime entry not counted as Break → actual_break = 60 from Break entry
            assert_eq!(r.standard_minutes(), 480);
        }

        #[test]
        fn mixed_work_break_entries_order_respected() {
            // First entry's punch_in → punch_in, last entry's punch_out → punch_out
            let r = ComputeDailyInputBuilder::new()
                .entries(vec![
                    mock_work_entry(dt(8, 0), dt(12, 0), 240),
                    mock_break_entry(dt(12, 0), dt(13, 0), 60),
                    mock_work_entry(dt(13, 0), dt(17, 0), 240),
                ])
                .compute()
                .unwrap();
            assert_eq!(r.standard_minutes(), 480); // 540 - 60 break
        }

        #[test]
        fn overnight_break_end_less_than_start_adds_day() {
            // Shift 22:00-06:00, break 01:00-00:30 (break_end < break_start)
            let schedule = EffectiveSchedule::new(
                TEST_DATE,
                Some(Uuid::new_v4()),
                "test".to_string(),
                Some("Night".to_string()),
                None,
                NaiveTime::from_hms_opt(22, 0, 0),
                NaiveTime::from_hms_opt(6, 0, 0),
                Some(30),
                NaiveTime::from_hms_opt(1, 0, 0), // break_start = 01:00
                NaiveTime::from_hms_opt(0, 30, 0), // break_end = 00:30 (next day)
                Some(true),
                None,
                None,
                None,
                None,
                vec![],
                false,
                false,
                None,
            );
            let builder = ComputeDailyInputBuilder::new()
                .schedule(schedule)
                .rules_mut(|r| r.set_is_break_auto_deduct(true))
                .work_entry(dt(22, 0), dt_utc_d2(6, 0));
            let r = builder.compute().unwrap();
            // break after adjustment: end=date+1 00:30 → overlap calc should work
            // shift is 22:00-06:00(+1) → 8h = 480 min
            assert!(
                r.standard_minutes() > 0,
                "overnight shift with overnight break should compute"
            );
        }
    }

    // =========================================================================
    // DST pipeline tests — verify compute_daily_attendance on transition days
    // =========================================================================

    mod dst_pipeline {
        use super::*;

        fn ny() -> Tz {
            "America/New_York".parse().unwrap()
        }

        /// Construct a UTC DateTime from a local date/time in a specific timezone.
        fn dt_local(tz: Tz, date: NaiveDate, h: u32, m: u32) -> chrono::DateTime<Utc> {
            crate::utils::tz::naive_time_to_tz(NaiveTime::from_hms_opt(h, m, 0).unwrap(), date, tz)
                .expect("valid local time")
                .with_timezone(&Utc)
        }

        #[test]
        fn overnight_shift_spring_forward_7h() {
            // Shift 22:00-06:00 on Mar 7 NY (day before spring-forward).
            // Punch: 22:00 EST Mar 7 → 06:00 EDT Mar 8 = 7 real hours.
            let tz = ny();
            let date = NaiveDate::from_ymd_opt(2026, 3, 7).unwrap();

            let punch_in = dt_local(tz, date, 22, 0); // 22:00 EST = 03:00 UTC Mar 8
            let punch_out = dt_local(tz, NaiveDate::from_ymd_opt(2026, 3, 8).unwrap(), 6, 0); // 06:00 EDT = 10:00 UTC Mar 8

            let r = ComputeDailyInputBuilder::new()
                .tz(tz)
                .date(date)
                .schedule(mock_schedule(22, 0, 6, 0))
                .work_entry(punch_in, punch_out)
                .compute()
                .unwrap();

            // 7 real hours = 420 minutes (spring-forward lost 1h)
            assert_eq!(
                r.standard_minutes(),
                420,
                "overnight spring-forward shift should be 7h, got {}min",
                r.standard_minutes()
            );
        }

        #[test]
        fn overnight_shift_fall_back_9h() {
            // Shift 22:00-06:00 on Oct 31 NY (day before fall-back).
            // Punch: 22:00 EDT Oct 31 → 06:00 EST Nov 1 = 9 real hours.
            let tz = ny();
            let date = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();

            let punch_in = dt_local(tz, date, 22, 0); // 22:00 EDT = 02:00 UTC Nov 1
            let punch_out = dt_local(tz, NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(), 6, 0); // 06:00 EST = 11:00 UTC Nov 1

            let r = ComputeDailyInputBuilder::new()
                .tz(tz)
                .date(date)
                .schedule(mock_schedule(22, 0, 6, 0))
                .work_entry(punch_in, punch_out)
                .compute()
                .unwrap();

            // 9 real hours = 540 minutes (fall-back gained 1h)
            assert_eq!(
                r.standard_minutes(),
                540,
                "overnight fall-back shift should be 9h, got {}min",
                r.standard_minutes()
            );
        }

        #[test]
        fn normal_day_shift_on_spring_forward_day() {
            // Shift 08:00-17:00 on Mar 8 NY (spring-forward day itself).
            // DST gap at 02:00-03:00 doesn't affect 08:00-17:00 shift.
            let tz = ny();
            let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();

            let punch_in = dt_local(tz, date, 8, 0);
            let punch_out = dt_local(tz, date, 17, 0);

            let r = ComputeDailyInputBuilder::new()
                .tz(tz)
                .date(date)
                .schedule(mock_schedule(8, 0, 17, 0))
                .work_entry(punch_in, punch_out)
                .compute()
                .unwrap();

            // 9 hours = 540 minutes (gap is outside shift, so unaffected)
            assert_eq!(
                r.standard_minutes(),
                540,
                "day shift on spring-forward day should be 9h, got {}min",
                r.standard_minutes()
            );
        }

        #[test]
        fn overnight_shift_no_dst() {
            // Normal overnight shift 22:00-06:00, no DST transition.
            let tz = ny();
            let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

            let punch_in = dt_local(tz, date, 22, 0);
            let punch_out = dt_local(tz, NaiveDate::from_ymd_opt(2026, 6, 16).unwrap(), 6, 0);

            let r = ComputeDailyInputBuilder::new()
                .tz(tz)
                .date(date)
                .schedule(mock_schedule(22, 0, 6, 0))
                .work_entry(punch_in, punch_out)
                .compute()
                .unwrap();

            // 8 hours = 480 minutes
            assert_eq!(
                r.standard_minutes(),
                480,
                "normal overnight shift should be 8h, got {}min",
                r.standard_minutes()
            );
        }
    }
}
