use crate::models::{AssignmentRule, ScheduleTrace};
use uuid::Uuid;

pub struct EmployeeMatchContext {
    pub department_id: Option<Uuid>,
    pub employee_id_number: String,
    pub tags: Vec<String>,
}

pub struct RuleEvaluator;

impl RuleEvaluator {
    pub fn matches(ctx: &EmployeeMatchContext, rule: &AssignmentRule) -> bool {
        let conditions = &rule.conditions;

        // 1. Department match (Strict)
        if let Some(dept_id) = conditions.department_id {
            if ctx.department_id != Some(dept_id) {
                return false;
            }
        }

        // 2. Employee ID Numbers match (Whitelist)
        if let Some(ids) = &conditions.employee_id_numbers {
            if !ids.contains(&ctx.employee_id_number) {
                return false;
            }
        }

        // 3. Tags match (Intersection - AT LEAST ONE)
        if let Some(rule_tags) = &conditions.tags {
            if !rule_tags.is_empty() {
                let has_tag_match = ctx.tags.iter().any(|t| rule_tags.contains(t));
                if !has_tag_match {
                    return false;
                }
            }
        }

        true
    }

    /// Finds the first matching rule based on priority
    pub fn resolve_rule<'a>(
        ctx: &EmployeeMatchContext,
        rules: &'a [AssignmentRule],
    ) -> Option<&'a AssignmentRule> {
        rules.iter().find(|r| Self::matches(ctx, r))
    }

    /// Resolves rules and returns a list of traces for transparency
    pub fn resolve_with_trace<'a>(
        ctx: &EmployeeMatchContext,
        rules: &'a [AssignmentRule],
    ) -> (Option<&'a AssignmentRule>, Vec<ScheduleTrace>) {
        let mut traces = Vec::new();
        let mut winner = None;

        for rule in rules {
            if Self::matches(ctx, rule) {
                if winner.is_none() {
                    winner = Some(rule);
                    traces.push(ScheduleTrace {
                        rule_id: Some(rule.id),
                        rule_name: rule.name.clone(),
                        priority: rule.priority,
                        status: "applied".to_string(),
                        reason: Some("Highest priority matching rule".to_string()),
                    });
                } else {
                    traces.push(ScheduleTrace {
                        rule_id: Some(rule.id),
                        rule_name: rule.name.clone(),
                        priority: rule.priority,
                        status: "overridden".to_string(),
                        reason: Some("Overridden by higher priority rule".to_string()),
                    });
                }
            }
        }

        (winner, traces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AssignmentRuleConditions;
    use chrono::Utc;
    use sqlx::types::Json;

    fn make_rule(
        name: &str,
        priority: i32,
        dept: Option<Uuid>,
        ids: Option<Vec<String>>,
        tags: Option<Vec<String>>,
    ) -> AssignmentRule {
        AssignmentRule {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            pattern_id: Uuid::new_v4(),
            name: name.to_string(),
            priority,
            conditions: Json(AssignmentRuleConditions {
                department_id: dept,
                employee_id_numbers: ids,
                tags,
            }),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_ctx(dept: Option<Uuid>, id_number: &str, tags: Vec<&str>) -> EmployeeMatchContext {
        EmployeeMatchContext {
            department_id: dept,
            employee_id_number: id_number.to_string(),
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    // --- matches() tests ---

    #[test]
    fn test_matches_all_none_conditions() {
        let rule = make_rule("Catch-all", 1, None, None, None);
        let ctx = make_ctx(None, "EMP001", vec![]);
        assert!(RuleEvaluator::matches(&ctx, &rule));
    }

    #[test]
    fn test_matches_department_exact() {
        let dept_id = Uuid::new_v4();
        let rule = make_rule("Dept Only", 1, Some(dept_id), None, None);

        let ctx_match = make_ctx(Some(dept_id), "EMP001", vec![]);
        assert!(RuleEvaluator::matches(&ctx_match, &rule));

        let ctx_no_match = make_ctx(Some(Uuid::new_v4()), "EMP001", vec![]);
        assert!(!RuleEvaluator::matches(&ctx_no_match, &rule));

        let ctx_none = make_ctx(None, "EMP001", vec![]);
        assert!(!RuleEvaluator::matches(&ctx_none, &rule));
    }

    #[test]
    fn test_matches_employee_id_whitelist() {
        let rule = make_rule(
            "ID Whitelist",
            1,
            None,
            Some(vec!["EMP001".into(), "EMP002".into()]),
            None,
        );

        assert!(RuleEvaluator::matches(&make_ctx(None, "EMP001", vec![]), &rule));
        assert!(RuleEvaluator::matches(&make_ctx(None, "EMP002", vec![]), &rule));
        assert!(!RuleEvaluator::matches(&make_ctx(None, "EMP999", vec![]), &rule));
    }

    #[test]
    fn test_matches_tag_intersection() {
        let rule = make_rule("Tag Rule", 1, None, None, Some(vec!["vip".into(), "night".into()]));

        // Employee has one matching tag
        assert!(RuleEvaluator::matches(&make_ctx(None, "EMP001", vec!["vip"]), &rule));
        // Employee has different matching tag
        assert!(RuleEvaluator::matches(&make_ctx(None, "EMP001", vec!["night", "other"]), &rule));
        // Employee has no matching tags
        assert!(!RuleEvaluator::matches(&make_ctx(None, "EMP001", vec!["day"]), &rule));
        // Employee has no tags
        assert!(!RuleEvaluator::matches(&make_ctx(None, "EMP001", vec![]), &rule));
    }

    #[test]
    fn test_matches_empty_tags_is_catch_all() {
        let rule = make_rule("Empty Tags", 1, None, None, Some(vec![]));
        // Empty tag list means "no tag restriction"
        assert!(RuleEvaluator::matches(&make_ctx(None, "EMP001", vec![]), &rule));
    }

    #[test]
    fn test_matches_combined_conditions_all_must_pass() {
        let dept_id = Uuid::new_v4();
        let rule = make_rule(
            "Combined",
            1,
            Some(dept_id),
            Some(vec!["EMP001".into()]),
            Some(vec!["vip".into()]),
        );

        // All match
        assert!(RuleEvaluator::matches(&make_ctx(Some(dept_id), "EMP001", vec!["vip"]), &rule));
        // Dept mismatch
        assert!(!RuleEvaluator::matches(&make_ctx(Some(Uuid::new_v4()), "EMP001", vec!["vip"]), &rule));
        // ID mismatch
        assert!(!RuleEvaluator::matches(&make_ctx(Some(dept_id), "EMP999", vec!["vip"]), &rule));
        // Tag mismatch
        assert!(!RuleEvaluator::matches(&make_ctx(Some(dept_id), "EMP001", vec!["day"]), &rule));
    }

    // --- resolve_rule() tests ---

    #[test]
    fn test_resolve_rule_returns_first_match() {
        let rules = vec![
            make_rule("Low", 1, None, None, Some(vec!["night".into()])),
            make_rule("Catch-all", 2, None, None, None),
        ];
        let ctx = make_ctx(None, "EMP001", vec!["day"]);
        let result = RuleEvaluator::resolve_rule(&ctx, &rules);
        assert_eq!(result.unwrap().name, "Catch-all");
    }

    #[test]
    fn test_resolve_rule_none_when_no_match() {
        let dept_id = Uuid::new_v4();
        let rules = vec![make_rule("Dept Only", 1, Some(dept_id), None, None)];
        let ctx = make_ctx(None, "EMP001", vec![]);
        assert!(RuleEvaluator::resolve_rule(&ctx, &rules).is_none());
    }

    #[test]
    fn test_resolve_rule_empty_rules() {
        let ctx = make_ctx(None, "EMP001", vec![]);
        assert!(RuleEvaluator::resolve_rule(&ctx, &[]).is_none());
    }

    // --- resolve_with_trace() tests ---

    #[test]
    fn test_trace_marks_applied_and_overridden() {
        let rules = vec![
            make_rule("First", 1, None, None, None),
            make_rule("Second", 2, None, None, None),
        ];
        let ctx = make_ctx(None, "EMP001", vec![]);
        let (winner, traces) = RuleEvaluator::resolve_with_trace(&ctx, &rules);

        assert_eq!(winner.unwrap().name, "First");
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].status, "applied");
        assert_eq!(traces[1].status, "overridden");
    }
}
