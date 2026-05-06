#![allow(clippy::expect_used)]
use serde::{Deserialize, Serialize};

/// Stored attendance log in Redb (serialized with bincode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: i64, // UTC Unix timestamp
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: u8,           // 0=unsynced, 1=synced, 2=in_flight
    pub created_at: i64,         // UTC Unix timestamp
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
    /// Detected checksum mode from last successful connect.
    /// `Some(true)` = legacy, `Some(false)` = modern, `None` = not yet detected.
    /// Backward compatible: old data without this field deserializes as `None`.
    #[serde(default)]
    pub use_legacy_checksum: Option<bool>,
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
    Internal(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CapacityExceeded => write!(f, "Store capacity exceeded"),
            Self::InvalidTimestamp => write!(f, "Invalid timestamp format"),
            Self::NotFound => write!(f, "Log not found"),
            Self::Duplicate => write!(f, "Duplicate log rejected"),
            Self::DbError(s) => write!(f, "Database error: {s}"),
            Self::DeserializeError(s) => write!(f, "Deserialization error: {s}"),
            Self::Internal(s) => write!(f, "Internal error: {s}"),
        }
    }
}

impl std::error::Error for StoreError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stored_log_roundtrip() {
        let log = StoredLog {
            id: 42,
            device_id: "DEV1".into(),
            user_id: "USER1".into(),
            timestamp: 1700000000,
            status: Some(1),
            punch: Some(0),
            is_synced: 0,
            created_at: 1700000000,
            updated_at: None,
            retry_count: 0,
        };
        let bytes = log.to_bytes();
        let decoded = StoredLog::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.id, 42);
        assert_eq!(decoded.device_id, "DEV1");
        assert_eq!(decoded.user_id, "USER1");
        assert_eq!(decoded.timestamp, 1700000000);
        assert_eq!(decoded.status, Some(1));
        assert_eq!(decoded.punch, Some(0));
    }

    #[test]
    fn test_stored_log_composite_key() {
        let log = StoredLog {
            id: 1,
            device_id: "DEV".into(),
            user_id: "USR".into(),
            timestamp: 100,
            status: Some(1),
            punch: Some(2),
            is_synced: 0,
            created_at: 0,
            updated_at: None,
            retry_count: 0,
        };
        assert_eq!(log.composite_key(), "DEV:USR:100:2:1");
    }

    #[test]
    fn test_stored_log_composite_key_none_values() {
        let log = StoredLog {
            id: 1,
            device_id: "D".into(),
            user_id: "U".into(),
            timestamp: 50,
            status: None,
            punch: None,
            is_synced: 0,
            created_at: 0,
            updated_at: None,
            retry_count: 0,
        };
        assert_eq!(log.composite_key(), "D:U:50:-1:-1");
    }

    #[test]
    fn test_stored_device_roundtrip() {
        let device = StoredDevice {
            id: "DEV1".into(),
            name: "Test".into(),
            ip: "1.2.3.4".into(),
            port: 4370,
            brand: "zkteco".into(),
            timezone: "UTC".into(),
            last_seen: Some(100),
            is_active: true,
            mac: Some("AA:BB".into()),
            serial: None,
            firmware: None,
            platform: None,
            hw_name: None,
            users_count: Some(50),
            logs_count: Some(1000),
            users_capacity: None,
            logs_capacity: None,
            password: None,
            device_time: None,
            last_info_update: None,
            use_legacy_checksum: None,
        };
        let bytes = device.to_bytes();
        let decoded = StoredDevice::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.id, "DEV1");
        assert_eq!(decoded.mac, Some("AA:BB".into()));
        assert_eq!(decoded.users_count, Some(50));
    }

    #[test]
    fn test_stored_failed_log_roundtrip() {
        let fl = StoredFailedLog {
            id: 99,
            device_id: "D".into(),
            user_id: "U".into(),
            timestamp: 500,
            status: Some(1),
            punch: Some(0),
            failed_at: 600,
            failure_reason: "timeout".into(),
            retry_attempts: 5,
            first_attempt_at: Some(100),
            last_attempt_at: Some(500),
        };
        let bytes = fl.to_bytes();
        let decoded = StoredFailedLog::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.id, 99);
        assert_eq!(decoded.failure_reason, "timeout");
        assert_eq!(decoded.retry_attempts, 5);
    }

    #[test]
    fn test_store_error_display() {
        assert_eq!(
            format!("{}", StoreError::CapacityExceeded),
            "Store capacity exceeded"
        );
        assert_eq!(
            format!("{}", StoreError::InvalidTimestamp),
            "Invalid timestamp format"
        );
        assert_eq!(format!("{}", StoreError::NotFound), "Log not found");
        assert_eq!(
            format!("{}", StoreError::Duplicate),
            "Duplicate log rejected"
        );
        assert_eq!(
            format!("{}", StoreError::DbError("test".into())),
            "Database error: test"
        );
        assert_eq!(
            format!("{}", StoreError::DeserializeError("bad".into())),
            "Deserialization error: bad"
        );
    }

    #[test]
    fn test_from_bytes_invalid_data() {
        let result = StoredLog::from_bytes(&[0, 1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_stored_log_with_updated_at() {
        let log = StoredLog {
            id: 1,
            device_id: "D".into(),
            user_id: "U".into(),
            timestamp: 100,
            status: None,
            punch: None,
            is_synced: 2,
            created_at: 50,
            updated_at: Some(75),
            retry_count: 3,
        };
        let bytes = log.to_bytes();
        let decoded = StoredLog::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.updated_at, Some(75));
        assert_eq!(decoded.retry_count, 3);
        assert_eq!(decoded.is_synced, 2);
    }
}
