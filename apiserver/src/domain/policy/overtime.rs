use crate::models::OvertimeRule;
use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Timelike};
use chrono_tz::Tz;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct OvertimeSegment {
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub multiplier: Decimal,
}

pub struct OvertimeEngine;

impl OvertimeEngine {
    pub fn calculate_segments(
        rules: &[OvertimeRule],
        date: NaiveDate,
        is_holiday: bool,
    ) -> Vec<OvertimeSegment> {
        let weekday = date.weekday();
        let day_mask = 1 << (weekday.num_days_from_monday() as i32);

        let active_rules: Vec<&OvertimeRule> = rules
            .iter()
            .filter(|r| {
                let day_matches = (r.days_of_week & day_mask) != 0;
                let holiday_matches = !r.is_holiday || is_holiday;
                day_matches && holiday_matches
            })
            .collect();

        if active_rules.is_empty() {
            return vec![];
        }

        let mut time_points = std::collections::BTreeSet::new();
        time_points.insert(NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is valid"));
        time_points.insert(NaiveTime::from_hms_opt(23, 59, 59).expect("23:59:59 is valid"));

        for rule in &active_rules {
            if let Some(start) = rule.time_start {
                time_points.insert(start);
            }
            if let Some(end) = rule.time_end {
                time_points.insert(end);
            }
        }

        let sorted_points: Vec<NaiveTime> = time_points.into_iter().collect();
        let mut segments = Vec::new();

        for i in 0..sorted_points.len() - 1 {
            let start = sorted_points[i];
            let end = sorted_points[i + 1];

            let mid_secs =
                (start.num_seconds_from_midnight() + end.num_seconds_from_midnight()) / 2;
            let mid_time = NaiveTime::from_num_seconds_from_midnight_opt(mid_secs, 0)
                .expect("midpoint time is valid");

            let best_rule = active_rules
                .iter()
                .filter(|r| {
                    let r_start = r
                        .time_start
                        .unwrap_or(NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is valid"));
                    let r_end = r
                        .time_end
                        .unwrap_or(NaiveTime::from_hms_opt(23, 59, 59).expect("23:59:59 is valid"));

                    if r_start <= r_end {
                        mid_time >= r_start && mid_time < r_end
                    } else {
                        mid_time >= r_start || mid_time < r_end
                    }
                })
                .max_by_key(|r| r.priority);

            if let Some(rule) = best_rule {
                segments.push(OvertimeSegment {
                    start,
                    end,
                    multiplier: rule.multiplier,
                });
            }
        }

        if segments.is_empty() {
            return vec![];
        }

        let mut merged = Vec::new();
        let mut current = segments[0].clone();

        for next in segments.into_iter().skip(1) {
            if next.multiplier == current.multiplier {
                current.end = next.end;
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
                let inter_start = NaiveTime::max(*ot_start, segment.start);
                let inter_end = NaiveTime::min(*ot_end, segment.end);

                if inter_end > inter_start {
                    let duration_mins = (inter_end.num_seconds_from_midnight() as i32
                        - inter_start.num_seconds_from_midnight() as i32)
                        / 60;
                    let duration_hours = Decimal::from(duration_mins) / Decimal::from(60);
                    total_pay += duration_hours * hourly_rate * segment.multiplier;
                }
            }
        }

        total_pay
    }

    /// Splits a DateTime interval into daily NaiveTime segments for calculation
    pub fn split_interval_by_day<T: TimeZone>(
        start: DateTime<T>,
        end: DateTime<T>,
        tz: &Tz,
    ) -> Vec<(NaiveDate, Vec<(NaiveTime, NaiveTime)>)> {
        let mut result = Vec::new();
        if start >= end {
            return result;
        }

        let mut current = start.with_timezone(tz);
        let end_tz = end.with_timezone(tz);

        while current < end_tz {
            let date = current.date_naive();
            // 23:59:59 is always valid; DST can make from_local ambiguous but we pick first
            let day_end =
                match tz.from_local_datetime(&date.and_hms_opt(23, 59, 59).expect("valid time")) {
                    chrono::LocalResult::Single(dt) => dt,
                    chrono::LocalResult::Ambiguous(dt, _) => dt,
                    chrono::LocalResult::None => {
                        // Rare: DST gap at 23:59:59 - use 22:59:59 + 1 hour
                        let shifted = date.and_hms_opt(22, 59, 59).expect("valid time")
                            + chrono::Duration::hours(1);
                        tz.from_local_datetime(&shifted).single().unwrap_or(current)
                    }
                };

            let segment_end = if day_end < end_tz {
                // We add 1 second to day_end to reach the next day boundary roughly
                // But NaiveTime max is better
                day_end
            } else {
                end_tz
            };

            let mut daily_intervals = Vec::new();

            // Special case for NaiveTime max
            let s_time = current.time();
            let e_time = if segment_end.date_naive() > date {
                NaiveTime::from_hms_opt(23, 59, 59).expect("23:59:59 is valid")
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
            // Move to next day
            let Some(next_date) = date.succ_opt() else {
                break;
            };
            current = match tz
                .from_local_datetime(&next_date.and_hms_opt(0, 0, 0).expect("midnight valid"))
            {
                chrono::LocalResult::Single(dt) => dt,
                chrono::LocalResult::Ambiguous(dt, _) => dt,
                chrono::LocalResult::None => {
                    // DST gap at midnight - advance by 1 hour
                    let shifted = next_date.and_hms_opt(1, 0, 0).expect("1am valid");
                    tz.from_local_datetime(&shifted)
                        .single()
                        .unwrap_or(segment_end)
                }
            };
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use uuid::Uuid;

    fn mock_rule(
        priority: i32,
        multiplier: Decimal,
        start: Option<&str>,
        end: Option<&str>,
        days: i32,
    ) -> OvertimeRule {
        OvertimeRule {
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
        }
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
        assert_eq!(segments[0].multiplier, Decimal::new(20, 1));
        assert_eq!(segments[1].multiplier, Decimal::new(15, 1));
        assert_eq!(segments[2].multiplier, Decimal::new(20, 1));
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

        let days = OvertimeEngine::split_interval_by_day(start, end, &UTC);

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
        holiday_rule.is_holiday = true;

        let rules = vec![normal_rule, holiday_rule];

        // Non-holiday: only normal rule should produce segments (holiday rule excluded)
        let segments_normal = OvertimeEngine::calculate_segments(&rules, date, false);
        assert!(!segments_normal.is_empty());
        for seg in &segments_normal {
            assert_eq!(seg.multiplier, Decimal::new(15, 1), "Non-holiday should use 1.5x");
        }

        // Holiday: holiday rule (priority 10) wins over normal rule (priority 1)
        let segments_holiday = OvertimeEngine::calculate_segments(&rules, date, true);
        assert!(!segments_holiday.is_empty());
        for seg in &segments_holiday {
            assert_eq!(seg.multiplier, Decimal::new(30, 1), "Holiday should use 3.0x");
        }
    }

    /// calculate_overtime_pay_from_times should use holiday multiplier on holiday
    #[test]
    fn test_holiday_ot_pay_uses_correct_multiplier() {
        let date = NaiveDate::from_ymd_opt(2026, 5, 1).expect("valid date");
        let normal_rule = mock_rule(1, Decimal::new(15, 1), Some("17:00:00"), Some("22:00:00"), 127);
        let mut holiday_rule = mock_rule(10, Decimal::new(30, 1), Some("17:00:00"), Some("22:00:00"), 127);
        holiday_rule.is_holiday = true;
        let rules = vec![normal_rule, holiday_rule];

        let ot_intervals = vec![(
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(), // 2 hours OT
        )];
        let hourly_rate = Decimal::new(100, 0);

        // Non-holiday: 2h × 100 × 1.5 = 300
        let pay_normal = OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, false, &ot_intervals, hourly_rate);
        assert_eq!(pay_normal, Decimal::new(300, 0));

        // Holiday: 2h × 100 × 3.0 = 600
        let pay_holiday = OvertimeEngine::calculate_overtime_pay_from_times(&rules, date, true, &ot_intervals, hourly_rate);
        assert_eq!(pay_holiday, Decimal::new(600, 0));
    }
}
