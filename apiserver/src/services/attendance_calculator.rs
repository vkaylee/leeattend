use crate::models::{EarlyArrivalStrategy, PolicyRules};
use chrono::{DateTime, Duration, TimeZone};

#[derive(Debug, Default)]
pub struct WorkBuckets {
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
}

/// Encapsulates all inputs for daily attendance calculation.
///
/// Replaces the 10-parameter function signatures in `calculate_daily_units`
/// and `calculate_buckets`, improving readability at call sites.
#[derive(Debug)]
pub struct DailyCalcInput<T: TimeZone> {
    pub punch_in: DateTime<T>,
    pub punch_out: DateTime<T>,
    pub shift_start: Option<DateTime<T>>,
    pub shift_end: Option<DateTime<T>>,
    pub scheduled_break_minutes: i32,
    pub actual_break_minutes: i32,
    pub work_unit: i32,
    pub rounding_policy: String,
    pub rules: PolicyRules,
    pub ot_ceiling: Option<DateTime<T>>,
}

pub struct AttendanceCalculator;

impl AttendanceCalculator {
    /// Calculate standard and OT buckets using structured input.
    pub fn calculate_buckets<T: TimeZone>(input: &DailyCalcInput<T>) -> WorkBuckets {
        let punch_in = &input.punch_in;
        let punch_out = &input.punch_out;
        let shift_start_dt = input.shift_start.clone();
        let shift_end_dt = input.shift_end.clone();
        let scheduled_break_minutes = input.scheduled_break_minutes;
        let actual_break_minutes = input.actual_break_minutes;
        let rules = &input.rules;
        let ot_ceiling = &input.ot_ceiling;

        let mut buckets = WorkBuckets::default();

        if punch_out <= punch_in {
            return buckets;
        }

        match (shift_start_dt, shift_end_dt) {
            (Some(start), Some(mut end)) => {
                // Handle Overnight Shift linearization
                if end < start {
                    end += Duration::days(1);
                }

                // Apply Grace Period to Shift Start for Early OT calculation
                let mut effective_ot_start = punch_in.clone();
                if rules.grace_period_minutes > 0 {
                    let grace_threshold =
                        start.clone() - Duration::minutes(rules.grace_period_minutes as i64);

                    match rules.early_arrival_strategy {
                        EarlyArrivalStrategy::Block | EarlyArrivalStrategy::Allow => {
                            if punch_in >= &grace_threshold && punch_in < &start {
                                effective_ot_start = start.clone();
                            } else if punch_in < &grace_threshold {
                                effective_ot_start = punch_in.clone();
                            }
                        }
                        EarlyArrivalStrategy::Clamp => {
                            effective_ot_start = DateTime::max(punch_in.clone(), grace_threshold);
                        }
                    }
                } else {
                    effective_ot_start = punch_in.clone();
                }

                // 1. Standard Bucket: Intersection of [In, Out] and [Start, End]
                let standard_in = DateTime::max(punch_in.clone(), start.clone());
                let mut standard_out = DateTime::min(punch_out.clone(), end.clone());

                // Apply Authorized Early Leave (Paid)
                if rules.is_early_leave_paid && rules.authorized_early_leave_minutes > 0 {
                    let missing_minutes = end
                        .clone()
                        .signed_duration_since(standard_out.clone())
                        .num_minutes();
                    if missing_minutes > 0
                        && missing_minutes <= rules.authorized_early_leave_minutes as i64
                    {
                        standard_out = end.clone();
                    }
                }

                if standard_out > standard_in {
                    let mut standard_raw = standard_out
                        .signed_duration_since(standard_in)
                        .num_minutes() as i32;

                    // Deduct break from standard time
                    let break_to_deduct = Self::calculate_break_deduction(
                        actual_break_minutes,
                        scheduled_break_minutes,
                        rules.is_break_auto_deduct,
                        rules.is_break_overtime,
                    );
                    standard_raw = i32::max(0, standard_raw - break_to_deduct);
                    buckets.standard_minutes = standard_raw;
                }

                // 2. Early OT: Time before ShiftStart
                if rules.allow_early_ot && punch_in < &start {
                    let early_out = DateTime::min(punch_out.clone(), start);
                    if early_out > effective_ot_start {
                        buckets.early_ot_minutes = early_out
                            .signed_duration_since(effective_ot_start)
                            .num_minutes()
                            as i32;
                    }
                }

                // 3. Late OT: Time after ShiftEnd, capped at ot_ceiling
                if punch_out > &end {
                    let effective_punch_out = match ot_ceiling {
                        Some(ceil) => DateTime::min(punch_out.clone(), ceil.clone()),
                        None => punch_out.clone(),
                    };
                    let late_in = DateTime::max(punch_in.clone(), end.clone());
                    if effective_punch_out > late_in {
                        buckets.late_ot_minutes = effective_punch_out
                            .signed_duration_since(late_in)
                            .num_minutes() as i32;
                    }
                }
            }
            _ => {
                // No shift: everything is standard
                let raw_duration = punch_out
                    .clone()
                    .signed_duration_since(punch_in.clone())
                    .num_minutes() as i32;
                buckets.standard_minutes = i32::max(0, raw_duration);
            }
        }

        buckets
    }

    /// Break Deduction Logic
    pub fn calculate_break_deduction(
        actual_break_minutes: i32,
        scheduled_break_minutes: i32,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
    ) -> i32 {
        if actual_break_minutes > 0 {
            actual_break_minutes
        } else if is_break_auto_deduct && !is_break_overtime {
            scheduled_break_minutes
        } else {
            0
        }
    }

    /// Unit Calculation Logic
    pub fn calculate_units(minutes: i32, unit_size: i32, policy: &str) -> i32 {
        let size = if unit_size <= 1 {
            1.0
        } else {
            unit_size as f64
        };
        let mins = minutes as f64;

        match policy {
            "floor" => (mins / size).floor() as i32,
            "ceil" => (mins / size).ceil() as i32,
            "round" => (mins / size).round() as i32,
            _ => (mins / size) as i32,
        }
    }

    /// High-level method to calculate daily standard and OT units in one go.
    /// `ot_ceiling` caps late OT at the next shift's start (for split shifts).
    pub fn calculate_daily_units<T: TimeZone>(input: &DailyCalcInput<T>) -> (i32, i32) {
        let buckets = Self::calculate_buckets(input);

        let std_units = Self::calculate_units(
            buckets.standard_minutes,
            input.work_unit,
            &input.rounding_policy,
        );
        let ot_units = Self::calculate_units(
            buckets.early_ot_minutes + buckets.late_ot_minutes,
            input.work_unit,
            &input.rounding_policy,
        );

        (std_units, ot_units)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, Utc};

    fn dt(h: u32, m: u32) -> DateTime<Utc> {
        Utc.from_local_datetime(
            &NaiveDate::from_ymd_opt(2026, 1, 1)
                .expect("valid date")
                .and_time(NaiveTime::from_hms_opt(h, m, 0).expect("valid time")),
        )
        .single()
        .expect("non-ambiguous time")
    }

    fn mock_rules() -> PolicyRules {
        PolicyRules {
            grace_period_minutes: 0,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_auto_deduct: false,
            is_break_overtime: false,
            allow_early_ot: false,
        }
    }

    #[test]
    fn test_standard_shift_within_bounds() {
        let start = Some(dt(7, 30));
        let end = Some(dt(17, 0));

        let b1 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(7, 30),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b1.standard_minutes, 570); // 9.5 hours (no break deducted)

        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;
        let b2 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(7, 30),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b2.standard_minutes, 510); // 8.5 hours
    }

    #[test]
    fn test_clamping_and_grace() {
        let start = Some(dt(7, 30));
        let end = Some(dt(17, 0));
        let mut rules = mock_rules();
        rules.grace_period_minutes = 30;
        rules.allow_early_ot = true;

        // Case 1: Within grace -> No OT
        let b1 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(7, 15),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b1.early_ot_minutes, 0);

        // Case 2: Before grace -> Full OT
        let b2 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(6, 45),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b2.early_ot_minutes, 45);

        // Case 3: Clamp
        rules.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
        let b3 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(6, 45),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b3.early_ot_minutes, 30);
    }

    #[test]
    fn test_authorized_early_leave() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let mut rules = mock_rules();
        rules.authorized_early_leave_minutes = 60;
        rules.is_early_leave_paid = true;

        // Exit 30m early -> Paid as full day
        let b1 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(16, 30),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b1.standard_minutes, 540);

        // Exit 90m early -> Not paid
        let b2 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(15, 30),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b2.standard_minutes, 450);
    }

    #[test]
    fn test_rounding_and_units() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let rules = mock_rules();

        let (std, _) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(16, 55),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: rules.clone(),
            ot_ceiling: None,
        });
        // (17:00 - 08:00) = 540m.
        // Actual work = 08:00 - 16:55 = 535m.
        // Net = 535 - 0 (no auto deduct) = 535.
        // Floor(535, 30) = 17 units.
        assert_eq!(std, 17);

        let mut r2 = rules.clone();
        r2.is_break_auto_deduct = true;
        let (std2, _) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(16, 55),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: r2.clone(),
            ot_ceiling: None,
        });
        // Net = 535 - 60 = 475. Floor(475, 30) = 15 units.
        assert_eq!(std2, 15);
    }

    #[test]
    fn test_split_shift_ot_capped_at_next_shift() {
        // Shift 1: 08:00-12:00, OT ceiling = 13:00 (next shift starts)
        // Employee works until 13:30 → OT should be 60min (12:00-13:00), NOT 90min
        let start = Some(dt(8, 0));
        let end = Some(dt(12, 0));
        let ceiling = Some(dt(13, 0));

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(13, 30),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: ceiling,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b.standard_minutes, 240); // 08:00-12:00 = 4h
        assert_eq!(b.late_ot_minutes, 60); // 12:00-13:00 (capped, not 90)
    }

    #[test]
    fn test_split_shift_no_ceiling_ot_unlimited() {
        // Same scenario but no ceiling → OT = 90min
        let start = Some(dt(8, 0));
        let end = Some(dt(12, 0));

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(13, 30),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b.standard_minutes, 240);
        assert_eq!(b.late_ot_minutes, 90); // 12:00-13:30 = 90min (no cap)
    }

    #[test]
    fn test_split_shift_gap_fully_utilized_as_ot() {
        // Shift 1: 08:00-12:00, next shift at 13:00
        // Employee works until exactly 13:00 → full gap = OT
        let start = Some(dt(8, 0));
        let end = Some(dt(12, 0));
        let ceiling = Some(dt(13, 0));

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(13, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: ceiling,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b.standard_minutes, 240);
        assert_eq!(b.late_ot_minutes, 60); // 12:00-13:00 = full gap
    }

    // --- calculate_units rounding tests ---

    #[test]
    fn test_calculate_units_floor_rounding() {
        // 75 minutes / 30-minute unit = 2.5 → floor = 2
        assert_eq!(AttendanceCalculator::calculate_units(75, 30, "floor"), 2);
        // 90 minutes / 30 = 3.0 → exactly 3
        assert_eq!(AttendanceCalculator::calculate_units(90, 30, "floor"), 3);
        // 29 minutes / 30 = 0.966 → floor = 0
        assert_eq!(AttendanceCalculator::calculate_units(29, 30, "floor"), 0);
    }

    #[test]
    fn test_calculate_units_ceil_rounding() {
        // 75 / 30 = 2.5 → ceil = 3
        assert_eq!(AttendanceCalculator::calculate_units(75, 30, "ceil"), 3);
        // 90 / 30 = 3.0 → exactly 3
        assert_eq!(AttendanceCalculator::calculate_units(90, 30, "ceil"), 3);
        // 1 / 30 = 0.033 → ceil = 1
        assert_eq!(AttendanceCalculator::calculate_units(1, 30, "ceil"), 1);
    }

    #[test]
    fn test_calculate_units_round_rounding() {
        // 75 / 30 = 2.5 → round = 3
        assert_eq!(AttendanceCalculator::calculate_units(75, 30, "round"), 3);
        // 44 / 30 = 1.47 → round = 1
        assert_eq!(AttendanceCalculator::calculate_units(44, 30, "round"), 1);
        // 46 / 30 = 1.53 → round = 2
        assert_eq!(AttendanceCalculator::calculate_units(46, 30, "round"), 2);
    }

    #[test]
    fn test_calculate_units_zero_minutes() {
        assert_eq!(AttendanceCalculator::calculate_units(0, 30, "floor"), 0);
        assert_eq!(AttendanceCalculator::calculate_units(0, 30, "ceil"), 0);
        assert_eq!(AttendanceCalculator::calculate_units(0, 30, "round"), 0);
    }

    #[test]
    fn test_calculate_units_default_rounding() {
        // Unknown rounding policy defaults to floor
        assert_eq!(AttendanceCalculator::calculate_units(75, 30, "unknown"), 2);
    }

    // --- DailyCalcInput struct tests ---

    #[test]
    fn test_daily_calc_input_via_calculate_from_input() {
        let input = DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: mock_rules(),
            ot_ceiling: None,
        };
        let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(&input);
        // 9h = 540m, floor(540/30) = 18 units (no break auto-deduct in mock_rules)
        assert_eq!(std_units, 18);
        assert_eq!(ot_units, 0);
    }

    #[test]
    fn test_daily_calc_input_with_break_auto_deduct() {
        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;
        let input = DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules,
            ot_ceiling: None,
        };
        let (std_units, _) = AttendanceCalculator::calculate_daily_units(&input);
        // 540m - 60m break = 480m, floor(480/30) = 16 units
        assert_eq!(std_units, 16);
    }

    #[test]
    fn test_daily_calc_input_debug_trait() {
        let input = DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: None,
            shift_end: None,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: mock_rules(),
            ot_ceiling: None,
        };
        // Verify Debug trait works without panic
        let debug_str = format!("{:?}", input);
        assert!(debug_str.contains("DailyCalcInput"));
    }

    // =====================================================================
    // Edge case: punch_out <= punch_in returns zero buckets
    // =====================================================================

    #[test]
    fn test_punch_out_before_punch_in_returns_zero() {
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(17, 0),
            punch_out: dt(8, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b.standard_minutes, 0);
        assert_eq!(b.early_ot_minutes, 0);
        assert_eq!(b.late_ot_minutes, 0);
    }

    #[test]
    fn test_punch_out_equals_punch_in_returns_zero() {
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(8, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b.standard_minutes, 0);
        assert_eq!(b.early_ot_minutes, 0);
        assert_eq!(b.late_ot_minutes, 0);
    }

    // =====================================================================
    // No shift boundaries: everything is standard
    // =====================================================================

    #[test]
    fn test_no_shift_all_standard() {
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(6, 0),
            punch_out: dt(18, 0),
            shift_start: None,
            shift_end: None,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        // 12 hours = 720 minutes, all standard
        assert_eq!(b.standard_minutes, 720);
        assert_eq!(b.early_ot_minutes, 0);
        assert_eq!(b.late_ot_minutes, 0);
    }

    #[test]
    fn test_no_shift_short_session() {
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(9, 0),
            punch_out: dt(9, 15),
            shift_start: None,
            shift_end: None,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        assert_eq!(b.standard_minutes, 15);
    }

    /// Proves that the calculator produces non-zero standard hours even without
    /// shift boundaries.  `report_service::calculate_day_units()` guards against
    /// this by returning `(0, 0)` when `shift_id` is `None`.
    ///
    /// Without that guard, a roster day-off with stale time entries would still
    /// generate work-unit counts in the attendance report.
    #[test]
    fn test_daily_units_no_shift_returns_nonzero_proving_guard_needed() {
        let rules = mock_rules();

        // Simulate an employee who punched 08:00→17:00 but has NO shift
        let (std, ot) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
punch_in: dt(8, 0),
punch_out: dt(17, 0),
shift_start: None,
shift_end: None,
scheduled_break_minutes: // no shift boundaries
            0,
actual_break_minutes: 0,
work_unit: // no break
            30,
rounding_policy: "floor".to_string(),
rules: rules.clone(),
ot_ceiling: None
});

        // Without the report guard, the calculator happily returns 18 standard units
        // (540 min / 30 = 18) and 0 OT.  The report_service guard prevents this
        // from reaching final reports when the day is a roster day-off.
        assert_eq!(
            std, 18,
            "calculator gives 18 std units without shift — report guard needed"
        );
        assert_eq!(ot, 0, "no shift boundaries means no OT");
    }

    /// Same as above but with break auto-deduction enabled.
    /// Break deduction only applies when shift boundaries exist, so no-shift
    /// still produces the full raw duration as standard hours.
    #[test]
    fn test_daily_units_no_shift_with_auto_break() {
        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;

        let (std, ot) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: None,
            shift_end: None,
            scheduled_break_minutes: 60, // 60-min scheduled break, no actual break logged
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: rules.clone(),
            ot_ceiling: None,
        });

        // Break deduction is only applied inside the shift-boundary branch.
        // Without shift boundaries → raw 540 min → 18 units (no break deducted).
        assert_eq!(std, 18);
        assert_eq!(ot, 0);
    }

    /// Edge case: very short accidental session (face recognition walk-by).
    /// Without a shift, even 5 minutes of "work" generates 0 units (floor rounding).
    #[test]
    fn test_daily_units_no_shift_accidental_face_recognition() {
        let rules = mock_rules();

        // Walk past device at 07:55, walk again at 08:00 → 5-min session
        let (std, ot) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(7, 55),
            punch_out: dt(8, 0),
            shift_start: None,
            shift_end: None,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: rules.clone(), // 5 / 30 = 0.17 → floor = 0
            ot_ceiling: None,
        });

        assert_eq!(std, 0, "5-min accidental session rounds to 0 units");
        assert_eq!(ot, 0);
    }

    // =====================================================================
    // Overnight shift: end_time < start_time
    // =====================================================================

    #[test]
    fn test_overnight_shift_standard_hours() {
        // Night shift 22:00 → 06:00 next day
        // dt uses a single day, end < start triggers +1 day
        let start = Some(dt(22, 0));
        let end = Some(dt(6, 0)); // will be +1 day internally
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(22, 0), // punch in at 22:00
            punch_out: Utc
                .from_local_datetime(
                    &chrono::NaiveDate::from_ymd_opt(2026, 1, 2)
                        .unwrap()
                        .and_time(chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap()),
                )
                .single()
                .unwrap(),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            work_unit: 0,
            rounding_policy: "".to_string(),
            rules: mock_rules(),
            ot_ceiling: None,
        });
        // 22:00 → 06:00+1 = 480 min standard
        assert_eq!(b.standard_minutes, 480);
        assert_eq!(b.late_ot_minutes, 0);
    }

    // =====================================================================
    // Break deduction edge cases
    // =====================================================================

    #[test]
    fn test_break_deduction_actual_break_takes_precedence() {
        // If actual_break_minutes > 0, use that instead of scheduled
        let deduction = AttendanceCalculator::calculate_break_deduction(45, 60, true, false);
        assert_eq!(deduction, 45); // actual break (45) used, not scheduled (60)
    }

    #[test]
    fn test_break_deduction_auto_deduct_when_no_actual() {
        let deduction = AttendanceCalculator::calculate_break_deduction(0, 60, true, false);
        assert_eq!(deduction, 60); // auto-deducted scheduled break
    }

    #[test]
    fn test_break_deduction_skip_when_break_is_overtime() {
        // is_break_overtime = true → don't deduct even if auto_deduct is on
        let deduction = AttendanceCalculator::calculate_break_deduction(0, 60, true, true);
        assert_eq!(deduction, 0);
    }

    #[test]
    fn test_break_deduction_no_auto_deduct_no_actual() {
        let deduction = AttendanceCalculator::calculate_break_deduction(0, 60, false, false);
        assert_eq!(deduction, 0);
    }

    #[test]
    fn test_break_deduction_zero_scheduled() {
        let deduction = AttendanceCalculator::calculate_break_deduction(0, 0, true, false);
        assert_eq!(deduction, 0);
    }

    // =====================================================================
    // calculate_units edge cases
    // =====================================================================

    #[test]
    fn test_calculate_units_with_unit_size_1() {
        // Edge: unit_size <= 1 → treated as 1.0
        assert_eq!(AttendanceCalculator::calculate_units(480, 1, "floor"), 480);
        assert_eq!(AttendanceCalculator::calculate_units(480, 0, "floor"), 480);
    }

    #[test]
    fn test_calculate_units_15min_unit() {
        // 480 min / 15-min unit = 32 units
        assert_eq!(AttendanceCalculator::calculate_units(480, 15, "floor"), 32);
        // 481 / 15 = 32.07 → floor=32, ceil=33
        assert_eq!(AttendanceCalculator::calculate_units(481, 15, "floor"), 32);
        assert_eq!(AttendanceCalculator::calculate_units(481, 15, "ceil"), 33);
    }

    #[test]
    fn test_calculate_units_45min_unit() {
        // 480 / 45 = 10.67 → floor=10, ceil=11, round=11
        assert_eq!(AttendanceCalculator::calculate_units(480, 45, "floor"), 10);
        assert_eq!(AttendanceCalculator::calculate_units(480, 45, "ceil"), 11);
        assert_eq!(AttendanceCalculator::calculate_units(480, 45, "round"), 11);
    }

    // =====================================================================
    // Early OT with allow_early_ot disabled
    // =====================================================================

    #[test]
    fn test_early_arrival_no_ot_when_disabled() {
        let rules = mock_rules(); // allow_early_ot = false
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(6, 0),
            punch_out: dt(17, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        // Punch in 2h early, but allow_early_ot = false → no early OT
        assert_eq!(b.early_ot_minutes, 0);
        // Standard: intersection of [6:00, 17:00] ∩ [8:00, 17:00] = 540 min
        assert_eq!(b.standard_minutes, 540);
    }

    // =====================================================================
    // Late OT without ceiling
    // =====================================================================

    #[test]
    fn test_late_ot_uncapped() {
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(20, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        // Standard: 08→17 = 540, Late OT: 17→20 = 180
        assert_eq!(b.standard_minutes, 540);
        assert_eq!(b.late_ot_minutes, 180);
    }

    // =====================================================================
    // Partial shift: punch in late and leave early
    // =====================================================================

    #[test]
    fn test_partial_shift_late_in_early_out() {
        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(9, 30),
            punch_out: dt(15, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: mock_rules().clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        // Standard: intersection [9:30, 15:00] ∩ [8:00, 17:00] = 330 min
        assert_eq!(b.standard_minutes, 330);
        assert_eq!(b.early_ot_minutes, 0);
        assert_eq!(b.late_ot_minutes, 0);
    }

    // =====================================================================
    // Combined: early OT + late OT in same day
    // =====================================================================

    #[test]
    fn test_early_and_late_ot_combined() {
        let mut rules = mock_rules();
        rules.allow_early_ot = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(6, 0),
            punch_out: dt(20, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: "".to_string(),
        });
        // Early OT: 06:00→08:00 = 120 min
        assert_eq!(b.early_ot_minutes, 120);
        // Standard: 08:00→17:00 = 540 min
        assert_eq!(b.standard_minutes, 540);
        // Late OT: 17:00→20:00 = 180 min
        assert_eq!(b.late_ot_minutes, 180);
    }

    // =====================================================================
    // calculate_daily_units integration: std + OT units
    // =====================================================================

    #[test]
    fn test_daily_units_with_ot() {
        let mut rules = mock_rules();
        rules.allow_early_ot = true;

        let (std, ot) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(7, 0),
            punch_out: dt(19, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: rules.clone(),
            ot_ceiling: None,
        });
        // Standard: 540 min - 0 break = 540 min; floor(540/30) = 18 units
        assert_eq!(std, 18);
        // Early OT: 60 min (7→8), Late OT: 120 min (17→19) = 180; floor(180/30) = 6
        assert_eq!(ot, 6);
    }

    #[test]
    fn test_daily_units_with_auto_break_and_ot() {
        let mut rules = mock_rules();
        rules.allow_early_ot = true;
        rules.is_break_auto_deduct = true;

        let (std, ot) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(7, 0),
            punch_out: dt(19, 0),
            shift_start: Some(dt(8, 0)),
            shift_end: Some(dt(17, 0)),
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: rules.clone(),
            ot_ceiling: None,
        });
        // Standard: 540 - 60 break = 480 min; floor(480/30) = 16
        assert_eq!(std, 16);
        // OT: 60 + 120 = 180; floor(180/30) = 6
        assert_eq!(ot, 6);
    }
}
