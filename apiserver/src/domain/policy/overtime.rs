use crate::models::OvertimeRule;
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Timelike};
use chrono_tz::Tz;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct OvertimeSegment {
    start: NaiveTime,
    end: NaiveTime,
    multiplier: Decimal,
}

impl OvertimeSegment {
    pub fn new(start: NaiveTime, end: NaiveTime, multiplier: Decimal) -> Self {
        Self {
            start,
            end,
            multiplier,
        }
    }

    pub fn start(&self) -> NaiveTime {
        self.start
    }

    pub fn end(&self) -> NaiveTime {
        self.end
    }

    pub fn set_end(&mut self, end: NaiveTime) {
        self.end = end;
    }

    pub fn multiplier(&self) -> Decimal {
        self.multiplier
    }
}

pub struct OvertimeEngine;

impl OvertimeEngine {
    pub fn calculate_segments(
        rules: &[OvertimeRule],
        date: NaiveDate,
        is_holiday: bool,
    ) -> Vec<OvertimeSegment> {
        let weekday = date.weekday();
        let day_mask = 1_i32 << weekday.num_days_from_monday();

        let active_rules: Vec<&OvertimeRule> = rules
            .iter()
            .filter(|r| {
                let day_matches = (r.days_of_week() & day_mask) != 0;
                let holiday_matches = !r.is_holiday() || is_holiday;
                day_matches && holiday_matches
            })
            .collect();

        if active_rules.is_empty() {
            return vec![];
        }

        let mut time_points = std::collections::BTreeSet::new();
        time_points.insert(NaiveTime::from_hms_opt(0, 0, 0).unwrap_or_default());
        time_points.insert(NaiveTime::from_hms_opt(23, 59, 59).unwrap_or_default());

        for rule in &active_rules {
            if let Some(start) = rule.time_start() {
                time_points.insert(start);
            }
            if let Some(end) = rule.time_end() {
                time_points.insert(end);
            }
        }

        let sorted_points: Vec<NaiveTime> = time_points.into_iter().collect();
        let mut segments = Vec::new();

        for i in 0..sorted_points.len() - 1 {
            let start = sorted_points[i];
            let end = sorted_points[i + 1];

            let mid_secs = u32::midpoint(
                start.num_seconds_from_midnight(),
                end.num_seconds_from_midnight(),
            );
            let mid_time =
                NaiveTime::from_num_seconds_from_midnight_opt(mid_secs, 0).unwrap_or_default();

            let best_rule = active_rules
                .iter()
                .filter(|r| {
                    let r_start = r.time_start().unwrap_or(NaiveTime::MIN);
                    let r_end = r
                        .time_end()
                        .unwrap_or_else(|| NaiveTime::from_hms_opt(23, 59, 59).unwrap_or_default());

                    if r_start <= r_end {
                        mid_time >= r_start && mid_time < r_end
                    } else {
                        mid_time >= r_start || mid_time < r_end
                    }
                })
                .max_by_key(|r| r.priority());

            if let Some(rule) = best_rule {
                segments.push(OvertimeSegment::new(start, end, rule.multiplier()));
            }
        }

        if segments.is_empty() {
            return vec![];
        }

        let mut merged = Vec::new();
        let mut current = segments[0].clone();

        for next in segments.into_iter().skip(1) {
            if next.multiplier() == current.multiplier() {
                current.set_end(next.end());
            } else {
                merged.push(current);
                current = next;
            }
        }
        merged.push(current);

        merged
    }

    pub fn calculate_overtime_pay_from_times(
        rules: &[OvertimeRule],
        date: NaiveDate,
        is_holiday: bool,
        ot_intervals: &[(NaiveTime, NaiveTime)],
        hourly_rate: Decimal,
    ) -> Decimal {
        let segments = Self::calculate_segments(rules, date, is_holiday);
        if segments.is_empty() {
            return Decimal::ZERO;
        }

        let mut total_pay = Decimal::ZERO;

        for (ot_start, ot_end) in ot_intervals {
            for segment in &segments {
                let inter_start = NaiveTime::max(*ot_start, segment.start());
                let inter_end = NaiveTime::min(*ot_end, segment.end());

                if inter_end > inter_start {
                    let duration_mins = (inter_end.num_seconds_from_midnight()
                        - inter_start.num_seconds_from_midnight())
                        / 60;
                    let duration_hours = Decimal::from(duration_mins) / Decimal::from(60);
                    total_pay += duration_hours * hourly_rate * segment.multiplier();
                }
            }
        }

        total_pay
    }

    /// Splits a DateTime interval into daily NaiveTime segments for calculation
    pub fn split_interval_by_day<T: TimeZone>(
        start: &DateTime<T>,
        end: &DateTime<T>,
        tz: &Tz,
    ) -> Vec<(NaiveDate, Vec<(NaiveTime, NaiveTime)>)> {
        let mut result = Vec::new();
        if *start >= *end {
            return result;
        }

        let mut current = start.with_timezone(tz);
        let end_tz = end.with_timezone(tz);

        while current < end_tz {
            let date = current.date_naive();
            // End of day in local time — delegate to tz.rs for DST handling
            let day_end = crate::utils::tz::local_to_tz(tz, date, 23, 59, 59).unwrap_or(current);

            let segment_end = if day_end < end_tz { day_end } else { end_tz };

            let mut daily_intervals = Vec::new();

            // Special case for NaiveTime max
            let s_time = current.time();
            let e_time = if segment_end.date_naive() > date {
                NaiveTime::from_hms_opt(23, 59, 59).unwrap_or_default()
            } else {
                segment_end.time()
            };

            if e_time > s_time {
                daily_intervals.push((s_time, e_time));
            }

            result.push((date, daily_intervals));

            if segment_end >= end_tz {
                break;
            }
            // Move to next day — delegate to tz.rs for DST handling at midnight
            let Some(next_date) = date.succ_opt() else {
                break;
            };
            current = crate::utils::tz::local_to_tz(tz, next_date, 0, 0, 0).unwrap_or(segment_end);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use chrono::NaiveTime;
    use uuid::Uuid;

    fn mock_rule(
        priority: i32,
        multiplier: Decimal,
        start: Option<&str>,
        end: Option<&str>,
        days: i32,
    ) -> OvertimeRule {
        OvertimeRule::new(NewOvertimeRuleParams {
            id: Uuid::new_v4(),
            policy_id: Uuid::new_v4(),
            priority,
            multiplier,
            time_start: start
                .map(|s| NaiveTime::parse_from_str(s, "%H:%M:%S").expect("valid mock time")),
            time_end: end
                .map(|s| NaiveTime::parse_from_str(s, "%H:%M:%S").expect("valid mock time")),
            days_of_week: days,
            is_holiday: false,
            created_at: None,
            updated_at: None,
        })
    }

    #[test]
    fn test_priority_resolution() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date"); // Monday
        let rules = vec![
            mock_rule(1, Decimal::new(15, 1), None, None, 127), // Base 1.5x all day
            mock_rule(
                10,
                Decimal::new(20, 1),
                Some("22:00:00"),
                Some("06:00:00"),
                127,
            ), // Night 2.0x (Overnight)
        ];

        let segments = OvertimeEngine::calculate_segments(&rules, date, false);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].multiplier(), Decimal::new(20, 1));
        assert_eq!(segments[1].multiplier(), Decimal::new(15, 1));
        assert_eq!(segments[2].multiplier(), Decimal::new(20, 1));
    }

    #[test]
    fn test_pay_calculation() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        let rules = vec![
            mock_rule(
                1,
                Decimal::new(15, 1),
                Some("17:00:00"),
                Some("22:00:00"),
                127,
            ),
            mock_rule(
                10,
                Decimal::new(20, 1),
                Some("22:00:00"),
                Some("06:00:00"),
                127,
            ),
        ];

        let ot_intervals = vec![(
            NaiveTime::from_hms_opt(18, 0, 0).expect("valid time"),
            NaiveTime::from_hms_opt(23, 0, 0).expect("valid time"),
        )];

        let hourly_rate = Decimal::new(100, 0);
        let pay = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            false,
            &ot_intervals,
            hourly_rate,
        );

        assert_eq!(pay, Decimal::new(800, 0));
    }

    #[test]
    fn test_split_interval_by_day() {
        use chrono::TimeZone;
        use chrono_tz::UTC;
        let start = UTC
            .with_ymd_and_hms(2026, 1, 26, 22, 0, 0)
            .single()
            .expect("valid date");
        let end = UTC
            .with_ymd_and_hms(2026, 1, 27, 2, 0, 0)
            .single()
            .expect("valid date");

        let days = OvertimeEngine::split_interval_by_day(&start, &end, &UTC);

        assert_eq!(days.len(), 2);
        assert_eq!(
            days[0].0,
            NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date")
        );
        assert_eq!(
            days[0].1[0].0,
            NaiveTime::from_hms_opt(22, 0, 0).expect("valid time")
        );

        assert_eq!(
            days[1].0,
            NaiveDate::from_ymd_opt(2026, 1, 27).expect("valid date")
        );
        assert_eq!(
            days[1].1[0].0,
            NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is valid")
        );
        assert_eq!(
            days[1].1[0].1,
            NaiveTime::from_hms_opt(2, 0, 0).expect("valid time")
        );
    }

    /// Holiday-specific rule (is_holiday=true) should only activate when is_holiday=true.
    #[test]
    fn test_holiday_rule_filtering() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 1).expect("valid date"); // Thursday
        let normal_rule = mock_rule(1, Decimal::new(15, 1), None, None, 127); // 1.5x all week
        let mut holiday_rule = mock_rule(10, Decimal::new(30, 1), None, None, 127); // 3.0x holiday
        holiday_rule.set_is_holiday(true);

        let rules = vec![normal_rule, holiday_rule];

        // Non-holiday: only normal rule should produce segments (holiday rule excluded)
        let segments_normal = OvertimeEngine::calculate_segments(&rules, date, false);
        assert!(!segments_normal.is_empty());
        for seg in &segments_normal {
            assert_eq!(
                seg.multiplier(),
                Decimal::new(15, 1),
                "Non-holiday should use 1.5x"
            );
        }

        // Holiday: holiday rule (priority 10) wins over normal rule (priority 1)
        let segments_holiday = OvertimeEngine::calculate_segments(&rules, date, true);
        assert!(!segments_holiday.is_empty());
        for seg in &segments_holiday {
            assert_eq!(
                seg.multiplier(),
                Decimal::new(30, 1),
                "Holiday should use 3.0x"
            );
        }
    }

    /// calculate_overtime_pay_from_times should use holiday multiplier on holiday
    #[test]
    fn test_holiday_ot_pay_uses_correct_multiplier() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 1).expect("valid date");
        let normal_rule = mock_rule(
            1,
            Decimal::new(15, 1),
            Some("17:00:00"),
            Some("22:00:00"),
            127,
        );
        let mut holiday_rule = mock_rule(
            10,
            Decimal::new(30, 1),
            Some("17:00:00"),
            Some("22:00:00"),
            127,
        );
        holiday_rule.set_is_holiday(true);
        let rules = vec![normal_rule, holiday_rule];

        let ot_intervals = vec![(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(), // 2 hours OT
        )];
        let hourly_rate = Decimal::new(100, 0);

        // Non-holiday: 2h × 100 × 1.5 = 300
        let pay_normal = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            false,
            &ot_intervals,
            hourly_rate,
        );
        assert_eq!(pay_normal, Decimal::new(300, 0));

        // Holiday: 2h × 100 × 3.0 = 600
        let pay_holiday = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            true,
            &ot_intervals,
            hourly_rate,
        );
        assert_eq!(pay_holiday, Decimal::new(600, 0));
    }

    #[test]
    fn test_no_rules_returns_empty_segments() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        let segments = OvertimeEngine::calculate_segments(&[], date, false);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_no_matching_day_returns_empty() {
        // Rule only applies to Saturday (day_mask = 1 << 5 = 32)
        let rule = mock_rule(1, Decimal::new(20, 1), None, None, 32);
        // Monday
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        let segments = OvertimeEngine::calculate_segments(&[rule], date, false);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_multiple_ot_intervals() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        let rules = vec![mock_rule(1, Decimal::new(15, 1), None, None, 127)];

        // Two separate intervals: 1h evening + 1h night
        let intervals = vec![
            (
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            ),
            (
                NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
            ),
        ];
        let hourly_rate = Decimal::new(100, 0);
        let pay = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            false,
            &intervals,
            hourly_rate,
        );
        // 2h total × 100 × 1.5 = 300
        assert_eq!(pay, Decimal::new(300, 0));
    }

    #[test]
    fn test_zero_hourly_rate() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        let rules = vec![mock_rule(1, Decimal::new(15, 1), None, None, 127)];
        let intervals = vec![(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
        )];
        let pay = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            false,
            &intervals,
            Decimal::ZERO,
        );
        assert_eq!(pay, Decimal::ZERO);
    }

    // =====================================================================
    // Additional OT engine edge cases
    // =====================================================================

    #[test]
    fn test_empty_ot_intervals_returns_zero() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        let rules = vec![mock_rule(1, Decimal::new(15, 1), None, None, 127)];
        let pay = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            false,
            &[],
            Decimal::new(100, 0),
        );
        assert_eq!(pay, Decimal::ZERO);
    }

    #[test]
    fn test_ot_interval_outside_all_segments() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        // Rule only covers 17:00-22:00
        let rules = vec![mock_rule(
            1,
            Decimal::new(15, 1),
            Some("17:00:00"),
            Some("22:00:00"),
            127,
        )];
        // OT interval is 08:00-09:00 — outside rule coverage
        let intervals = vec![(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
        )];
        let pay = OvertimeEngine::calculate_overtime_pay_from_times(
            &rules,
            date,
            false,
            &intervals,
            Decimal::new(100, 0),
        );
        assert_eq!(pay, Decimal::ZERO);
    }

    #[test]
    fn test_split_interval_start_equals_end() {
        use chrono::TimeZone;
        use chrono_tz::UTC;
        let start = UTC
            .with_ymd_and_hms(2026, 1, 26, 10, 0, 0)
            .single()
            .unwrap();
        let days = OvertimeEngine::split_interval_by_day(&start, &start, &UTC);
        assert!(days.is_empty(), "start == end should yield empty result");
    }

    #[test]
    fn test_split_interval_start_after_end() {
        use chrono::TimeZone;
        use chrono_tz::UTC;
        let start = UTC
            .with_ymd_and_hms(2026, 1, 27, 10, 0, 0)
            .single()
            .unwrap();
        let end = UTC
            .with_ymd_and_hms(2026, 1, 26, 10, 0, 0)
            .single()
            .unwrap();
        let days = OvertimeEngine::split_interval_by_day(&start, &end, &UTC);
        assert!(days.is_empty(), "start > end should yield empty result");
    }

    #[test]
    fn test_split_interval_same_day() {
        use chrono::TimeZone;
        use chrono_tz::UTC;
        let start = UTC
            .with_ymd_and_hms(2026, 1, 26, 10, 0, 0)
            .single()
            .unwrap();
        let end = UTC
            .with_ymd_and_hms(2026, 1, 26, 15, 0, 0)
            .single()
            .unwrap();
        let days = OvertimeEngine::split_interval_by_day(&start, &end, &UTC);
        assert_eq!(days.len(), 1);
        assert_eq!(days[0].1.len(), 1);
        assert_eq!(days[0].1[0].0, NaiveTime::from_hms_opt(10, 0, 0).unwrap());
        assert_eq!(days[0].1[0].1, NaiveTime::from_hms_opt(15, 0, 0).unwrap());
    }

    #[test]
    fn test_split_interval_three_days() {
        use chrono::TimeZone;
        use chrono_tz::UTC;
        let start = UTC
            .with_ymd_and_hms(2026, 1, 25, 20, 0, 0)
            .single()
            .unwrap();
        let end = UTC.with_ymd_and_hms(2026, 1, 27, 4, 0, 0).single().unwrap();
        let days = OvertimeEngine::split_interval_by_day(&start, &end, &UTC);
        assert_eq!(days.len(), 3);
        // Day 1: Jan 25, 20:00 → 23:59:59
        assert_eq!(days[0].0, NaiveDate::from_ymd_opt(2026, 1, 25).unwrap());
        // Day 2: Jan 26, 00:00 → 23:59:59
        assert_eq!(days[1].0, NaiveDate::from_ymd_opt(2026, 1, 26).unwrap());
        // Day 3: Jan 27, 00:00 → 04:00
        assert_eq!(days[2].0, NaiveDate::from_ymd_opt(2026, 1, 27).unwrap());
    }

    #[test]
    fn test_segment_merging() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 26).expect("valid date");
        // Two rules with same multiplier covering adjacent times should merge
        let rules = vec![
            mock_rule(
                1,
                Decimal::new(15, 1),
                Some("06:00:00"),
                Some("12:00:00"),
                127,
            ),
            mock_rule(
                1,
                Decimal::new(15, 1),
                Some("12:00:00"),
                Some("18:00:00"),
                127,
            ),
        ];
        let segments = OvertimeEngine::calculate_segments(&rules, date, false);
        // Should merge into one segment since multiplier is the same
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].multiplier(), Decimal::new(15, 1));
    }
}
