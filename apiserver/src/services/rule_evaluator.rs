use uuid::Uuid;
use crate::models::AssignmentRule;

pub struct EmployeeMatchContext {
    pub department_id: Option<Uuid>,
    pub employee_id_number: String,
    pub tags: Vec<String>,
}

pub struct RuleEvaluator;

impl RuleEvaluator {
    pub fn matches(
        ctx: &EmployeeMatchContext,
        rule: &AssignmentRule,
    ) -> bool {
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
}
