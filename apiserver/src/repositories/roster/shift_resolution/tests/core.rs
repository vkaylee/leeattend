//! Decomposed from shift_resolution.rs
use super::*;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::NaiveDate;

    #[test]
    fn test_no_match_returns_none() {
        let data = empty_roster_data();
        let roster_map = HashMap::new();
        let emp = Uuid::new_v4();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &data,
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, vec![None]);
        assert_eq!(source, "none");
        assert!(traces.is_empty());
    }

    #[test]
    fn test_p1_roster_wins() {
        let data = empty_roster_data();
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), Some(shift_id));

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &data,
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, vec![Some(shift_id)]);
        assert_eq!(source, "roster");
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].priority, 100);
    }

    #[test]
    fn test_p1_roster_day_off() {
        let data = empty_roster_data();
        let emp = Uuid::new_v4();

        let mut roster_map = HashMap::new();
        roster_map.insert((emp, date(2026, 3, 3)), None); // Day off

        let (shift, source, _) = resolve_effective_shift_from_data(
            &data,
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, vec![None]);
        assert_eq!(source, "roster");
    }

    #[test]
    fn test_p2_individual_pattern_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        // 2026-03-03 is a Tuesday = num_days_from_sunday = 2
        let target_date = date(2026, 3, 3);
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

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(shift, vec![Some(shift_id)]);
        assert_eq!(source, "pattern");
        assert_eq!(traces[0].priority, 80);
    }

    #[test]
    fn test_p2_split_shift_returns_both_shifts() {
        let emp = Uuid::new_v4();
        let shift_morning = Uuid::new_v4();
        let shift_afternoon = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        // Two shifts on the same day for the same pattern
        data.pattern_assignments.insert(
            emp,
            vec![
                weekly_entry(pattern_id, date(2026, 1, 1), None, dow, Some(shift_morning)),
                weekly_entry(
                    pattern_id,
                    date(2026, 1, 1),
                    None,
                    dow,
                    Some(shift_afternoon),
                ),
            ],
        );

        let roster_map = HashMap::new();

        let (shift_ids, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(shift_ids.len(), 2, "Split shift should return 2 shifts");
        assert_eq!(shift_ids, vec![Some(shift_morning), Some(shift_afternoon)]);
        assert_eq!(source, "pattern");
        assert_eq!(traces[0].priority, 80);
    }

    #[test]
    fn test_p3_rule_split_shift_returns_both() {
        let emp = Uuid::new_v4();
        let shift_morning = Uuid::new_v4();
        let shift_afternoon = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let rule_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.active_rules.push(crate::models::AssignmentRule::new(
            rule_id,
            Uuid::new_v4(),
            pattern_id,
            "Split shift rule".to_string(),
            10,
            crate::models::AssignmentRuleConditions::new(None, None, None, None, None),
            true,
            chrono::Utc::now(),
            chrono::Utc::now(),
            None,
            None,
        ));
        // Two shifts for the same (pattern, day)
        data.rule_patterns.insert(
            (pattern_id, dow),
            vec![Some(shift_morning), Some(shift_afternoon)],
        );

        let roster_map = HashMap::new();

        let (shift_ids, source, _) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            target_date,
        );

        assert_eq!(
            shift_ids.len(),
            2,
            "Rule split shift should return 2 shifts"
        );
        assert_eq!(shift_ids, vec![Some(shift_morning), Some(shift_afternoon)]);
        assert_eq!(source, "assignment_rule");
    }

    #[test]
    fn test_p5_fixed_assignment_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let mut data = (*empty_roster_data()).clone();
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );

        let roster_map = HashMap::new();

        let (shift, source, traces) = resolve_effective_shift_from_data(
            &Arc::new(data),
            &roster_map,
            emp,
            "EMP001",
            None,
            date(2026, 3, 3),
        );

        assert_eq!(shift, vec![Some(shift_id)]);
        assert_eq!(source, "fixed");
        assert_eq!(traces[0].priority, 20);
    }

    #[test]
    fn test_p1_beats_all_lower_priorities() {
        let emp = Uuid::new_v4();
        let roster_shift = Uuid::new_v4();
        let fixed_shift = Uuid::new_v4();
        let target_date = date(2026, 3, 3);

        let mut data = (*empty_roster_data()).clone();
        // P5
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id: fixed_shift,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
        );

        // P1 roster override
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

        assert_eq!(shift, vec![Some(roster_shift)], "P1 roster should beat P5");
        assert_eq!(source, "roster");
    }

    #[test]
    fn test_expired_pattern_falls_through() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let fixed_shift = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        // P2: expired pattern (ended before target date)
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
        // P5: active fixed assignment
        data.fixed_assignments.insert(
            emp,
            vec![RosterFixedAssignment {
                shift_id: fixed_shift,
                start_date: date(2026, 1, 1),
                end_date: None,
            }],
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

        assert_eq!(
            shift,
            vec![Some(fixed_shift)],
            "expired P2 should fall through to P5"
        );
        assert_eq!(source, "fixed");
    }

    // ─── Rotation pattern tests ───────────────────────────────────────────

    /// Build a rotation `RosterPatternEntry` for tests.
    fn rotation_entry(
        pattern_id: Uuid,
        start: NaiveDate,
        end: Option<NaiveDate>,
        cycle_length_days: i32,
        anchor_date: NaiveDate,
        schedule_day: i32,
        shift_id: Option<Uuid>,
    ) -> RosterPatternEntry {
        RosterPatternEntry {
            pattern_id,
            start_date: start,
            end_date: end,
            schedule_day,
            shift_id,
            pattern_type: "rotation".to_string(),
            cycle_length_days: Some(cycle_length_days),
            anchor_date: Some(anchor_date),
        }
    }
