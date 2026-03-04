use serde::{Serialize, Deserialize};

/// Stored attendance log in Redb (serialized with bincode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: i64,         // UTC Unix timestamp
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: u8,          // 0=unsynced, 1=synced, 2=in_flight
    pub created_at: i64,        // UTC Unix timestamp
    pub updated_at: Option<i64>, // UTC Unix timestamp
    pub retry_count: i64,
}

impl StoredLog {
    /// Generate composite key string for ACK correlation.
    /// Format: "device_id:user_id:timestamp_unix:punch:status"
    pub fn composite_key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.device_id,
            self.user_id,
            self.timestamp,
            self.punch.unwrap_or(-1),
            self.status.unwrap_or(-1)
        )
    }

    /// Encode to bincode bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("StoredLog serialization should never fail")
    }

    /// Decode from bincode bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoreError> {
        bincode::deserialize(bytes).map_err(|e| StoreError::DeserializeError(e.to_string()))
    }
}

/// Stored device configuration in Redb.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDevice {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: i64,
    pub brand: String,
    pub timezone: String,
    pub last_seen: Option<i64>,
    pub is_active: bool,
    pub mac: Option<String>,
    pub serial: Option<String>,
    pub firmware: Option<String>,
    pub platform: Option<String>,
    pub hw_name: Option<String>,
    pub users_count: Option<i64>,
    pub logs_count: Option<i64>,
    pub users_capacity: Option<i64>,
    pub logs_capacity: Option<i64>,
    pub password: Option<String>,
    pub device_time: Option<String>,
    pub last_info_update: Option<i64>,
}

impl StoredDevice {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("StoredDevice serialization should never fail")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoreError> {
        bincode::deserialize(bytes).map_err(|e| StoreError::DeserializeError(e.to_string()))
    }
}

/// Stored failed log in Redb (dead letter queue).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFailedLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: i64,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub failed_at: i64,
    pub failure_reason: String,
    pub retry_attempts: i64,
    pub first_attempt_at: Option<i64>,
    pub last_attempt_at: Option<i64>,
}

impl StoredFailedLog {
    pub fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).expect("StoredFailedLog serialization should never fail")
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, StoreError> {
        bincode::deserialize(bytes).map_err(|e| StoreError::DeserializeError(e.to_string()))
    }
}

/// Error types for store operations.
#[derive(Debug)]
pub enum StoreError {
    CapacityExceeded,
    InvalidTimestamp,
    NotFound,
    Duplicate,
    DbError(String),
    DeserializeError(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::CapacityExceeded => write!(f, "Store capacity exceeded"),
            StoreError::InvalidTimestamp => write!(f, "Invalid timestamp format"),
            StoreError::NotFound => write!(f, "Log not found"),
            StoreError::Duplicate => write!(f, "Duplicate log rejected"),
            StoreError::DbError(s) => write!(f, "Database error: {s}"),
            StoreError::DeserializeError(s) => write!(f, "Deserialization error: {s}"),
        }
    }
}

impl std::error::Error for StoreError {}
