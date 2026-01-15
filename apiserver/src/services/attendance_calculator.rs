use chrono::{DateTime, Duration, TimeZone};

pub struct AttendanceCalculator;

impl AttendanceCalculator {
    /// 1. Grace Period Logic
    /// Determines the effective punch-in time based on the shift schedule and grace period.
    /// If the employee punches in within the grace window (e.g., 30 mins before start),
    /// it counts as starting exactly at the shift start time.
    pub fn calculate_effective_punch_in<T: TimeZone>(
        actual_punch_in: DateTime<T>,
        shift_start_dt: Option<DateTime<T>>,
        grace_period_minutes: i32,
    ) -> DateTime<T> {
        if let Some(shift_start) = shift_start_dt {
            if grace_period_minutes > 0 {
                let grace_start = shift_start.clone() - Duration::minutes(grace_period_minutes as i64);
                // If punch is between [Shift Start - Grace] and [Shift Start]
                if actual_punch_in >= grace_start && actual_punch_in < shift_start {
                    return shift_start;
                }
            }
        }
        actual_punch_in
    }

    /// 2. Break Deduction Logic
    /// Calculates how many minutes should be deducted for breaks.
    /// - Priority 1: Actual break duration (if > 0)
    /// - Priority 2: Auto-deduct (if enabled and not overtime-break)
    /// - Priority 3: 0
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

    /// 3. Rounding Logic
    /// Applies Floor, Ceil, or Round logic based on the work unit (e.g., 15 minutes).
    pub fn apply_rounding(minutes: i32, unit: i32, policy: &str) -> i32 {
        if unit <= 1 || policy == "none" {
            return minutes;
        }

        let unit = unit as f64;
        let mins = minutes as f64;

        match policy {
            "floor" => (mins / unit).floor() as i32 * unit as i32,
            "ceil" => (mins / unit).ceil() as i32 * unit as i32,
            "round" => (mins / unit).round() as i32 * unit as i32,
            _ => minutes,
        }
    }

    /// 4. Main Calculation (Orchestrator)
    /// Calculates the net standard hours and overtime minutes for a specific day.
    /// 
    /// Returns: (Standard Minutes, Overtime Minutes)
    pub fn calculate_daily_net_hours<T: TimeZone>(
        punch_in: DateTime<T>,
        punch_out: DateTime<T>,
        shift_start_dt: Option<DateTime<T>>, // Scheduled Start (Absolute time for that day)
        shift_end_dt: Option<DateTime<T>>,   // Scheduled End (Absolute time for that day)
        scheduled_break_minutes: i32,
        grace_period_minutes: i32,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
        actual_break_minutes: i32,
    ) -> (i32, i32) {
        // A. Determine Effective Punch In
        let effective_punch_in = Self::calculate_effective_punch_in(
            punch_in,
            shift_start_dt.clone(),
            grace_period_minutes
        );

        // B. Calculate Raw Duration (from Effective In to Out)
        let raw_duration = punch_out
            .signed_duration_since(effective_punch_in)
            .num_minutes();
        
        let raw_duration = i32::max(0, raw_duration as i32);

        // C. Calculate Break Deduction
        let break_to_deduct = Self::calculate_break_deduction(
            actual_break_minutes,
            scheduled_break_minutes,
            is_break_auto_deduct,
            is_break_overtime,
        );

        // D. Calculate Net Worked Minutes
        let worked_minutes = i32::max(0, raw_duration - break_to_deduct);

        // E. Split Standard vs Overtime
        if let (Some(start), Some(mut end)) = (shift_start_dt, shift_end_dt) {
            // Handle Overnight Shift linearization for capacity check
            if end < start {
                end = end + Duration::days(1);
            }

            let shift_duration = end.signed_duration_since(start).num_minutes() as i32;
            let standard_work_capacity = shift_duration - scheduled_break_minutes; // Standard hours capacity

            if worked_minutes > standard_work_capacity {
                let standard = standard_work_capacity;
                let overtime = worked_minutes - standard_work_capacity;
                return (standard, overtime);
            }
        }

        // If no shift or under capacity, everything is standard (or based on policy, 
        // usually if no shift, logic might vary, but here we assume standard).
        (worked_minutes, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Helper to create UTC DateTime for testing
    fn dt(h: u32, m: u32) -> DateTime<Utc> {
        Utc.from_local_datetime(
            &NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
                .and_time(NaiveTime::from_hms_opt(h, m, 0).unwrap())
        ).single().unwrap()
    }

    #[test]
    fn test_grace_period_logic() {
        let shift_start = Some(dt(8, 0)); // 08:00
        let grace = 30; // 30 mins grace

        // Case 1: Punch inside grace window (07:45) -> Should be 08:00
        let p1 = dt(7, 45);
        assert_eq!(
            AttendanceCalculator::calculate_effective_punch_in(p1, shift_start, grace),
            shift_start.unwrap()
        );

        // Case 2: Punch before grace window (07:29) -> Should keep 07:29 (Early OT)
        let p2 = dt(7, 29);
        assert_eq!(
            AttendanceCalculator::calculate_effective_punch_in(p2, shift_start, grace),
            p2
        );

        // Case 3: Punch exactly at start (08:00) -> 08:00
        let p3 = dt(8, 0);
        assert_eq!(
            AttendanceCalculator::calculate_effective_punch_in(p3, shift_start, grace),
            p3
        );

        // Case 4: Punch Late (08:01) -> 08:01
        let p4 = dt(8, 1);
        assert_eq!(
            AttendanceCalculator::calculate_effective_punch_in(p4, shift_start, grace),
            p4
        );
    }

    #[test]
    fn test_break_deduction() {
        // Case 1: Actual break exists (Priority 1)
        assert_eq!(
            AttendanceCalculator::calculate_break_deduction(60, 30, true, false),
            60
        );

        // Case 2: No actual break, Auto-deduct ON (Priority 2)
        assert_eq!(
            AttendanceCalculator::calculate_break_deduction(0, 45, true, false),
            45
        );

        // Case 3: No actual break, Auto-deduct OFF
        assert_eq!(
            AttendanceCalculator::calculate_break_deduction(0, 45, false, false),
            0
        );

        // Case 4: No actual break, Auto-deduct ON but Is Overtime Break (Should NOT deduct)
        assert_eq!(
            AttendanceCalculator::calculate_break_deduction(0, 45, true, true),
            0
        );
    }

    #[test]
    fn test_rounding_logic() {
        // Floor
        assert_eq!(AttendanceCalculator::apply_rounding(29, 15, "floor"), 15);
        // Ceil
        assert_eq!(AttendanceCalculator::apply_rounding(16, 15, "ceil"), 30);
        // Round
        assert_eq!(AttendanceCalculator::apply_rounding(22, 15, "round"), 15); // < 7.5
        assert_eq!(AttendanceCalculator::apply_rounding(23, 15, "round"), 30); // > 7.5
    }

    #[test]
    fn test_full_calculation_scenario() {
        let shift_start = Some(dt(8, 0));
        let shift_end = Some(dt(17, 0)); // 9 hours total
        let scheduled_break = 60; // 1 hour break -> 8 hours standard capacity

        // Scenario: Worked 08:00 to 18:00 (10 hours raw).
        // Auto-deduct break (60 mins).
        // Expect: 
        // Raw Work: 10h (600m)
        // Net Work: 600 - 60 = 540m (9h)
        // Capacity: 8h (480m)
        // Result: 480 Standard, 60 Overtime.

        let (std, ot) = AttendanceCalculator::calculate_daily_net_hours(
            dt(8, 0),
            dt(18, 0),
            shift_start,
            shift_end,
            scheduled_break,
            0,    // No grace
            true, // Auto deduct
            false,
            0     // No actual break
        );

        assert_eq!(std, 480); // 8 hours
        assert_eq!(ot, 60);   // 1 hour OT
    }

    #[test]
    fn test_grace_period_calculation_scenario() {
        let shift_start = Some(dt(8, 0));
        let shift_end = Some(dt(17, 0));
        let scheduled_break = 60; 

        // Scenario: Punch In 07:45 (Grace 30m) -> Effective 08:00.
        // Punch Out 17:00.
        // Raw Duration: 08:00 to 17:00 = 9h (540m).
        // Break: 60m.
        // Net: 480m.
        // Standard Cap: 480m.
        // Result: 480 Std, 0 OT.

        let (std, ot) = AttendanceCalculator::calculate_daily_net_hours(
            dt(7, 45),
            dt(17, 0),
            shift_start,
            shift_end,
            scheduled_break,
            30,   // 30m Grace
            true,
            false,
            0
        );

        assert_eq!(std, 480);
        assert_eq!(ot, 0);
    }
}
