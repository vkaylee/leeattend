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
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
}

impl From<AssignmentRuleEntity> for AssignmentRule {
    fn from(e: AssignmentRuleEntity) -> Self {
        let conditions = serde_json::from_value(e.conditions)
            .unwrap_or_else(|_| AssignmentRuleConditions::new(None, None, None, None, None));
        Self::new(
            e.id,
            e.workspace_id,
            e.pattern_id,
            e.name,
            e.priority,
            conditions,
            e.is_active,
            e.created_at.unwrap_or_else(Utc::now),
            e.updated_at.unwrap_or_else(Utc::now),
            e.start_date,
            e.end_date,
        )
    }
}
