use crate::models::{
    NewOvertimePolicyAssignmentParams, NewOvertimeRuleParams, OvertimePolicy, OvertimeRule,
};
use crate::schema::{overtime_policy_assignments, overtime_policy_definitions, overtime_rules};
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use diesel::prelude::*;
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = overtime_policy_definitions)]
pub struct OvertimePolicyEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<OvertimePolicyEntity> for OvertimePolicy {
    fn from(entity: OvertimePolicyEntity) -> Self {
        Self::new(
            entity.id,
            entity.workspace_id,
            entity.name,
            entity.description,
            entity.created_at,
            entity.updated_at,
        )
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = overtime_rules)]
pub struct OvertimeRuleEntity {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub priority: i32,
    pub multiplier: Decimal,
    pub time_start: Option<NaiveTime>,
    pub time_end: Option<NaiveTime>,
    pub days_of_week: i32,
    pub is_holiday: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl From<OvertimeRuleEntity> for OvertimeRule {
    fn from(entity: OvertimeRuleEntity) -> Self {
        Self::new(NewOvertimeRuleParams {
            id: entity.id,
            policy_id: entity.policy_id,
            priority: entity.priority,
            multiplier: entity.multiplier,
            time_start: entity.time_start,
            time_end: entity.time_end,
            days_of_week: entity.days_of_week,
            is_holiday: entity.is_holiday,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        })
    }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = overtime_policy_assignments)]
pub struct OvertimePolicyAssignmentEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub created_at: Option<DateTime<Utc>>,
}

impl From<OvertimePolicyAssignmentEntity> for crate::models::OvertimePolicyAssignment {
    fn from(entity: OvertimePolicyAssignmentEntity) -> Self {
        Self::new(NewOvertimePolicyAssignmentParams {
            id: entity.id,
            workspace_id: entity.workspace_id,
            assignee_type: entity.assignee_type,
            assignee_id: entity.assignee_id,
            policy_id: entity.policy_id,
            effective_from: entity.effective_from,
            effective_to: entity.effective_to,
            created_at: entity.created_at,
        })
    }
}
