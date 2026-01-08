use serde::{Deserialize, Serialize};
use serde_json::Value;

// Database Models
#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct RawAttendanceLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: chrono::NaiveDateTime,
    pub status: Option<i64>, // SQLite INTEGER is i64
    pub punch: Option<i64>,  // SQLite INTEGER is i64
    pub is_synced: i64,      // SQLite INTEGER
    pub created_at: Option<chrono::NaiveDateTime>,
}

#[derive(Debug, sqlx::FromRow)]
#[allow(dead_code)]
pub struct DeviceConfig {
    pub id: String,
    pub name: Option<String>,
    pub ip: String,
    pub port: i64,
    pub last_seen: Option<chrono::NaiveDateTime>,
    pub is_active: Option<bool>,
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
    },
    ConfigPush {
        data: Value,
    },
    Command {
        id: String,
        method: String,
        params: Option<Value>,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AttendanceData {
    pub user_id: String,
    pub timestamp: String,
    pub status: Option<i32>,
    pub punch: Option<i32>,
}