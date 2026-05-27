use crate::schema::{approval_action_log, approval_workflow_steps, approval_workflows};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWorkflowRequest {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub steps: Vec<WorkflowStepRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkflowStepRequest {
    pub step_order: i32,
    pub step_name: String,
    pub approver_type: String,
    pub approver_config: serde_json::Value,
    pub action_required: String,
    pub can_skip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = approval_workflows)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApprovalWorkflow {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub total_steps: i32,
    pub is_active: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = approval_workflow_steps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApprovalWorkflowStep {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub step_order: i32,
    pub step_name: String,
    pub approver_type: String,
    pub approver_config: serde_json::Value,
    pub action_required: String,
    pub can_skip: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, ToSchema)]
#[diesel(table_name = approval_action_log)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApprovalActionLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub step_order: i32,
    pub step_name: Option<String>,
    pub acted_by_user_id: Uuid,
    pub action: String,
    pub note: Option<String>,
    pub acted_at: DateTime<Utc>,
}
