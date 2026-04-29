//! Pure shift resolution logic — no DB access.
//!
//! The `resolve_effective_shift_from_data` function implements the 6-tier
//! priority chain for resolving which shift applies to an employee on a date.

use crate::cache::RosterData;
use crate::models::ScheduleTrace;
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
pub fn resolve_effective_shift_from_data(
    roster_data: &RosterData,
    roster_map: &HashMap<(Uuid, NaiveDate), Option<Uuid>>,
    employee_id: Uuid,
    employee_id_number: &str,
    employee_tags: Option<&Vec<String>>,
    date: NaiveDate,
) -> (Vec<Option<Uuid>>, String, Vec<ScheduleTrace>) {
    let mut traces = Vec::new();

    // P1: Roster override
    if let Some(shift_opt) = roster_map.get(&(employee_id, date)) {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Manual Roster".to_string(),
            priority: 100,
            status: "applied".to_string(),
            reason: Some("Directly assigned via roster".to_string()),
        });
        return (vec![*shift_opt], "roster".to_string(), traces);
    }

    let dow = chrono::Datelike::weekday(&date).num_days_from_sunday() as i32;

    // P2: Individual Working Pattern — collect ALL matching shifts for the day
    let patterns: Vec<_> = roster_data
        .pattern_assignments
        .get(&employee_id)
        .map(|v| {
            v.iter()
                .filter(|a| {
                    a.start_date <= date
                        && a.end_date.is_none_or(|ed| ed >= date)
                        && a.day_of_week == dow
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
        department_id: emp_dept_id,
        employee_id_number: employee_id_number.to_string(),
        tags: employee_tags.cloned().unwrap_or_default(),
    };

    let (matched_rule, rule_traces) =
        crate::services::rule_evaluator::RuleEvaluator::resolve_with_trace(
            &match_ctx,
            &roster_data.active_rules,
        );
    traces.extend(rule_traces);

    if let Some(rule) = matched_rule {
        if let Some(shift_opts) = roster_data.rule_patterns.get(&(rule.pattern_id, dow)) {
            return (shift_opts.clone(), "assignment_rule".to_string(), traces);
        }
    }

    // P4: Department Working Pattern — collect ALL matching shifts
    let dept_ps: Vec<_> = roster_data
        .dept_patterns
        .get(&employee_id)
        .map(|v| {
            v.iter()
                .filter(|a| a.effective_date <= date && a.day_of_week == dow)
                .collect()
        })
        .unwrap_or_default();

    if !dept_ps.is_empty() {
        traces.push(ScheduleTrace {
            rule_id: dept_ps[0].default_working_pattern_id,
            rule_name: "Department Pattern".to_string(),
            priority: 40,
            status: "applied".to_string(),
            reason: Some("Inherited from department default pattern".to_string()),
        });
        return (
            dept_ps.iter().map(|dp| dp.shift_id).collect(),
            "department_pattern".to_string(),
            traces,
        );
    }

    // P5: Fixed Assignment (Individual)
    let fixed = roster_data
        .fixed_assignments
        .get(&employee_id)
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

    // P6: Department Fixed Shift
    let dept_f = roster_data
        .dept_fixed_shifts
        .get(&employee_id)
        .and_then(|v| v.iter().find(|a| a.effective_date <= date));

    if let Some(df) = dept_f {
        traces.push(ScheduleTrace {
            rule_id: None,
            rule_name: "Department Fixed".to_string(),
            priority: 10,
            status: "applied".to_string(),
            reason: Some("Inherited from department default shift".to_string()),
        });
        return (
            vec![Some(df.shift_id)],
            "department_fixed".to_string(),
            traces,
        );
    }

    (vec![None], "none".to_string(), traces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::{
        RosterData, RosterDeptFixedShift, RosterDeptPattern, RosterFixedAssignment,
        RosterPatternEntry,
    };
    use chrono::NaiveDate;
    use std::sync::Arc;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn empty_roster_data() -> Arc<RosterData> {
        Arc::new(RosterData {
            shifts: HashMap::new(),
            fixed_assignments: HashMap::new(),
            pattern_assignments: HashMap::new(),
            dept_patterns: HashMap::new(),
            dept_fixed_shifts: HashMap::new(),
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
            vec![RosterPatternEntry {
                pattern_id,
                start_date: date(2026, 1, 1),
                end_date: None,
                day_of_week: dow,
                shift_id: Some(shift_id),
            }],
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
                RosterPatternEntry {
                    pattern_id,
                    start_date: date(2026, 1, 1),
                    end_date: None,
                    day_of_week: dow,
                    shift_id: Some(shift_morning),
                },
                RosterPatternEntry {
                    pattern_id,
                    start_date: date(2026, 1, 1),
                    end_date: None,
                    day_of_week: dow,
                    shift_id: Some(shift_afternoon),
                },
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
        data.active_rules.push(crate::models::AssignmentRule {
            id: rule_id,
            workspace_id: Uuid::new_v4(),
            name: "Split shift rule".to_string(),
            pattern_id,
            priority: 10,
            is_active: true,
            conditions: sqlx::types::Json(crate::models::AssignmentRuleConditions {
                department_id: None,
                employee_id_numbers: None,
                tags: None,
            }),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });
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
    fn test_p4_dept_split_shift_returns_both() {
        let emp = Uuid::new_v4();
        let shift_morning = Uuid::new_v4();
        let shift_afternoon = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.dept_patterns.insert(
            emp,
            vec![
                RosterDeptPattern {
                    default_working_pattern_id: Some(pattern_id),
                    effective_date: date(2026, 1, 1),
                    day_of_week: dow,
                    shift_id: Some(shift_morning),
                },
                RosterDeptPattern {
                    default_working_pattern_id: Some(pattern_id),
                    effective_date: date(2026, 1, 1),
                    day_of_week: dow,
                    shift_id: Some(shift_afternoon),
                },
            ],
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
            "Dept split shift should return 2 shifts"
        );
        assert_eq!(shift_ids, vec![Some(shift_morning), Some(shift_afternoon)]);
        assert_eq!(source, "department_pattern");
    }

    #[test]
    fn test_p4_department_pattern_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let pattern_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);
        let dow = chrono::Datelike::weekday(&target_date).num_days_from_sunday() as i32;

        let mut data = (*empty_roster_data()).clone();
        data.dept_patterns.insert(
            emp,
            vec![RosterDeptPattern {
                default_working_pattern_id: Some(pattern_id),
                effective_date: date(2026, 1, 1),
                day_of_week: dow,
                shift_id: Some(shift_id),
            }],
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
        assert_eq!(source, "department_pattern");
        assert_eq!(traces[0].priority, 40);
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
    fn test_p6_department_fixed_wins() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();

        let mut data = (*empty_roster_data()).clone();
        data.dept_fixed_shifts.insert(
            emp,
            vec![RosterDeptFixedShift {
                shift_id,
                effective_date: date(2026, 1, 1),
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
        assert_eq!(source, "department_fixed");
        assert_eq!(traces[0].priority, 10);
    }

    #[test]
    fn test_p1_beats_all_lower_priorities() {
        let emp = Uuid::new_v4();
        let roster_shift = Uuid::new_v4();
        let fixed_shift = Uuid::new_v4();
        let dept_fixed_shift = Uuid::new_v4();
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
        // P6
        data.dept_fixed_shifts.insert(
            emp,
            vec![RosterDeptFixedShift {
                shift_id: dept_fixed_shift,
                effective_date: date(2026, 1, 1),
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

        assert_eq!(
            shift,
            vec![Some(roster_shift)],
            "P1 roster should beat P5 and P6"
        );
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
            vec![RosterPatternEntry {
                pattern_id,
                start_date: date(2026, 1, 1),
                end_date: Some(date(2026, 2, 28)), // expired
                day_of_week: dow,
                shift_id: Some(shift_id),
            }],
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

    #[test]
    fn test_future_dept_history_ignored() {
        let emp = Uuid::new_v4();
        let shift_id = Uuid::new_v4();
        let target_date = date(2026, 3, 3);

        let mut data = (*empty_roster_data()).clone();
        // Department fixed with future effective date
        data.dept_fixed_shifts.insert(
            emp,
            vec![RosterDeptFixedShift {
                shift_id,
                effective_date: date(2026, 4, 1), // future
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

        assert_eq!(shift, vec![None], "future effective_date should be ignored");
        assert_eq!(source, "none");
    }
}
