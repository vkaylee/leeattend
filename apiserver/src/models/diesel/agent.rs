use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::{AuthorizedAgent, Device};

#[derive(Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = crate::schema::authorized_agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AuthorizedAgentEntity {
    pub id: String,
    pub name: String,
    pub token: String,
    pub is_trusted: Option<bool>,
    pub last_ip: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub workspace_id: Uuid,
    pub last_seen_at: Option<DateTime<Utc>>,
    pub max_concurrent_workers: i32,
    pub success_cooldown_secs: i32,
    pub retention_days: i32,
}

impl From<AuthorizedAgentEntity> for AuthorizedAgent {
    fn from(entity: AuthorizedAgentEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            token: entity.token,
            is_trusted: entity.is_trusted.unwrap_or(false),
            last_ip: entity.last_ip,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            is_active: entity.is_active,
            workspace_id: entity.workspace_id,
            last_seen_at: entity.last_seen_at.unwrap_or_else(Utc::now),
            max_concurrent_workers: entity.max_concurrent_workers,
            success_cooldown_secs: entity.success_cooldown_secs,
            retention_days: entity.retention_days,
        }
    }
}

#[derive(Queryable, Selectable, Insertable, Identifiable)]
#[diesel(table_name = crate::schema::devices)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DeviceEntity {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: Option<i32>,
    pub is_enabled: Option<bool>,
    pub last_status: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub workspace_id: Uuid,
    pub brand: String,
    pub password: Option<String>,
    pub timezone: Option<String>,
}

impl From<DeviceEntity> for Device {
    fn from(entity: DeviceEntity) -> Self {
        Self {
            id: entity.id,
            agent_id: entity.agent_id,
            name: entity.name,
            ip: entity.ip,
            port: entity.port.unwrap_or(0),
            is_enabled: entity.is_enabled.unwrap_or(false),
            last_status: entity.last_status.unwrap_or_else(|| "unknown".to_string()),
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            workspace_id: entity.workspace_id,
            brand: entity.brand,
            password: entity.password,
            timezone: entity.timezone.unwrap_or_else(|| "UTC".to_string()),
            max_concurrent_workers: 0,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::devices)]
pub struct UpdateDieselDevice {
    pub name: String,
    pub ip: String,
    pub port: Option<i32>,
    pub is_enabled: Option<bool>,
    pub brand: String,
    pub password: Option<String>,
    pub timezone: Option<String>,
}
