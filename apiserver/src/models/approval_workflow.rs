use crate::schema::{
    approval_action_log, approval_workflow_instances, approval_workflow_steps,
    approval_workflow_tasks, approval_workflows,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalAction {
    Approve,
    Reject,
    Return,
    Reassign,
    Skip,
}

impl ApprovalAction {
    pub fn parse_value(s: &str) -> Option<Self> {
        match s {
            "approve" | "approved" => Some(Self::Approve),
            "reject" | "rejected" => Some(Self::Reject),
            "return" | "returned" => Some(Self::Return),
            "reassign" => Some(Self::Reassign),
            "skip" | "skipped" => Some(Self::Skip),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approve => "approved",
            Self::Reject => "rejected",
            Self::Return => "returned",
            Self::Reassign => "reassigned",
            Self::Skip => "skipped",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateWorkflowRequest {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub steps: Vec<WorkflowStepRequest>,
}

/// Partial update request for an approval workflow.
/// All fields are optional — only provided values are applied.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_active: Option<bool>,
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

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WorkflowInstanceStepSnapshot {
    pub step_order: i32,
    pub step_name: String,
    pub approver_type: String,
    pub approver_config: serde_json::Value,
    pub resolved_approver_ids: Vec<Uuid>,
    pub consensus_mode: String,
    pub action_required: String,
    pub can_skip: bool,
    pub status: String,
    pub sla_hours: Option<i32>,
    pub escalation_action: Option<serde_json::Value>,
    pub approvers_acted: Vec<ApproverActionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApproverActionInfo {
    pub user_id: Uuid,
    pub action: String,
    pub note: Option<String>,
    pub acted_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset, ToSchema,
)]
#[diesel(table_name = approval_workflow_instances)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApprovalWorkflowInstance {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub workflow_id: Option<Uuid>,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub status: String,
    pub current_step_order: i32,
    pub steps_snapshot: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Insertable, AsChangeset, ToSchema,
)]
#[diesel(table_name = approval_workflow_tasks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApprovalWorkflowTask {
    id: Uuid,
    instance_id: Uuid,
    step_order: i32,
    status: String,
    assignee_type: String,
    assignee_target_id: Option<Uuid>,
    assignee_target_name: Option<String>,
    created_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    completed_by: Option<Uuid>,
}

impl ApprovalWorkflowTask {
    pub fn new(
        instance_id: Uuid,
        step_order: i32,
        assignee_type: String,
        assignee_target_id: Option<Uuid>,
        assignee_target_name: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            instance_id,
            step_order,
            status: "pending".to_string(),
            assignee_type,
            assignee_target_id,
            assignee_target_name,
            created_at: Utc::now(),
            completed_at: None,
            completed_by: None,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn instance_id(&self) -> Uuid {
        self.instance_id
    }

    pub fn step_order(&self) -> i32 {
        self.step_order
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn assignee_type(&self) -> &str {
        &self.assignee_type
    }

    pub fn assignee_target_id(&self) -> Option<Uuid> {
        self.assignee_target_id
    }

    pub fn assignee_target_name(&self) -> Option<&str> {
        self.assignee_target_name.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    pub fn completed_by(&self) -> Option<Uuid> {
        self.completed_by
    }

    pub fn complete(&mut self, completed_by: Uuid) {
        self.status = "completed".to_string();
        self.completed_at = Some(Utc::now());
        self.completed_by = Some(completed_by);
    }

    pub fn cancel(&mut self) {
        self.status = "cancelled".to_string();
        self.completed_at = Some(Utc::now());
    }
}
