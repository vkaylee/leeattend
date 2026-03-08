use crate::redb_store::{RedbStore, StoredLog, StoreError};
use crate::models::AttendanceData;
use std::sync::Arc;
use log::info;

/// HybridDatabase — now a thin wrapper around RedbStore.
/// Maintains the same public API for backward compatibility.
/// Redb handles persistence automatically (no checkpoint/flush needed).
#[derive(Clone)]
pub struct HybridDatabase {
    store: Arc<RedbStore>,
}

impl HybridDatabase {
    /// Create a new HybridDatabase backed by Redb.
    /// `db_path` is the path to the Redb file (or ":memory:" for testing).
    /// `_max_memory_logs` is kept for API compat but ignored (Redb manages its own cache).
    pub fn new(db_path: &str, _max_memory_logs: usize, signature_ttl_days: i64) -> Result<Self, Box<dyn std::error::Error>> {
        // Convert SQLite-style URLs to plain paths for Redb
        let redb_path = Self::normalize_path(db_path);
        let store = RedbStore::new(&redb_path, signature_ttl_days)
            .map_err(|e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::other(e.to_string())) })?;
        Ok(Self { store: Arc::new(store) })
    }

    /// Get direct access to the underlying RedbStore.
    pub fn store(&self) -> &RedbStore {
        &self.store
    }

    /// Normalize database path from SQLite-style URL to plain file path.
    fn normalize_path(url: &str) -> String {
        if url == ":memory:" || url == "sqlite::memory:" || url.contains("mode=memory") {
            return ":memory:".to_string();
        }
        // Strip "sqlite:" prefix and query params
        let path = url.strip_prefix("sqlite:").unwrap_or(url);
        let path = path.split('?').next().unwrap_or(path);
        // Change extension from .db to .redb
        if let Some(stripped) = path.strip_suffix(".db") {
            format!("{stripped}.redb")
        } else {
            format!("{path}.redb")
        }
    }

    /// Save attendance — converts local time to UTC and inserts into Redb.
    pub fn save_raw_attendance(&self, device_id: &str, data: AttendanceData) -> Result<i64, StoreError> {
        let tz = self.store.get_device_timezone(device_id)?;
        let timestamp_utc = RedbStore::parse_local_to_utc_timestamp(&data.timestamp, tz)
            .map_err(|_| StoreError::InvalidTimestamp)?;
        self.store.insert(device_id, &data, timestamp_utc)
    }

    /// Batch save — converts timestamps and inserts into Redb.
    pub fn save_raw_attendance_batch(&self, batch: Vec<(Arc<String>, AttendanceData)>) -> Result<usize, StoreError> {
        let mut batch_with_ts: Vec<(String, AttendanceData, i64)> = Vec::new();
        for (device_id, data) in batch {
            let tz = self.store.get_device_timezone(&device_id).unwrap_or(chrono_tz::UTC);
            if let Ok(ts) = RedbStore::parse_local_to_utc_timestamp(&data.timestamp, tz) {
                batch_with_ts.push((device_id.to_string(), data, ts));
            }
        }
        self.store.insert_batch(&batch_with_ts)
    }

    /// Get unsynced logs from Redb.
    pub fn get_unsynced_logs(&self, limit: usize) -> Vec<StoredLog> {
        self.store.get_unsynced(limit).unwrap_or_default()
    }

    /// Count unsynced logs.
    pub fn count_unsynced_logs(&self) -> usize {
        self.store.count_unsynced().unwrap_or(0)
    }

    /// Mark logs as in-flight.
    pub fn mark_in_flight(&self, ids: &[i64]) {
        if let Err(e) = self.store.mark_in_flight(ids) {
            log::error!("Failed to mark logs as in-flight: {e}");
        }
    }

    /// Mark logs as synced (removes from active queue, keeps signature).
    pub fn mark_synced(&self, ids: &[i64]) {
        let _ = self.store.mark_synced(ids);
    }

    /// Reset log to unsynced (failed ACK).
    pub fn reset_to_unsynced(&self, id: i64) {
        let _ = self.store.reset_to_unsynced(id);
    }

    /// Mark synced by composite key (for ACK handling).
    pub fn mark_synced_by_key(&self, composite_key: &str) -> bool {
        self.store.mark_synced_by_key(composite_key)
    }

    /// Reset to unsynced by composite key (for failed ACK).
    pub fn reset_to_unsynced_by_key(&self, composite_key: &str) -> bool {
        self.store.reset_to_unsynced_by_key(composite_key)
    }

    /// Evict signatures older than TTL. Returns (evicted, before, after).
    pub fn evict_old_signatures(&self) -> (usize, usize, usize) {
        self.store.evict_old_signatures().unwrap_or((0, 0, 0))
    }



    /// Unload device memory — removes signatures and logs for disabled device.
    pub fn unload_device_memory(&self, device_id: &str) -> Result<(), StoreError> {
        let removed = self.store.remove_signatures_for_device(device_id)?;
        info!("HybridDatabase: Unloaded {removed} entries for disabled device '{device_id}'");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attendance(user_id: &str, timestamp: &str) -> AttendanceData {
        AttendanceData {
            user_id: user_id.to_string(),
            timestamp: timestamp.to_string(),
            status: Some(1),
            punch: Some(0),
        }
    }

    fn setup_db() -> HybridDatabase {
        let db = HybridDatabase::new(":memory:", 1000, 90).unwrap();
        // Setup a device with UTC timezone for test
        db.store().upsert_device("DEV001", "Test Device", "192.168.1.1", 4370, "zkteco", "UTC", None, true, 123456).unwrap();
        db
    }

    #[test]
    fn test_insert_and_get_unsynced() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001", data).unwrap();
        assert!(id > 0);

        let logs = db.get_unsynced_logs(10);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "user1");
    }

    #[test]
    fn test_deduplication() {
        let db = setup_db();
        let data1 = make_attendance("user1", "2025-06-01 08:00:00");
        let data2 = make_attendance("user1", "2025-06-01 08:00:00");

        let r1 = db.save_raw_attendance("DEV001", data1);
        let r2 = db.save_raw_attendance("DEV001", data2);

        assert!(r1.is_ok());
        assert!(matches!(r2, Err(StoreError::Duplicate)));
    }

    #[test]
    fn test_mark_in_flight_and_synced() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001", data).unwrap();

        db.mark_in_flight(&[id]);
        assert_eq!(db.count_unsynced_logs(), 0);

        db.mark_synced(&[id]);
        assert_eq!(db.count_unsynced_logs(), 0);
        assert_eq!(db.get_unsynced_logs(10).len(), 0);
    }

    #[test]
    fn test_reset_to_unsynced() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001", data).unwrap();

        db.mark_in_flight(&[id]);
        assert_eq!(db.count_unsynced_logs(), 0);

        db.reset_to_unsynced(id);
        assert_eq!(db.count_unsynced_logs(), 1);
    }

    #[test]
    fn test_composite_key_ack() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001", data).unwrap();

        let logs = db.get_unsynced_logs(10);
        let key = logs[0].composite_key();

        db.mark_in_flight(&[id]);
        assert!(db.mark_synced_by_key(&key));
        assert_eq!(db.count_unsynced_logs(), 0);
    }

    #[test]
    fn test_batch_insert() {
        let db = setup_db();
        let batch = vec![
            (Arc::new("DEV001".to_string()), make_attendance("user1", "2025-06-01 08:00:00")),
            (Arc::new("DEV001".to_string()), make_attendance("user2", "2025-06-01 08:01:00")),
            (Arc::new("DEV001".to_string()), make_attendance("user1", "2025-06-01 08:00:00")), // duplicate
        ];

        let inserted = db.save_raw_attendance_batch(batch).unwrap();
        assert_eq!(inserted, 2);
        assert_eq!(db.count_unsynced_logs(), 2);
    }

    #[test]
    fn test_fifo_ordering() {
        let db = setup_db();
        db.save_raw_attendance("DEV001", make_attendance("user1", "2025-06-01 08:00:00")).unwrap();
        db.save_raw_attendance("DEV001", make_attendance("user2", "2025-06-01 08:01:00")).unwrap();
        db.save_raw_attendance("DEV001", make_attendance("user3", "2025-06-01 08:02:00")).unwrap();

        let logs = db.get_unsynced_logs(10);
        assert_eq!(logs.len(), 3);
        assert!(logs[0].id < logs[1].id);
        assert!(logs[1].id < logs[2].id);
    }

    #[test]
    fn test_device_management() {
        let db = HybridDatabase::new(":memory:", 1000, 90).unwrap();

        db.store().upsert_device("DEV001", "Device 1", "192.168.1.1", 4370, "zkteco", "Asia/Ho_Chi_Minh", None, true, 123456).unwrap();
        db.store().upsert_device("DEV002", "Device 2", "192.168.1.2", 4370, "zkteco", "UTC", None, false, 123456).unwrap();

        let active = db.store().get_active_devices().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "DEV001");
    }

    #[test]
    fn test_agent_config() {
        let db = HybridDatabase::new(":memory:", 1000, 90).unwrap();

        db.store().upsert_agent_config("max_concurrent_workers", "5").unwrap();
        let val = db.store().get_agent_config("max_concurrent_workers").unwrap();
        assert_eq!(val, Some("5".to_string()));

        let missing = db.store().get_agent_config("nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_eviction() {
        let db = HybridDatabase::new(":memory:", 1000, 1).unwrap(); // 1 day TTL
        db.store().upsert_device("DEV001", "Test", "192.168.1.1", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

        // Insert log with current timestamp (within TTL)
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        db.save_raw_attendance("DEV001", data).unwrap();

        let sig_before = db.store().count_signatures().unwrap();
        let (evicted, _, _) = db.evict_old_signatures();
        // Old signatures (before threshold) should be evicted
        // Since test inserts happen at "2025-06-01" which is in the past, they should be evicted
        assert!(evicted > 0 || sig_before == 0);
    }
}
