use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::employee::{AssignmentRule, AssignmentRuleConditions};
use crate::schema::assignment_rules;

#[derive(Debug, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = assignment_rules)]
pub struct AssignmentRuleEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub pattern_id: Uuid,
    pub name: String,
    pub priority: i32,
    pub conditions: serde_json::Value,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<AssignmentRuleEntity> for AssignmentRule {
    fn from(e: AssignmentRuleEntity) -> Self {
        Self {
            id: e.id,
            workspace_id: e.workspace_id,
            pattern_id: e.pattern_id,
            name: e.name,
            priority: e.priority,
            conditions: serde_json::from_value(e.conditions).unwrap_or(AssignmentRuleConditions {
                department_id: None,
                employee_id_numbers: None,
                tags: None,
            }),
            is_active: e.is_active,
            created_at: e.created_at.unwrap_or_else(Utc::now),
            updated_at: e.updated_at.unwrap_or_else(Utc::now),
        }
    }
}
