use serde::{Deserialize, Serialize};
use serde_json::Value;

// Database Models
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct RawAttendanceLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: i64,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub updated_at: Option<chrono::NaiveDateTime>,
    pub retry_count: i64,
}

/// Minimal struct for signature loading (Phase 3)
#[derive(Debug, sqlx::FromRow)]
pub struct SignatureRow {
    pub device_id: String,
    pub user_id: String,
    pub timestamp: i64,
    pub punch: Option<i32>,
    pub status: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct FailedLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: i64,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub failed_at: chrono::NaiveDateTime,
    pub failure_reason: String,
    pub retry_attempts: i64,
    pub first_attempt_at: Option<chrono::NaiveDateTime>,
    pub last_attempt_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct DeviceConfig {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: i64,
    pub brand: String,
    pub timezone: String,
    pub last_seen: Option<chrono::NaiveDateTime>,
    pub is_active: Option<bool>,
    pub mac: Option<String>,
    pub serial: Option<String>,
    pub firmware: Option<String>,
    pub platform: Option<String>,
    pub hw_name: Option<String>,
    pub users_count: Option<i64>,
    pub logs_count: Option<i64>,
    pub last_info_update: Option<chrono::NaiveDateTime>,
}

// Config Push Models
fn default_enabled() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub timezone: Option<String>,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigData {
    pub devices: Vec<DeviceConfigPush>,
    pub max_concurrent_workers: Option<i32>,
}

// WebSocket / API Models
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CentralMessage {
    Handshake {
        agent_id: String,
        token: String,
        timestamp: String,
    },
    Heartbeat {
        agent_id: String,
        timestamp: String,
    },
    Attendance {
        msg_id: String,
        agent_id: String,
        device_id: String,
        data: AttendanceData,
        timestamp: String,
    },
    Ack {
        msg_id: String,
        status: Option<String>,
        error: Option<String>,
    },
    ConfigPush {
        data: ConfigData,
    },
    StatusUpdate {
        agent_id: String,
        devices: Vec<DeviceStatus>,
        timestamp: String,
    },
    Command {
        id: String,
        method: String,
        device_id: Option<String>,
        data: Option<Value>,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceStatus {
    pub device_id: String,
    pub status: String,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
    pub info: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttendanceData {
    pub user_id: String,
    pub timestamp: String,
    pub status: Option<i32>,
    pub punch: Option<i32>,
}
