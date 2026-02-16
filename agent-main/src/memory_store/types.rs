use chrono::NaiveDateTime;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use rustc_hash::FxHasher;

/// In-memory log with sync status and persistence tracking
#[derive(Clone, Debug)]
pub struct MemoryLog {
    pub id: i64,
    pub device_id: Arc<str>,
    pub user_id: Arc<str>,
    pub timestamp: i64,  // Unix timestamp (UTC seconds)
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,           // 0=unsynced, 1=synced, 2=in_flight
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub retry_count: i64,
    pub persisted: bool,          // true if already on disk
}

impl MemoryLog {
    /// Generate composite key string for ACK correlation
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

    /// Compute hash of composite key components for memory index
    /// Uses FxHasher for fast, deterministic hashing
    pub fn composite_key_hash(&self) -> u64 {
        Self::compute_composite_hash(&self.device_id, &self.user_id, self.timestamp, self.punch, self.status)
    }

    /// Static method to compute composite key hash from components
    pub(crate) fn compute_composite_hash(device_id: &str, user_id: &str, timestamp: i64, punch: Option<i32>, status: Option<i32>) -> u64 {
        let mut hasher = FxHasher::default();
        device_id.hash(&mut hasher);
        user_id.hash(&mut hasher);
        timestamp.hash(&mut hasher);
        punch.hash(&mut hasher);
        status.hash(&mut hasher);
        hasher.finish()
    }
}

/// Error types for store operations
#[derive(Debug)]
pub enum StoreError {
    CapacityExceeded,
    InvalidTimestamp,
    NotFound,
    Duplicate,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::CapacityExceeded => write!(f, "Store capacity exceeded"),
            StoreError::InvalidTimestamp => write!(f, "Invalid timestamp format"),
            StoreError::NotFound => write!(f, "Log not found"),
            StoreError::Duplicate => write!(f, "Duplicate log rejected"),
        }
    }
}

impl std::error::Error for StoreError {}
