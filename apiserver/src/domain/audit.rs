use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuditLog {
    id: Uuid,
    workspace_id: Uuid,
    actor_id: Option<Uuid>,
    actor_type: Option<String>,
    actor_name: Option<String>,
    actor_email: Option<String>,
    target_user_id: Option<Uuid>,
    target_resource_id: Option<Uuid>,
    action: String,
    details: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: DateTime<Utc>,
}

pub struct NewAuditLogParams {
    id: Uuid,
    workspace_id: Uuid,
    actor_id: Option<Uuid>,
    actor_type: Option<String>,
    actor_name: Option<String>,
    actor_email: Option<String>,
    target_user_id: Option<Uuid>,
    target_resource_id: Option<Uuid>,
    action: String,
    details: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
    created_at: DateTime<Utc>,
}

impl NewAuditLogParams {
    pub fn new(
        id: Uuid,
        workspace_id: Uuid,
        action: impl Into<String>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            workspace_id,
            actor_id: None,
            actor_type: None,
            actor_name: None,
            actor_email: None,
            target_user_id: None,
            target_resource_id: None,
            action: action.into(),
            details: None,
            ip_address: None,
            user_agent: None,
            created_at,
        }
    }

    #[must_use]
    pub fn with_actor(
        mut self,
        actor_id: Option<Uuid>,
        actor_type: Option<String>,
        actor_name: Option<String>,
        actor_email: Option<String>,
    ) -> Self {
        self.actor_id = actor_id;
        self.actor_type = actor_type;
        self.actor_name = actor_name;
        self.actor_email = actor_email;
        self
    }

    #[must_use]
    pub fn with_target(
        mut self,
        target_user_id: Option<Uuid>,
        target_resource_id: Option<Uuid>,
    ) -> Self {
        self.target_user_id = target_user_id;
        self.target_resource_id = target_resource_id;
        self
    }

    #[must_use]
    pub fn with_details(mut self, details: Option<serde_json::Value>) -> Self {
        self.details = details;
        self
    }

    #[must_use]
    pub fn with_client_context(
        mut self,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        self.ip_address = ip_address;
        self.user_agent = user_agent;
        self
    }
}

impl AuditLog {
    pub fn new(params: NewAuditLogParams) -> Self {
        Self {
            id: params.id,
            workspace_id: params.workspace_id,
            actor_id: params.actor_id,
            actor_type: params.actor_type,
            actor_name: params.actor_name,
            actor_email: params.actor_email,
            target_user_id: params.target_user_id,
            target_resource_id: params.target_resource_id,
            action: params.action,
            details: params.details,
            ip_address: params.ip_address,
            user_agent: params.user_agent,
            created_at: params.created_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn workspace_id(&self) -> Uuid {
        self.workspace_id
    }

    pub fn actor_id(&self) -> Option<Uuid> {
        self.actor_id
    }

    pub fn actor_type(&self) -> Option<&str> {
        self.actor_type.as_deref()
    }

    pub fn actor_name(&self) -> Option<&str> {
        self.actor_name.as_deref()
    }

    pub fn actor_email(&self) -> Option<&str> {
        self.actor_email.as_deref()
    }

    pub fn target_user_id(&self) -> Option<Uuid> {
        self.target_user_id
    }

    pub fn target_resource_id(&self) -> Option<Uuid> {
        self.target_resource_id
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn details(&self) -> Option<&serde_json::Value> {
        self.details.as_ref()
    }

    pub fn ip_address(&self) -> Option<&str> {
        self.ip_address.as_deref()
    }

    pub fn user_agent(&self) -> Option<&str> {
        self.user_agent.as_deref()
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
