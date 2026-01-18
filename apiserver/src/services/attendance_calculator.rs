use chrono::{DateTime, Datelike, Duration, NaiveDate, TimeZone, Weekday};

#[derive(Debug, Default)]
pub struct WorkBuckets {
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
}

pub struct AttendanceCalculator;

impl AttendanceCalculator {
    /// Calculate standard and OT buckets based on shift boundaries and policy.
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_buckets<T: TimeZone>(
        punch_in: DateTime<T>,
        punch_out: DateTime<T>,
        shift_start_dt: Option<DateTime<T>>,
        shift_end_dt: Option<DateTime<T>>,
        scheduled_break_minutes: i32,
        grace_period_minutes: i32,
        actual_break_minutes: i32,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
        allow_early_ot: bool,
    ) -> WorkBuckets {
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
                // If punch_in is within [start - grace, start], we treat it as 'start' for OT purposes.
                // If punch_in is before 'start - grace', we count full OT from punch_in.
                let mut effective_ot_start = start.clone();
                if grace_period_minutes > 0 {
                    let grace_threshold =
                        start.clone() - Duration::minutes(grace_period_minutes as i64);
                    if punch_in >= grace_threshold && punch_in < start {
                        effective_ot_start = start.clone(); // Within grace, no OT
                    } else if punch_in < grace_threshold {
                        effective_ot_start = punch_in.clone(); // Before grace, full OT
                    }
                } else {
                    effective_ot_start = punch_in.clone();
                }

                // 1. Standard Bucket: Intersection of [In, Out] and [Start, End]
                let standard_in = DateTime::max(punch_in.clone(), start.clone());
                let standard_out = DateTime::min(punch_out.clone(), end.clone());

                if standard_out > standard_in {
                    let mut standard_raw = standard_out
                        .signed_duration_since(standard_in)
                        .num_minutes() as i32;

                    // Deduct break from standard time
                    let break_to_deduct = Self::calculate_break_deduction(
                        actual_break_minutes,
                        scheduled_break_minutes,
                        is_break_auto_deduct,
                        is_break_overtime,
                    );
                    standard_raw = i32::max(0, standard_raw - break_to_deduct);
                    buckets.standard_minutes = standard_raw;
                }

                // 2. Early OT: Time before ShiftStart
                if allow_early_ot && punch_in < start {
                    let early_out = DateTime::min(punch_out.clone(), start.clone());
                    if early_out > effective_ot_start {
                        buckets.early_ot_minutes = early_out
                            .signed_duration_since(effective_ot_start)
                            .num_minutes()
                            as i32;
                    }
                }

                // 3. Late OT: Time after ShiftEnd
                if punch_out > end {
                    let late_in = DateTime::max(punch_in.clone(), end.clone());
                    if punch_out > late_in {
                        buckets.late_ot_minutes =
                            punch_out.signed_duration_since(late_in).num_minutes() as i32;
                    }
                }
            }
            _ => {
                // No shift: everything is standard (or OT based on global policy, but here we keep it standard)
                let raw_duration = punch_out.signed_duration_since(punch_in).num_minutes() as i32;
                buckets.standard_minutes = i32::max(0, raw_duration);
            }
        }

        buckets
    }

    /// 2. Break Deduction Logic
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

    /// 3. Unit Calculation Logic
    ///
    /// Converts minutes to the number of work units based on the policy.
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
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_daily_units<T: TimeZone>(
        punch_in: DateTime<T>,
        punch_out: DateTime<T>,
        shift_start_dt: Option<DateTime<T>>,
        shift_end_dt: Option<DateTime<T>>,
        scheduled_break_minutes: i32,
        grace_period_minutes: i32,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
        actual_break_minutes: i32,
        allow_early_ot: bool,
        work_unit: i32,
        rounding_policy: &str,
    ) -> (i32, i32) {
        let buckets = Self::calculate_buckets(
            punch_in,
            punch_out,
            shift_start_dt,
            shift_end_dt,
            scheduled_break_minutes,
            grace_period_minutes,
            actual_break_minutes,
            is_break_auto_deduct,
            is_break_overtime,
            allow_early_ot,
        );

        // Apply rounding to standard time (Result in Units)
        let standard_units =
            Self::calculate_units(buckets.standard_minutes, work_unit, rounding_policy);

        // Overtime units (Summed Early + Late)
        let total_ot_units = Self::calculate_units(
            buckets.early_ot_minutes + buckets.late_ot_minutes,
            work_unit,
            rounding_policy,
        );

        (standard_units, total_ot_units)
    }

    /// Calculate the number of standard working days in a month based on policy.
    pub fn count_working_days(year: i32, month: u32, days_per_week: i32) -> f64 {
        let start_date = match NaiveDate::from_ymd_opt(year, month, 1) {
            Some(d) => d,
            None => return 26.0,
        };

        let next_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
        };

        let mut working_days = 0;
        let mut current_date = start_date;
        while current_date < next_month {
            let weekday = current_date.weekday();
            let is_working = match days_per_week {
                7 => true,
                6 => weekday != Weekday::Sun,
                5 => weekday != Weekday::Sat && weekday != Weekday::Sun,
                _ => true,
            };
            if is_working {
                working_days += 1;
            }
            if let Some(next) = current_date.succ_opt() {
                current_date = next;
            } else {
                break;
            }
        }
        working_days as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, Utc};

    fn dt(h: u32, m: u32) -> DateTime<Utc> {
        Utc.from_local_datetime(
            &NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap()),
        )
        .single()
        .unwrap()
    }

    #[test]
    fn test_clamping_and_buckets() {
        let start = Some(dt(7, 30));
        let end = Some(dt(17, 0));

        // Case 1: Early In (07:25), Out (17:00), No Early OT
        let b1 = AttendanceCalculator::calculate_buckets(
            dt(7, 25),
            dt(17, 0),
            start,
            end,
            60,
            0,
            0,
            true,
            false,
            false,
        );
        assert_eq!(b1.standard_minutes, 510); // (17:00 - 07:30) - 60m = 8.5h = 510m
        assert_eq!(b1.early_ot_minutes, 0);

        // Case 2: Early In (07:25), Allow Early OT
        let b2 = AttendanceCalculator::calculate_buckets(
            dt(7, 25),
            dt(17, 0),
            start,
            end,
            60,
            0,
            0,
            true,
            false,
            true,
        );
        assert_eq!(b2.standard_minutes, 510);
        assert_eq!(b2.early_ot_minutes, 5); // 07:25 to 07:30

        // Case 3: Late In (07:40), Unit 30, Policy Floor
        let b3 = AttendanceCalculator::calculate_buckets(
            dt(7, 40),
            dt(17, 0),
            start,
            end,
            60,
            0,
            0,
            true,
            false,
            false,
        );
        // (17:00 - 07:40) = 9h20m = 560m.
        // 560 - 60 (break) = 500m.
        assert_eq!(b3.standard_minutes, 500);

        let rounded = AttendanceCalculator::calculate_units(b3.standard_minutes, 30, "floor");
        assert_eq!(rounded, 16); // 16 units (8 hours)

        // Case 4: Grace Period (30m). Start 07:30 -> Grace starts 07:00.
        // Punch in 07:15 (within grace) -> No OT
        let b4 = AttendanceCalculator::calculate_buckets(
            dt(7, 15),
            dt(17, 0),
            start,
            end,
            60,
            30,
            0,
            true,
            false,
            true,
        );
        assert_eq!(b4.early_ot_minutes, 0);

        // Case 5: Before Grace Period. Punch in 06:45 -> OT from 06:45 (45m)
        let b5 = AttendanceCalculator::calculate_buckets(
            dt(6, 45),
            dt(17, 0),
            start,
            end,
            60,
            30,
            0,
            true,
            false,
            true,
        );
        assert_eq!(b5.early_ot_minutes, 45);
    }

    #[test]
    fn test_overnight_shift_edge_case() {
        // Shift: 22:00 (Jan 1) to 06:00 (Jan 2) -> 8 hours
        let start = Some(dt(22, 0));
        let end = Some(dt(6, 0)); // Note: dt helper uses Jan 1. End is 06:00 Jan 1 in helper.
                                  // The calculator logic linearizes this by adding 1 day.

        // Scenario: Punch In 21:50, Punch Out 06:10.
        // Standard: [22:00, 06:00] -> 8 hours (480m)
        // Early OT: 21:50 to 22:00 -> 10m
        // Late OT: 06:00 to 06:10 -> 10m

        // We need a helper for Jan 2 to test accurately
        let punch_in = dt(21, 50);
        let punch_out = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2026, 1, 2)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(6, 10, 0).unwrap()),
            )
            .single()
            .unwrap();

        let b = AttendanceCalculator::calculate_buckets(
            punch_in, punch_out, start, end, 0, 0, 0, false, false, true,
        );

        assert_eq!(b.standard_minutes, 480);
        assert_eq!(b.early_ot_minutes, 10);
        assert_eq!(b.late_ot_minutes, 10);
    }

    #[test]
    fn test_early_departure_rounding() {
        // Shift 08:00 - 17:00. Unit 30, Policy Floor.
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));

        // Punch In 08:00, Punch Out 16:55 (Left 5 mins early)
        // Standard: 08:00 to 16:55 = 8h55m = 535m
        // Net: 535 - 60 = 475m
        // Floor(475, 30) = 15 units (7.5h) -> Lost 1 unit for leaving 5 mins early.
        let (std, _) = AttendanceCalculator::calculate_daily_units(
            dt(8, 0),
            dt(16, 55),
            start,
            end,
            60,
            0,
            true,
            false,
            0,
            false,
            30,
            "floor",
        );
        assert_eq!(std, 15);
    }

    #[test]
    fn test_punch_entirely_outside_shift() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));

        // Case 1: Entirely before shift (06:00 to 07:00)
        let b1 = AttendanceCalculator::calculate_buckets(
            dt(6, 0),
            dt(7, 0),
            start,
            end,
            0,
            0,
            0,
            false,
            false,
            true,
        );
        assert_eq!(b1.standard_minutes, 0);
        assert_eq!(b1.early_ot_minutes, 60);
        assert_eq!(b1.late_ot_minutes, 0);

        // Case 2: Entirely after shift (18:00 to 19:00)
        let b2 = AttendanceCalculator::calculate_buckets(
            dt(18, 0),
            dt(19, 0),
            start,
            end,
            0,
            0,
            0,
            false,
            false,
            true,
        );
        assert_eq!(b2.standard_minutes, 0);
        assert_eq!(b2.early_ot_minutes, 0);
        assert_eq!(b2.late_ot_minutes, 60);
    }

    #[test]
    fn test_break_auto_deduct_safety() {
        // Shift 08:00 - 17:00. Break 60m.
        // Employee only worked 30 mins (08:00 to 08:30).
        // Standard duration: 30m.
        // Break to deduct: 60m.
        // Net: max(0, 30 - 60) = 0.
        let (std, _) = AttendanceCalculator::calculate_daily_units(
            dt(8, 0),
            dt(8, 30),
            Some(dt(8, 0)),
            Some(dt(17, 0)),
            60,
            0,
            true,
            false,
            0,
            false,
            1,
            "none",
        );
        assert_eq!(std, 0);
    }

    #[test]
    fn test_unit_calculation_policies() {
        // Unit 30
        assert_eq!(AttendanceCalculator::calculate_units(29, 30, "floor"), 0);
        assert_eq!(AttendanceCalculator::calculate_units(31, 30, "floor"), 1);

        assert_eq!(AttendanceCalculator::calculate_units(1, 30, "ceil"), 1);
        assert_eq!(AttendanceCalculator::calculate_units(30, 30, "ceil"), 1);
        assert_eq!(AttendanceCalculator::calculate_units(31, 30, "ceil"), 2);

        assert_eq!(AttendanceCalculator::calculate_units(14, 30, "round"), 0);
        assert_eq!(AttendanceCalculator::calculate_units(15, 30, "round"), 1); // Boundary
        assert_eq!(AttendanceCalculator::calculate_units(44, 30, "round"), 1);
        assert_eq!(AttendanceCalculator::calculate_units(45, 30, "round"), 2);

        // Policy "none" (truncates)
        assert_eq!(AttendanceCalculator::calculate_units(17, 30, "none"), 0);
        assert_eq!(AttendanceCalculator::calculate_units(45, 30, "none"), 1);
        // Unit <= 1
        assert_eq!(AttendanceCalculator::calculate_units(17, 1, "floor"), 17);
    }

    #[test]
    fn test_break_priorities() {
        // Case 1: Actual break > 0, overrides everything
        let b1 = AttendanceCalculator::calculate_break_deduction(45, 60, true, false);
        assert_eq!(b1, 45);

        // Case 2: Actual break = 0, auto-deduct is true
        let b2 = AttendanceCalculator::calculate_break_deduction(0, 60, true, false);
        assert_eq!(b2, 60);

        // Case 3: Actual break = 0, auto-deduct is true BUT is overtime break
        let b3 = AttendanceCalculator::calculate_break_deduction(0, 60, true, true);
        assert_eq!(b3, 0);

        // Case 4: Actual break = 0, auto-deduct is false
        let b4 = AttendanceCalculator::calculate_break_deduction(0, 60, false, false);
        assert_eq!(b4, 0);
    }

    #[test]
    fn test_boundary_punches() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));

        // Punch In exactly at Shift End (No standard time)
        let b1 = AttendanceCalculator::calculate_buckets(
            dt(17, 0),
            dt(18, 0),
            start,
            end,
            0,
            0,
            0,
            false,
            false,
            true,
        );
        assert_eq!(b1.standard_minutes, 0);
        assert_eq!(b1.late_ot_minutes, 60);

        // Punch Out exactly at Shift Start (No standard time)
        let b2 = AttendanceCalculator::calculate_buckets(
            dt(7, 0),
            dt(8, 0),
            start,
            end,
            0,
            0,
            0,
            false,
            false,
            true,
        );
        assert_eq!(b2.standard_minutes, 0);
        assert_eq!(b2.early_ot_minutes, 60);
    }

    #[test]
    fn test_zero_duration_work() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));

        // Punch In == Punch Out
        let b = AttendanceCalculator::calculate_buckets(
            dt(9, 0),
            dt(9, 0),
            start,
            end,
            0,
            0,
            0,
            false,
            false,
            true,
        );
        assert_eq!(b.standard_minutes, 0);
        assert_eq!(b.early_ot_minutes, 0);
        assert_eq!(b.late_ot_minutes, 0);
    }

    #[test]
    fn test_no_shift_logic() {
        // Without shift, everything is standard duration
        let b = AttendanceCalculator::calculate_buckets(
            dt(9, 0),
            dt(11, 0),
            None,
            None,
            0,
            0,
            0,
            false,
            false,
            true,
        );
        assert_eq!(b.standard_minutes, 120);
        assert_eq!(b.early_ot_minutes, 0);
        assert_eq!(b.late_ot_minutes, 0);
    }

    #[test]
    fn test_overnight_shift_late_start_late_end() {
        // Shift: 22:00 (Jan 1) to 06:00 (Jan 2)
        let start = Some(dt(22, 0));
        let end = Some(dt(6, 0));

        // Employee arrives late on Jan 2: 01:00 to 07:00
        let punch_in = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2026, 1, 2)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(1, 0, 0).unwrap()),
            )
            .single()
            .unwrap();

        let punch_out = Utc
            .from_local_datetime(
                &NaiveDate::from_ymd_opt(2026, 1, 2)
                    .unwrap()
                    .and_time(NaiveTime::from_hms_opt(7, 0, 0).unwrap()),
            )
            .single()
            .unwrap();

        let b = AttendanceCalculator::calculate_buckets(
            punch_in, punch_out, start, end, 0, 0, 0, false, false, true,
        );

        // Standard: intersection of [01:00, 07:00] and [22:00 Jan 1, 06:00 Jan 2]
        // Standard In = max(01:00 Jan 2, 22:00 Jan 1) = 01:00 Jan 2
        // Standard Out = min(07:00 Jan 2, 06:00 Jan 2) = 06:00 Jan 2
        // Standard: 01:00 to 06:00 = 5 hours (300m)
        assert_eq!(b.standard_minutes, 300);

        // Late OT: max(0, 07:00 - 06:00) = 60m
        assert_eq!(b.late_ot_minutes, 60);
        assert_eq!(b.early_ot_minutes, 0);
    }

    #[test]
    fn test_rounding_late_entry() {
        // Shift 08:00 - 17:00 (9h raw, 8h net if 60m break)
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));

        // Arrive 08:01 (Late 1 min)
        // Duration: 08:01 to 17:00 = 8h59m = 539m
        // Net: 539 - 60 = 479m
        // Floor(479, 30) = 15 units (7.5h) -> Lost 1 unit for being 1 min late.
        let (std, _) = AttendanceCalculator::calculate_daily_units(
            dt(8, 1),
            dt(17, 0),
            start,
            end,
            60,
            0,
            true,
            false,
            0,
            false,
            30,
            "floor",
        );
        assert_eq!(std, 15);
    }
}
