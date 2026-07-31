//! Decomposed from shift_resolution.rs
use super::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;

    #[test]
    fn test_single_interval_produces_one_segment_no_split() {
        // Regression: a shift with ONE interval should produce exactly ONE segment.
        // This is the fix for the original bug where shifts without mid-shift
        // attendance requirement still got split into 2 sessions in the report.
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Standard Day");

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(sid, shift);
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );
        // Single interval spanning the full shift
        data.shift_intervals.insert(
            shift_id,
            vec![RosterInterval {
                interval_id,
                sort_order: 0,
                interval_name: Some("Full Day".to_string()),
                start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 60,
                break_start: Some(chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                break_end: Some(chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap()),
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 15,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                is_night_shift: false,
            }],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 6, 15));

        // Single interval → exactly 1 segment — NO SPLIT
        assert_eq!(
            segments.len(),
            1,
            "single interval should produce 1 segment, not split"
        );
        let seg = &segments[0];
        assert_eq!(seg.template_id, shift_id);
        assert_eq!(seg.interval_id, Some(interval_id));
        assert_eq!(
            seg.scheduled_start,
            chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            seg.scheduled_end,
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        );
        assert_eq!(seg.break_duration_minutes, Some(60));
        assert_eq!(seg.is_break_auto_deduct, Some(true));
    }

    #[test]
    fn test_two_intervals_produce_two_segments_with_independent_config() {
        // Two intervals = 2 segments, each with its own break config
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let iv_morning = Uuid::new_v4();
        let iv_afternoon = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Split Day");

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(sid, shift);
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );
        data.shift_intervals.insert(
            shift_id,
            vec![
                RosterInterval {
                    interval_id: iv_morning,
                    sort_order: 0,
                    interval_name: Some("Morning".to_string()),
                    start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                    end_time: chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                    break_duration_minutes: 15,
                    break_start: None,
                    break_end: None,
                    is_break_auto_deduct: true,
                    break_is_overtime: false,
                    early_punch_in_grace_period_minutes: 10,
                    allow_early_ot: false,
                    max_early_ot_minutes: None,
                    ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                    is_night_shift: false,
                },
                RosterInterval {
                    interval_id: iv_afternoon,
                    sort_order: 1,
                    interval_name: Some("Afternoon".to_string()),
                    start_time: chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
                    end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                    break_duration_minutes: 0,
                    break_start: None,
                    break_end: None,
                    is_break_auto_deduct: false,
                    break_is_overtime: false,
                    early_punch_in_grace_period_minutes: 5,
                    allow_early_ot: true,
                    max_early_ot_minutes: Some(30),
                    ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::PerSegment,
                    is_night_shift: false,
                },
            ],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 6, 15));

        assert_eq!(segments.len(), 2, "two intervals should produce 2 segments");
        // Morning
        assert_eq!(segments[0].interval_name.as_deref(), Some("Morning"));
        assert_eq!(segments[0].break_duration_minutes, Some(15));
        assert_eq!(segments[0].is_break_auto_deduct, Some(true));
        assert_eq!(segments[0].early_punch_in_grace_period_minutes, 10);
        // Afternoon — independent config, no fallback
        assert_eq!(segments[1].interval_name.as_deref(), Some("Afternoon"));
        assert_eq!(segments[1].break_duration_minutes, Some(0));
        assert_eq!(segments[1].is_break_auto_deduct, Some(false));
        assert_eq!(segments[1].early_punch_in_grace_period_minutes, 5);
        assert!(segments[1].allow_early_ot);
        assert_eq!(segments[1].ot_aggregation_mode, "per_segment");
    }

    #[test]
    fn test_night_shift_auto_derived_from_interval_times() {
        // is_night_shift is now a property of the interval, not hardcoded.
        // When start_time > end_time, it's a night shift.
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Night");

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(sid, shift);
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );
        data.shift_intervals.insert(
            shift_id,
            vec![RosterInterval {
                interval_id,
                sort_order: 0,
                interval_name: Some("Night Block".to_string()),
                start_time: chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
                break_duration_minutes: 30,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: true,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 15,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                is_night_shift: true, // explicitly night
            }],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 6, 15));

        assert_eq!(segments.len(), 1);
        assert!(segments[0].is_night_shift, "night shift interval");
        assert_eq!(
            segments[0].scheduled_start,
            chrono::NaiveTime::from_hms_opt(22, 0, 0).unwrap()
        );
        assert_eq!(
            segments[0].scheduled_end,
            chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_no_shift_assigned_produces_zero_segments() {
        // No assignment → no shift → no intervals → 0 segments (unchanged behavior)
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Unassigned");
        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));
        assert_eq!(segments.len(), 0, "no assignment → 0 segments");
    }

    #[test]
    fn test_interval_without_shift_in_roster_data_is_skipped() {
        // If a shift_id resolves but the shift isn't in roster_data.shifts, skip it
        let emp = Uuid::new_v4();
        let missing_shift_id = Uuid::new_v4();

        let data = empty_roster_data(); // no shifts at all
        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(missing_shift_id));

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));
        assert_eq!(segments.len(), 0, "missing shift in roster_data → skipped");
    }
