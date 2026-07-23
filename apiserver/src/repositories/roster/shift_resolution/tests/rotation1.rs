//! Decomposed from shift_resolution.rs
use super::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;

    #[test]
    fn test_p2_rotation_pattern_resolves_correctly() {
        // Cycle = 6 days: Day0,1 = Morning, Day2,3 = Night, Day4,5 = OFF
        let emp = Uuid::new_v4();
        let morning = Uuid::new_v4();
        let night = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let anchor = date(2026, 1, 5); // Monday

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    anchor,
                    0,
                    Some(morning),
                ),
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    anchor,
                    1,
                    Some(morning),
                ),
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    anchor,
                    2,
                    Some(night),
                ),
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    anchor,
                    3,
                    Some(night),
                ),
                rotation_entry(pattern_id, date(2026, 1, 1), None, 6, anchor, 4, None), // OFF
                rotation_entry(pattern_id, date(2026, 1, 1), None, 6, anchor, 5, None), // OFF
            ],
        );

        let roster_map = HashMap::new();

        // Day 0 from anchor (2026-01-05) → Morning
        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 5),
        );
        assert_eq!(shift, vec![Some(morning)]);
        assert_eq!(source, "pattern");

        // Day 2 from anchor (2026-01-07) → Night
        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 7),
        );
        assert_eq!(shift, vec![Some(night)]);
        assert_eq!(source, "pattern");

        // Day 5 from anchor (2026-01-10) → OFF
        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 10),
        );
        assert_eq!(shift, vec![None]);
        assert_eq!(source, "pattern");

        // Day 6 = wrap to Day 0 (2026-01-11) → Morning
        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 11),
        );
        assert_eq!(shift, vec![Some(morning)]);
        assert_eq!(source, "pattern");
    }

    #[test]
    fn test_p2_rotation_with_negative_date_difference() {
        // Date before anchor should still work via rem_euclid
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let anchor = date(2026, 3, 10);

        let mut data = (*empty_roster_data()).clone();
        // Cycle = 4, Day 2 = shift_id
        data.pattern_assignments.insert(
            emp,
            vec![rotation_entry(
                pattern_id,
                date(2026, 1, 1),
                None,
                4,
                anchor,
                2,
                Some(shift_id),
            )],
        );

        let roster_map = HashMap::new();

        // 2026-03-08 is 2 days BEFORE anchor → (−2) rem_euclid(4) = 2 → Day 2 → match!
        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 8),
        );
        assert_eq!(shift, vec![Some(shift_id)]);
        assert_eq!(source, "pattern");
    }

    #[test]
    fn test_p1_roster_overrides_rotation() {
        let emp = Uuid::new_v4();
        let rotation_shift = Uuid::new_v4();
        let roster_shift = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let anchor = date(2026, 1, 1);
        let target_date = date(2026, 1, 1);

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![rotation_entry(
                pattern_id,
                date(2026, 1, 1),
                None,
                4,
                anchor,
                0,
                Some(rotation_shift),
            )],
        );

        let mut roster_map = HashMap::new();
        roster_map.insert((emp, target_date), Some(roster_shift));

        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );
        assert_eq!(shift, vec![Some(roster_shift)], "P1 roster > P2 rotation");
        assert_eq!(source, "roster");
    }

    #[test]
    fn test_weekly_pattern_unchanged_after_rotation_support() {
        // Regression: weekly patterns should work exactly as before
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3); // Tuesday
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![weekly_entry(
                pattern_id,
                date(2026, 1, 1),
                None,
                dow,
                Some(shift_id),
            )],
        );

        let roster_map = HashMap::new();

        let (shift, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );
        assert_eq!(shift, vec![Some(shift_id)]);
        assert_eq!(source, "pattern");
    }

    #[test]
    fn test_rotation_different_anchor_different_result() {
        // Same pattern, same date, different anchor → different cycle_day
        let emp_a = Uuid::new_v4();
        let emp_b = Uuid::new_v4();
        let morning = Uuid::new_v4();
        let night = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 1, 7);

        let mut data = (*empty_roster_data()).clone();
        // Employee A: anchor Jan 5 → cycle_day = (7-5) % 6 = 2 → Night
        data.pattern_assignments.insert(
            emp_a,
            vec![
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    date(2026, 1, 5),
                    0,
                    Some(morning),
                ),
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    date(2026, 1, 5),
                    2,
                    Some(night),
                ),
            ],
        );
        // Employee B: anchor Jan 7 → cycle_day = (7-7) % 6 = 0 → Morning
        data.pattern_assignments.insert(
            emp_b,
            vec![
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    date(2026, 1, 7),
                    0,
                    Some(morning),
                ),
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    6,
                    date(2026, 1, 7),
                    2,
                    Some(night),
                ),
            ],
        );

        let roster_map = HashMap::new();

        let (shift_a, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp_a,
            "EMP001",
            None,
            target_date,
        );
        let (shift_b, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp_b,
            "EMP002",
            None,
            target_date,
        );

        assert_eq!(
            shift_a,
            vec![Some(night)],
            "Employee A on cycle_day 2 → Night"
        );
        assert_eq!(
            shift_b,
            vec![Some(morning)],
            "Employee B on cycle_day 0 → Morning"
        );
    }

    #[test]
    fn test_rotation_cycle_length_one() {
        // Boundary case: cycle_length = 1. Every day should resolve to the same shift.
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let anchor = date(2026, 1, 1);

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![rotation_entry(
                pattern_id,
                date(2026, 1, 1),
                None,
                1,
                anchor,
                0,
                Some(shift_id),
            )],
        );

        let roster_map = HashMap::new();

        // Check different dates, they should all map to Day 0
        for offset in 0..10 {
            let target_date = anchor + chrono::Duration::days(offset);
            let (shift, _, _) = resolve_effective_shift_from_data(
                &Arc::new(data.clone()),
                &roster_map,
                emp,
                "EMP001",
                None,
                target_date,
            );
            assert_eq!(shift, vec![Some(shift_id)]);
        }
    }

