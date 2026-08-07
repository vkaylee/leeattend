use serde::{Deserialize, Serialize};
use serde_json::Value;

// Config Push Models
const fn default_enabled() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub password: Option<String>,
    pub timezone: Option<String>,
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConfigData {
    pub devices: Vec<DeviceConfigPush>,
    pub max_concurrent_workers: Option<i32>,
    pub success_cooldown_secs: Option<i32>,
    pub retention_days: Option<i32>,
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
        #[serde(default)]
        attempt: Option<u32>,
        #[serde(default)]
        timestamp: Option<String>,
    },
    HeartbeatAck {
        server_time: String,
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
    CommandResponse {
        command_id: String,
        status: String,
        data: Option<Value>,
        error: Option<String>,
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
