#[cfg(test)]
mod tests {
    use crate::models::{EarlyArrivalStrategy, PolicyRules};
    use crate::services::attendance_calculator::{
        AttendanceCalculator, BreakDeductionInput, DailyCalcInput, WorkBuckets,
    };
    use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};

    // =========================================================================
    // Test helpers
    // =========================================================================

    fn dt(h: u32, m: u32) -> DateTime<Utc> {
        Utc.from_local_datetime(
            &NaiveDate::from_ymd_opt(2026, 1, 1)
                .expect("valid date")
                .and_time(NaiveTime::from_hms_opt(h, m, 0).expect("valid time")),
        )
        .single()
        .expect("non-ambiguous time")
    }

    fn dt_day2(h: u32, m: u32) -> DateTime<Utc> {
        Utc.from_local_datetime(
            &NaiveDate::from_ymd_opt(2026, 1, 2)
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

    // =========================================================================
    // TestInputBuilder — eliminates repetitive 14-field struct boilerplate
    // =========================================================================

    struct TestInputBuilder {
        punch_in: DateTime<Utc>,
        punch_out: DateTime<Utc>,
        shift_start: Option<DateTime<Utc>>,
        shift_end: Option<DateTime<Utc>>,
        scheduled_break_minutes: i32,
        actual_break_minutes: i32,
        break_start: Option<DateTime<Utc>>,
        break_end: Option<DateTime<Utc>>,
        work_unit: i32,
        rounding_policy: String,
        rules: PolicyRules,
        ot_ceiling: Option<DateTime<Utc>>,
        max_early_ot_minutes: Option<i32>,
    }

    impl TestInputBuilder {
        /// Defaults: punch 08:00-17:00, shift 08:00-17:00, no breaks, no OT.
        fn new() -> Self {
            Self {
                punch_in: dt(8, 0),
                punch_out: dt(17, 0),
                shift_start: Some(dt(8, 0)),
                shift_end: Some(dt(17, 0)),
                scheduled_break_minutes: 0,
                actual_break_minutes: 0,
                break_start: None,
                break_end: None,
                work_unit: 0,
                rounding_policy: String::new(),
                rules: mock_rules(),
                ot_ceiling: None,
                max_early_ot_minutes: None,
            }
        }

        fn punch(mut self, h_in: u32, m_in: u32, h_out: u32, m_out: u32) -> Self {
            self.punch_in = dt(h_in, m_in);
            self.punch_out = dt(h_out, m_out);
            self
        }

        fn punch_raw(mut self, pin: DateTime<Utc>, pout: DateTime<Utc>) -> Self {
            self.punch_in = pin;
            self.punch_out = pout;
            self
        }

        fn shift(mut self, h_start: u32, m_start: u32, h_end: u32, m_end: u32) -> Self {
            self.shift_start = Some(dt(h_start, m_start));
            self.shift_end = Some(dt(h_end, m_end));
            self
        }

        fn shift_raw(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
            self.shift_start = Some(start);
            self.shift_end = Some(end);
            self
        }

        fn no_shift(mut self) -> Self {
            self.shift_start = None;
            self.shift_end = None;
            self
        }

        fn scheduled_break(mut self, minutes: i32) -> Self {
            self.scheduled_break_minutes = minutes;
            self
        }

        fn actual_break(mut self, minutes: i32) -> Self {
            self.actual_break_minutes = minutes;
            self
        }

        fn fixed_break(mut self, h_start: u32, m_start: u32, h_end: u32, m_end: u32) -> Self {
            self.break_start = Some(dt(h_start, m_start));
            self.break_end = Some(dt(h_end, m_end));
            self
        }

        fn rules_mut(mut self, f: impl FnOnce(&mut PolicyRules)) -> Self {
            f(&mut self.rules);
            self
        }

        fn ot_ceiling_at(mut self, h: u32, m: u32) -> Self {
            self.ot_ceiling = Some(dt(h, m));
            self
        }

        fn max_early_ot(mut self, minutes: i32) -> Self {
            self.max_early_ot_minutes = Some(minutes);
            self
        }

        fn work_unit_cfg(mut self, unit: i32, policy: &str) -> Self {
            self.work_unit = unit;
            self.rounding_policy = policy.to_string();
            self
        }

        fn build(self) -> DailyCalcInput<Utc> {
            DailyCalcInput {
                punch_in: self.punch_in,
                punch_out: self.punch_out,
                shift_start: self.shift_start,
                shift_end: self.shift_end,
                scheduled_break_minutes: self.scheduled_break_minutes,
                actual_break_minutes: self.actual_break_minutes,
                break_start: self.break_start,
                break_end: self.break_end,
                work_unit: self.work_unit,
                rounding_policy: self.rounding_policy,
                rules: self.rules,
                ot_ceiling: self.ot_ceiling,
                max_early_ot_minutes: self.max_early_ot_minutes,
            }
        }

        fn buckets(self) -> WorkBuckets {
            AttendanceCalculator::calculate_buckets(&self.build())
        }

        fn daily_units(self) -> (i32, i32) {
            AttendanceCalculator::calculate_daily_units(&self.build())
        }
    }

    // =========================================================================
    // Standard shift tests
    // =========================================================================

    mod standard_shift {
        use super::*;

        #[test]
        fn within_bounds_no_break_deduct() {
            let b = TestInputBuilder::new()
                .punch(7, 30, 17, 0)
                .shift(7, 30, 17, 0)
                .scheduled_break(60)
                .buckets();
            assert_eq!(b.standard_minutes, 570); // 9.5 hours (no break deducted)
        }

        #[test]
        fn within_bounds_with_auto_deduct() {
            let b = TestInputBuilder::new()
                .punch(7, 30, 17, 0)
                .shift(7, 30, 17, 0)
                .scheduled_break(60)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .buckets();
            assert_eq!(b.standard_minutes, 510); // 8.5 hours
        }

        #[test]
        fn partial_shift_late_in_early_out() {
            let b = TestInputBuilder::new()
                .punch(9, 30, 15, 0)
                .buckets();
            assert_eq!(b.standard_minutes, 330);
            assert_eq!(b.early_ot_minutes, 0);
            assert_eq!(b.late_ot_minutes, 0);
        }

        #[test]
        fn only_ot_worked() {
            let b = TestInputBuilder::new()
                .punch(18, 0, 20, 0)
                .buckets();
            assert_eq!(b.standard_minutes, 0);
            assert_eq!(b.early_ot_minutes, 0);
            assert_eq!(b.late_ot_minutes, 120);
        }
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    mod edge_cases {
        use super::*;

        #[test]
        fn punch_out_before_punch_in_returns_zero() {
            let b = TestInputBuilder::new().punch(17, 0, 8, 0).buckets();
            assert_eq!(b.standard_minutes, 0);
            assert_eq!(b.early_ot_minutes, 0);
            assert_eq!(b.late_ot_minutes, 0);
        }

        #[test]
        fn punch_out_equals_punch_in_returns_zero() {
            let b = TestInputBuilder::new().punch(8, 0, 8, 0).buckets();
            assert_eq!(b.standard_minutes, 0);
            assert_eq!(b.early_ot_minutes, 0);
            assert_eq!(b.late_ot_minutes, 0);
        }

        #[test]
        fn no_shift_all_standard() {
            let b = TestInputBuilder::new()
                .punch(6, 0, 18, 0)
                .no_shift()
                .buckets();
            assert_eq!(b.standard_minutes, 720);
            assert_eq!(b.early_ot_minutes, 0);
            assert_eq!(b.late_ot_minutes, 0);
        }

        #[test]
        fn no_shift_short_session() {
            let b = TestInputBuilder::new()
                .punch(9, 0, 9, 15)
                .no_shift()
                .buckets();
            assert_eq!(b.standard_minutes, 15);
        }

        #[test]
        fn daily_units_no_shift_returns_nonzero_proving_guard_needed() {
            let (std, ot) = TestInputBuilder::new()
                .no_shift()
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 18, "calculator gives 18 std units without shift — report guard needed");
            assert_eq!(ot, 0, "no shift boundaries means no OT");
        }

        #[test]
        fn daily_units_no_shift_with_auto_break() {
            let (std, ot) = TestInputBuilder::new()
                .no_shift()
                .scheduled_break(60)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 18);
            assert_eq!(ot, 0);
        }

        #[test]
        fn daily_units_no_shift_accidental_face_recognition() {
            let (std, ot) = TestInputBuilder::new()
                .punch(7, 55, 8, 0)
                .no_shift()
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 0, "5-min accidental session rounds to 0 units");
            assert_eq!(ot, 0);
        }

        #[test]
        fn overnight_shift_standard_hours() {
            let b = TestInputBuilder::new()
                .punch_raw(dt(22, 0), dt_day2(6, 0))
                .shift(22, 0, 6, 0) // end < start triggers +1 day internally
                .buckets();
            assert_eq!(b.standard_minutes, 480);
            assert_eq!(b.late_ot_minutes, 0);
        }

        #[test]
        fn daily_calc_input_debug_trait() {
            let input = TestInputBuilder::new().no_shift().build();
            let debug_str = format!("{input:?}");
            assert!(debug_str.contains("DailyCalcInput"));
        }
    }

    // =========================================================================
    // Grace period & clamping
    // =========================================================================

    mod grace_period {
        use super::*;

        #[test]
        fn within_grace_no_ot() {
            let b = TestInputBuilder::new()
                .punch(7, 15, 17, 0)
                .shift(7, 30, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 0);
        }

        #[test]
        fn before_grace_full_ot() {
            let b = TestInputBuilder::new()
                .punch(6, 45, 17, 0)
                .shift(7, 30, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 45);
        }

        #[test]
        fn clamp_strategy() {
            let b = TestInputBuilder::new()
                .punch(6, 45, 17, 0)
                .shift(7, 30, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 30);
        }

        #[test]
        fn boundaries_allow_on_threshold() {
            let b = TestInputBuilder::new()
                .punch(7, 30, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Allow;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 0);
        }

        #[test]
        fn boundaries_allow_1min_before() {
            let b = TestInputBuilder::new()
                .punch(7, 29, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Allow;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 31);
        }

        #[test]
        fn boundaries_allow_1min_after() {
            let b = TestInputBuilder::new()
                .punch(7, 31, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Allow;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 0);
        }

        #[test]
        fn boundaries_block_on_threshold() {
            let b = TestInputBuilder::new()
                .punch(7, 30, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Block;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 0);
        }

        #[test]
        fn boundaries_block_1min_before() {
            let b = TestInputBuilder::new()
                .punch(7, 29, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Block;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 31);
        }

        #[test]
        fn boundaries_clamp_on_threshold() {
            let b = TestInputBuilder::new()
                .punch(7, 30, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 30);
        }

        #[test]
        fn boundaries_clamp_1min_before() {
            let b = TestInputBuilder::new()
                .punch(7, 29, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 30);
        }

        #[test]
        fn boundaries_clamp_1min_after() {
            let b = TestInputBuilder::new()
                .punch(7, 31, 17, 0)
                .rules_mut(|r| {
                    r.grace_period_minutes = 30;
                    r.allow_early_ot = true;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
                })
                .buckets();
            assert_eq!(b.early_ot_minutes, 29);
        }
    }

    // =========================================================================
    // Authorized early leave
    // =========================================================================

    mod early_leave {
        use super::*;

        #[test]
        fn exit_30m_early_paid() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 16, 30)
                .rules_mut(|r| {
                    r.authorized_early_leave_minutes = 60;
                    r.is_early_leave_paid = true;
                })
                .buckets();
            assert_eq!(b.standard_minutes, 540);
        }

        #[test]
        fn exit_90m_early_not_paid() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 15, 30)
                .rules_mut(|r| {
                    r.authorized_early_leave_minutes = 60;
                    r.is_early_leave_paid = true;
                })
                .buckets();
            assert_eq!(b.standard_minutes, 450);
        }

        #[test]
        fn exactly_60m_early_paid() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 16, 0)
                .rules_mut(|r| {
                    r.authorized_early_leave_minutes = 60;
                    r.is_early_leave_paid = true;
                })
                .buckets();
            assert_eq!(b.standard_minutes, 540);
        }

        #[test]
        fn exactly_61m_early_not_paid() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 15, 59)
                .rules_mut(|r| {
                    r.authorized_early_leave_minutes = 60;
                    r.is_early_leave_paid = true;
                })
                .buckets();
            assert_eq!(b.standard_minutes, 479);
        }

        #[test]
        fn exactly_59m_early_paid() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 16, 1)
                .rules_mut(|r| {
                    r.authorized_early_leave_minutes = 60;
                    r.is_early_leave_paid = true;
                })
                .buckets();
            assert_eq!(b.standard_minutes, 540);
        }

        #[test]
        fn not_paid_when_flag_disabled() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 16, 0)
                .rules_mut(|r| {
                    r.authorized_early_leave_minutes = 60;
                    r.is_early_leave_paid = false;
                })
                .buckets();
            assert_eq!(b.standard_minutes, 480);
        }
    }

    // =========================================================================
    // OT tests
    // =========================================================================

    mod ot_tests {
        use super::*;

        #[test]
        fn early_arrival_no_ot_when_disabled() {
            let b = TestInputBuilder::new().punch(6, 0, 17, 0).buckets();
            assert_eq!(b.early_ot_minutes, 0);
            assert_eq!(b.standard_minutes, 540);
        }

        #[test]
        fn late_ot_uncapped() {
            let b = TestInputBuilder::new().punch(8, 0, 20, 0).buckets();
            assert_eq!(b.standard_minutes, 540);
            assert_eq!(b.late_ot_minutes, 180);
        }

        #[test]
        fn early_and_late_combined() {
            let b = TestInputBuilder::new()
                .punch(6, 0, 20, 0)
                .rules_mut(|r| r.allow_early_ot = true)
                .buckets();
            assert_eq!(b.early_ot_minutes, 120);
            assert_eq!(b.standard_minutes, 540);
            assert_eq!(b.late_ot_minutes, 180);
        }

        #[test]
        fn daily_units_with_ot() {
            let (std, ot) = TestInputBuilder::new()
                .punch(7, 0, 19, 0)
                .scheduled_break(60)
                .rules_mut(|r| r.allow_early_ot = true)
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 18);
            assert_eq!(ot, 6);
        }

        #[test]
        fn daily_units_with_auto_break_and_ot() {
            let (std, ot) = TestInputBuilder::new()
                .punch(7, 0, 19, 0)
                .scheduled_break(60)
                .rules_mut(|r| {
                    r.allow_early_ot = true;
                    r.is_break_auto_deduct = true;
                })
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 16);
            assert_eq!(ot, 6);
        }

        // --- Split shift OT ceiling ---

        #[test]
        fn split_shift_ot_capped_at_next_shift() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 13, 30)
                .shift(8, 0, 12, 0)
                .ot_ceiling_at(13, 0)
                .buckets();
            assert_eq!(b.standard_minutes, 240);
            assert_eq!(b.late_ot_minutes, 60);
        }

        #[test]
        fn split_shift_no_ceiling_ot_unlimited() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 13, 30)
                .shift(8, 0, 12, 0)
                .buckets();
            assert_eq!(b.standard_minutes, 240);
            assert_eq!(b.late_ot_minutes, 90);
        }

        #[test]
        fn split_shift_gap_fully_utilized() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 13, 0)
                .shift(8, 0, 12, 0)
                .ot_ceiling_at(13, 0)
                .buckets();
            assert_eq!(b.standard_minutes, 240);
            assert_eq!(b.late_ot_minutes, 60);
        }

        #[test]
        fn ceiling_exact() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 13, 0)
                .shift(8, 0, 12, 0)
                .ot_ceiling_at(13, 0)
                .buckets();
            assert_eq!(b.late_ot_minutes, 60);
        }

        #[test]
        fn ceiling_1min_before() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 12, 59)
                .shift(8, 0, 12, 0)
                .ot_ceiling_at(13, 0)
                .buckets();
            assert_eq!(b.late_ot_minutes, 59);
        }

        #[test]
        fn ceiling_1min_after() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 13, 1)
                .shift(8, 0, 12, 0)
                .ot_ceiling_at(13, 0)
                .buckets();
            assert_eq!(b.late_ot_minutes, 60);
        }

        // --- Early OT capping ---

        #[test]
        fn early_ot_capped() {
            let b = TestInputBuilder::new()
                .punch(5, 0, 17, 0)
                .rules_mut(|r| r.allow_early_ot = true)
                .max_early_ot(120)
                .buckets();
            assert_eq!(b.early_ot_minutes, 120);
        }

        #[test]
        fn early_ot_uncapped() {
            let b = TestInputBuilder::new()
                .punch(5, 0, 17, 0)
                .rules_mut(|r| r.allow_early_ot = true)
                .buckets();
            assert_eq!(b.early_ot_minutes, 180);
        }

        #[test]
        fn early_ot_under_cap() {
            let b = TestInputBuilder::new()
                .punch(5, 0, 17, 0)
                .rules_mut(|r| r.allow_early_ot = true)
                .max_early_ot(240)
                .buckets();
            assert_eq!(b.early_ot_minutes, 180);
        }

        #[test]
        fn early_ot_cap_zero() {
            let b = TestInputBuilder::new()
                .punch(5, 0, 17, 0)
                .rules_mut(|r| r.allow_early_ot = true)
                .max_early_ot(0)
                .buckets();
            assert_eq!(b.early_ot_minutes, 0);
        }

        #[test]
        fn early_ot_cap_with_clamp_grace_limited() {
            let b = TestInputBuilder::new()
                .punch(7, 15, 17, 0)
                .rules_mut(|r| {
                    r.allow_early_ot = true;
                    r.grace_period_minutes = 30;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
                })
                .max_early_ot(15)
                .buckets();
            assert_eq!(b.early_ot_minutes, 15);
        }

        #[test]
        fn early_ot_cap_with_clamp_grace_not_limiting() {
            let b = TestInputBuilder::new()
                .punch(7, 15, 17, 0)
                .rules_mut(|r| {
                    r.allow_early_ot = true;
                    r.grace_period_minutes = 30;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Clamp;
                })
                .max_early_ot(60)
                .buckets();
            assert_eq!(b.early_ot_minutes, 30);
        }

        #[test]
        fn early_ot_cap_with_block_grace() {
            let b = TestInputBuilder::new()
                .punch(7, 45, 17, 0)
                .rules_mut(|r| {
                    r.allow_early_ot = true;
                    r.grace_period_minutes = 30;
                    r.early_arrival_strategy = EarlyArrivalStrategy::Block;
                })
                .max_early_ot(15)
                .buckets();
            assert_eq!(b.early_ot_minutes, 0);
        }

        #[test]
        fn early_ot_cap_overnight() {
            let b = TestInputBuilder::new()
                .punch_raw(dt(20, 0), dt(6, 0) + chrono::Duration::days(1))
                .shift_raw(dt(22, 0), dt(6, 0) + chrono::Duration::days(1))
                .rules_mut(|r| r.allow_early_ot = true)
                .max_early_ot(45)
                .buckets();
            assert_eq!(b.early_ot_minutes, 45);
        }
    }

    // =========================================================================
    // Break deduction tests
    // =========================================================================

    mod break_deduction {
        use super::*;

        fn punch_range() -> (DateTime<Utc>, DateTime<Utc>) {
            (dt(7, 0), dt(18, 0))
        }

        fn deduct(
            actual: i32, scheduled: i32,
            auto_deduct: bool, is_ot: bool,
            bs: Option<&DateTime<Utc>>, be: Option<&DateTime<Utc>>,
            pi: &DateTime<Utc>, po: &DateTime<Utc>,
        ) -> i32 {
            AttendanceCalculator::calculate_break_deduction(&BreakDeductionInput {
                actual_break_minutes: actual,
                scheduled_break_minutes: scheduled,
                is_break_auto_deduct: auto_deduct,
                is_break_overtime: is_ot,
                break_start: bs, break_end: be,
                punch_in: pi, punch_out: po,
            })
        }

        #[test]
        fn actual_break_takes_precedence() {
            let (pi, po) = punch_range();
            let d = deduct(45, 60, true, false, None, None, &pi, &po);
            assert_eq!(d, 45);
        }

        #[test]
        fn auto_deduct_when_no_actual() {
            let (pi, po) = punch_range();
            let d = deduct(0, 60, true, false, None, None, &pi, &po);
            assert_eq!(d, 60);
        }

        #[test]
        fn skip_when_break_is_overtime() {
            let (pi, po) = punch_range();
            let d = deduct(0, 60, true, true, None, None, &pi, &po);
            assert_eq!(d, 0);
        }

        #[test]
        fn no_auto_deduct_no_actual() {
            let (pi, po) = punch_range();
            let d = deduct(0, 60, false, false, None, None, &pi, &po);
            assert_eq!(d, 0);
        }

        #[test]
        fn zero_scheduled() {
            let (pi, po) = punch_range();
            let d = deduct(0, 0, true, false, None, None, &pi, &po);
            assert_eq!(d, 0);
        }
    }

    // =========================================================================
    // Frame intersection tests (fixed break mode)
    // =========================================================================

    mod frame_intersection {
        use super::*;

        fn deduct_framed(
            bs: Option<&DateTime<Utc>>, be: Option<&DateTime<Utc>>,
            pi: &DateTime<Utc>, po: &DateTime<Utc>,
            is_ot: bool,
        ) -> i32 {
            AttendanceCalculator::calculate_break_deduction(&BreakDeductionInput {
                actual_break_minutes: 0,
                scheduled_break_minutes: 60,
                is_break_auto_deduct: true,
                is_break_overtime: is_ot,
                break_start: bs, break_end: be,
                punch_in: pi, punch_out: po,
            })
        }

        #[test]
        fn full_overlap() {
            let (pi, po) = (dt(8, 0), dt(17, 0));
            let (bs, be) = (Some(dt(11, 30)), Some(dt(12, 30)));
            let d = deduct_framed(bs.as_ref(), be.as_ref(), &pi, &po, false);
            assert_eq!(d, 60);
        }

        #[test]
        fn partial_overlap_leave_during_break() {
            let (pi, po) = (dt(8, 0), dt(12, 0));
            let (bs, be) = (Some(dt(11, 30)), Some(dt(12, 30)));
            let d = deduct_framed(bs.as_ref(), be.as_ref(), &pi, &po, false);
            assert_eq!(d, 30);
        }

        #[test]
        fn no_overlap_leave_before_break() {
            let (pi, po) = (dt(8, 0), dt(11, 0));
            let (bs, be) = (Some(dt(11, 30)), Some(dt(12, 30)));
            let d = deduct_framed(bs.as_ref(), be.as_ref(), &pi, &po, false);
            assert_eq!(d, 0);
        }

        #[test]
        fn arrive_during_break() {
            let (pi, po) = (dt(12, 0), dt(17, 0));
            let (bs, be) = (Some(dt(11, 30)), Some(dt(12, 30)));
            let d = deduct_framed(bs.as_ref(), be.as_ref(), &pi, &po, false);
            assert_eq!(d, 30);
        }

        #[test]
        fn break_ot_no_deduction() {
            let (pi, po) = (dt(8, 0), dt(17, 0));
            let (bs, be) = (Some(dt(11, 30)), Some(dt(12, 30)));
            let d = deduct_framed(bs.as_ref(), be.as_ref(), &pi, &po, true);
            assert_eq!(d, 0);
        }

        // --- Fixed break via calculate_buckets ---

        #[test]
        fn buckets_partial_overlap() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 12, 0)
                .scheduled_break(60)
                .fixed_break(11, 30, 12, 30)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .buckets();
            assert_eq!(b.standard_minutes, 210);
        }

        #[test]
        fn buckets_no_overlap() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 11, 0)
                .scheduled_break(60)
                .fixed_break(11, 30, 12, 30)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .buckets();
            assert_eq!(b.standard_minutes, 180);
        }

        #[test]
        fn buckets_full_overlap() {
            let b = TestInputBuilder::new()
                .scheduled_break(60)
                .fixed_break(11, 30, 12, 30)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .buckets();
            assert_eq!(b.standard_minutes, 480);
        }
    }

    // =========================================================================
    // Break is overtime tests
    // =========================================================================

    mod break_overtime {
        use super::*;

        #[test]
        fn fixed_worked() {
            let b = TestInputBuilder::new()
                .scheduled_break(60)
                .fixed_break(12, 0, 13, 0)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 480);
            assert_eq!(b.break_ot_minutes, 60);
        }

        #[test]
        fn fixed_taken() {
            let b = TestInputBuilder::new()
                .scheduled_break(60)
                .actual_break(60)
                .fixed_break(12, 0, 13, 0)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 480);
            assert_eq!(b.break_ot_minutes, 0);
        }

        #[test]
        fn flexible_worked() {
            let b = TestInputBuilder::new()
                .scheduled_break(60)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 480);
            assert_eq!(b.break_ot_minutes, 60);
        }

        #[test]
        fn partial_overlap() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 12, 30)
                .scheduled_break(60)
                .fixed_break(12, 0, 13, 0)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 240);
            assert_eq!(b.break_ot_minutes, 30);
        }

        #[test]
        fn no_overlap() {
            let b = TestInputBuilder::new()
                .punch(8, 0, 11, 30)
                .scheduled_break(60)
                .fixed_break(12, 0, 13, 0)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 210);
            assert_eq!(b.break_ot_minutes, 0);
        }

        #[test]
        fn actual_shorter() {
            let b = TestInputBuilder::new()
                .scheduled_break(60)
                .actual_break(30)
                .fixed_break(12, 0, 13, 0)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 480);
            assert_eq!(b.break_ot_minutes, 30);
        }

        #[test]
        fn actual_longer() {
            let b = TestInputBuilder::new()
                .scheduled_break(60)
                .actual_break(90)
                .fixed_break(12, 0, 13, 0)
                .rules_mut(|r| r.is_break_overtime = true)
                .buckets();
            assert_eq!(b.standard_minutes, 450);
            assert_eq!(b.break_ot_minutes, 0);
        }

        #[test]
        fn overnight() {
            let b = TestInputBuilder::new()
                .punch_raw(dt(22, 0), dt_day2(6, 0))
                .shift_raw(dt(22, 0), dt_day2(6, 0))
                .scheduled_break(60)
                .rules_mut(|r| r.is_break_overtime = true)
                .build();

            // Rebuild with break frame on day 2
            let input = DailyCalcInput {
                break_start: Some(dt_day2(1, 0)),
                break_end: Some(dt_day2(2, 0)),
                ..b
            };
            let buckets = AttendanceCalculator::calculate_buckets(&input);
            assert_eq!(buckets.standard_minutes, 420);
            assert_eq!(buckets.break_ot_minutes, 60);
        }
    }

    // =========================================================================
    // Unit calculation tests
    // =========================================================================

    mod unit_calculation {
        use super::*;

        #[test]
        fn floor_rounding() {
            assert_eq!(AttendanceCalculator::calculate_units(75, 30, "floor"), 2);
            assert_eq!(AttendanceCalculator::calculate_units(90, 30, "floor"), 3);
            assert_eq!(AttendanceCalculator::calculate_units(29, 30, "floor"), 0);
        }

        #[test]
        fn ceil_rounding() {
            assert_eq!(AttendanceCalculator::calculate_units(75, 30, "ceil"), 3);
            assert_eq!(AttendanceCalculator::calculate_units(90, 30, "ceil"), 3);
            assert_eq!(AttendanceCalculator::calculate_units(1, 30, "ceil"), 1);
        }

        #[test]
        fn round_rounding() {
            assert_eq!(AttendanceCalculator::calculate_units(75, 30, "round"), 3);
            assert_eq!(AttendanceCalculator::calculate_units(44, 30, "round"), 1);
            assert_eq!(AttendanceCalculator::calculate_units(46, 30, "round"), 2);
        }

        #[test]
        fn zero_minutes() {
            assert_eq!(AttendanceCalculator::calculate_units(0, 30, "floor"), 0);
            assert_eq!(AttendanceCalculator::calculate_units(0, 30, "ceil"), 0);
            assert_eq!(AttendanceCalculator::calculate_units(0, 30, "round"), 0);
        }

        #[test]
        fn default_rounding() {
            assert_eq!(AttendanceCalculator::calculate_units(75, 30, "unknown"), 2);
        }

        #[test]
        fn unit_size_1() {
            assert_eq!(AttendanceCalculator::calculate_units(480, 1, "floor"), 480);
            assert_eq!(AttendanceCalculator::calculate_units(480, 0, "floor"), 480);
        }

        #[test]
        fn unit_size_15() {
            assert_eq!(AttendanceCalculator::calculate_units(480, 15, "floor"), 32);
            assert_eq!(AttendanceCalculator::calculate_units(481, 15, "floor"), 32);
            assert_eq!(AttendanceCalculator::calculate_units(481, 15, "ceil"), 33);
        }

        #[test]
        fn unit_size_45() {
            assert_eq!(AttendanceCalculator::calculate_units(480, 45, "floor"), 10);
            assert_eq!(AttendanceCalculator::calculate_units(480, 45, "ceil"), 11);
            assert_eq!(AttendanceCalculator::calculate_units(480, 45, "round"), 11);
        }

        #[test]
        fn work_unit_edge_cases() {
            assert_eq!(AttendanceCalculator::calculate_units(480, 0, "floor"), 480);
            assert_eq!(AttendanceCalculator::calculate_units(480, 1, "floor"), 480);
            assert_eq!(AttendanceCalculator::calculate_units(75, 30, "invalid_policy"), 2);
        }

        // --- Daily units integration ---

        #[test]
        fn daily_calc_via_builder() {
            let (std, ot) = TestInputBuilder::new()
                .scheduled_break(60)
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 18);
            assert_eq!(ot, 0);
        }

        #[test]
        fn daily_calc_with_auto_deduct() {
            let (std, _) = TestInputBuilder::new()
                .scheduled_break(60)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 16);
        }

        #[test]
        fn rounding_and_units() {
            let (std, _) = TestInputBuilder::new()
                .punch(8, 0, 16, 55)
                .scheduled_break(60)
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std, 17);

            let (std2, _) = TestInputBuilder::new()
                .punch(8, 0, 16, 55)
                .scheduled_break(60)
                .rules_mut(|r| r.is_break_auto_deduct = true)
                .work_unit_cfg(30, "floor")
                .daily_units();
            assert_eq!(std2, 15);
        }
    }
}
