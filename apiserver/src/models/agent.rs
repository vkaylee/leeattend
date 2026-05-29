use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AgentStatus {
    pub agent_id: String,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
    pub devices: Vec<DeviceStatus>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct AuthorizedAgent {
    pub id: String,
    pub name: String,
    pub token: String,
    pub is_trusted: bool,
    pub last_ip: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub is_active: bool,
    pub workspace_id: Uuid,
    pub max_concurrent_workers: i32,
    pub success_cooldown_secs: i32,
    pub retention_days: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateAuthorizedAgentRequest {
    pub name: Option<String>,
    pub token: Option<String>,
    pub is_trusted: Option<bool>,
    pub is_active: Option<bool>,
    pub max_concurrent_workers: Option<i32>,
    pub success_cooldown_secs: Option<i32>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateAuthorizedAgentRequest {
    pub id: String,
    pub name: String,
    pub token: String,
    pub workspace_id: Option<Uuid>,
    pub max_concurrent_workers: Option<i32>,
    pub success_cooldown_secs: Option<i32>,
    pub retention_days: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TrustAgentRequest {
    pub is_trusted: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Device {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub password: Option<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub is_enabled: bool,
    pub last_status: String,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Uuid,
    pub max_concurrent_workers: i32,
}

fn default_timezone() -> String {
    "UTC".to_string()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDeviceRequest {
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: Option<i32>,
    pub brand: Option<String>,
    pub password: Option<String>,
    pub timezone: Option<String>,
    pub workspace_id: Option<Uuid>,
    pub max_concurrent_workers: Option<i32>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub brand: Option<String>,
    pub password: Option<String>,
    pub timezone: Option<String>,
    pub is_enabled: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timezone() {
        assert_eq!(default_timezone(), "UTC");
    }

    #[test]
    fn test_device_status_serialization() {
        let status = DeviceStatus {
            device_id: "dev-1".to_string(),
            status: "online".to_string(),
            last_sync: Some(Utc::now()),
            error: None,
            info: None,
        };
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains("dev-1"));
        assert!(serialized.contains("online"));
    }

    #[test]
    fn test_agent_status_serialization() {
        let status = AgentStatus {
            agent_id: "agent-1".to_string(),
            last_seen: Utc::now(),
            is_online: true,
            devices: vec![],
        };
        let serialized = serde_json::to_string(&status).unwrap();
        assert!(serialized.contains("agent-1"));
        assert!(serialized.contains("is_online"));
    }
}
