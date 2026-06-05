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

impl AuditLog {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
    ) -> Self {
        Self {
            id,
            workspace_id,
            actor_id,
            actor_type,
            actor_name,
            actor_email,
            target_user_id,
            target_resource_id,
            action,
            details,
            ip_address,
            user_agent,
            created_at,
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
