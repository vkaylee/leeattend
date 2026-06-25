//! Pure shift resolution logic — no DB access.
//!
//! The `resolve_effective_shift_from_data` function implements the 6-tier
//! priority chain for resolving which shift applies to an employee on a date.
//!
//! The `resolve_work_segments_from_data` function extends this by expanding
//! resolved shift IDs into full `ResolvedSegment` structs, supporting
//! split-shift patterns where a single day can have multiple work segments.

use crate::cache::RosterData;
use crate::models::{ResolvedSegment, ScheduleTrace};
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

/// Resolve the effective shift for a single employee on a single date using
/// pre-loaded [`RosterData`] and a roster override map.
///
/// Priority chain (highest wins):
///   P1 Roster → P2 Individual Pattern → P3 Dynamic Rules →
///   P4 Department Pattern → P5 Fixed Assignment → P6 Department Fixed
///
/// Returns `(shift_ids, source_label, traces)` where `shift_ids` may contain
/// multiple entries for split-shift patterns.
///
/// This is a **pure function** — no DB access — making it cheap to call in a
/// tight loop and easy to unit-test.
#[allow(clippy::too_many_lines)]
pub fn resolve_effective_shift_from_data<S: ::std::hash::BuildHasher>(
    roster_data: &RosterData,
    roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>, S>,
    employment_id: Uuid,
    employee_id: Uuid,
    employee_id_number: &str,
    employee_tags: Option<&Vec<String>>,
    date: NaiveDate,
    active_location_id: Option<Uuid>,
    active_employer_id: Option<Uuid>,
) -> (Vec<Option<Uuid>>, String, Vec<ScheduleTrace>) {
    let mut traces = Vec::new();

    // P1: Roster override
    if let Some(shift_opt) = roster_map.get(&(employment_id, date)) {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Manual Roster".to_string(),
            priority: 100,
            status: "applied".to_string(),
            reason: Some("Directly assigned via roster".to_string()),
        });
        return (vec![*shift_opt], "roster".to_string(), traces);
    }

    let dow = i32::try_from(chrono::Datelike::weekday(&date).num_days_from_sunday()).unwrap_or(0);

    // P2: Individual Working Pattern — collect ALL matching shifts for the day
    let patterns: Vec<_> = roster_data
        .pattern_assignments
        .get(&employment_id)
        .map(|v| {
            v.iter()
                .filter(|a| {
                    a.start_date <= date
                        && a.end_date.is_none_or(|ed| ed >= date)
                        && match a.pattern_type.as_str() {
                            "rotation" => {
                                let anchor = a.anchor_date.unwrap_or(a.start_date);
                                let cycle_len = a.cycle_length_days.unwrap_or(7);
                                let days_since =
                                    (date - anchor).num_days().rem_euclid(i64::from(cycle_len));
                                // Safety: days_since is in [0, cycle_len) where cycle_len <= 366
                                #[allow(clippy::cast_possible_truncation)]
                                let cycle_day = days_since as i32;
                                a.schedule_day == cycle_day
                            }
                            _ => a.schedule_day == dow, // weekly (unchanged)
                        }
                })
                .collect()
        })
        .unwrap_or_default();

    if !patterns.is_empty() {
        traces.push(ScheduleTrace {
            rule_id: Some(patterns[0].pattern_id),
            rule_name: "Individual Pattern".to_string(),
            priority: 80,
            status: "applied".to_string(),
            reason: Some("Assigned via individual working pattern".to_string()),
        });
        return (
            patterns.iter().map(|p| p.shift_id).collect(),
            "pattern".to_string(),
            traces,
        );
    }

    // P3: Dynamic Assignment Rules
    let emp_dept_id = roster_data
        .dept_history
        .get(&employee_id)
        .and_then(|v| v.iter().find(|h| h.effective_date <= date))
        .map(|h| h.department_id);

    let match_ctx = crate::services::rule_evaluator::EmployeeMatchContext {
        date,
        department_id: emp_dept_id,
        employee_id_number: employee_id_number.to_string(),
        tags: employee_tags.cloned().unwrap_or_default(),
        location_id: active_location_id,
        employer_id: active_employer_id,
    };

    let (matched_rule, rule_traces) =
        crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
            &match_ctx,
            &roster_data.active_rules,
        );
    traces.extend(rule_traces);

    if let Some(rule) = matched_rule {
        if let Some(shift_opts) = roster_data.rule_patterns.get(&(rule.pattern_id(), dow)) {
            return (shift_opts.clone(), "assignment_rule".to_string(), traces);
        }
    }

    // P5: Fixed Assignment (Individual)
    let fixed = roster_data
        .fixed_assignments
        .get(&employment_id)
        .and_then(|v| {
            v.iter()
                .find(|a| a.start_date <= date && a.end_date.is_none_or(|ed| ed >= date))
        });

    if let Some(f) = fixed {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Fixed Assignment".to_string(),
            priority: 20,
            status: "applied".to_string(),
            reason: Some("Directly assigned shift".to_string()),
        });
        return (vec![Some(f.shift_id)], "fixed".to_string(), traces);
    }

    (vec![None], "none".to_string(), traces)
}

/// Resolve full work segments for a single employee on a single date.
///
/// Wraps [`resolve_effective_shift_from_data`] and expands each resolved
/// shift ID into a [`ResolvedSegment`] by looking up the shift definition
/// in `roster_data.shifts`.
///
/// When a shift has intervals defined in `shift_default_intervals`, they
/// will be used (once the lookup is wired). For now, each shift produces
/// a single segment using the shift's own start_time/end_time.
///
/// Returns segments sorted by `scheduled_start`.
#[allow(clippy::too_many_lines)]
pub fn resolve_work_segments_from_data<S: ::std::hash::BuildHasher>(
    roster_data: &RosterData,
    roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>, S>,
    employment_id: Uuid,
    employee_id: Uuid,
    employee_id_number: &str,
    employee_tags: Option<&Vec<String>>,
    date: NaiveDate,
    active_location_id: Option<Uuid>,
    active_employer_id: Option<Uuid>,
) -> Vec<ResolvedSegment> {
    let (shift_ids, source, _traces) = resolve_effective_shift_from_data(
        roster_data,
        roster_map,
        employment_id,
        employee_id,
        employee_id_number,
        employee_tags,
        date,
        active_location_id,
        active_employer_id,
    );

    let mut segments: Vec<ResolvedSegment> = shift_ids
        .iter()
        .enumerate()
        .filter_map(|(idx, shift_opt)| {
            let shift_id = (*shift_opt)?;
            let shift = roster_data.shifts.get(&shift_id)?;

            Some(ResolvedSegment {
                shift_id,
                interval_id: None,
                sort_order: i32::try_from(idx).unwrap_or(i32::MAX),
                scheduled_start: shift.start_time(),
                scheduled_end: shift.end_time(),
                source: source.clone(),
                break_duration_minutes: Some(shift.break_duration_minutes()),
                break_start: shift.break_start(),
                break_end: shift.break_end(),
                is_break_auto_deduct: Some(shift.is_break_auto_deduct()),
                break_is_overtime: Some(shift.break_is_overtime()),
                allow_early_ot: shift.allow_early_ot(),
                max_early_ot_minutes: shift.max_early_ot_minutes(),
                early_punch_in_grace_period_minutes: shift.early_punch_in_grace_period_minutes(),
                ot_aggregation_mode: shift.ot_aggregation_mode().to_string(),
                is_night_shift: shift.is_night_shift(),
            })
        })
        .collect();

    // Sort by scheduled start time for correct segment ordering
    segments.sort_by_key(|s| s.scheduled_start);

    // Re-assign sort_order after sorting
    for (i, seg) in segments.iter_mut().enumerate() {
        seg.sort_order = i32::try_from(i).unwrap_or(i32::MAX);
    }

    segments
}

#[cfg(test)]
#[allow(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::similar_names,
    clippy::clone_on_ref_ptr
)]
mod tests {
    use super::*;
    use crate::cache::{RosterData, RosterFixedAssignment, RosterPatternEntry};
    use chrono::{NaiveDate, NaiveTime};
    use std::sync::Arc;

    fn resolve_effective_shift_from_data(
        roster_data: &RosterData,
        roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>>,
        employee_id: Uuid,
        employee_id_number: &str,
        employee_tags: Option<&Vec<String>>,
        date: NaiveDate,
    ) -> (Vec<Option<Uuid>>, String, Vec<ScheduleTrace>) {
        super::resolve_effective_shift_from_data(
            roster_data,
            roster_map,
            employee_id,
            employee_id,
            employee_id_number,
            employee_tags,
            date,
            None,
            None,
        )
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// Build a weekly `RosterPatternEntry` for tests (rotation fields defaulted).
    fn weekly_entry(
        pattern_id: Uuid,
        start: NaiveDate,
        end: Option<NaiveDate>,
        schedule_day: i32,
        shift_id: Option<Uuid>,
    ) -> RosterPatternEntry {
        RosterPatternEntry {
            pattern_id,
            start_date: start,
            end_date: end,
            schedule_day,
            shift_id,
            pattern_type: "weekly".to_string(),
            cycle_length_days: None,
            anchor_date: None,
        }
    }

    fn empty_roster_data() -> Arc<RosterData> {
        Arc::new(RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            active_rules: Vec::new(),
            rule_patterns: HashMap::new(),
            dept_history: HashMap::new(),
        })
    }

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

    fn make_test_shift(
        id: Uuid,
        name: &str,
        start_h: u32,
        start_m: u32,
        end_h: u32,
        end_m: u32,
    ) -> (Uuid, RosterShift) {
        let shift = RosterShift::new(
            name.to_string(),
            None,
            NaiveTime::from_hms_opt(start_h, start_m, 0).unwrap(),
            NaiveTime::from_hms_opt(end_h, end_m, 0).unwrap(),
            60,
            None,
            None,
            true,
            0,
            false,
            false,
            None,
            "aggregate".to_string(),
            false,
        );
        (id, shift)
    }

    fn make_night_shift(id: Uuid, name: &str) -> (Uuid, RosterShift) {
        let shift = RosterShift::new(
            name.to_string(),
            None,
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
            30,
            None,
            None,
            true,
            0,
            false,
            false,
            None,
            "aggregate".to_string(),
            true,
        );
        (id, shift)
    }

    fn make_roster_data_with_shifts(shifts_map: Vec<(Uuid, RosterShift)>) -> Arc<RosterData> {
        let mut data = (*empty_roster_data()).clone();
        data.shifts = shifts_map.into_iter().collect();
        Arc::new(data)
    }

    fn resolve_work_segments_wrapper(
        roster_data: &RosterData,
        roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>>,
        emp_id: Uuid,
        date: NaiveDate,
    ) -> Vec<crate::models::ResolvedSegment> {
        super::resolve_work_segments_from_data(
            roster_data,
            roster_map,
            emp_id,
            emp_id,
            "EMP001",
            None,
            date,
            None,
            None,
        )
    }

    #[test]
    fn test_resolve_single_segment_backward_compat() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let (sid, shift) = make_test_shift(shift_id, "Morning", 8, 0, 17, 0);

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let roster_map = HashMap::new();

        // No assignments → falls through to none → 0 segments
        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));
        assert_eq!(segments.len(), 0, "No shift assigned → 0 segments");

        // Now add a fixed assignment
        let mut data2 = (*data).clone();
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
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (m_id, morning) = make_test_shift(morning_id, "Ca Sáng", 7, 0, 11, 0);
        let (e_id, evening) = make_test_shift(evening_id, "Ca Chiều", 17, 0, 21, 0);

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(m_id, morning), (e_id, evening)].into_iter().collect();
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
        let (sid, shift) = make_test_shift(shift_id, "Roster Shift", 9, 0, 18, 0);

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
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
        let (sid, shift) = make_test_shift(shift_id, "Some Shift", 8, 0, 17, 0);

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
        let (sid, shift) = make_test_shift(shift_id, "Orphan Shift", 8, 0, 17, 0);

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
        let roster_map = HashMap::new();

        let segments = resolve_work_segments_wrapper(&data, &roster_map, emp, date(2026, 3, 3));

        assert_eq!(segments.len(), 0, "No match → 0 segments");
    }

    #[test]
    fn test_segment_inherits_shift_config() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let shift = RosterShift::new(
            "Config Test".to_string(),
            Some("CT".to_string()),
            NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
            45,
            Some(NaiveTime::from_hms_opt(11, 0, 0).unwrap()),
            Some(NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
            true,
            10,
            true,
            false,
            Some(120),
            "per_segment".to_string(),
            false,
        );

        let data = make_roster_data_with_shifts(vec![(shift_id, shift)]);
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
        assert_eq!(seg.allow_early_ot, true);
        assert_eq!(seg.max_early_ot_minutes, Some(120));
        assert_eq!(seg.early_punch_in_grace_period_minutes, 10);
        assert_eq!(seg.ot_aggregation_mode, "per_segment");
        assert_eq!(seg.is_night_shift, false);
    }

    #[test]
    fn test_night_shift_segment() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let (sid, shift) = make_night_shift(shift_id, "Ca Đêm");

        let data = make_roster_data_with_shifts(vec![(sid, shift)]);
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
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (id1, s1) = make_test_shift(s1_id, "Sáng", 6, 0, 10, 0);
        let (id2, s2) = make_test_shift(s2_id, "Chiều", 13, 0, 17, 0);
        let (id3, s3) = make_test_shift(s3_id, "Tối", 18, 0, 22, 0);

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(id1, s1), (id2, s2), (id3, s3)].into_iter().collect();
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

        let (sid, shift) = make_test_shift(shift_id, "Expired", 8, 0, 17, 0);
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
        let shift = RosterShift::new(
            "No Break".to_string(),
            None,
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            0,
            None,
            None,
            false,
            0,
            false,
            false,
            None,
            "aggregate".to_string(),
            false,
        );

        let data = make_roster_data_with_shifts(vec![(shift_id, shift)]);
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
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let (e_id, evening) = make_test_shift(evening_id, "Tối", 18, 0, 22, 0);
        let (m_id, morning) = make_test_shift(morning_id, "Sáng", 6, 0, 10, 0);

        let mut data = (*empty_roster_data()).clone();
        data.shifts = vec![(e_id, evening), (m_id, morning)].into_iter().collect();
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
}
