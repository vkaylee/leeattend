use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GenericResponse {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: String,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentStatus {
    pub agent_id: String,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
    pub devices: Vec<DeviceStatus>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Device {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub is_enabled: bool,
    pub last_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeviceRequest {
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub is_enabled: Option<bool>,
}