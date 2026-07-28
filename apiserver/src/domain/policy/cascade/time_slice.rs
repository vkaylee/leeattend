//! Time-slice decomposition ([[ADR-003]]).
//!
//! Cuts a shift at local-midnight boundaries in the workspace timezone; each
//! slice carries its own resolved `day_type`. Phase 1 targets a single
//! timezone (Asia/Ho_Chi_Minh) — the interface accepts an explicit `Tz` so
//! Phase 2 can extend to per-workspace timezones without changing the
//! signature.
//!
//! A slice whose duration is `<= MIN_SLICE_MINUTES` is merged into its right
//! neighbor (or into its left neighbor if it is the trailing slice). This
//! prevents pathological shifts like 23:59 → 00:01 from producing two
//! 1-minute slivers that each pay the cost of cascade evaluation.
//!
//! This module only *produces* slices; it does not decide anchoring per rule
//! type — that is the eval layer's responsibility. The `Anchoring` enum is
//! the vocabulary evaluators use to pick which slice(s) apply per rule.

use chrono::{DateTime, NaiveDate, TimeZone};
use chrono_tz::Tz;

/// Slices with total duration ≤ this threshold merge into a neighbor.
/// See ADR-003 §Consequences.
pub const MIN_SLICE_MINUTES: i64 = 1;

/// The shift's local-time window, tz-aware.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShiftSchedule {
    start: DateTime<Tz>,
    end: DateTime<Tz>,
}

impl ShiftSchedule {
    pub fn new(start: DateTime<Tz>, end: DateTime<Tz>) -> Self {
        Self { start, end }
    }

    pub fn start(&self) -> DateTime<Tz> {
        self.start
    }
    pub fn end(&self) -> DateTime<Tz> {
        self.end
    }
}

/// One evaluation-unit produced by `decompose_shift`.
///
/// `day_type` is a free-form identifier resolved by the caller — the
/// day-type taxonomy (`"weekday"`, `"sunday"`, `"holiday_national"`, …) is
/// owned by the policy/holiday layer, not by this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice {
    start: DateTime<Tz>,
    end: DateTime<Tz>,
    day_type: String,
}

impl Slice {
    pub fn new(start: DateTime<Tz>, end: DateTime<Tz>, day_type: impl Into<String>) -> Self {
        Self {
            start,
            end,
            day_type: day_type.into(),
        }
    }

    pub fn start(&self) -> DateTime<Tz> {
        self.start
    }
    pub fn end(&self) -> DateTime<Tz> {
        self.end
    }
    pub fn day_type(&self) -> &str {
        &self.day_type
    }
    pub fn duration_minutes(&self) -> i64 {
        (self.end - self.start).num_minutes()
    }
}

/// Per-rule anchoring — how each rule type reads slice output.
/// See ADR-003 §Anchoring Matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchoring {
    /// Punch validation (in/out grace, geofence) — use the slice containing
    /// the real tap timestamp.
    RealTapSlice,
    /// Late/early classification — use the overall shift schedule, not
    /// slices. Keeps HR mental model stable when a shift crosses day types.
    OverallShift,
    /// OT & pay multipliers — evaluate independently on each slice.
    PerSlice,
    /// Work-hour aggregation — total across slices for display, plus
    /// per-slice breakdown for payroll.
    TotalPlusBreakdown,
}

/// Decompose a shift into slices at local-midnight boundaries.
///
/// `day_type_of` resolves a local date to its day-type identifier; called
/// once per raw slice's start date, before sliver merging. Kept as a closure
/// so day-type resolution (holiday calendar, weekday map) stays in the
/// caller layer where DB access lives.
///
/// Returns an empty vec for zero-length or reversed ranges.
pub fn decompose_shift<F>(schedule: &ShiftSchedule, tz: &Tz, day_type_of: F) -> Vec<Slice>
where
    F: Fn(NaiveDate) -> String,
{
    if schedule.end() <= schedule.start() {
        return Vec::new();
    }

    let start_local = schedule.start().with_timezone(tz);
    let end_local = schedule.end().with_timezone(tz);

    let mut cuts: Vec<DateTime<Tz>> = vec![start_local];
    let mut cursor_date = start_local.date_naive();
    while let Some(next_date) = cursor_date.succ_opt() {
        let Some(midnight) = local_midnight(*tz, next_date) else {
            break;
        };
        if midnight >= end_local {
            break;
        }
        cuts.push(midnight);
        cursor_date = next_date;
    }
    cuts.push(end_local);

    let mut slices: Vec<Slice> = cuts
        .windows(2)
        .map(|w| Slice::new(w[0], w[1], day_type_of(w[0].date_naive())))
        .collect();

    merge_slivers(&mut slices);
    slices
}

/// Local 00:00 of `date` in `tz`, with a Phase-1 tolerance for the DST-gap
/// edge case (Vietnam has no DST, so this branch is never exercised in
/// production — kept so Phase 2 tz expansion doesn't panic).
fn local_midnight(tz: Tz, date: NaiveDate) -> Option<DateTime<Tz>> {
    let naive = date.and_hms_opt(0, 0, 0)?;
    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) | chrono::LocalResult::Ambiguous(dt, _) => Some(dt),
        chrono::LocalResult::None => {
            let shifted = date.and_hms_opt(1, 0, 0)?;
            tz.from_local_datetime(&shifted).single()
        }
    }
}

/// In-place sliver merge: any slice with duration `<= MIN_SLICE_MINUTES`
/// merges into its right neighbor. A trailing sliver merges into its left
/// neighbor (there is no right). Idempotent — a single pass is enough
/// because merging only ever extends a neighbor, never shortens.
fn merge_slivers(slices: &mut Vec<Slice>) {
    if slices.len() <= 1 {
        return;
    }
    let mut i = 0;
    while i + 1 < slices.len() {
        if slices[i].duration_minutes() <= MIN_SLICE_MINUTES {
            let sliver_start = slices[i].start();
            slices[i + 1].start = sliver_start;
            slices.remove(i);
        } else {
            i += 1;
        }
    }
    if slices.len() >= 2 {
        let last = slices.len() - 1;
        if slices[last].duration_minutes() <= MIN_SLICE_MINUTES {
            let sliver_end = slices[last].end();
            slices[last - 1].end = sliver_end;
            slices.remove(last);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::tz::local_to_tz;
    use chrono::Datelike;

    fn hcm() -> Tz {
        chrono_tz::Asia::Ho_Chi_Minh
    }

    fn weekday_or_sunday(date: NaiveDate) -> String {
        match date.weekday() {
            chrono::Weekday::Sun => "sunday".to_string(),
            _ => "weekday".to_string(),
        }
    }

    fn schedule(start: DateTime<Tz>, end: DateTime<Tz>) -> ShiftSchedule {
        ShiftSchedule::new(start, end)
    }

    #[test]
    fn same_day_shift_produces_one_slice() {
        let tz = hcm();
        // 2026-07-22 is a Wednesday.
        let d = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let start = local_to_tz(&tz, d, 8, 0, 0).unwrap();
        let end = local_to_tz(&tz, d, 17, 0, 0).unwrap();
        let slices = decompose_shift(&schedule(start, end), &tz, weekday_or_sunday);
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].start(), start);
        assert_eq!(slices[0].end(), end);
        assert_eq!(slices[0].day_type(), "weekday");
        assert_eq!(slices[0].duration_minutes(), 9 * 60);
    }

    #[test]
    fn overnight_shift_across_day_type_splits_at_midnight() {
        // Ca 22:00 CN → 06:00 T2 (Lễ) — ADR-003 canonical example.
        let tz = hcm();
        // 2026-07-26 is Sunday, 2026-07-27 is Monday.
        let sun = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
        let mon = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
        let start = local_to_tz(&tz, sun, 22, 0, 0).unwrap();
        let end = local_to_tz(&tz, mon, 6, 0, 0).unwrap();
        // Caller marks Monday as a holiday for this test.
        let resolver = |d: NaiveDate| -> String {
            if d == mon {
                "holiday_national".to_string()
            } else {
                "sunday".to_string()
            }
        };
        let slices = decompose_shift(&schedule(start, end), &tz, resolver);
        assert_eq!(slices.len(), 2);
        assert_eq!(slices[0].day_type(), "sunday");
        assert_eq!(slices[1].day_type(), "holiday_national");
        assert_eq!(slices[0].duration_minutes(), 2 * 60);
        assert_eq!(slices[1].duration_minutes(), 6 * 60);
        // Boundary continuity: slice[0].end == slice[1].start.
        assert_eq!(slices[0].end(), slices[1].start());
    }

    #[test]
    fn short_cross_midnight_merges_into_single_slice() {
        // 23:59 → 00:01 — each half is 1 minute, ≤ MIN → merge.
        let tz = hcm();
        let sun = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
        let mon = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
        let start = local_to_tz(&tz, sun, 23, 59, 0).unwrap();
        let end = local_to_tz(&tz, mon, 0, 1, 0).unwrap();
        let slices = decompose_shift(&schedule(start, end), &tz, weekday_or_sunday);
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].start(), start);
        assert_eq!(slices[0].end(), end);
        // Left sliver absorbs into right → right's day_type wins.
        assert_eq!(slices[0].day_type(), "weekday");
    }

    #[test]
    fn three_day_marathon_produces_three_slices() {
        let tz = hcm();
        let d1 = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap(); // Wed
        let d3 = NaiveDate::from_ymd_opt(2026, 7, 24).unwrap(); // Fri
        let start = local_to_tz(&tz, d1, 8, 0, 0).unwrap();
        let end = local_to_tz(&tz, d3, 12, 0, 0).unwrap();
        let slices = decompose_shift(&schedule(start, end), &tz, weekday_or_sunday);
        assert_eq!(slices.len(), 3);
        assert_eq!(slices[0].duration_minutes(), 16 * 60); // 08:00→24:00
        assert_eq!(slices[1].duration_minutes(), 24 * 60); // full day
        assert_eq!(slices[2].duration_minutes(), 12 * 60); // 00:00→12:00
        assert_eq!(slices[0].end(), slices[1].start());
        assert_eq!(slices[1].end(), slices[2].start());
    }

    #[test]
    fn trailing_sliver_absorbs_into_left() {
        // 08:00 → 24:00:30 (30 s after midnight is a sliver on the right side).
        let tz = hcm();
        let d = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let next = NaiveDate::from_ymd_opt(2026, 7, 23).unwrap();
        let start = local_to_tz(&tz, d, 8, 0, 0).unwrap();
        let end = local_to_tz(&tz, next, 0, 0, 30).unwrap();
        let slices = decompose_shift(&schedule(start, end), &tz, weekday_or_sunday);
        // Trailing 30-s slice merges into the 16h slice on the left.
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].start(), start);
        assert_eq!(slices[0].end(), end);
    }

    #[test]
    fn zero_length_shift_returns_empty() {
        let tz = hcm();
        let d = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let t = local_to_tz(&tz, d, 8, 0, 0).unwrap();
        let slices = decompose_shift(&schedule(t, t), &tz, weekday_or_sunday);
        assert!(slices.is_empty());
    }

    #[test]
    fn reversed_range_returns_empty() {
        let tz = hcm();
        let d = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let start = local_to_tz(&tz, d, 10, 0, 0).unwrap();
        let end = local_to_tz(&tz, d, 8, 0, 0).unwrap();
        let slices = decompose_shift(&schedule(start, end), &tz, weekday_or_sunday);
        assert!(slices.is_empty());
    }

    #[test]
    fn shift_ending_exactly_at_midnight_stays_single_slice() {
        // 08:00 → 24:00 exact. No slice on the "next" day.
        let tz = hcm();
        let d = NaiveDate::from_ymd_opt(2026, 7, 22).unwrap();
        let next = NaiveDate::from_ymd_opt(2026, 7, 23).unwrap();
        let start = local_to_tz(&tz, d, 8, 0, 0).unwrap();
        let end = local_to_tz(&tz, next, 0, 0, 0).unwrap();
        let slices = decompose_shift(&schedule(start, end), &tz, weekday_or_sunday);
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].duration_minutes(), 16 * 60);
    }

    #[test]
    fn anchoring_variants_are_distinct() {
        // Guards against accidental collapse of Anchoring variants.
        let all = [
            Anchoring::RealTapSlice,
            Anchoring::OverallShift,
            Anchoring::PerSlice,
            Anchoring::TotalPlusBreakdown,
        ];
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                assert_eq!(i == j, a == b);
            }
        }
    }
}
