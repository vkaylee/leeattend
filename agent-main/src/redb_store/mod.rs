mod attendance_ops;
mod device_ops;
mod maintenance_ops;
mod sync_ops;
mod types;

pub use types::{StoreError, StoredDevice, StoredFailedLog, StoredLog};

use parking_lot::RwLock;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

// ── Table Definitions ──────────────────────────────────────────────────────

/// Primary attendance log storage: ID → bincode(StoredLog)
const ATTENDANCE_LOGS: TableDefinition<i64, &[u8]> = TableDefinition::new("attendance_logs");

/// Dedup signature index: FxHash(device_id,user_id,ts,punch,status) → timestamp_utc
const SIGNATURES: TableDefinition<u64, i64> = TableDefinition::new("signatures");

/// Composite key index for ACK correlation: hash → log_id
const COMPOSITE_INDEX: TableDefinition<u64, i64> = TableDefinition::new("composite_index");

/// Sync state index: log_id → sync_state (0=unsynced, 1=synced, 2=in_flight)
const SYNC_INDEX: TableDefinition<i64, u8> = TableDefinition::new("sync_index");

/// Device configuration: device_id → bincode(StoredDevice)
const DEVICES: TableDefinition<&str, &[u8]> = TableDefinition::new("devices");

/// Agent key-value config: key → value
const AGENT_CONFIG: TableDefinition<&str, &str> = TableDefinition::new("agent_config");

/// Dead letter queue: id → bincode(StoredFailedLog)
const FAILED_LOGS: TableDefinition<i64, &[u8]> = TableDefinition::new("failed_logs");

/// Parsing errors: auto-id → bincode(error_record)
const PARSING_ERRORS: TableDefinition<i64, &[u8]> = TableDefinition::new("parsing_errors");

// ── RedbStore ──────────────────────────────────────────────────────────────

/// Unified persistent store backed by Redb.
/// Replaces both InMemoryStore (DashMap) and Database (SQLite).
pub struct RedbStore {
    db: RwLock<Database>,
    next_id: Arc<AtomicI64>,
    signature_ttl_days: i64,
}

impl RedbStore {
    /// Open or create a Redb database at the given path.
    pub fn new(path: &str, signature_ttl_days: i64) -> Result<Self, redb::Error> {
        let db = if path == ":memory:" || path.contains("mode=memory") {
            // In-memory for tests
            Database::builder().create_with_backend(redb::backends::InMemoryBackend::new())?
        } else {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(path).parent()
                && !parent.as_os_str().is_empty()
            {
                let _ = std::fs::create_dir_all(parent);
            }
            Database::create(path)?
        };

        // Create all tables (idempotent)
        {
            let txn = db.begin_write()?;
            {
                let _ = txn.open_table(ATTENDANCE_LOGS)?;
                let _ = txn.open_table(SIGNATURES)?;
                let _ = txn.open_table(COMPOSITE_INDEX)?;
                let _ = txn.open_table(SYNC_INDEX)?;
                let _ = txn.open_table(DEVICES)?;
                let _ = txn.open_table(AGENT_CONFIG)?;
                let _ = txn.open_table(FAILED_LOGS)?;
                let _ = txn.open_table(PARSING_ERRORS)?;
            }
            txn.commit()?;
        }

        // Seed ID counter from existing data
        let max_id = {
            let txn = db.begin_read()?;
            let table = txn.open_table(ATTENDANCE_LOGS)?;
            table.last()?.map(|(k, _)| k.value()).unwrap_or(0)
        };

        Ok(Self {
            db: RwLock::new(db),
            next_id: Arc::new(AtomicI64::new(max_id + 1001)), // Safety gap
            signature_ttl_days,
        })
    }

    /// Get a reference to the underlying Redb database lock.
    pub const fn db(&self) -> &RwLock<Database> {
        &self.db
    }

    /// Get the configured signature TTL in days.
    pub const fn signature_ttl_days(&self) -> i64 {
        self.signature_ttl_days
    }

    /// Allocate the next unique ID.
    fn next_id(&self) -> i64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Compute 64-bit FxHash signature for deduplication.
    fn compute_signature(
        device_id: &str,
        user_id: &str,
        timestamp: i64,
        punch: Option<i32>,
        status: Option<i32>,
    ) -> u64 {
        // Use a simple FNV-1a style hasher (no external dependency needed)
        let mut h: u64 = 0xcbf29ce484222325;
        for b in device_id.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for b in user_id.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        for b in timestamp.to_le_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        // Hash punch
        match punch {
            Some(p) => {
                h ^= 1u64;
                h = h.wrapping_mul(0x100000001b3);
                for b in p.to_le_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
            }
            None => {
                h ^= 0u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        // Hash status
        match status {
            Some(s) => {
                h ^= 1u64;
                h = h.wrapping_mul(0x100000001b3);
                for b in s.to_le_bytes() {
                    h ^= b as u64;
                    h = h.wrapping_mul(0x100000001b3);
                }
            }
            None => {
                h ^= 0u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h
    }

    /// Compute composite key hash (same algorithm as signature but for ACK lookup).
    fn compute_composite_hash(
        device_id: &str,
        user_id: &str,
        timestamp: i64,
        punch: Option<i32>,
        status: Option<i32>,
    ) -> u64 {
        // Use different seed to avoid collision with signature hash
        let sig = Self::compute_signature(device_id, user_id, timestamp, punch, status);
        sig.wrapping_mul(0x9e3779b97f4a7c15) // Mix with golden ratio
    }

    /// Count total attendance logs.
    pub fn count_total(&self) -> Result<usize, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(format!("{e}")))?;
        let table = txn
            .open_table(ATTENDANCE_LOGS)
            .map_err(|e| StoreError::DbError(format!("{e}")))?;
        Ok(table
            .len()
            .map_err(|e| StoreError::DbError(format!("{e}")))? as usize)
    }

    /// Count signatures.
    pub fn count_signatures(&self) -> Result<usize, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(format!("{e}")))?;
        let table = txn
            .open_table(SIGNATURES)
            .map_err(|e| StoreError::DbError(format!("{e}")))?;
        Ok(table
            .len()
            .map_err(|e| StoreError::DbError(format!("{e}")))? as usize)
    }

    /// Check if a specific signature exists.
    pub fn has_signature(
        &self,
        device_id: &str,
        user_id: &str,
        timestamp: i64,
        punch: Option<i32>,
        status: Option<i32>,
    ) -> Result<bool, StoreError> {
        let sig = Self::compute_signature(device_id, user_id, timestamp, punch, status);
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(format!("{e}")))?;
        let table = txn
            .open_table(SIGNATURES)
            .map_err(|e| StoreError::DbError(format!("{e}")))?;
        Ok(table
            .get(sig)
            .map_err(|e| StoreError::DbError(format!("{e}")))?
            .is_some())
    }

    /// Parse composite key string and look up log ID.
    /// Format: "device_id:user_id:timestamp_unix:punch:status"
    pub fn get_id_by_composite_key(&self, composite_key: &str) -> Option<i64> {
        let mut parts = composite_key.rsplitn(4, ':');

        let status_str = parts.next()?;
        let punch_str = parts.next()?;
        let timestamp_str = parts.next()?;
        let rest = parts.next()?;

        let status: Option<i32> = status_str.parse().ok().filter(|&v| v != -1);
        let punch: Option<i32> = punch_str.parse().ok().filter(|&v| v != -1);
        let timestamp_unix: i64 = timestamp_str.parse().ok()?;

        let (device_id, user_id) = if rest.len() > 36 && rest.as_bytes()[36] == b':' {
            (&rest[..36], &rest[37..])
        } else {
            match rest.rsplit_once(':') {
                Some((d, u)) => (d, u),
                None => return None,
            }
        };

        let hash = Self::compute_composite_hash(device_id, user_id, timestamp_unix, punch, status);
        let txn = self.db.read().begin_read().ok()?;
        let table: redb::ReadOnlyTable<u64, i64> = txn.open_table(COMPOSITE_INDEX).ok()?;
        table
            .get(hash)
            .ok()?
            .map(|v: redb::AccessGuard<'_, i64>| v.value())
    }

    /// Compact the database file to reclaim unused space.
    pub fn compact(&self) -> Result<bool, StoreError> {
        self.db
            .write()
            .compact()
            .map_err(|e| StoreError::DbError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> RedbStore {
        RedbStore::new(":memory:", 90).expect("in-memory store")
    }

    #[test]
    fn test_new_in_memory() {
        let store = make_store();
        assert_eq!(store.count_total().unwrap(), 0);
        assert_eq!(store.count_signatures().unwrap(), 0);
        assert_eq!(store.signature_ttl_days(), 90);
    }

    #[test]
    fn test_count_total_empty() {
        let store = make_store();
        assert_eq!(store.count_total().unwrap(), 0);
    }

    #[test]
    fn test_count_signatures_empty() {
        let store = make_store();
        assert_eq!(store.count_signatures().unwrap(), 0);
    }

    #[test]
    fn test_has_signature_empty() {
        let store = make_store();
        assert!(
            !store
                .has_signature("DEV", "USR", 100, Some(1), Some(1))
                .unwrap()
        );
    }

    #[test]
    fn test_has_signature_after_insert() {
        let store = make_store();
        let data = crate::models::AttendanceData {
            user_id: "U1".into(),
            timestamp: "2026-01-01 10:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("DEV1", &data, 5000).unwrap();
        assert!(
            store
                .has_signature("DEV1", "U1", 5000, Some(1), Some(1))
                .unwrap()
        );
        // Different params = no signature
        assert!(
            !store
                .has_signature("DEV1", "U1", 5001, Some(1), Some(1))
                .unwrap()
        );
    }

    #[test]
    fn test_compute_signature_deterministic() {
        let sig1 = RedbStore::compute_signature("DEV", "USR", 100, Some(1), Some(0));
        let sig2 = RedbStore::compute_signature("DEV", "USR", 100, Some(1), Some(0));
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_compute_signature_different_inputs() {
        let sig1 = RedbStore::compute_signature("DEV1", "USR", 100, Some(1), Some(0));
        let sig2 = RedbStore::compute_signature("DEV2", "USR", 100, Some(1), Some(0));
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_compute_signature_none_vs_some() {
        let sig1 = RedbStore::compute_signature("D", "U", 100, None, None);
        let sig2 = RedbStore::compute_signature("D", "U", 100, Some(0), Some(0));
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_get_id_by_composite_key() {
        let store = make_store();
        let data = crate::models::AttendanceData {
            user_id: "USR1".into(),
            timestamp: "2026-01-01 10:00:00".into(),
            status: Some(1),
            punch: Some(2),
        };
        let id = store.insert("DEV1", &data, 50000).unwrap();

        let key = "DEV1:USR1:50000:2:1".to_string();
        let found = store.get_id_by_composite_key(&key);
        assert_eq!(found, Some(id));
    }

    #[test]
    fn test_get_id_by_composite_key_not_found() {
        let store = make_store();
        assert_eq!(store.get_id_by_composite_key("X:Y:0:1:1"), None);
    }

    #[test]
    fn test_get_id_by_composite_key_invalid_format() {
        let store = make_store();
        assert_eq!(store.get_id_by_composite_key("invalid"), None);
    }

    #[test]
    fn test_next_id_increments() {
        let store = make_store();
        let id1 = store.next_id();
        let id2 = store.next_id();
        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_compute_composite_hash_differs_from_signature() {
        let sig = RedbStore::compute_signature("D", "U", 100, Some(1), Some(0));
        let comp = RedbStore::compute_composite_hash("D", "U", 100, Some(1), Some(0));
        assert_ne!(sig, comp);
    }
}
