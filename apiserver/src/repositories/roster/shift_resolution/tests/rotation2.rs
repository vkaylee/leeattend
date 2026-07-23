//! Decomposed from shift_resolution.rs
use super::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;

    #[test]
    fn test_expired_rotation_pattern() {
        // If target date is past the end_date, the rotation pattern should not apply.
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let anchor = date(2026, 1, 1);
        let end_date = date(2026, 1, 10);

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![rotation_entry(
                pattern_id,
                date(2026, 1, 1),
                Some(end_date),
                4,
                anchor,
                0,
                Some(shift_id),
            )],
        );

        let roster_map = HashMap::new();

        // Target is before end_date (matches cycle_day 0 on 2026-01-09, which is 8 days after anchor → day 0)
        let (shift_active, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 9),
        );
        assert_eq!(shift_active, vec![Some(shift_id)]);

        // Target is after end_date (2026-01-13 is 12 days after anchor → day 0, but pattern is expired)
        let (shift_expired, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 13),
        );
        assert_eq!(shift_expired, vec![None]); // falls through to none
    }

    #[test]
    fn test_rotation_split_shift() {
        // Multiple entries for the same cycle day (e.g. morning + evening shifts on the same day)
        let emp = Uuid::new_v4();
        let morning = Uuid::new_v4();
        let evening = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let anchor = date(2026, 1, 1);

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    3,
                    anchor,
                    0,
                    Some(morning),
                ),
                rotation_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    3,
                    anchor,
                    0,
                    Some(evening),
                ),
                rotation_entry(pattern_id, date(2026, 1, 1), None, 3, anchor, 1, None),
                rotation_entry(pattern_id, date(2026, 1, 1), None, 3, anchor, 2, None),
            ],
        );

        let roster_map = HashMap::new();

        // 2026-01-04 is 3 days after anchor → Day 0 → should return both shifts
        let (shifts, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 1, 4),
        );
        assert_eq!(shifts.len(), 2);
        assert!(shifts.contains(&Some(morning)));
        assert!(shifts.contains(&Some(evening)));
    }

    #[test]
    fn test_rotation_anchor_date_fallback() {
        // If anchor_date is None, should fallback to start_date
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let start_date = date(2026, 1, 5); // Monday

        let mut data = (*empty_roster_data()).clone();
        let mut entry = rotation_entry(
            pattern_id,
            start_date,
            None,
            4,
            start_date, // will be ignored as we set it to None below
            0,
            Some(shift_id),
        );
        entry.anchor_date = None; // Force anchor to None

        data.pattern_assignments.insert(emp, vec![entry]);

        let roster_map = HashMap::new();

        // Target date matches start_date (which acts as anchor_date) → cycle_day = (5 - 5) % 4 = 0 → Matches!
        let (shift, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            start_date,
        );
        assert_eq!(shift, vec![Some(shift_id)]);
    }

    #[test]
    fn test_rotation_far_future_date() {
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
                5,
                anchor,
                2,
                Some(shift_id),
            )],
        );

        let roster_map = HashMap::new();

        // 2036-01-01 is 10 years later (leap years included). Let's calculate day difference
        let target_date = date(2036, 1, 1);
        let days_diff = (target_date - anchor).num_days();
        let cycle_day = days_diff.rem_euclid(5) as i32;

        let (shift, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        if cycle_day == 2 {
            assert_eq!(shift, vec![Some(shift_id)]);
        } else {
            assert_eq!(shift, vec![None]);
        }
    }

    #[test]
    fn test_coexistence_rotation_and_weekly_patterns() {
        // If an employee has both an active weekly pattern and active rotation pattern at the same time,
        // it should collect both matching shifts.
        let emp = Uuid::new_v4();
        let shift_rotation = Uuid::new_v4();
        let shift_weekly = Uuid::new_v4();
        let pattern_rot_id = Uuid::new_v4();
        let pattern_wk_id = Uuid::new_v4();

        let target_date = date(2026, 1, 5); // Monday
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.pattern_assignments.insert(
            emp,
            vec![
                rotation_entry(
                    pattern_rot_id,
                    date(2026, 1, 1),
                    None,
                    4,
                    date(2026, 1, 1),
                    0, // 2026-01-05 is 4 days after anchor → Day 0 → matches
                    Some(shift_rotation),
                ),
                weekly_entry(
                    pattern_wk_id,
                    date(2026, 1, 1),
                    None,
                    dow, // Monday → matches
                    Some(shift_weekly),
                ),
            ],
        );

        let roster_map = HashMap::new();

        let (shifts, _, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        // Verify that shifts from both patterns are returned
        assert_eq!(shifts.len(), 2);
        assert!(shifts.contains(&Some(shift_rotation)));
        assert!(shifts.contains(&Some(shift_weekly)));
    }

    #[test]
    fn test_p3_rule_with_location_and_employer() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let rule_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3); // Tuesday (dow = 2)
        let dow = 2;

        let loc_match = Uuid::new_v4();
        let emp_match = Uuid::new_v4();

        let mut data = (*empty_roster_data()).clone();
        data.active_rules.push(crate::models::AssignmentRule::new(
            rule_id,
            Uuid::new_v4(),
            pattern_id,
            "Location/Employer matching rule".to_string(),
            10,
            crate::models::AssignmentRuleConditions::new(
                None,
                None,
                None,
                Some(loc_match),
                Some(emp_match),
            ),
            true,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
            None,
        ));
        data.rule_patterns
            .insert((pattern_id, dow), vec![Some(shift_id)]);

        let roster_map = HashMap::new();

        // 1. Mismatch location and employer (should not match, returns none)
        let (shifts, source, _) = super::resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp,
            emp,
            "EMP001",
            None,
            target_date,
            None,
            None,
        );
        assert_eq!(shifts, vec![None]);
        assert_eq!(source, "none");

        // 2. Mismatch location only (should not match, returns none)
        let (shifts, _source, _) = super::resolve_effective_shift_from_data(
            &Arc::new(data.clone()),
            &roster_map,
            emp,
            emp,
            "EMP001",
            None,
            target_date,
            None,
            Some(emp_match),
        );
        assert_eq!(shifts, vec![None]);

        // 3. Match both location and employer (should match, returns shift_id)
        let (shifts, source, _) = super::resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            emp,
            "EMP001",
            None,
            target_date,
            Some(loc_match),
            Some(emp_match),
        );
        assert_eq!(shifts, vec![Some(shift_id)]);
        assert_eq!(source, "assignment_rule");
    }

    // ─── Work Segments resolution tests ─────────────────────────────────

    use crate::cache::RosterShift;
