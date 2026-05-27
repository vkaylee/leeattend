use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct AuditLog {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_type: Option<String>,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
    pub target_user_id: Option<Uuid>,
    pub target_resource_id: Option<Uuid>,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
