use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::domain::audit::AuditLog;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::audit_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuditLogEntity {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub target_user_id: Option<Uuid>,
    pub target_resource_id: Option<Uuid>,
    pub action: String,
    pub details: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub actor_type: Option<String>,
    pub actor_name: Option<String>,
    pub actor_email: Option<String>,
}

impl From<AuditLogEntity> for AuditLog {
    fn from(entity: AuditLogEntity) -> Self {
        Self::new(crate::domain::audit::NewAuditLogParams {
            id: entity.id,
            workspace_id: entity.workspace_id,
            actor_id: entity.actor_id,
            actor_type: entity.actor_type,
            actor_name: entity.actor_name,
            actor_email: entity.actor_email,
            target_user_id: entity.target_user_id,
            target_resource_id: entity.target_resource_id,
            action: entity.action,
            details: entity.details,
            ip_address: entity.ip_address,
            user_agent: entity.user_agent,
            created_at: entity.created_at,
        })
    }
}

impl From<AuditLog> for AuditLogEntity {
    fn from(log: AuditLog) -> Self {
        Self {
            id: log.id(),
            workspace_id: log.workspace_id(),
            actor_id: log.actor_id(),
            actor_type: log.actor_type().map(String::from),
            actor_name: log.actor_name().map(String::from),
            actor_email: log.actor_email().map(String::from),
            target_user_id: log.target_user_id(),
            target_resource_id: log.target_resource_id(),
            action: log.action().to_string(),
            details: log.details().cloned(),
            ip_address: log.ip_address().map(String::from),
            user_agent: log.user_agent().map(String::from),
            created_at: log.created_at(),
        }
    }
}
