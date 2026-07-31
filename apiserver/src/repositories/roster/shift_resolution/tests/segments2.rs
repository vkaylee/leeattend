//! Decomposed from shift_resolution.rs
use super::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;

    #[test]
    fn test_interval_expansion_two_intervals_produces_two_segments() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_1_id = Uuid::new_v4();
        let interval_2_id = Uuid::new_v4();

        let shift = RosterShift::new("Split Shift".to_string(), Some("SS".to_string()));

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(shift_id, shift);
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
                    interval_id: interval_1_id,
                    sort_order: 0,
                    interval_name: Some("Morning".to_string()),
                    start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                    end_time: chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                    break_duration_minutes: 15,
                    break_start: None,
                    break_end: None,
                    is_break_auto_deduct: false,
                    break_is_overtime: false,
                    early_punch_in_grace_period_minutes: 0,
                    allow_early_ot: false,
                    max_early_ot_minutes: None,
                    ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                    is_night_shift: false,
                },
                RosterInterval {
                    interval_id: interval_2_id,
                    sort_order: 1,
                    interval_name: Some("Afternoon".to_string()),
                    start_time: chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
                    end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                    break_duration_minutes: 0,
                    break_start: None,
                    break_end: None,
                    is_break_auto_deduct: false,
                    break_is_overtime: false,
                    early_punch_in_grace_period_minutes: 0,
                    allow_early_ot: false,
                    max_early_ot_minutes: None,
                    ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                    is_night_shift: false,
                },
            ],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 2, "2 intervals → 2 segments");
        // Morning segment
        assert_eq!(segments[0].interval_id, Some(interval_1_id));
        assert_eq!(segments[0].interval_name.as_deref(), Some("Morning"));
        assert_eq!(
            segments[0].scheduled_start,
            chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap()
        );
        assert_eq!(
            segments[0].scheduled_end,
            chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()
        );
        assert_eq!(segments[0].break_duration_minutes, Some(15));
        assert_eq!(segments[0].sort_order, 0);

        // Afternoon segment
        assert_eq!(segments[1].interval_id, Some(interval_2_id));
        assert_eq!(segments[1].interval_name.as_deref(), Some("Afternoon"));
        assert_eq!(
            segments[1].scheduled_start,
            chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap()
        );
        assert_eq!(
            segments[1].scheduled_end,
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap()
        );
        // Break is 0 (no fallback — interval is sole source)
        assert_eq!(segments[1].break_duration_minutes, Some(0));
        assert_eq!(segments[1].sort_order, 1);

        // Both inherit shift metadata
        assert_eq!(segments[0].template_id, shift_id);
        assert_eq!(segments[1].template_id, shift_id);
        assert_eq!(segments[0].shift_name.as_deref(), Some("Split Shift"));
        assert_eq!(segments[1].shift_name.as_deref(), Some("Split Shift"));
        assert_eq!(segments[0].segment_type, "work");
    }

    #[test]
    fn test_interval_break_config_is_used_as_is() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();

        // Shift only provides identity — no time/break/policy data
        let shift = RosterShift::new("With Break".to_string(), None);

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(shift_id, shift);
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );
        // Interval is the sole source for break/policy config
        data.shift_intervals.insert(
            shift_id,
            vec![RosterInterval {
                interval_id,
                sort_order: 0,
                interval_name: None,
                start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 30,
                break_start: Some(chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap()),
                break_end: Some(chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                is_break_auto_deduct: true,
                break_is_overtime: true,
                early_punch_in_grace_period_minutes: 15,
                allow_early_ot: true,
                max_early_ot_minutes: Some(120),
                ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::PerSegment,
                is_night_shift: false,
            }],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 1);
        let seg = &segments[0];
        // Interval values are used as-is — no override/fallback logic
        assert_eq!(seg.break_duration_minutes, Some(30));
        assert_eq!(
            seg.break_start,
            Some(chrono::NaiveTime::from_hms_opt(11, 30, 0).unwrap())
        );
        assert_eq!(
            seg.break_end,
            Some(chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap())
        );
        assert_eq!(seg.is_break_auto_deduct, Some(true));
        assert_eq!(seg.break_is_overtime, Some(true));
        assert!(seg.allow_early_ot);
        assert_eq!(seg.max_early_ot_minutes, Some(120));
        assert_eq!(seg.early_punch_in_grace_period_minutes, 15);
        assert_eq!(seg.ot_aggregation_mode, "per_segment");
        assert_eq!(seg.interval_id, Some(interval_id));
    }

    #[test]
    fn test_enriched_segment_fields_populated() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let interval_id = Uuid::new_v4();
        let test_date = date(2026, 6, 15);

        let shift = RosterShift::new("Enriched Test".to_string(), Some("ET".to_string()));

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(shift_id, shift);
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
                interval_name: Some("Early".to_string()),
                start_time: chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
                break_duration_minutes: 0,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 5,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                is_night_shift: false,
            }],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, test_date);

        assert_eq!(segments.len(), 1);
        let seg = &segments[0];
        // Enriched fields from ResolvedSegment
        assert_eq!(seg.template_id, shift_id);
        assert_eq!(seg.shift_name.as_deref(), Some("Enriched Test"));
        assert_eq!(seg.shift_symbol.as_deref(), Some("ET"));
        assert_eq!(seg.interval_name.as_deref(), Some("Early"));
        assert_eq!(seg.segment_type, "work");
        assert_eq!(seg.date, Some(test_date));
        // Holiday fields default to false/None (populated later by caller)
        assert!(!seg.is_holiday);
        assert!(!seg.is_holiday_paid);
        assert!(seg.holiday_name.is_none());
    }

    #[test]
    fn test_interval_from_different_shift_not_leaked() {
        // Regression: intervals should only expand the shift they belong to.
        // Shift A has intervals, but Shift B is the one assigned.
        let emp = Uuid::new_v4();
        let shift_a = Uuid::new_v4();
        let shift_b = Uuid::new_v4();
        let interval_a = Uuid::new_v4();
        let interval_b = Uuid::new_v4();

        let (sa_id, shift_a_data) = make_test_shift(shift_a, "Shift A");
        let (sb_id, shift_b_data) = make_test_shift(shift_b, "Shift B");

        let mut data = (*empty_roster_data()).clone();
        data.shifts.insert(sa_id, shift_a_data);
        data.shifts.insert(sb_id, shift_b_data);
        // Only Shift A has intervals defined — must NOT leak to Shift B
        data.shift_intervals.insert(
            shift_a,
            vec![RosterInterval {
                interval_id: interval_a,
                sort_order: 0,
                interval_name: Some("A-Morning".to_string()),
                start_time: chrono::NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                break_duration_minutes: 0,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                is_night_shift: false,
            }],
        );
        // Also give Shift B its own interval — the point is it should NOT get shift A's interval
        data.shift_intervals.insert(
            shift_b,
            vec![RosterInterval {
                interval_id: interval_b,
                sort_order: 0,
                interval_name: Some("B-Afternoon".to_string()),
                start_time: chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
                end_time: chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
                break_duration_minutes: 0,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: false,
                break_is_overtime: false,
                early_punch_in_grace_period_minutes: 0,
                allow_early_ot: false,
                max_early_ot_minutes: None,
                ot_aggregation_mode: crate::services::attendance_calculator::OtAggregationMode::Aggregate,
                is_night_shift: false,
            }],
        );
        // Fixed assignment gives Shift B
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id: shift_b,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );

        let data = Arc::new(data);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        // Shift B is assigned — should get B's interval, NOT A's
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].template_id, shift_b);
        assert_eq!(segments[0].interval_id, Some(interval_b));
        assert_eq!(segments[0].interval_name.as_deref(), Some("B-Afternoon"));
        assert_eq!(
            segments[0].scheduled_start,
            chrono::NaiveTime::from_hms_opt(13, 0, 0).unwrap()
        );
    }

    // ─── Post-refactor: single interval = single segment (no split) ──────

