use crate::database_store::DatabaseStore;
use crate::models::AttendanceData;
use crate::redb_store::{RedbStore, StoreError, StoredDevice, StoredLog};
use async_trait::async_trait;
use log::info;
use std::sync::Arc;

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
    pub fn new(
        db_path: &str,
        _max_memory_logs: usize,
        signature_ttl_days: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Convert SQLite-style URLs to plain paths for Redb
        let redb_path = Self::normalize_path(db_path);
        let store = RedbStore::new(&redb_path, signature_ttl_days).map_err(
            |e| -> Box<dyn std::error::Error> { Box::new(std::io::Error::other(e.to_string())) },
        )?;
        Ok(Self {
            store: Arc::new(store),
        })
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
    pub async fn save_raw_attendance(
        &self,
        device_id: String,
        data: AttendanceData,
    ) -> Result<i64, StoreError> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            let tz = store.get_device_timezone(&device_id)?;
            let timestamp_utc = RedbStore::parse_local_to_utc_timestamp(&data.timestamp, tz)
                .map_err(|_| StoreError::InvalidTimestamp)?;
            store.insert(&device_id, &data, timestamp_utc)
        })
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?
    }

    /// Batch save — converts timestamps and inserts into Redb.
    pub async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(Arc<String>, AttendanceData)>,
    ) -> Result<usize, StoreError> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            let mut batch_with_ts: Vec<(String, AttendanceData, i64)> = Vec::new();
            for (device_id, data) in batch {
                let tz = match store.get_device_timezone(&device_id) {
                    Ok(tz) => tz,
                    Err(error) => {
                        store.record_parsing_error(
                            &device_id,
                            &data.timestamp,
                            &format!("device timezone unavailable: {error}"),
                        )?;
                        continue;
                    }
                };

                match RedbStore::parse_local_to_utc_timestamp(&data.timestamp, tz) {
                    Ok(timestamp_utc) => {
                        batch_with_ts.push((device_id.to_string(), data, timestamp_utc));
                    }
                    Err(error) => {
                        store.record_parsing_error(&device_id, &data.timestamp, &error)?;
                    }
                }
            }
            store.insert_batch(&batch_with_ts)
        })
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?
    }

    /// Get unsynced logs from Redb.
    pub async fn get_unsynced_logs(&self, limit: usize) -> Result<Vec<StoredLog>, StoreError> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.get_unsynced(limit))
            .await
            .map_err(|e| StoreError::Internal(e.to_string()))?
    }

    /// Count unsynced logs.
    pub async fn count_unsynced_logs(&self) -> usize {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.count_unsynced().unwrap_or(0))
            .await
            .unwrap_or(0)
    }

    /// Mark logs as in-flight.
    pub async fn mark_in_flight(&self, ids: Vec<i64>) {
        let store = self.store.clone();
        if let Err(e) = tokio::task::spawn_blocking(move || store.mark_in_flight(&ids))
            .await
            .unwrap_or(Err(StoreError::Internal("Task paniced".into())))
        {
            log::error!("Failed to mark logs as in-flight: {e}");
        }
    }

    /// Mark logs as synced (removes from active queue, keeps signature).
    pub async fn mark_synced(&self, ids: Vec<i64>) {
        let store = self.store.clone();
        let _ = tokio::task::spawn_blocking(move || store.mark_synced(&ids)).await;
    }

    /// Reset log to unsynced (failed ACK).
    pub async fn reset_to_unsynced(&self, id: i64) {
        let store = self.store.clone();
        let _ = tokio::task::spawn_blocking(move || store.reset_to_unsynced(id)).await;
    }

    /// Mark synced by opaque V2 message UUID.
    pub async fn mark_synced_by_msg_id(&self, msg_id: uuid::Uuid) -> bool {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.mark_synced_by_msg_id(msg_id))
            .await
            .unwrap_or(false)
    }

    /// Mark synced by composite key (legacy ACK fallback).
    pub async fn mark_synced_by_key(&self, composite_key: String) -> bool {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.mark_synced_by_key(&composite_key))
            .await
            .unwrap_or(false)
    }

    /// Reset by opaque V2 message UUID.
    pub async fn reset_to_unsynced_by_msg_id(&self, msg_id: uuid::Uuid) -> bool {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.reset_to_unsynced_by_msg_id(msg_id))
            .await
            .unwrap_or(false)
    }

    /// Reset to unsynced by composite key (legacy ACK fallback).
    pub async fn reset_to_unsynced_by_key(&self, composite_key: String) -> bool {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.reset_to_unsynced_by_key(&composite_key))
            .await
            .unwrap_or(false)
    }

    /// Evict signatures older than TTL. Returns (evicted, before, after).
    pub async fn evict_old_signatures(&self) -> (usize, usize, usize) {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.evict_old_signatures().unwrap_or((0, 0, 0)))
            .await
            .unwrap_or((0, 0, 0))
    }

    /// Unload device memory — removes signatures and logs for disabled device.
    pub async fn unload_device_memory(&self, device_id: String) -> Result<(), StoreError> {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || {
            let removed = store.remove_signatures_for_device(&device_id)?;
            info!("HybridDatabase: Unloaded {removed} entries for disabled device '{device_id}'");
            Ok(())
        })
        .await
        .map_err(|e| StoreError::Internal(e.to_string()))?
    }
}

#[async_trait]
impl DatabaseStore for HybridDatabase {
    async fn save_raw_attendance(
        &self,
        device_id: String,
        data: AttendanceData,
    ) -> Result<i64, StoreError> {
        self.save_raw_attendance(device_id, data).await
    }

    async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(Arc<String>, AttendanceData)>,
    ) -> Result<usize, StoreError> {
        self.save_raw_attendance_batch(batch).await
    }

    async fn get_unsynced_logs(&self, limit: usize) -> Result<Vec<StoredLog>, StoreError> {
        self.get_unsynced_logs(limit).await
    }

    fn get_msg_id(&self, id: i64) -> Option<uuid::Uuid> {
        self.store.get_msg_id(id)
    }

    async fn count_unsynced_logs(&self) -> usize {
        self.count_unsynced_logs().await
    }

    async fn mark_in_flight(&self, ids: Vec<i64>) {
        self.mark_in_flight(ids).await;
    }

    async fn mark_synced(&self, ids: Vec<i64>) {
        self.mark_synced(ids).await;
    }

    async fn reset_to_unsynced(&self, id: i64) {
        self.reset_to_unsynced(id).await;
    }

    async fn mark_synced_by_key(&self, composite_key: String) -> bool {
        self.mark_synced_by_key(composite_key).await
    }

    async fn mark_synced_by_msg_id(&self, msg_id: uuid::Uuid) -> bool {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.mark_synced_by_msg_id(msg_id))
            .await
            .unwrap_or(false)
    }

    async fn reset_to_unsynced_by_key(&self, composite_key: String) -> bool {
        self.reset_to_unsynced_by_key(composite_key).await
    }

    async fn reset_to_unsynced_by_msg_id(&self, msg_id: uuid::Uuid) -> bool {
        let store = self.store.clone();
        tokio::task::spawn_blocking(move || store.reset_to_unsynced_by_msg_id(msg_id))
            .await
            .unwrap_or(false)
    }

    async fn evict_old_signatures(&self) -> (usize, usize, usize) {
        self.evict_old_signatures().await
    }

    async fn unload_device_memory(&self, device_id: String) -> Result<(), StoreError> {
        self.unload_device_memory(device_id).await
    }

    fn get_active_devices(&self) -> Result<Vec<StoredDevice>, StoreError> {
        self.store.get_active_devices()
    }

    fn get_agent_config(&self, key: &str) -> Result<Option<String>, StoreError> {
        self.store.get_agent_config(key)
    }

    fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), StoreError> {
        self.store.upsert_agent_config(key, value)
    }

    fn deactivate_devices_not_in(&self, device_ids: &[String]) -> Result<u64, StoreError> {
        self.store.deactivate_devices_not_in(device_ids)
    }

    fn upsert_device(
        &self,
        id: &str,
        name: &str,
        ip: &str,
        port: i32,
        brand: &str,
        timezone: &str,
        password: Option<&str>,
        is_enabled: bool,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        self.store.upsert_device(
            id, name, ip, port, brand, timezone, password, is_enabled, now_ts,
        )
    }

    fn get_device_metadata(&self, device_id: &str) -> Result<Option<StoredDevice>, StoreError> {
        self.store.get_device_metadata(device_id)
    }

    fn update_device_metadata(
        &self,
        device_id: &str,
        mac: &str,
        serial: &str,
        firmware: &str,
        platform: &str,
        hw_name: &str,
        users_count: i64,
        logs_count: i64,
        users_capacity: i64,
        logs_capacity: i64,
        device_time: &str,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        self.store.update_device_metadata(
            device_id,
            mac,
            serial,
            firmware,
            platform,
            hw_name,
            users_count,
            logs_count,
            users_capacity,
            logs_capacity,
            device_time,
            now_ts,
        )
    }

    fn update_device_checksum(
        &self,
        device_id: &str,
        use_legacy_checksum: bool,
    ) -> Result<(), StoreError> {
        self.store
            .update_device_checksum(device_id, use_legacy_checksum)
    }

    fn update_device_counts(
        &self,
        device_id: &str,
        users_count: i64,
        logs_count: i64,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        self.store
            .update_device_counts(device_id, users_count, logs_count, now_ts)
    }

    fn move_expired_logs_to_dlq(&self, max_retries: i64) -> Result<u64, StoreError> {
        self.store.move_expired_logs_to_dlq(max_retries)
    }

    fn reset_stale_syncing(&self, stale_timeout_secs: i64) -> Result<u64, StoreError> {
        self.store.reset_stale_syncing(stale_timeout_secs)
    }

    fn cleanup_old_synced_logs(&self, retention_days: i64) -> Result<u64, StoreError> {
        self.store.cleanup_old_synced_logs(retention_days)
    }

    fn compact(&self) -> Result<bool, StoreError> {
        self.store.compact()
    }

    fn recover_from_crash(&self) -> Result<u64, StoreError> {
        crate::crash_recovery::recover_from_crash(&self.store)
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
        db.store()
            .upsert_device(
                "DEV001",
                "Test Device",
                "192.168.1.1",
                4370,
                "zkteco",
                "UTC",
                None,
                true,
                123456,
            )
            .unwrap();
        db
    }

    #[tokio::test]
    async fn test_insert_and_get_unsynced() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001".into(), data).await.unwrap();
        assert!(id > 0);

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "user1");
    }

    #[tokio::test]
    async fn test_deduplication() {
        let db = setup_db();
        let data1 = make_attendance("user1", "2025-06-01 08:00:00");
        let data2 = make_attendance("user1", "2025-06-01 08:00:00");

        let r1 = db.save_raw_attendance("DEV001".into(), data1).await;
        let r2 = db.save_raw_attendance("DEV001".into(), data2).await;

        assert!(r1.is_ok());
        assert!(matches!(r2, Err(StoreError::Duplicate)));
    }

    #[tokio::test]
    async fn test_mark_in_flight_and_synced() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001".into(), data).await.unwrap();

        db.mark_in_flight(vec![id]).await;
        assert_eq!(db.count_unsynced_logs().await, 0);

        db.mark_synced(vec![id]).await;
        assert_eq!(db.count_unsynced_logs().await, 0);
        assert_eq!(db.get_unsynced_logs(10).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_reset_to_unsynced() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001".into(), data).await.unwrap();

        db.mark_in_flight(vec![id]).await;
        assert_eq!(db.count_unsynced_logs().await, 0);

        db.reset_to_unsynced(id).await;
        assert_eq!(db.count_unsynced_logs().await, 1);
    }

    #[tokio::test]
    async fn test_composite_key_ack() {
        let db = setup_db();
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        let id = db.save_raw_attendance("DEV001".into(), data).await.unwrap();

        let logs = db.get_unsynced_logs(10).await.unwrap();
        let key = logs[0].composite_key();

        db.mark_in_flight(vec![id]).await;
        assert!(db.mark_synced_by_key(key).await);
        assert_eq!(db.count_unsynced_logs().await, 0);
    }

    #[tokio::test]
    async fn test_batch_insert() {
        let db = setup_db();
        let batch = vec![
            (
                Arc::new("DEV001".to_string()),
                make_attendance("user1", "2025-06-01 08:00:00"),
            ),
            (
                Arc::new("DEV001".to_string()),
                make_attendance("user2", "2025-06-01 08:01:00"),
            ),
            (
                Arc::new("DEV001".to_string()),
                make_attendance("user1", "2025-06-01 08:00:00"),
            ), // duplicate
        ];

        let inserted = db.save_raw_attendance_batch(batch).await.unwrap();
        assert_eq!(inserted, 2);
        assert_eq!(db.count_unsynced_logs().await, 2);
    }

    #[tokio::test]
    async fn test_fifo_ordering() {
        let db = setup_db();
        db.save_raw_attendance(
            "DEV001".into(),
            make_attendance("user1", "2025-06-01 08:00:00"),
        )
        .await
        .unwrap();
        db.save_raw_attendance(
            "DEV001".into(),
            make_attendance("user2", "2025-06-01 08:01:00"),
        )
        .await
        .unwrap();
        db.save_raw_attendance(
            "DEV001".into(),
            make_attendance("user3", "2025-06-01 08:02:00"),
        )
        .await
        .unwrap();

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 3);
        assert!(logs[0].id < logs[1].id);
        assert!(logs[1].id < logs[2].id);
    }

    #[test]
    fn test_device_management() {
        let db = HybridDatabase::new(":memory:", 1000, 90).unwrap();

        db.store()
            .upsert_device(
                "DEV001",
                "Device 1",
                "192.168.1.1",
                4370,
                "zkteco",
                "Asia/Ho_Chi_Minh",
                None,
                true,
                123456,
            )
            .unwrap();
        db.store()
            .upsert_device(
                "DEV002",
                "Device 2",
                "192.168.1.2",
                4370,
                "zkteco",
                "UTC",
                None,
                false,
                123456,
            )
            .unwrap();

        let active = db.store().get_active_devices().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "DEV001");
    }

    #[test]
    fn test_agent_config() {
        let db = HybridDatabase::new(":memory:", 1000, 90).unwrap();

        db.store()
            .upsert_agent_config("max_concurrent_workers", "5")
            .unwrap();
        let val = db
            .store()
            .get_agent_config("max_concurrent_workers")
            .unwrap();
        assert_eq!(val, Some("5".to_string()));

        let missing = db.store().get_agent_config("nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[tokio::test]
    async fn test_eviction() {
        let db = HybridDatabase::new(":memory:", 1000, 1).unwrap(); // 1 day TTL
        db.store()
            .upsert_device(
                "DEV001",
                "Test",
                "192.168.1.1",
                4370,
                "zkteco",
                "UTC",
                None,
                true,
                123456,
            )
            .unwrap();

        // Insert log with current timestamp (within TTL)
        let data = make_attendance("user1", "2025-06-01 08:00:00");
        db.save_raw_attendance("DEV001".into(), data).await.unwrap();

        let sig_before = db.store().count_signatures().unwrap();
        let (evicted, _, _) = db.evict_old_signatures().await;
        // Old signatures (before threshold) should be evicted
        // Since test inserts happen at "2025-06-01" which is in the past, they should be evicted
        assert!(evicted > 0 || sig_before == 0);
    }
}
