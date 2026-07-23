use crate::models::compliance::ComplianceRules;
use crate::schema::workspace_compliance_rules;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// Storage-layer row for workspace_compliance_rules.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = workspace_compliance_rules)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkspaceComplianceRuleEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub min_rest_period_hours: i32,
    pub max_weekly_hours: i32,
    pub max_monthly_ot_hours: i32,
    pub allow_manager_override: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Insert a new compliance rule row for a workspace.
#[derive(Debug, Insertable)]
#[diesel(table_name = workspace_compliance_rules)]
pub struct NewComplianceRule {
    pub workspace_id: Uuid,
    pub min_rest_period_hours: i32,
    pub max_weekly_hours: i32,
    pub max_monthly_ot_hours: i32,
    pub allow_manager_override: bool,
    pub is_enabled: bool,
}

impl From<WorkspaceComplianceRuleEntity> for ComplianceRules {
    fn from(e: WorkspaceComplianceRuleEntity) -> Self {
        Self {
            id: e.id,
            workspace_id: e.workspace_id,
            min_rest_period_hours: e.min_rest_period_hours,
            max_weekly_hours: e.max_weekly_hours,
            max_monthly_ot_hours: e.max_monthly_ot_hours,
            allow_manager_override: e.allow_manager_override,
            is_enabled: e.is_enabled,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}
