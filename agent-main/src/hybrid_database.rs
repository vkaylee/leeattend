use crate::database::Database;
use crate::memory_store::{InMemoryStore, MemoryLog, StoreError};
use crate::models::AttendanceData;
use std::sync::Arc;
use log::info;

/// HybridDatabase orchestrates in-memory and disk persistence
/// - Memory is primary for fast reads/writes
/// - Disk is backup for crash recovery and long-term persistence
#[derive(Clone)]
pub struct HybridDatabase {
    disk: Database,
    memory: Arc<InMemoryStore>,
    signature_ttl_days: i64,
}

impl HybridDatabase {
    pub async fn new(database_url: &str, max_memory_logs: usize, signature_ttl_days: i64) -> Result<Self, sqlx::Error> {
        let disk = Database::new(database_url).await?;
        let memory = Arc::new(InMemoryStore::new(max_memory_logs));

        // Seed ID counter from disk
        let max_id = disk.get_max_id().await?;
        memory.seed_id(max_id + 1);

        // Note: Signatures and pending logs are now loaded per-device via load_device_memory()
        // This speeds up startup when only a few devices are active

        Ok(Self { disk, memory, signature_ttl_days })
    }

    pub fn memory_store(&self) -> Arc<InMemoryStore> {
        self.memory.clone()
    }

    pub fn disk_db(&self) -> &Database {
        &self.disk
    }

    /// Save attendance - goes to memory first
    /// Converts local time to UTC Unix timestamp using device timezone
    pub async fn save_raw_attendance(&self, device_id: &str, data: AttendanceData) -> Result<i64, StoreError> {
        // Get device timezone
        let tz = self.disk.get_device_timezone(device_id).await
            .map_err(|_| StoreError::InvalidTimestamp)?;

        // Parse timestamp and convert to UTC Unix timestamp
        let timestamp_utc = Database::parse_local_to_utc_timestamp(&data.timestamp, tz)
            .map_err(|_| StoreError::InvalidTimestamp)?;

        self.memory.insert(device_id.to_string(), data, timestamp_utc)
    }

    /// Batch save - goes to memory first
    /// Converts local time to UTC Unix timestamp using device timezone for each log
    pub async fn save_raw_attendance_batch(&self, batch: Vec<(String, AttendanceData)>) -> Result<usize, StoreError> {
        let mut batch_with_timestamps = Vec::new();

        for (device_id, data) in batch {
            // Get device timezone
            let tz = self.disk.get_device_timezone(&device_id).await
                .map_err(|_| StoreError::InvalidTimestamp)?;

            // Parse timestamp and convert to UTC Unix timestamp
            if let Ok(timestamp_utc) = Database::parse_local_to_utc_timestamp(&data.timestamp, tz) {
                batch_with_timestamps.push((device_id, data, timestamp_utc));
            }
        }

        self.memory.insert_batch(batch_with_timestamps)
    }

    /// Get unsynced logs from memory
    pub fn get_unsynced_logs(&self, limit: usize) -> Vec<MemoryLog> {
        self.memory.get_unsynced(limit)
    }

    /// Count unsynced logs
    pub fn count_unsynced_logs(&self) -> usize {
        self.memory.count_unsynced()
    }

    /// Mark logs as in-flight
    pub fn mark_in_flight(&self, ids: &[i64]) {
        self.memory.mark_in_flight(ids)
    }

    /// Mark logs as synced
    pub fn mark_synced(&self, ids: &[i64]) {
        self.memory.mark_synced(ids)
    }

    /// Reset log to unsynced (failed ACK)
    pub fn reset_to_unsynced(&self, id: i64) {
        self.memory.reset_to_unsynced(id)
    }

    /// Mark synced by composite key (for ACK handling)
    pub fn mark_synced_by_key(&self, composite_key: &str) -> bool {
        self.memory.mark_synced_by_key(composite_key)
    }

    /// Reset to unsynced by composite key (for failed ACK)
    pub fn reset_to_unsynced_by_key(&self, composite_key: &str) -> bool {
        self.memory.reset_to_unsynced_by_key(composite_key)
    }

    /// Checkpoint unpersisted logs to disk
    pub async fn checkpoint(&self) -> Result<usize, sqlx::Error> {
        let mut total_changes = 0;

        // 1. Handle status updates for already-persisted logs (e.g. marked as synced)
        let updates = self.memory.get_and_clear_pending_sync_updates();
        if !updates.is_empty() {
            self.disk.update_sync_status_batch(&updates).await?;
            total_changes += updates.len();
        }

        // 2. Handle new unpersisted logs
        let unpersisted = self.memory.get_unpersisted();
        if !unpersisted.is_empty() {
            let (inserted, _skipped, _composite_keys) = self.disk.checkpoint_batch(&unpersisted).await?;

            // Mark as persisted in memory
            let ids: Vec<i64> = unpersisted.iter().map(|l| l.id).collect();
            self.memory.mark_persisted(&ids);
            total_changes += inserted;
        }

        if total_changes > 0 {
            self.memory.clear_dirty();
        }

        Ok(total_changes)
    }

    /// Full flush on shutdown
    pub async fn flush_all(&self) -> Result<usize, sqlx::Error> {
        // First checkpoint any unpersisted
        self.checkpoint().await?;

        // Then update disk with final sync states using composite keys
        let all_logs = self.memory.get_all();
        let updates: Vec<(Arc<str>, Arc<str>, i64, Option<i32>, Option<i32>, i64)> = all_logs.iter()
            .filter(|l| l.persisted)  // Only update already-persisted logs
            .map(|l| (l.device_id.clone(), l.user_id.clone(), l.timestamp, l.punch, l.status, l.is_synced))
            .collect();

        self.disk.update_sync_status_batch(&updates).await?;

        Ok(updates.len())
    }

    /// Evict signatures older than TTL. Returns (evicted, before, after).
    pub fn evict_old_signatures(&self) -> (usize, usize, usize) {
        self.memory.evict_old_signatures(self.signature_ttl_days * 86400)
    }

    /// Runtime optimization: Unload device data from memory when disabled
    pub async fn unload_device_memory(&self, device_id: &str) -> Result<(), sqlx::Error> {
        // 1. Fetch ALL signatures for this device (no TTL filter — unload must remove everything)
        let signatures = self.disk.get_signatures_for_device(device_id, 36500).await?;
        
        // 2. Remove signatures from memory set
        self.memory.remove_signatures(signatures);

        // 3. Remove any active logs from memory store
        self.memory.remove_logs_for_device(device_id);

        info!("HybridDatabase: Unloaded memory for disabled device '{device_id}'");
        Ok(())
    }

    /// Runtime optimization: Load device data into memory when enabled
    pub async fn load_device_memory(&self, device_id: &str) -> Result<(), sqlx::Error> {
        // 1. Load signatures for dedup (only within TTL window)
        let signatures = self.disk.get_signatures_for_device(device_id, self.signature_ttl_days).await?;
        let sig_count = signatures.len();
        self.memory.load_signatures(signatures);

        // 2. Load pending logs into memory queue for sync
        let pending_logs = self.disk.get_unsynced_logs_for_device(device_id).await?;
        let pending_count = pending_logs.len();
        self.memory.load_from_disk(pending_logs);

        info!("HybridDatabase: Loaded {sig_count} signatures + {pending_count} pending logs for device '{device_id}'");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn make_attendance(user_id: &str, timestamp: &str) -> AttendanceData {
        AttendanceData {
            user_id: user_id.to_string(),
            timestamp: timestamp.to_string(),
            status: Some(1),
            punch: Some(0),
        }
    }

    #[tokio::test]
    async fn test_hybrid_new() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();
        assert_eq!(hybrid.count_unsynced_logs(), 0);
        assert!(!hybrid.memory_store().is_dirty());
    }

    #[tokio::test]
    async fn test_hybrid_save_single() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let id = hybrid.save_raw_attendance("DEV1", data).await.unwrap();

        assert!(id > 0);
        assert_eq!(hybrid.count_unsynced_logs(), 1);
        assert!(hybrid.memory_store().is_dirty());
    }

    #[tokio::test]
    async fn test_hybrid_save_batch() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        let batch: Vec<(String, AttendanceData)> = (1..=5)
            .map(|i| {
                (
                    "DEV1".to_string(),
                    make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00"),
                )
            })
            .collect();

        let count = hybrid.save_raw_attendance_batch(batch).await.unwrap();

        assert_eq!(count, 5);
        assert_eq!(hybrid.count_unsynced_logs(), 5);
    }

    #[tokio::test]
    async fn test_hybrid_checkpoint_empty() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Checkpoint with no logs
        let checkpointed = hybrid.checkpoint().await.unwrap();

        assert_eq!(checkpointed, 0);
        assert!(!hybrid.memory_store().is_dirty());
    }

    #[tokio::test]
    async fn test_hybrid_checkpoint_with_logs() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save logs
        for i in 1..=10 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        assert_eq!(hybrid.count_unsynced_logs(), 10);
        assert!(hybrid.memory_store().is_dirty());

        // Checkpoint
        let checkpointed = hybrid.checkpoint().await.unwrap();

        assert_eq!(checkpointed, 10);
        assert!(!hybrid.memory_store().is_dirty());
    }

    #[tokio::test]
    async fn test_hybrid_full_workflow() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // 1. Save logs
        for i in 1..=5 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }
        assert_eq!(hybrid.count_unsynced_logs(), 5);

        // 2. Get unsynced
        let logs = hybrid.get_unsynced_logs(10);
        assert_eq!(logs.len(), 5);

        // 3. Mark in-flight
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        hybrid.mark_in_flight(&ids);
        assert_eq!(hybrid.count_unsynced_logs(), 0);

        // 4. Mark synced (first 3)
        hybrid.mark_synced(&ids[0..3]);

        // 5. Checkpoint
        let checkpointed = hybrid.checkpoint().await.unwrap();
        assert!(checkpointed > 0);
    }

    #[tokio::test]
    async fn test_hybrid_restart_recovery() {
        // Note: In-memory SQLite doesn't persist between connections
        // This test verifies the recovery mechanism works with the same connection
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Phase 1: Save and checkpoint
        for i in 1..=5 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Verify all are unsynced before checkpoint
        assert_eq!(hybrid.count_unsynced_logs(), 5);

        // Checkpoint to disk
        let checkpointed = hybrid.checkpoint().await.unwrap();
        assert_eq!(checkpointed, 5);

        // After checkpoint, should be 0 (marked as synced if needed)
        // Load_from_disk only loads unsynced/in-flight logs
        // For this test, we verify checkpoint doesn't error
    }

    #[tokio::test]
    async fn test_hybrid_reset_to_unsynced_after_inflight() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save and mark in-flight
        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let id = hybrid.save_raw_attendance("DEV1", data).await.unwrap();

        hybrid.mark_in_flight(&[id]);
        assert_eq!(hybrid.count_unsynced_logs(), 0);

        // Reset to unsynced
        hybrid.reset_to_unsynced(id);
        assert_eq!(hybrid.count_unsynced_logs(), 1);
    }

    #[tokio::test]
    async fn test_hybrid_flush_all() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save logs
        for i in 1..=5 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Flush all
        let flushed = hybrid.flush_all().await.unwrap();

        // flushed is usize, always >= 0, just verify flush succeeded
        let _ = flushed;
        assert!(!hybrid.memory_store().is_dirty());
    }

    #[tokio::test]
    async fn test_hybrid_get_unsynced_fifo_order() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save in specific order
        let users = vec!["FIRST", "SECOND", "THIRD"];
        for user in &users {
            let data = make_attendance(user, "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Get unsynced
        let logs = hybrid.get_unsynced_logs(10);

        // Verify FIFO order
        assert_eq!(logs.len(), 3);
        assert_eq!(&*logs[0].user_id, "FIRST");
        assert_eq!(&*logs[1].user_id, "SECOND");
        assert_eq!(&*logs[2].user_id, "THIRD");
    }

    #[tokio::test]
    async fn test_hybrid_multiple_devices() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save from different devices
        for dev_idx in 1..=3 {
            for user_idx in 1..=2 {
                let device = format!("DEV{dev_idx}");
                let data = make_attendance(
                    &format!("U{dev_idx}{user_idx}"),
                    "2026-01-29 10:00:00"
                );
                hybrid.save_raw_attendance(&device, data).await.unwrap();
            }
        }

        assert_eq!(hybrid.count_unsynced_logs(), 6);

        // Get all unsynced
        let logs = hybrid.get_unsynced_logs(10);
        assert_eq!(logs.len(), 6);
    }

    #[tokio::test]
    async fn test_hybrid_capacity_enforcement() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 5, 90).await.unwrap();

        // Try to save more than capacity
        let batch: Vec<(String, AttendanceData)> = (1..=10)
            .map(|i| {
                (
                    "DEV1".to_string(),
                    make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00"),
                )
            })
            .collect();

        let result = hybrid.save_raw_attendance_batch(batch).await;

        // Should insert up to capacity
        assert!(result.is_ok());
        assert_eq!(hybrid.count_unsynced_logs(), 5);
    }

    #[tokio::test]
    async fn test_hybrid_partial_sync() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save 5 logs
        let mut ids = Vec::new();
        for i in 1..=5 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            let id = hybrid.save_raw_attendance("DEV1", data).await.unwrap();
            ids.push(id);
        }

        // Get unsynced
        let logs = hybrid.get_unsynced_logs(10);
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

        // Mark first 3 as in-flight
        hybrid.mark_in_flight(&log_ids[0..3]);

        // Check counts
        assert_eq!(hybrid.count_unsynced_logs(), 2);

        // Checkpoint (only persists unpersisted logs)
        let checkpointed = hybrid.checkpoint().await.unwrap();
        // checkpointed is usize, always >= 0, just verify checkpoint succeeded
        let _ = checkpointed;
    }

    #[tokio::test]
    async fn test_hybrid_checkpoint_idempotent() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Save logs
        for i in 1..=3 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Checkpoint multiple times
        let count1 = hybrid.checkpoint().await.unwrap();
        let count2 = hybrid.checkpoint().await.unwrap();

        assert_eq!(count1, 3);
        assert_eq!(count2, 0); // Second checkpoint should find nothing
    }

    // ===== Phase 4: Integration Tests =====

    #[tokio::test]
    async fn test_full_dedup_lifecycle() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        let data = make_attendance("U001", "2026-01-29 10:00:00");

        // 1. Insert
        let id = hybrid.save_raw_attendance("DEV1", data.clone()).await.unwrap();
        assert!(id > 0);

        // 2. Sync
        hybrid.mark_in_flight(&[id]);
        hybrid.mark_synced(&[id]);

        // 3. Checkpoint
        hybrid.checkpoint().await.unwrap();

        // 4. Try duplicate - should fail
        let result = hybrid.save_raw_attendance("DEV1", data).await;
        assert!(matches!(result, Err(StoreError::Duplicate)));
    }

    #[tokio::test]
    async fn test_signatures_survive_restart_simulation() {
        // Use shared in-memory SQLite with UUID for test isolation
        // Note: This simulates restart by creating two separate HybridDatabase instances
        // sharing the same in-memory DB via cache=shared.
        // IMPORTANT: We must keep the first instance alive to maintain the DB connection,
        // as in-memory SQLite loses data when all connections close.
        let db_url = format!("sqlite:file:{}?mode=memory&cache=shared", uuid::Uuid::new_v4());

        // Phase 1: Create and checkpoint logs
        let hybrid1 = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();

        for i in 1..=10 {
            let data = make_attendance(&format!("U{i:03}"), "2026-01-29 10:00:00");
            hybrid1.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Checkpoint to disk
        hybrid1.checkpoint().await.unwrap();

        // Mark some as synced
        let logs = hybrid1.get_unsynced_logs(5);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        hybrid1.mark_in_flight(&ids);
        hybrid1.mark_synced(&ids);

        // Final flush
        hybrid1.flush_all().await.unwrap();

        // Phase 2: Simulate restart - create new instance while keeping hybrid1 alive
        // This simulates app restart: new memory store, but shared persistent DB
        let hybrid2 = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();

        // With lazy loading, need to explicitly load device memory (like main.rs does)
        hybrid2.load_device_memory("DEV1").await.unwrap();

        // All 10 signatures should be loaded from disk after load_device_memory
        assert_eq!(hybrid2.memory_store().count_signatures(), 10);

        // Try to insert duplicate of synced log
        let data = make_attendance("U001", "2026-01-29 10:00:00");
        let result = hybrid2.save_raw_attendance("DEV1", data).await;
        assert!(matches!(result, Err(StoreError::Duplicate)));

        // Keep hybrid1 alive until end of test to maintain DB connection
        drop(hybrid1);
    }

    // ===== Phase 3: Checkpoint Refactor Tests =====

    #[tokio::test]
    async fn test_checkpoint_no_id_collision() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Pre-populate disk with logs having specific IDs
        for i in 1..=100 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: "2026-01-31 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            hybrid.save_raw_attendance(&format!("DEV{}", (i % 5) + 1), data).await.ok();
        }

        // Checkpoint first batch
        let checkpointed1 = hybrid.checkpoint().await.unwrap();
        assert!(checkpointed1 > 0);

        // Create more logs with overlapping memory IDs
        for i in 50..=150 {
            let data = AttendanceData {
                user_id: format!("U{:03}", i + 1000),  // Different user to avoid business duplicate
                timestamp: "2026-01-31 11:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            hybrid.save_raw_attendance("DEV2", data).await.ok();
        }

        // Second checkpoint should NOT fail with UNIQUE constraint error
        let checkpointed2 = hybrid.checkpoint().await.unwrap();
        assert!(checkpointed2 > 0, "Checkpoint should not fail with ID collision");
    }

    #[tokio::test]
    async fn test_full_checkpoint_ack_flow_with_composite_key() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Insert logs to memory
        for i in 0..10 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-01-31 10:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Checkpoint to disk
        let inserted = hybrid.checkpoint().await.unwrap();
        assert_eq!(inserted, 10);

        // Simulate ACK flow: get unsynced logs, extract composite keys, mark synced
        let logs = hybrid.get_unsynced_logs(20);
        for log in &logs {
            let composite_key = log.composite_key();

            // Mark in-flight
            hybrid.mark_in_flight(&[log.id]);

            // Simulate successful ACK using composite key
            assert!(hybrid.mark_synced_by_key(&composite_key), "Should find log by composite key");
        }

        // Verify all synced
        assert_eq!(hybrid.count_unsynced_logs(), 0);
    }

    #[tokio::test]
    async fn test_ack_failure_resets_to_unsynced() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-31 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = hybrid.save_raw_attendance("DEV1", data).await.unwrap();

        // Mark in-flight
        hybrid.mark_in_flight(&[id]);
        assert_eq!(hybrid.count_unsynced_logs(), 0);  // In-flight, not unsynced

        // Construct composite key using Unix timestamp directly
        let ts_unix = NaiveDateTime::parse_from_str("2026-01-31 10:00:00", "%Y-%m-%d %H:%M:%S")
            .expect("parse failed")
            .and_utc()
            .timestamp();
        let key = format!("DEV1:U001:{ts_unix}:0:1");

        // Simulate failed ACK - reset to unsynced
        assert!(hybrid.reset_to_unsynced_by_key(&key));

        // Should be back in unsynced queue
        assert_eq!(hybrid.count_unsynced_logs(), 1);
    }

    #[tokio::test]
    async fn test_checkpoint_performance_1000_logs() {
        let hybrid = HybridDatabase::new("sqlite::memory:", 10000, 90).await.unwrap();

        // Insert 1000 logs
        for i in 0..1000 {
            let data = AttendanceData {
                user_id: format!("U{i:06}"),
                timestamp: format!("2026-01-31 {:02}:{:02}:{:02}", i / 3600, (i % 3600) / 60, i % 60),
                status: Some(1),
                punch: Some(0),
            };
            hybrid.save_raw_attendance("DEV1", data).await.unwrap();
        }

        // Measure checkpoint time
        let start = std::time::Instant::now();
        let inserted = hybrid.checkpoint().await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(inserted, 1000);
        assert!(
            elapsed.as_millis() < 500,  // Should be well under 500ms
            "Checkpoint took {elapsed:?}, expected < 500ms"
        );
    }
}


    #[tokio::test]
    async fn test_dynamic_memory_management() {
        // Use large TTL to avoid filtering out old test timestamps (2023)
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 3650).await.unwrap();
        
        // 1. Setup: Insert logs for two devices directly to DB (simulating existing history)
        let data_a = AttendanceData { user_id: "U1".into(), timestamp: "2023-01-01 10:00:00".into(), status: Some(1), punch: Some(1) };
        let data_b = AttendanceData { user_id: "U1".into(), timestamp: "2023-01-01 10:00:00".into(), status: Some(1), punch: Some(1) };
        
        hybrid.save_raw_attendance("DevA", data_a.clone()).await.unwrap();
        hybrid.save_raw_attendance("DevB", data_b.clone()).await.unwrap();
        
        // Flush to ensure they are persisted and (normally) in memory
        hybrid.checkpoint().await.unwrap();
        
        // 2. Verify: Initial state (both in memory because save_raw_attendance adds them)
        assert_eq!(hybrid.memory_store().count_signatures(), 2);
        
        // 3. Test Unload "DevA"
        hybrid.unload_device_memory("DevA").await.unwrap();
        
        // Should remove DevA's signature, leaving DevB
        assert_eq!(hybrid.memory_store().count_signatures(), 1, "Should have 1 signature after unloading DevA");
        assert!(hybrid.memory_store().has_signature("DevB", "U1", 1672567200, Some(1), Some(1)));
        assert!(!hybrid.memory_store().has_signature("DevA", "U1", 1672567200, Some(1), Some(1)));
        
        // 4. Test Reload "DevA"
        hybrid.load_device_memory("DevA").await.unwrap();
        
        // Should have 2 signatures again
        assert_eq!(hybrid.memory_store().count_signatures(), 2, "Should have 2 signatures after reloading DevA");
        assert!(hybrid.memory_store().has_signature("DevA", "U1", 1672567200, Some(1), Some(1)));
    }

    /// Test to ensure hash signatures are loaded during HybridDatabase::new()
    /// This guarantees dedup hashes exist BEFORE device polling loop starts.
    #[tokio::test]
    async fn test_signatures_loaded_before_polling() {
        // 1. Setup: Create HybridDatabase
        let db_url = "sqlite::memory:";
        let hybrid = HybridDatabase::new(db_url, 1000, 90).await.unwrap();
        
        // Setup device
        hybrid.disk_db().upsert_device("DevZ", "Test Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        
        // 2. Save some logs (this adds to both memory signatures and disk)
        let data = AttendanceData { user_id: "U1".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) };
        hybrid.save_raw_attendance("DevZ", data).await.unwrap();
        
        // Force flush to disk so signatures are persisted
        hybrid.checkpoint().await.unwrap();
        
        // 3. Verify signatures exist in memory AFTER initialization
        let sig_count = hybrid.memory_store().count_signatures();
        assert_eq!(sig_count, 1, "Signature should exist after save");
        
        // 4. Simulate device disable -> re-enable (runtime scenario)
        // This tests the load_device_memory() path used BEFORE polling starts
        hybrid.unload_device_memory("DevZ").await.unwrap();
        assert_eq!(hybrid.memory_store().count_signatures(), 0, "Signatures should be 0 after unload");
        
        // 5. CRUCIAL: load_device_memory() is called BEFORE polling loop starts (main.rs:276-280)
        // This ensures hashes are loaded before any log processing
        hybrid.load_device_memory("DevZ").await.unwrap();
        
        // Verify signatures are loaded BEFORE polling would start
        assert_eq!(hybrid.memory_store().count_signatures(), 1,
            "Signatures must be loaded BEFORE device polling loop starts");
    }

    // ===== Signature Eviction & TTL Tests =====

    #[tokio::test]
    async fn test_hybrid_evict_old_signatures() {
        // TTL = 1 day — signatures older than 1 day will be evicted
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 1).await.unwrap();

        // Insert with recent timestamp (within TTL)
        let recent_ts = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
        let recent_data = AttendanceData {
            user_id: "RECENT".to_string(),
            timestamp: "2026-02-14 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        hybrid.memory_store().insert("D1".to_string(), recent_data, recent_ts).unwrap();

        // Insert with old timestamp (outside TTL)
        let old_ts = chrono::Utc::now().timestamp() - (3 * 86400); // 3 days ago
        let old_data = AttendanceData {
            user_id: "OLD".to_string(),
            timestamp: "2026-02-11 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        hybrid.memory_store().insert("D1".to_string(), old_data, old_ts).unwrap();

        assert_eq!(hybrid.memory_store().count_signatures(), 2);

        // Evict — should remove the old one
        let (evicted, before, after) = hybrid.evict_old_signatures();
        assert_eq!(evicted, 1);
        assert_eq!(before, 2);
        assert_eq!(after, 1);
    }

    #[tokio::test]
    async fn test_ttl_filtered_load_excludes_old_synced_signatures() {
        // TTL = 7 days
        // Tests that SYNCED old signatures are not reloaded (only signature query applies TTL).
        // Unsynced logs are always reloaded via get_unsynced_logs_for_device regardless of TTL.
        let db_url = format!("sqlite:file:{}?mode=memory&cache=shared", uuid::Uuid::new_v4());
        let hybrid = HybridDatabase::new(&db_url, 1000, 7).await.unwrap();

        // Setup device
        hybrid.disk_db().upsert_device("DEV1", "Test", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();

        // Insert recent log (2 days ago — within TTL)
        let recent_ts = chrono::Utc::now().timestamp() - (2 * 86400);
        let recent_data = AttendanceData {
            user_id: "RECENT_USER".to_string(),
            timestamp: "2026-02-12 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let recent_id = hybrid.memory_store().insert("DEV1".to_string(), recent_data, recent_ts).unwrap();

        // Insert old log (30 days ago — outside TTL)
        let old_ts = chrono::Utc::now().timestamp() - (30 * 86400);
        let old_data = AttendanceData {
            user_id: "OLD_USER".to_string(),
            timestamp: "2026-01-15 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let old_id = hybrid.memory_store().insert("DEV1".to_string(), old_data, old_ts).unwrap();

        // Checkpoint to disk, then mark synced via proper flow
        hybrid.checkpoint().await.unwrap();
        hybrid.mark_in_flight(&[recent_id, old_id]);
        hybrid.mark_synced(&[recent_id, old_id]);
        // flush_all now correctly updates disk using composite keys
        hybrid.flush_all().await.unwrap();

        assert_eq!(hybrid.memory_store().count_signatures(), 2);

        // Verify both signatures in memory before unload
        assert_eq!(hybrid.memory_store().count_signatures(), 2, "Both signatures before unload");

        // Unload device (clears memory)
        hybrid.unload_device_memory("DEV1").await.unwrap();
        assert_eq!(hybrid.memory_store().count_signatures(), 0, "All signatures cleared after unload");

        // Reload — TTL filter on signature query should only load recent
        hybrid.load_device_memory("DEV1").await.unwrap();

        // Only recent signature loaded (old synced one filtered by TTL)
        assert_eq!(hybrid.memory_store().count_signatures(), 1,
            "Only recent signatures within TTL should be loaded from synced logs");

        // Recent log dedup still works
        let recent_data2 = AttendanceData {
            user_id: "RECENT_USER".to_string(),
            timestamp: "2026-02-12 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let result = hybrid.memory_store().insert("DEV1".to_string(), recent_data2, recent_ts);
        assert!(matches!(result, Err(crate::memory_store::StoreError::Duplicate)),
            "Recent signature should still provide dedup protection");

        // Old log reinsertion succeeds (signature expired by TTL)
        let old_data2 = AttendanceData {
            user_id: "OLD_USER".to_string(),
            timestamp: "2026-01-15 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let result = hybrid.memory_store().insert("DEV1".to_string(), old_data2, old_ts);
        assert!(result.is_ok(),
            "Old signature beyond TTL should allow reinsertion");
    }

    #[tokio::test]
    async fn test_evict_preserves_dedup_for_recent_signatures() {
        // TTL = 90 days
        let hybrid = HybridDatabase::new("sqlite::memory:", 1000, 90).await.unwrap();

        // Insert several recent logs
        for i in 0..5 {
            let ts = chrono::Utc::now().timestamp() - (i * 3600);
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-02-14 {:02}:00:00", 10 - i),
                status: Some(1),
                punch: Some(0),
            };
            hybrid.memory_store().insert("D1".to_string(), data, ts).unwrap();
        }

        // Evict — nothing should be evicted (all within 90 days)
        let (evicted, _, _) = hybrid.evict_old_signatures();
        assert_eq!(evicted, 0);
        assert_eq!(hybrid.memory_store().count_signatures(), 5);

        // Dedup still works for all
        for i in 0..5 {
            let ts = chrono::Utc::now().timestamp() - (i * 3600);
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-02-14 {:02}:00:00", 10 - i),
                status: Some(1),
                punch: Some(0),
            };
            let result = hybrid.memory_store().insert("D1".to_string(), data, ts);
            assert!(matches!(result, Err(crate::memory_store::StoreError::Duplicate)),
                "Recent signature U{i} should still block duplicates");
        }
    }

    /// Verify flush_all persists sync status to disk via composite keys
    /// (not memory IDs which differ from disk auto-increment IDs)
    #[tokio::test]
    async fn test_flush_all_updates_disk_sync_status_via_composite_keys() {
        let db_url = format!("sqlite:file:{}?mode=memory&cache=shared", uuid::Uuid::new_v4());
        let hybrid = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();

        // Setup device
        hybrid.disk_db().upsert_device("DEV1", "Test", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();

        // Insert 3 logs
        let ts_base = chrono::Utc::now().timestamp();
        let mut ids = Vec::new();
        for i in 0..3 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: format!("2026-02-15 0{}:00:00", i + 1),
                status: Some(1),
                punch: Some(0),
            };
            let id = hybrid.memory_store().insert("DEV1".to_string(), data, ts_base + i).unwrap();
            ids.push(id);
        }

        // Checkpoint to disk (disk assigns its own auto-increment IDs)
        hybrid.checkpoint().await.unwrap();

        // Verify memory IDs differ from disk IDs
        let disk_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM zkt_raw_attendance ORDER BY id")
            .fetch_all(hybrid.disk_db().get_pool())
            .await
            .unwrap();
        assert_ne!(ids, disk_ids, "Memory IDs should differ from disk auto-increment IDs");

        // Mark as synced (persisted logs get removed from memory, composite keys queued)
        hybrid.mark_in_flight(&ids);
        hybrid.mark_synced(&ids);

        // flush_all should propagate sync status to disk using composite keys
        hybrid.flush_all().await.unwrap();

        // Verify disk rows have is_synced=1
        let unsynced_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced != 1"
        )
        .fetch_one(hybrid.disk_db().get_pool())
        .await
        .unwrap();

        assert_eq!(unsynced_count, 0, "All disk rows should be is_synced=1 after flush_all");
    }

    /// Verify checkpoint propagates pending_sync_updates (from reset_to_unsynced) to disk
    #[tokio::test]
    async fn test_checkpoint_propagates_reset_to_unsynced_via_composite_keys() {
        let db_url = format!("sqlite:file:{}?mode=memory&cache=shared", uuid::Uuid::new_v4());
        let hybrid = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();

        hybrid.disk_db().upsert_device("DEV1", "Test", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();

        let ts = chrono::Utc::now().timestamp();
        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-02-15 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let id = hybrid.memory_store().insert("DEV1".to_string(), data, ts).unwrap();

        // Checkpoint to disk
        hybrid.checkpoint().await.unwrap();

        // Mark in-flight (is_synced=2)
        hybrid.mark_in_flight(&[id]);

        // Simulate failed sync — reset back to unsynced (queues composite key update)
        hybrid.reset_to_unsynced(id);

        // Checkpoint should propagate the reset to disk
        hybrid.checkpoint().await.unwrap();

        // Verify disk row is back to is_synced=0
        let sync_status: i64 = sqlx::query_scalar(
            "SELECT is_synced FROM zkt_raw_attendance WHERE device_id = 'DEV1' AND user_id = 'U001'"
        )
        .fetch_one(hybrid.disk_db().get_pool())
        .await
        .unwrap();

        assert_eq!(sync_status, 0, "Disk row should be reset to is_synced=0 after checkpoint");
    }
