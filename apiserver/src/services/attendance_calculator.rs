use crate::models::{EarlyArrivalStrategy, PolicyRules};
use chrono::{DateTime, Duration, TimeZone};

#[derive(Debug, Default)]
pub struct WorkBuckets {
    pub standard_minutes: i32,
    pub early_ot_minutes: i32,
    pub late_ot_minutes: i32,
}

pub struct AttendanceCalculator;

impl AttendanceCalculator {
    /// Calculate standard and OT buckets based on shift boundaries and policy.
    pub fn calculate_buckets<T: TimeZone>(
        punch_in: DateTime<T>,
        punch_out: DateTime<T>,
        shift_start_dt: Option<DateTime<T>>,
        shift_end_dt: Option<DateTime<T>>,
        scheduled_break_minutes: i32,
        actual_break_minutes: i32,
        rules: &PolicyRules,
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
                let mut effective_ot_start = punch_in.clone();
                if rules.grace_period_minutes > 0 {
                    let grace_threshold =
                        start.clone() - Duration::minutes(rules.grace_period_minutes as i64);

                    match rules.early_arrival_strategy {
                        EarlyArrivalStrategy::Block | EarlyArrivalStrategy::Allow => {
                            if punch_in >= grace_threshold && punch_in < start {
                                effective_ot_start = start.clone();
                            } else if punch_in < grace_threshold {
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
                if rules.allow_early_ot && punch_in < start {
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
                // No shift: everything is standard
                let raw_duration = punch_out.signed_duration_since(punch_in).num_minutes() as i32;
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
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_daily_units<T: TimeZone>(
        punch_in: DateTime<T>,
        punch_out: DateTime<T>,
        shift_start_dt: Option<DateTime<T>>,
        shift_end_dt: Option<DateTime<T>>,
        scheduled_break_minutes: i32,
        actual_break_minutes: i32,
        work_unit: i32,
        rounding_policy: &str,
        rules: &PolicyRules,
    ) -> (i32, i32) {
        let buckets = Self::calculate_buckets(
            punch_in,
            punch_out,
            shift_start_dt,
            shift_end_dt,
            scheduled_break_minutes,
            actual_break_minutes,
            rules,
        );

        let std_units = Self::calculate_units(buckets.standard_minutes, work_unit, rounding_policy);
        let ot_units = Self::calculate_units(
            buckets.early_ot_minutes + buckets.late_ot_minutes,
            work_unit,
            rounding_policy,
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

        let b1 = AttendanceCalculator::calculate_buckets(
            dt(7, 30),
            dt(17, 0),
            start,
            end,
            60,
            0,
            &mock_rules(),
        );
        assert_eq!(b1.standard_minutes, 570); // 9.5 hours (no break deducted)

        let mut rules = mock_rules();
        rules.is_break_auto_deduct = true;
        let b2 = AttendanceCalculator::calculate_buckets(
            dt(7, 30),
            dt(17, 0),
            start,
            end,
            60,
            0,
            &rules,
        );
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
        let b1 =
            AttendanceCalculator::calculate_buckets(dt(7, 15), dt(17, 0), start, end, 0, 0, &rules);
        assert_eq!(b1.early_ot_minutes, 0);

        // Case 2: Before grace -> Full OT
        let b2 =
            AttendanceCalculator::calculate_buckets(dt(6, 45), dt(17, 0), start, end, 0, 0, &rules);
        assert_eq!(b2.early_ot_minutes, 45);

        // Case 3: Clamp
        rules.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
        let b3 =
            AttendanceCalculator::calculate_buckets(dt(6, 45), dt(17, 0), start, end, 0, 0, &rules);
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
        let b1 =
            AttendanceCalculator::calculate_buckets(dt(8, 0), dt(16, 30), start, end, 0, 0, &rules);
        assert_eq!(b1.standard_minutes, 540);

        // Exit 90m early -> Not paid
        let b2 =
            AttendanceCalculator::calculate_buckets(dt(8, 0), dt(15, 30), start, end, 0, 0, &rules);
        assert_eq!(b2.standard_minutes, 450);
    }

    #[test]
    fn test_rounding_and_units() {
        let start = Some(dt(8, 0));
        let end = Some(dt(17, 0));
        let rules = mock_rules();

        let (std, _) = AttendanceCalculator::calculate_daily_units(
            dt(8, 0),
            dt(16, 55),
            start,
            end,
            60,
            0,
            30,
            "floor",
            &rules,
        );
        // (17:00 - 08:00) = 540m.
        // Actual work = 08:00 - 16:55 = 535m.
        // Net = 535 - 0 (no auto deduct) = 535.
        // Floor(535, 30) = 17 units.
        assert_eq!(std, 17);

        let mut r2 = rules.clone();
        r2.is_break_auto_deduct = true;
        let (std2, _) = AttendanceCalculator::calculate_daily_units(
            dt(8, 0),
            dt(16, 55),
            start,
            end,
            60,
            0,
            30,
            "floor",
            &r2,
        );
        // Net = 535 - 60 = 475. Floor(475, 30) = 15 units.
        assert_eq!(std2, 15);
    }
}
