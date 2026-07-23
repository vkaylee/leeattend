//! Decomposed from shift_resolution.rs
use super::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;

    #[test]
    fn test_resolve_single_segment_backward_compat() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Morning");

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let roster_map = HashMap::new();

        // No assignments → falls through to none → 0 segments
        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));
        assert_eq!(segments.len(), 0, "No shift assigned → 0 segments");

        // Now add a fixed assignment and an interval (intervals are the sole source of segment data)
        let mut data2 = (*data).clone();
        data2.shift_intervals.insert(
            shift_id,
            vec![make_simple_interval(
                interval_id,
                NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            )],
        );
        data2.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );
        let data2 = Arc::new(data2);

        let segments = resolve_work_segments_wrapper(&data2, &roster_map, emp, date(2026, 3, 3));
        assert_eq!(segments.len(), 1, "Fixed assignment → 1 segment");
        assert_eq!(segments[0].shift_id, shift_id);
        assert_eq!(segments[0].source, "fixed");
        assert_eq!(
            segments[0].scheduled_start,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            segments[0].scheduled_end,
            NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_resolve_split_shift_two_segments() {
        let emp = Uuid::new_v4();
        let morning_id = Uuid::new_v4();
        let evening_id = Uuid::new_v4();
        let morning_iv_id = Uuid::new_v4();
        let evening_iv_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (m_id, morning) = make_test_shift(morning_id, "Ca Sáng");
        let (e_id, evening) = make_test_shift(evening_id, "Ca Chiều");

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(m_id, morning), (e_id, evening)].into_iter().collect();
        data.shift_intervals.insert(
            morning_id,
            vec![make_simple_interval(
                morning_iv_id,
                NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(11, 0, 0).unwrap(),
            )],
        );
        data.shift_intervals.insert(
            evening_id,
            vec![make_simple_interval(
                evening_iv_id,
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(21, 0, 0).unwrap(),
            )],
        );
        data.pattern_assignments.insert(
            emp,
            vec![
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(morning_id)),
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(evening_id)),
            ],
        );
        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, target_date);

        assert_eq!(segments.len(), 2, "Split shift → 2 segments");
        assert_eq!(segments[0].shift_id, morning_id);
        assert_eq!(segments[1].shift_id, evening_id);
        assert_eq!(segments[0].source, "pattern");
        assert_eq!(segments[1].source, "pattern");
        // Segments must be sorted by scheduled_start
        assert!(
            segments[0].scheduled_start < segments[1].scheduled_start,
            "Morning (07:00) should come before evening (17:00)"
        );
        assert_eq!(segments[0].sort_order, 0);
        assert_eq!(segments[1].sort_order, 1);
    }

    #[test]
    fn test_roster_override_produces_segment() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Roster Shift");

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(sid, shift)].into_iter().collect();
        data.shift_intervals.insert(
            shift_id,
            vec![make_simple_interval(
                interval_id,
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
            )],
        );
        let data = Arc::new(data);
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(shift_id));

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 1, "Roster override → 1 segment");
        assert_eq!(segments[0].shift_id, shift_id);
        assert_eq!(segments[0].source, "roster");
    }

    #[test]
    fn test_day_off_produces_zero_segments() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Some Shift");

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), None); // Day off

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 0, "Day off → 0 segments");
    }

    #[test]
    fn test_no_match_produces_zero_segments() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Orphan Shift");

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 0, "No match → 0 segments");
    }

    #[test]
    fn test_segment_inherits_interval_config() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let shift = RosterShift::new("Config Test".to_string(), Some("CT".to_string()));

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(shift_id, shift);
        data.shift_intervals.insert(
            shift_id,
            vec![RosterInterval {
                interval_id,
                sort_order: 0,
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
                break_duration_minutes: 45,
                break_start: Some(NaiveTime::from_hms_opt(11, 0, 0).unwrap()),
                break_end: Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 10,
                allow_early_ot: true,
                max_early_ot_minutes: Some(120),
                ot_aggregation_mode: "per_segment".to_string(),
                is_night_shift: false,
            }],
        );
        let data = Arc::new(data);
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(shift_id));

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 1);
        let seg = &segments[0];
        assert_eq!(seg.break_duration_minutes, Some(45));
        assert_eq!(
            seg.break_start,
            Some(NaiveTime::from_hms_opt(11, 0, 0).unwrap())
        );
        assert_eq!(
            seg.break_end,
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap())
        );
        assert_eq!(seg.is_break_auto_deduct, Some(true));
        assert!(seg.allow_early_ot);
        assert_eq!(seg.max_early_ot_minutes, Some(120));
        assert_eq!(seg.early_punch_in_grace_period_minutes, 10);
        assert_eq!(seg.ot_aggregation_mode, "per_segment");
        assert!(!seg.is_night_shift);
    }

    #[test]
    fn test_night_shift_segment() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let (sid, shift) = make_night_shift(shift_id, "Ca Đêm");

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(sid, shift);
        data.shift_intervals.insert(
            shift_id,
            vec![RosterInterval {
                interval_id,
                sort_order: 0,
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
                break_duration_minutes: 30,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: true,
            }],
        );
        let data = Arc::new(data);
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(shift_id));

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 1, "Night shift → 1 segment");
        let seg = &segments[0];
        assert!(seg.is_night_shift, "Night shift flag must be true");
        assert_eq!(
            seg.scheduled_start,
            NaiveTime::from_hms_opt(22, 0, 0).unwrap()
        );
        assert_eq!(seg.scheduled_end, NaiveTime::from_hms_opt(6, 0, 0).unwrap());
    }

    #[test]
    fn test_three_segment_split_shift() {
        let emp = Uuid::new_v4();
        let s1_id = Uuid::new_v4();
        let s2_id = Uuid::new_v4();
        let s3_id = Uuid::new_v4();
        let iv1_id = Uuid::new_v4();
        let iv2_id = Uuid::new_v4();
        let iv3_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (id1, s1) = make_test_shift(s1_id, "Sáng");
        let (id2, s2) = make_test_shift(s2_id, "Chiều");
        let (id3, s3) = make_test_shift(s3_id, "Tối");

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(id1, s1), (id2, s2), (id3, s3)].into_iter().collect();
        data.shift_intervals.insert(
            s1_id,
            vec![make_simple_interval(
                iv1_id,
                NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            )],
        );
        data.shift_intervals.insert(
            s2_id,
            vec![make_simple_interval(
                iv2_id,
                NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
            )],
        );
        data.shift_intervals.insert(
            s3_id,
            vec![make_simple_interval(
                iv3_id,
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            )],
        );
        data.pattern_assignments.insert(
            emp,
            vec![
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(s1_id)),
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(s2_id)),
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(s3_id)),
            ],
        );
        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, target_date);

        assert_eq!(segments.len(), 3, "3-segment split shift");
        assert_eq!(segments[0].sort_order, 0);
        assert_eq!(segments[1].sort_order, 1);
        assert_eq!(segments[2].sort_order, 2);
        // Verify ascending order
        assert!(segments[0].scheduled_start < segments[1].scheduled_start);
        assert!(segments[1].scheduled_start < segments[2].scheduled_start);
    }

    #[test]
    fn test_expired_pattern_no_segments() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (sid, shift) = make_test_shift(shift_id, "Expired");
        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let mut data = (*data).clone();
        // Pattern expired before target date
        data.pattern_assignments.insert(
            emp,
            vec![weekly_entry(
                pattern_id,
                date(2026, 1, 1),
                Some(date(2026, 2, 28)),
                dow,
                Some(shift_id),
            )],
        );
        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, target_date);
        assert_eq!(segments.len(), 0, "Expired pattern → no segments");
    }

    #[test]
    fn test_segment_with_null_break_fields() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let shift = RosterShift::new("No Break".to_string(), None);

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(shift_id, shift);
        data.shift_intervals.insert(
            shift_id,
            vec![RosterInterval {
                interval_id,
                sort_order: 0,
                interval_name: None,
                start_time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
                break_duration_minutes: 0,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: "aggregate".to_string(),
                is_night_shift: false,
            }],
        );
        let data = Arc::new(data);
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(shift_id));

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 1);
        let seg = &segments[0];
        assert_eq!(seg.break_duration_minutes, Some(0));
        assert_eq!(seg.break_start, None);
        assert_eq!(seg.break_end, None);
        assert_eq!(seg.is_break_auto_deduct, Some(false));
        assert_eq!(seg.break_is_overtime, Some(false));
    }

    #[test]
    fn test_segment_sort_order_reassigned_after_sort() {
        let emp = Uuid::new_v4();
        let evening_id = Uuid::new_v4();
        let morning_id = Uuid::new_v4();
        let evening_iv_id = Uuid::new_v4();
        let morning_iv_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (e_id, evening) = make_test_shift(evening_id, "Tối");
        let (m_id, morning) = make_test_shift(morning_id, "Sáng");

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(e_id, evening), (m_id, morning)].into_iter().collect();
        data.shift_intervals.insert(
            evening_id,
            vec![make_simple_interval(
                evening_iv_id,
                NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            )],
        );
        data.shift_intervals.insert(
            morning_id,
            vec![make_simple_interval(
                morning_iv_id,
                NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
            )],
        );
        // Intentionally add evening first, then morning — sorting should fix order
        data.pattern_assignments.insert(
            emp,
            vec![
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(evening_id)),
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(morning_id)),
            ],
        );
        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, target_date);

        assert_eq!(segments.len(), 2);
        // After sorting: morning (06:00) should be first
        assert_eq!(
            segments[0].scheduled_start,
            NaiveTime::from_hms_opt(6, 0, 0).unwrap()
        );
        assert_eq!(
            segments[1].scheduled_start,
            NaiveTime::from_hms_opt(18, 0, 0).unwrap()
        );
        assert_eq!(
            segments[0].sort_order, 0,
            "sort_order reassigned after sort"
        );
        assert_eq!(
            segments[1].sort_order, 1,
            "sort_order reassigned after sort"
        );
    }

    // ─── Interval expansion tests ───────────────────────────────────────

    use crate::cache::RosterInterval;

