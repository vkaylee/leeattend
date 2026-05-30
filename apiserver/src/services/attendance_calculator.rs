use crate::models::{EarlyArrivalStrategy, PolicyRules};
use chrono::{DateTime, Duration, TimeZone};

#[derive(Debug, Default)]
pub struct WorkBuckets {
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
    pub break_ot_minutes: i32,
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
    /// Break frame start (Fixed mode only). When both break_start and break_end
    /// are present, the calculator uses Frame Intersection (overlap algorithm)
    /// instead of flat deduction for auto-deduct mode.
    pub break_start: Option<DateTime<T>>,
    /// Break frame end (Fixed mode only).
    pub break_end: Option<DateTime<T>>,
    pub work_unit: i32,
    pub rounding_policy: String,
    pub rules: PolicyRules,
    pub ot_ceiling: Option<DateTime<T>>,
    pub max_early_ot_minutes: Option<i32>,
}

pub struct AttendanceCalculator;

impl AttendanceCalculator {
    /// Calculate standard and OT buckets using structured input.
    #[allow(clippy::too_many_lines)]
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

        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
            // Handle Overnight Shift linearization
            if end < start {
                end += Duration::days(1);
            }

            // Apply Grace Period to Shift Start for Early OT calculation
            let mut effective_ot_start = punch_in.clone();
            if rules.grace_period_minutes > 0 {
                let grace_threshold =
                    start.clone() - Duration::minutes(i64::from(rules.grace_period_minutes));

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
                    && missing_minutes <= i64::from(rules.authorized_early_leave_minutes)
                {
                    standard_out = end.clone();
                }
            }

            if standard_out > standard_in {
                let mut standard_raw = standard_out
                    .clone()
                    .signed_duration_since(standard_in.clone())
                    .num_minutes() as i32;

                // Calculate break deduction and break OT
                let (break_to_deduct, break_ot_minutes) = if rules.is_break_overtime {
                    if actual_break_minutes > 0 {
                        // Employee punched out for break -> no OT, but deduct break from standard
                        (actual_break_minutes, 0)
                    } else {
                        // Employee worked during break -> deduct from standard, add to OT
                        let overlap = if let (Some(bs), Some(be)) = (input.break_start.clone(), input.break_end.clone()) {
                            let overlap_start = DateTime::max(standard_in, bs);
                            let overlap_end = DateTime::min(standard_out, be);
                            if overlap_end > overlap_start {
                                overlap_end.signed_duration_since(overlap_start).num_minutes() as i32
                            } else {
                                0
                            }
                        } else {
                            scheduled_break_minutes
                        };
                        (overlap, overlap)
                    }
                } else {
                    let deduct = Self::calculate_break_deduction(
                        actual_break_minutes,
                        scheduled_break_minutes,
                        rules.is_break_auto_deduct,
                        rules.is_break_overtime,
                        input.break_start.as_ref(),
                        input.break_end.as_ref(),
                        punch_in,
                        punch_out,
                    );
                    (deduct, 0)
                };

                standard_raw = i32::max(0, standard_raw - break_to_deduct);
                buckets.standard_minutes = standard_raw;
                buckets.break_ot_minutes = break_ot_minutes;
            }

            // 2. Early OT: Time before ShiftStart
            if rules.allow_early_ot && punch_in < &start {
                let early_out = DateTime::min(punch_out.clone(), start);
                if early_out > effective_ot_start {
                    let mut capped_early_ot = early_out
                        .signed_duration_since(effective_ot_start)
                        .num_minutes() as i32;
                    if let Some(max_ot) = input.max_early_ot_minutes {
                        capped_early_ot = i32::min(capped_early_ot, max_ot);
                    }
                    buckets.early_ot_minutes = capped_early_ot;
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
        } else {
            // No shift: everything is standard
            let raw_duration = punch_out
                .clone()
                .signed_duration_since(punch_in.clone())
                .num_minutes() as i32;
            buckets.standard_minutes = i32::max(0, raw_duration);
        }

        buckets
    }

    /// Break Deduction Logic
    ///
    /// Supports two modes:
    /// - **Flat deduction** (Flexible): subtracts `scheduled_break_minutes` wholesale.
    /// - **Frame Intersection** (Fixed): when `break_start`/`break_end` are provided,
    ///   computes the actual overlap between `[punch_in, punch_out]` and
    ///   `[break_start, break_end]`. This prevents over-deduction when an employee's
    ///   working period only partially overlaps with the configured break window.
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_break_deduction<T: TimeZone>(
        actual_break_minutes: i32,
        scheduled_break_minutes: i32,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
        break_start: Option<&DateTime<T>>,
        break_end: Option<&DateTime<T>>,
        punch_in: &DateTime<T>,
        punch_out: &DateTime<T>,
    ) -> i32 {
        // 1. If there's actual break time recorded (4-punch mode), use it directly
        if actual_break_minutes > 0 {
            return actual_break_minutes;
        }

        // 2. No auto-deduct or break is OT → no deduction
        if !is_break_auto_deduct || is_break_overtime {
            return 0;
        }

        // 3. Frame Intersection: when break_start/break_end are provided (Fixed mode),
        //    calculate overlap between [punch_in, punch_out] ∩ [break_start, break_end]
        if let (Some(bs), Some(be)) = (break_start, break_end) {
            let overlap_start = DateTime::max(punch_in.clone(), bs.clone());
            let overlap_end = DateTime::min(punch_out.clone(), be.clone());
            let overlap_mins = overlap_end
                .signed_duration_since(overlap_start)
                .num_minutes();
            return i32::max(0, overlap_mins as i32);
        }

        // 4. Fallback: flat deduction (Flexible mode — no break frame configured)
        scheduled_break_minutes
    }

    /// Unit Calculation Logic
    pub fn calculate_units(minutes: i32, unit_size: i32, policy: &str) -> i32 {
        let size = if unit_size <= 1 {
            1.0
        } else {
            f64::from(unit_size)
        };
        let mins = f64::from(minutes);

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
            buckets.early_ot_minutes + buckets.late_ot_minutes + buckets.break_ot_minutes,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: rules.clone(),
            ot_ceiling: None,
            max_early_ot_minutes: None,
        });
        // (17:00 - 08:00) = 540m.
        // Actual work = 08:00 - 16:55 = 535m.
        // Net = 535 - 0 (no auto deduct) = 535.
        // Floor(535, 30) = 17 units.
        assert_eq!(std, 17);

        let mut r2 = rules;
        r2.is_break_auto_deduct = true;
        let (std2, _) = AttendanceCalculator::calculate_daily_units(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(16, 55),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: r2,
            ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: ceiling,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: ceiling,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: mock_rules(),
            ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules,
            ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules: mock_rules(),
            ot_ceiling: None,
            max_early_ot_minutes: None,
        };
        // Verify Debug trait works without panic
        let debug_str = format!("{input:?}");
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
break_start: None,
break_end: None,
work_unit: // no break
            30,
rounding_policy: "floor".to_string(),
rules,
ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules,
            ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules, // 5 / 30 = 0.17 → floor = 0
            ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 0,
            rounding_policy: String::new(),
            rules: mock_rules(),
            ot_ceiling: None,
            max_early_ot_minutes: None,
        });
        // 22:00 → 06:00+1 = 480 min standard
        assert_eq!(b.standard_minutes, 480);
        assert_eq!(b.late_ot_minutes, 0);
    }

    // =====================================================================
    // Break deduction edge cases
    // =====================================================================

    // Helper for break deduction tests — provides punch_in/punch_out that span
    // the full break window so they don't affect Flexible mode deduction.
    fn punch_range() -> (DateTime<Utc>, DateTime<Utc>) {
        (dt(7, 0), dt(18, 0))
    }

    #[test]
    fn test_break_deduction_actual_break_takes_precedence() {
        // If actual_break_minutes > 0, use that instead of scheduled
        let (pi, po) = punch_range();
        let deduction = AttendanceCalculator::calculate_break_deduction(
            45, 60, true, false, None, None, &pi, &po,
        );
        assert_eq!(deduction, 45); // actual break (45) used, not scheduled (60)
    }

    #[test]
    fn test_break_deduction_auto_deduct_when_no_actual() {
        let (pi, po) = punch_range();
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0, 60, true, false, None, None, &pi, &po,
        );
        assert_eq!(deduction, 60); // auto-deducted scheduled break (Flexible mode)
    }

    #[test]
    fn test_break_deduction_skip_when_break_is_overtime() {
        // is_break_overtime = true → don't deduct even if auto_deduct is on
        let (pi, po) = punch_range();
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0, 60, true, true, None, None, &pi, &po,
        );
        assert_eq!(deduction, 0);
    }

    #[test]
    fn test_break_deduction_no_auto_deduct_no_actual() {
        let (pi, po) = punch_range();
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0, 60, false, false, None, None, &pi, &po,
        );
        assert_eq!(deduction, 0);
    }

    #[test]
    fn test_break_deduction_zero_scheduled() {
        let (pi, po) = punch_range();
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0, 0, true, false, None, None, &pi, &po,
        );
        assert_eq!(deduction, 0);
    }

    // =====================================================================
    // Frame Intersection tests (Fixed break mode)
    // =====================================================================

    #[test]
    fn test_frame_intersection_full_overlap() {
        // Employee works full shift 08:00→17:00, break 11:30→12:30
        // Full overlap → deduct 60 minutes
        let pi = dt(8, 0);
        let po = dt(17, 0);
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0,
            60,
            true,
            false,
            bs.as_ref(),
            be.as_ref(),
            &pi,
            &po,
        );
        assert_eq!(deduction, 60);
    }

    #[test]
    fn test_frame_intersection_partial_overlap_leave_during_break() {
        // Employee leaves at 12:00 (mid-break), break 11:30→12:30
        // Overlap = min(12:00, 12:30) - max(08:00, 11:30) = 30 min
        let pi = dt(8, 0);
        let po = dt(12, 0);
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0,
            60,
            true,
            false,
            bs.as_ref(),
            be.as_ref(),
            &pi,
            &po,
        );
        assert_eq!(deduction, 30);
    }

    #[test]
    fn test_frame_intersection_no_overlap_leave_before_break() {
        // Employee leaves at 11:00 (before break starts), break 11:30→12:30
        // Overlap = 0
        let pi = dt(8, 0);
        let po = dt(11, 0);
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0,
            60,
            true,
            false,
            bs.as_ref(),
            be.as_ref(),
            &pi,
            &po,
        );
        assert_eq!(deduction, 0);
    }

    #[test]
    fn test_frame_intersection_arrive_during_break() {
        // Employee arrives at 12:00 (mid-break), break 11:30→12:30
        // Overlap = min(17:00, 12:30) - max(12:00, 11:30) = 30 min
        let pi = dt(12, 0);
        let po = dt(17, 0);
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0,
            60,
            true,
            false,
            bs.as_ref(),
            be.as_ref(),
            &pi,
            &po,
        );
        assert_eq!(deduction, 30);
    }

    #[test]
    fn test_frame_intersection_break_ot_no_deduction() {
        // Fixed break + Break OT → deduction = 0 (employee gets paid for break time)
        let pi = dt(8, 0);
        let po = dt(17, 0);
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let deduction = AttendanceCalculator::calculate_break_deduction(
            0,
            60,
            true,
            true,
            bs.as_ref(),
            be.as_ref(),
            &pi,
            &po,
        );
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
            break_start: None,
            break_end: None,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules: mock_rules(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules,
            ot_ceiling: None,
            max_early_ot_minutes: None,
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
            break_start: None,
            break_end: None,
            work_unit: 30,
            rounding_policy: "floor".to_string(),
            rules,
            ot_ceiling: None,
            max_early_ot_minutes: None,
        });
        // Standard: 540 - 60 break = 480 min; floor(480/30) = 16
        assert_eq!(std, 16);
        // OT: 60 + 120 = 180; floor(180/30) = 6
        assert_eq!(ot, 6);
    }

    #[test]
    fn test_calculate_buckets_fixed_break_frame_intersection_partial_overlap() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(12, 0), // leaves mid-break
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            break_start: bs,
            break_end: be,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });
        // Worked time: 240 min. Break overlap: 30 min. Standard work: 240 - 30 = 210 min.
        assert_eq!(b.standard_minutes, 210);
    }

    #[test]
    fn test_calculate_buckets_fixed_break_frame_intersection_no_overlap() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(11, 0), // leaves before break
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            break_start: bs,
            break_end: be,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });
        // Worked time: 180 min. Break overlap: 0 min. Standard work: 180 - 0 = 180 min.
        assert_eq!(b.standard_minutes, 180);
    }

    #[test]
    fn test_calculate_buckets_fixed_break_frame_intersection_full_overlap() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let bs = Some(dt(11, 30));
        let be = Some(dt(12, 30));
        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0), // works full shift
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            break_start: bs,
            break_end: be,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });
        // Worked time: 540 min. Break overlap: 60 min. Standard work: 540 - 60 = 480 min.
        assert_eq!(b.standard_minutes, 480);
    }

    #[test]
    fn test_early_ot_capping() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let mut rules = mock_rules();
        rules.allow_early_ot = true;

        // Case 1: Early OT is capped (punch in 180 min early, max early OT is 120 min)
        let b1 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(5, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(120),
        });
        assert_eq!(b1.early_ot_minutes, 120);

        // Case 2: Early OT is uncapped (max early OT is None)
        let b2 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(5, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });
        assert_eq!(b2.early_ot_minutes, 180);

        // Case 3: Early OT is under the cap (punch in 180 min early, max early OT is 240 min)
        let b3 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(5, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(240),
        });
        assert_eq!(b3.early_ot_minutes, 180);

        // Case 4: Early OT limit is 0 (should yield 0 OT)
        let b4 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(5, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(0),
        });
        assert_eq!(b4.early_ot_minutes, 0);

        // Case 5: Capping with Clamp Grace Period
        // Punch in: 7:15 (45 min early)
        // Shift start: 8:00
        // Grace period: 30 min (threshold 7:30)
        // Strategy: Clamp (clamps effective OT start to 7:30, i.e. max 30 min of early OT)
        // If max early OT is Some(15), early OT should be capped at 15 min.
        let mut clamp_rules = mock_rules();
        clamp_rules.allow_early_ot = true;
        clamp_rules.grace_period_minutes = 30;
        clamp_rules.early_arrival_strategy = EarlyArrivalStrategy::Clamp;

        let b5 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(7, 15),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: clamp_rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(15),
        });
        assert_eq!(b5.early_ot_minutes, 15);

        // If max early OT is Some(60), early OT should be 30 min (limited by clamp, not cap)
        let b6 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(7, 15),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: clamp_rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(60),
        });
        assert_eq!(b6.early_ot_minutes, 30);

        // Case 6: Capping with Block Grace Period
        // Punch in: 7:45 (15 min early, inside 30 min grace period threshold 7:30)
        // Shift start: 8:00
        // Strategy: Block (effective OT start becomes 8:00, i.e. 0 min early OT)
        // If max early OT is Some(15), early OT should still be 0.
        let mut block_rules = mock_rules();
        block_rules.allow_early_ot = true;
        block_rules.grace_period_minutes = 30;
        block_rules.early_arrival_strategy = EarlyArrivalStrategy::Block;

        let b7 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(7, 45),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: block_rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(15),
        });
        assert_eq!(b7.early_ot_minutes, 0);

        // Case 7: Overnight Shift early OT capping
        // Shift starts at 22:00 (starts on 2026-01-01), ends at 06:00 (on 2026-01-02)
        // Punch in: 20:00 (120 min early)
        // Max early OT: Some(45)
        // Expected early OT: 45 min
        let b8 = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(20, 0),
            punch_out: dt(6, 0) + chrono::Duration::days(1),
            shift_start: Some(dt(22, 0)),
            shift_end: Some(dt(6, 0) + chrono::Duration::days(1)),
            scheduled_break_minutes: 0,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules: rules.clone(),
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: Some(45),
        });
        assert_eq!(b8.early_ot_minutes, 45);
    }

    #[test]
    fn test_break_is_overtime_fixed_worked() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let bs = Some(dt(12, 0));
        let be = Some(dt(13, 0));
        let mut rules = mock_rules();
        rules.is_break_overtime = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            break_start: bs,
            break_end: be,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });

        // 9 hours total (540 mins). Break is 1 hour (60 mins).
        // Break is OT -> deducted from standard work (540 - 60 = 480 mins)
        // and added to break_ot_minutes = 60 mins.
        assert_eq!(b.standard_minutes, 480);
        assert_eq!(b.break_ot_minutes, 60);
    }

    #[test]
    fn test_break_is_overtime_fixed_taken() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let bs = Some(dt(12, 0));
        let be = Some(dt(13, 0));
        let mut rules = mock_rules();
        rules.is_break_overtime = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 60, // punched out for break
            break_start: bs,
            break_end: be,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });

        // Employee punched out for break -> standard deducted by 60 mins, but no break OT
        assert_eq!(b.standard_minutes, 480);
        assert_eq!(b.break_ot_minutes, 0);
    }

    #[test]
    fn test_break_is_overtime_flexible_worked() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let mut rules = mock_rules();
        rules.is_break_overtime = true;

        let b = AttendanceCalculator::calculate_buckets(&DailyCalcInput {
            punch_in: dt(8, 0),
            punch_out: dt(17, 0),
            shift_start: start,
            shift_end: end,
            scheduled_break_minutes: 60,
            actual_break_minutes: 0,
            break_start: None,
            break_end: None,
            rules,
            ot_ceiling: None,
            work_unit: 0,
            rounding_policy: String::new(),
            max_early_ot_minutes: None,
        });

        assert_eq!(b.standard_minutes, 480);
        assert_eq!(b.break_ot_minutes, 60);
    }
}
