use dashmap::{DashMap, DashSet};
use parking_lot::RwLock;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicI64, AtomicUsize, AtomicBool, Ordering};
use std::time::Instant;
use chrono::NaiveDateTime;
use crate::models::{RawAttendanceLog, AttendanceData, SignatureRow};
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;
/// Maximum retry delay cap in seconds (5 minutes)
const MAX_RETRY_DELAY_SECS: i64 = 300;

/// Maximum bit shift for exponential backoff (2^8 = 256s, then capped to 300s)
const MAX_RETRY_SHIFT: i64 = 8;

/// Jitter percentage (±10%)
const JITTER_PERCENT: f64 = 0.10;

/// Calculate exponential backoff delay with jitter.
/// Formula: min(2^retry_count, MAX_RETRY_DELAY_SECS) ± 10% jitter
///
/// Delay progression: 1s → 2s → 4s → 8s → 16s → 32s → 64s → 128s → 256s → 300s (capped)
fn calculate_backoff_with_jitter(retry_count: i64) -> i64 {
    use rand::Rng;

    // Base delay: 2^retry_count, capped at MAX_RETRY_SHIFT bits
    let base_delay = (1i64 << retry_count.min(MAX_RETRY_SHIFT)).min(MAX_RETRY_DELAY_SECS);

    // Apply ±10% jitter to prevent thundering herd
    let jitter_range = (base_delay as f64 * JITTER_PERCENT) as i64;
    if jitter_range > 0 {
        let jitter = rand::thread_rng().gen_range(-jitter_range..=jitter_range);
        // Ensure delay is at least 1 second
        (base_delay + jitter).max(1)
    } else {
        base_delay
    }
}

/// In-memory log with sync status and persistence tracking
#[derive(Clone, Debug)]
pub struct MemoryLog {
    pub id: i64,
    pub device_id: String,
    pub user_id: String,
    pub timestamp: NaiveDateTime,
    pub status: Option<i32>,
    pub punch: Option<i32>,
    pub is_synced: i64,           // 0=unsynced, 1=synced, 2=in_flight
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub retry_count: i64,
    pub persisted: bool,          // true if already on disk
}

/// In-memory store for attendance logs with FIFO ordering
pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,
    signatures: DashSet<u64>,  // Hash signatures for deduplication
    unsynced_index: Arc<RwLock<BTreeSet<i64>>>,
    inflight_index: Arc<RwLock<HashMap<i64, Instant>>>,
    next_id: AtomicI64,
    dirty: AtomicBool,
    count: AtomicUsize,
    max_capacity: usize,
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

impl InMemoryStore {
    /// Create a new in-memory store with specified capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            logs: DashMap::with_capacity(max_capacity / 2),
            signatures: DashSet::with_capacity(max_capacity),
            unsynced_index: Arc::new(RwLock::new(BTreeSet::new())),
            inflight_index: Arc::new(RwLock::new(HashMap::new())),
            next_id: AtomicI64::new(1),
            dirty: AtomicBool::new(false),
            count: AtomicUsize::new(0),
            max_capacity,
        }
    }

    /// Compute 64-bit signature for deduplication.
    /// Signature = hash(device_id, user_id, timestamp_unix, punch, status)
    fn compute_signature(device_id: &str, user_id: &str, timestamp: NaiveDateTime, punch: Option<i32>, status: Option<i32>) -> u64 {
        let mut hasher = FxHasher::default();
        device_id.hash(&mut hasher);
        user_id.hash(&mut hasher);
        // Use Unix timestamp for deterministic hashing
        timestamp.and_utc().timestamp().hash(&mut hasher);
        punch.hash(&mut hasher);
        status.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a signature already exists (public for testing)
    pub fn has_signature(&self, device_id: &str, user_id: &str, timestamp: NaiveDateTime, punch: Option<i32>, status: Option<i32>) -> bool {
        let sig = Self::compute_signature(device_id, user_id, timestamp, punch, status);
        self.signatures.contains(&sig)
    }

    /// Get signature count (for monitoring)
    pub fn count_signatures(&self) -> usize {
        self.signatures.len()
    }

    /// Seed the next ID counter (used during initialization)
    pub fn seed_id(&self, next_id: i64) {
        // Add gap to prevent ID collision with any in-flight logs
        // that might have been assigned but not persisted before crash
        let safe_start = next_id + 1000; // 1000 ID gap
        self.next_id.store(safe_start, Ordering::SeqCst);
    }

    /// Load signatures only (Phase A of startup - Phase 3)
    /// Does NOT create MemoryLog entries - just populates signature index
    pub fn load_signatures(&self, rows: Vec<SignatureRow>) {
        for row in rows {
            let sig = Self::compute_signature(&row.device_id, &row.user_id, row.timestamp, row.punch, row.status);
            self.signatures.insert(sig);
        }
    }

    /// Load existing logs from disk into memory (startup recovery)
    pub fn load_from_disk(&self, logs: Vec<RawAttendanceLog>) {
        let mut max_id = 0i64;
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        for log in logs {
            max_id = max_id.max(log.id);

            // Add signature for ALL loaded logs (including synced ones)
            let signature = Self::compute_signature(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
            self.signatures.insert(signature);

            let memory_log = MemoryLog {
                id: log.id,
                device_id: log.device_id,
                user_id: log.user_id,
                timestamp: log.timestamp,
                status: log.status,
                punch: log.punch,
                is_synced: log.is_synced,
                created_at: log.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()),
                updated_at: log.updated_at,
                retry_count: log.retry_count,
                persisted: true, // Loaded from disk = already persisted
            };

            // Update indices based on sync status
            if log.is_synced == 0 {
                unsynced.insert(log.id);
            } else if log.is_synced == 2 {
                inflight.insert(log.id, Instant::now());
            }

            self.logs.insert(log.id, memory_log);
        }

        self.next_id.store(max_id + 1, Ordering::SeqCst);
        self.count.store(self.logs.len(), Ordering::SeqCst);
    }

    /// Insert a new attendance log. Returns assigned ID.
    pub fn insert(&self, device_id: String, data: AttendanceData) -> Result<i64, StoreError> {
        // Parse timestamp first (needed for signature)
        let timestamp = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|_| StoreError::InvalidTimestamp)?;

        // Compute signature and check for duplicate
        let signature = Self::compute_signature(&device_id, &data.user_id, timestamp, data.punch, data.status);
        if self.signatures.contains(&signature) {
            return Err(StoreError::Duplicate);
        }

        // Check capacity (backpressure)
        if self.count.load(Ordering::Relaxed) >= self.max_capacity {
            return Err(StoreError::CapacityExceeded);
        }

        // Add signature FIRST (before log)
        self.signatures.insert(signature);

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = chrono::Utc::now().naive_utc();

        let log = MemoryLog {
            id,
            device_id,
            user_id: data.user_id,
            timestamp,
            status: data.status,
            punch: data.punch,
            is_synced: 0,
            created_at: now,
            updated_at: None,
            retry_count: 0,
            persisted: false,
        };

        self.logs.insert(id, log);
        self.unsynced_index.write().insert(id);
        self.count.fetch_add(1, Ordering::SeqCst);
        self.dirty.store(true, Ordering::SeqCst);

        Ok(id)
    }

    /// Batch insert with automatic capacity check
    pub fn insert_batch(&self, batch: Vec<(String, AttendanceData)>) -> Result<usize, StoreError> {
        if batch.is_empty() {
            return Err(StoreError::CapacityExceeded); // Maintain original behavior for empty batch
        }

        let available = self.max_capacity.saturating_sub(self.count.load(Ordering::Relaxed));
        if available == 0 {
            return Err(StoreError::CapacityExceeded);
        }

        let mut inserted = 0;
        let mut unsynced = self.unsynced_index.write();

        for (device_id, data) in batch.into_iter() {
            if inserted >= available {
                break; // Capacity limit reached
            }

            // Parse timestamp
            let timestamp = match NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"))
            {
                Ok(ts) => ts,
                Err(_) => continue, // Skip invalid timestamp
            };

            // Check for duplicate
            let signature = Self::compute_signature(&device_id, &data.user_id, timestamp, data.punch, data.status);
            if self.signatures.contains(&signature) {
                continue; // Skip duplicate
            }

            // Add signature FIRST
            self.signatures.insert(signature);

            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let now = chrono::Utc::now().naive_utc();

            let log = MemoryLog {
                id,
                device_id,
                user_id: data.user_id,
                timestamp,
                status: data.status,
                punch: data.punch,
                is_synced: 0,
                created_at: now,
                updated_at: None,
                retry_count: 0,
                persisted: false,
            };

            self.logs.insert(id, log);
            unsynced.insert(id);
            inserted += 1;
        }

        self.count.fetch_add(inserted, Ordering::SeqCst);
        if inserted > 0 {
            self.dirty.store(true, Ordering::SeqCst);
        }
        Ok(inserted)
    }

    /// Get unsynced logs in FIFO order (by ID) with exponential backoff filtering
    pub fn get_unsynced(&self, limit: usize) -> Vec<MemoryLog> {
        let unsynced = self.unsynced_index.read();
        let mut result = Vec::with_capacity(limit);
        let now = chrono::Utc::now().naive_utc();

        for id in unsynced.iter() {
            if result.len() >= limit {
                break;
            }
            if let Some(log) = self.logs.get(id) {
                // Skip logs still in backoff period (5-minute cap with jitter)
                if let Some(updated_at) = log.updated_at {
                    let delay_secs = calculate_backoff_with_jitter(log.retry_count);
                    let eligible_at = updated_at + chrono::Duration::seconds(delay_secs);
                    if now < eligible_at {
                        continue; // Still in backoff
                    }
                }
                result.push(log.clone());
            }
        }

        result
    }

    /// Count unsynced logs
    pub fn count_unsynced(&self) -> usize {
        self.unsynced_index.read().len()
    }

    /// Count logs currently in backoff (not eligible for sync)
    pub fn count_in_backoff(&self) -> usize {
        let unsynced = self.unsynced_index.read();
        let now = chrono::Utc::now().naive_utc();
        let mut count = 0;

        for id in unsynced.iter() {
            if let Some(log) = self.logs.get(id)
                && let Some(updated_at) = log.updated_at {
                    let delay_secs = calculate_backoff_with_jitter(log.retry_count);
                    let eligible_at = updated_at + chrono::Duration::seconds(delay_secs);
                    if now < eligible_at {
                        count += 1;
                    }
                }
        }

        count
    }

    /// Count total logs in memory
    pub fn count_total(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }

    /// Check if store has unpersisted changes
    pub fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    /// Mark logs as in-flight (is_synced=2)
    pub fn mark_in_flight(&self, ids: &[i64]) {
        let now = Instant::now();
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.is_synced = 2;
                log.retry_count += 1;
                log.updated_at = Some(chrono::Utc::now().naive_utc());
            }
            unsynced.remove(&id);
            inflight.insert(id, now);
        }

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Mark logs as synced (is_synced=1)
    pub fn mark_synced(&self, ids: &[i64]) {
        let mut inflight = self.inflight_index.write();

        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.is_synced = 1;
            }
            inflight.remove(&id);
        }

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Reset failed sync back to unsynced (is_synced=2 -> 0)
    pub fn reset_to_unsynced(&self, id: i64) {
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        if let Some(mut log) = self.logs.get_mut(&id) {
            log.is_synced = 0;
        }
        unsynced.insert(id);
        inflight.remove(&id);

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Get all unpersisted logs for checkpointing
    pub fn get_unpersisted(&self) -> Vec<MemoryLog> {
        self.logs.iter()
            .filter(|entry| !entry.persisted)
            .map(|entry| entry.clone())
            .collect()
    }

    /// Mark logs as persisted after successful checkpoint
    pub fn mark_persisted(&self, ids: &[i64]) {
        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.persisted = true;
            }
        }
    }

    /// Get all logs for full checkpoint
    pub fn get_all(&self) -> Vec<MemoryLog> {
        self.logs.iter().map(|entry| entry.clone()).collect()
    }

    /// Clear dirty flag after checkpoint
    pub fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::SeqCst);
    }

    /// Reset in-flight logs that have been stuck for too long
    pub fn reset_stale_inflight(&self, timeout: std::time::Duration) -> usize {
        let now = std::time::Instant::now();
        let mut to_reset = Vec::new();

        {
            let inflight = self.inflight_index.read();
            for (&id, &started_at) in inflight.iter() {
                if now.duration_since(started_at) > timeout {
                    to_reset.push(id);
                }
            }
        }

        for id in &to_reset {
            self.reset_to_unsynced(*id);
        }

        to_reset.len()
    }

    /// Remove synced logs older than threshold (memory cleanup)
    /// IMPORTANT: Signatures are NOT removed - they provide ongoing dedup protection
    pub fn cleanup_synced(&self, max_age_secs: u64) -> usize {
        let threshold = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(max_age_secs as i64);
        let mut to_remove = Vec::new();

        for entry in self.logs.iter() {
            if entry.is_synced == 1 && entry.created_at < threshold {
                to_remove.push(*entry.key());
            }
        }

        let count = to_remove.len();
        for id in to_remove {
            self.logs.remove(&id);
            // NOTE: DO NOT remove signature - keep for dedup protection
        }
        self.count.fetch_sub(count, Ordering::SeqCst);

        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_store() {
        let store = InMemoryStore::new(1000);
        assert_eq!(store.count_total(), 0);
        assert_eq!(store.count_unsynced(), 0);
        assert!(!store.is_dirty());
    }

    #[test]
    fn test_insert() {
        let store = InMemoryStore::new(1000);
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };

        let id = store.insert("D1".to_string(), data).unwrap();
        assert_eq!(id, 1);
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_unsynced(), 1);
        assert!(store.is_dirty());
    }

    #[test]
    fn test_insert_capacity_exceeded() {
        let store = InMemoryStore::new(2);

        // Insert 2 logs (should succeed)
        for i in 1..=2 {
            let data = AttendanceData {
                user_id: format!("U{}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        // Insert 3rd log (should fail)
        let data = AttendanceData {
            user_id: "U3".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let result = store.insert("D1".to_string(), data);
        assert!(matches!(result, Err(StoreError::CapacityExceeded)));
    }

    #[test]
    fn test_get_unsynced_fifo_order() {
        let store = InMemoryStore::new(1000);

        // Insert logs in order
        let users = vec!["FIRST", "SECOND", "THIRD"];
        for user in &users {
            let data = AttendanceData {
                user_id: user.to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        // Get unsynced logs
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 3);

        // Verify FIFO order
        assert_eq!(logs[0].user_id, "FIRST");
        assert_eq!(logs[1].user_id, "SECOND");
        assert_eq!(logs[2].user_id, "THIRD");

        // Verify IDs are ascending
        assert!(logs[1].id > logs[0].id);
        assert!(logs[2].id > logs[1].id);
    }

    #[test]
    fn test_mark_in_flight() {
        let store = InMemoryStore::new(1000);

        // Insert 3 logs
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        // Get logs and mark as in-flight
        let logs = store.get_unsynced(2);
        assert_eq!(logs.len(), 2);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();

        store.mark_in_flight(&ids);

        // Verify unsynced count decreased
        assert_eq!(store.count_unsynced(), 1);

        // Verify in-flight logs are not returned
        let remaining = store.get_unsynced(10);
        assert_eq!(remaining.len(), 1);
    }

    #[test]
    fn test_mark_synced() {
        let store = InMemoryStore::new(1000);

        // Insert and mark as in-flight
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();
        store.mark_in_flight(&[id]);

        // Mark as synced
        store.mark_synced(&[id]);

        // Verify log is synced
        let log = store.logs.get(&id).unwrap();
        assert_eq!(log.is_synced, 1);
        assert_eq!(store.count_unsynced(), 0);
    }

    #[test]
    fn test_reset_to_unsynced() {
        let store = InMemoryStore::new(1000);

        // Insert and mark as in-flight
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();
        store.mark_in_flight(&[id]);
        assert_eq!(store.count_unsynced(), 0);

        // Reset to unsynced
        store.reset_to_unsynced(id);

        // Verify log is back to unsynced
        let log = store.logs.get(&id).unwrap();
        assert_eq!(log.is_synced, 0);
        assert_eq!(store.count_unsynced(), 1);
    }

    #[test]
    fn test_insert_batch() {
        let store = InMemoryStore::new(1000);

        let batch = vec![
            ("D1".to_string(), AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }),
            ("D1".to_string(), AttendanceData {
                user_id: "U2".to_string(),
                timestamp: "2026-01-29 10:01:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }),
            ("D1".to_string(), AttendanceData {
                user_id: "U3".to_string(),
                timestamp: "2026-01-29 10:02:00".to_string(),
                status: Some(1),
                punch: Some(1),
            }),
        ];

        let inserted = store.insert_batch(batch).unwrap();
        assert_eq!(inserted, 3);
        assert_eq!(store.count_total(), 3);
        assert_eq!(store.count_unsynced(), 3);
    }

    #[test]
    fn test_get_unpersisted() {
        let store = InMemoryStore::new(1000);

        // Insert 2 logs (unpersisted)
        for i in 1..=2 {
            let data = AttendanceData {
                user_id: format!("U{}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        // All logs should be unpersisted
        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 2);

        // Mark one as persisted
        store.mark_persisted(&[1]);
        let unpersisted = store.get_unpersisted();
        assert_eq!(unpersisted.len(), 1);
    }

    #[test]
    fn test_cleanup_synced() {
        let store = InMemoryStore::new(1000);

        // Insert 3 logs
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        // Mark first 2 as synced
        store.mark_in_flight(&[1, 2]);
        store.mark_synced(&[1, 2]);

        // Manually set old created_at for synced logs
        for id in [1, 2] {
            if let Some(mut log) = store.logs.get_mut(&id) {
                log.created_at = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(7200);
            }
        }

        // Cleanup logs older than 1 hour (3600 seconds)
        let removed = store.cleanup_synced(3600);
        assert_eq!(removed, 2);
        assert_eq!(store.count_total(), 1);
    }

    #[test]
    fn test_load_from_disk() {
        let store = InMemoryStore::new(1000);

        let logs = vec![
            RawAttendanceLog {
                id: 1,
                device_id: "D1".to_string(),
                user_id: "U1".to_string(),
                timestamp: chrono::NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                status: Some(1),
                punch: Some(1),
                is_synced: 0,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: None,
                retry_count: 0,
            },
            RawAttendanceLog {
                id: 2,
                device_id: "D1".to_string(),
                user_id: "U2".to_string(),
                timestamp: chrono::NaiveDateTime::parse_from_str("2026-01-29 10:01:00", "%Y-%m-%d %H:%M:%S").unwrap(),
                status: Some(1),
                punch: Some(1),
                is_synced: 2,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: Some(chrono::Utc::now().naive_utc()),
                retry_count: 1,
            },
        ];

        store.load_from_disk(logs);

        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_unsynced(), 1); // Only first log is unsynced

        // Verify next_id is set correctly
        let data = AttendanceData {
            user_id: "U3".to_string(),
            timestamp: "2026-01-29 10:02:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();
        assert_eq!(id, 3); // Should continue from max_id + 1
    }

    #[test]
    fn test_concurrent_insert_and_mark() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(InMemoryStore::new(10000));
        let mut handles = vec![];

        // Insert thread
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            for i in 0..50 {
                let data = AttendanceData {
                    user_id: format!("U{:03}", i),
                    timestamp: "2026-01-29 10:00:00".to_string(),
                    status: Some(1),
                    punch: Some(1),
                };
                store_clone.insert("D1".to_string(), data).unwrap();
            }
        });
        handles.push(handle);

        // Mark in-flight thread
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let logs = store_clone.get_unsynced(25);
            let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
            store_clone.mark_in_flight(&ids);
        });
        handles.push(handle);

        // Mark synced thread
        let store_clone = store.clone();
        let handle = thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(20));
            let logs = store_clone.get_unsynced(10);
            let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
            store_clone.mark_synced(&ids);
        });
        handles.push(handle);

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(store.count_total(), 50);
    }

    #[test]
    fn test_reset_stale_inflight() {
        let store = InMemoryStore::new(1000);

        // Insert and mark in-flight
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{:03}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        let logs = store.get_unsynced(3);
        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        store.mark_in_flight(&ids);

        assert_eq!(store.count_unsynced(), 0);

        // Reset stale (with 0 timeout)
        let reset = store.reset_stale_inflight(std::time::Duration::from_secs(0));

        assert_eq!(reset, 3);
        assert_eq!(store.count_unsynced(), 3);
    }

    #[test]
    fn test_seed_id() {
        let store = InMemoryStore::new(1000);

        // Seed with starting ID 1000
        store.seed_id(1000);

        // Insert first log
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();

        // Should be at least 1000 + gap (1000 ID gap)
        assert!(id >= 2000, "ID {} should be >= 2000", id);
    }

    #[test]
    fn test_multiple_mark_synced_calls() {
        let store = InMemoryStore::new(1000);

        // Insert 5 logs
        let mut ids = Vec::new();
        for i in 1..=5 {
            let data = AttendanceData {
                user_id: format!("U{:03}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            let id = store.insert("D1".to_string(), data).unwrap();
            ids.push(id);
        }

        // Mark in-flight
        store.mark_in_flight(&ids);

        // Mark synced in two batches
        store.mark_synced(&ids[0..2]);
        store.mark_synced(&ids[2..5]);

        // All should be synced
        for id in &ids {
            let log = store.logs.get(id).unwrap();
            assert_eq!(log.is_synced, 1);
        }
    }

    #[test]
    fn test_get_unsynced_limit() {
        let store = InMemoryStore::new(1000);

        // Insert 10 logs
        for i in 1..=10 {
            let data = AttendanceData {
                user_id: format!("U{:03}", i),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap();
        }

        // Get with limit
        let logs_3 = store.get_unsynced(3);
        let logs_5 = store.get_unsynced(5);
        let logs_all = store.get_unsynced(100);

        assert_eq!(logs_3.len(), 3);
        assert_eq!(logs_5.len(), 5);
        assert_eq!(logs_all.len(), 10);
    }

    #[test]
    fn test_invalid_timestamp_iso_format() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29T10:00:00Z".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // Should accept ISO format
        let result = store.insert("DEV1".to_string(), data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_dirty_flag() {
        let store = InMemoryStore::new(1000);

        // Insert log (sets dirty)
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("D1".to_string(), data).unwrap();

        assert!(store.is_dirty());

        // Clear dirty
        store.clear_dirty();

        assert!(!store.is_dirty());
    }

    #[test]
    fn test_get_all_logs() {
        let store = InMemoryStore::new(1000);

        // Insert 3 logs in different states
        let id1 = {
            let data = AttendanceData {
                user_id: "U1".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap()
        };

        let id2 = {
            let data = AttendanceData {
                user_id: "U2".to_string(),
                timestamp: "2026-01-29 10:01:00".to_string(),
                status: Some(1),
                punch: Some(1),
            };
            store.insert("D1".to_string(), data).unwrap()
        };

        // Mark one as in-flight, one as synced
        store.mark_in_flight(&[id1]);
        store.mark_synced(&[id1]);
        store.mark_in_flight(&[id2]);

        // Get all should return both
        let all = store.get_all();
        assert_eq!(all.len(), 2);
    }

    // ===== Exponential Backoff Tests =====

    #[test]
    fn test_backoff_new_logs_immediate() {
        let store = InMemoryStore::new(1000);

        // Insert new log (retry_count=0, updated_at=None)
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("D1".to_string(), data).unwrap();

        // New log should be returned immediately
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_backoff_skips_recent_retry() {
        let store = InMemoryStore::new(1000);

        // Insert and simulate first retry
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();

        // Mark in-flight (sets retry_count=1, updated_at=now)
        store.mark_in_flight(&[id]);
        store.reset_to_unsynced(id);

        // Should NOT be returned (within 2s backoff)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 0);
        assert_eq!(store.count_in_backoff(), 1);
    }

    #[test]
    fn test_backoff_allows_after_delay() {
        let store = InMemoryStore::new(1000);

        // Insert log
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();

        // Simulate retry with old updated_at (3 seconds ago)
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.retry_count = 1;
            log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(3));
        }

        // Should be returned (past 2s backoff for retry_count=1)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_backoff_cap_at_300_seconds() {
        let store = InMemoryStore::new(1000);

        // Insert log with high retry count
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id = store.insert("D1".to_string(), data).unwrap();

        // Simulate many retries (retry_count=10, beyond 2^8=256s)
        // With jitter, delay can be 300 ± 10% = 270-330s
        // Use 200s ago - definitely still in backoff (less than min 270s)
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.retry_count = 10;
            log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(200));
        }

        // Should NOT be returned (200s < min backoff 270s)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 0);

        // After 400 seconds (well past max 330s with jitter), should be eligible
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.updated_at = Some(chrono::Utc::now().naive_utc() - chrono::Duration::seconds(400));
        }
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_backoff_new_logs_prioritized() {
        let store = InMemoryStore::new(1000);

        // Insert retry log (in backoff)
        let data1 = AttendanceData {
            user_id: "RETRY".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let id1 = store.insert("D1".to_string(), data1).unwrap();
        store.mark_in_flight(&[id1]);
        store.reset_to_unsynced(id1);

        // Insert new log
        let data2 = AttendanceData {
            user_id: "NEW".to_string(),
            timestamp: "2026-01-29 10:00:01".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert("D1".to_string(), data2).unwrap();

        // Only NEW should be returned (RETRY is in backoff)
        let logs = store.get_unsynced(10);
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "NEW");
    }

    // ===== Phase 1: Signature Tests =====

    #[test]
    fn test_compute_signature_deterministic() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        // Multiple calls with same input
        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));
        let sig3 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));

        assert_eq!(sig1, sig2);
        assert_eq!(sig2, sig3);
    }

    #[test]
    fn test_compute_signature_different_devices() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV2", "USER001", ts, Some(0), Some(1));

        assert_ne!(sig1, sig2, "Different device should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_users() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER002", ts, Some(0), Some(1));

        assert_ne!(sig1, sig2, "Different user should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_timestamps() {
        let ts1 = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let ts2 = NaiveDateTime::parse_from_str("2026-01-29 10:00:01", "%Y-%m-%d %H:%M:%S").unwrap();

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts1, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts2, Some(0), Some(1));

        assert_ne!(sig1, sig2, "Different timestamp should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_punch() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(1), Some(1));

        assert_ne!(sig1, sig2, "Different punch should produce different signature");
    }

    #[test]
    fn test_compute_signature_different_status() {
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let sig1 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(1));
        let sig2 = InMemoryStore::compute_signature("DEV1", "USER001", ts, Some(0), Some(2));

        assert_ne!(sig1, sig2, "Different status should produce different signature");
    }

    #[test]
    fn test_signature_count() {
        let store = InMemoryStore::new(1000);
        assert_eq!(store.count_signatures(), 0);
    }

    // ===== Phase 2: Deduplication Tests =====

    #[test]
    fn test_insert_rejects_exact_duplicate() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // First insert succeeds
        let result1 = store.insert("DEV1".to_string(), data.clone());
        assert!(result1.is_ok());

        // Second insert fails
        let result2 = store.insert("DEV1".to_string(), data);
        assert!(matches!(result2, Err(StoreError::Duplicate)));

        // Only 1 log in store
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_signatures(), 1);
    }

    #[test]
    fn test_insert_allows_different_timestamp() {
        let store = InMemoryStore::new(1000);

        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data2 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:01".to_string(), // 1 second later
            status: Some(1),
            punch: Some(0),
        };

        // Both should succeed
        assert!(store.insert("DEV1".to_string(), data1).is_ok());
        assert!(store.insert("DEV1".to_string(), data2).is_ok());

        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_signatures(), 2);
    }

    #[test]
    fn test_insert_allows_different_punch_same_timestamp() {
        let store = InMemoryStore::new(1000);

        // Same device, user, timestamp, status - but different punch
        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(),
            status: Some(1),
            punch: Some(0), // check-in
        };
        let data2 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(), // Same timestamp
            status: Some(1),
            punch: Some(1), // check-out
        };

        // Both should succeed - this is the critical bug fix
        let result1 = store.insert("DEV1".to_string(), data1);
        assert!(result1.is_ok(), "First insert (punch=0) should succeed");

        let result2 = store.insert("DEV1".to_string(), data2);
        assert!(result2.is_ok(), "Second insert (punch=1) should succeed - NOT duplicate");

        assert_eq!(store.count_total(), 2, "Both logs should be stored");
        assert_eq!(store.count_signatures(), 2, "Both signatures should be stored");
    }

    #[test]
    fn test_insert_allows_different_status_same_timestamp() {
        let store = InMemoryStore::new(1000);

        // Same device, user, timestamp, punch - but different status
        let data1 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let data2 = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 08:00:00".to_string(), // Same timestamp
            status: Some(2), // Different status
            punch: Some(0),
        };

        // Both should succeed
        assert!(store.insert("DEV1".to_string(), data1).is_ok());
        assert!(store.insert("DEV1".to_string(), data2).is_ok());

        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_signatures(), 2);
    }

    #[test]
    fn test_cleanup_synced_preserves_signatures() {
        let store = InMemoryStore::new(1000);

        let data = AttendanceData {
            user_id: "U001".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        // Insert and sync
        let id = store.insert("DEV1".to_string(), data.clone()).unwrap();
        store.mark_in_flight(&[id]);
        store.mark_synced(&[id]);

        // Set old created_at
        if let Some(mut log) = store.logs.get_mut(&id) {
            log.created_at = chrono::Utc::now().naive_utc() - chrono::Duration::hours(2);
        }

        // Cleanup (1 hour threshold)
        let cleaned = store.cleanup_synced(3600);
        assert_eq!(cleaned, 1);

        // Log removed, but signature remains
        assert_eq!(store.count_total(), 0);
        assert_eq!(store.count_signatures(), 1);

        // Duplicate still rejected
        let result = store.insert("DEV1".to_string(), data);
        assert!(matches!(result, Err(StoreError::Duplicate)));
    }

    #[test]
    fn test_batch_insert_handles_duplicates() {
        let store = InMemoryStore::new(1000);

        let batch = vec![
            ("DEV1".to_string(), AttendanceData {
                user_id: "U001".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            }),
            ("DEV1".to_string(), AttendanceData {
                user_id: "U001".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(), // Duplicate of first
                status: Some(1),
                punch: Some(0),
            }),
            ("DEV1".to_string(), AttendanceData {
                user_id: "U002".to_string(),
                timestamp: "2026-01-29 10:00:00".to_string(), // Different user
                status: Some(1),
                punch: Some(0),
            }),
        ];

        let inserted = store.insert_batch(batch).unwrap();

        assert_eq!(inserted, 2, "Should insert 2, skip 1 duplicate");
        assert_eq!(store.count_total(), 2);
        assert_eq!(store.count_signatures(), 2);
    }

    #[test]
    fn test_load_from_disk_populates_signatures() {
        let store = InMemoryStore::new(1000);
        let ts = NaiveDateTime::parse_from_str("2026-01-29 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let logs = vec![
            RawAttendanceLog {
                id: 1,
                device_id: "D1".to_string(),
                user_id: "U1".to_string(),
                timestamp: ts,
                status: Some(1),
                punch: Some(1),
                is_synced: 0,
                created_at: Some(chrono::Utc::now().naive_utc()),
                updated_at: None,
                retry_count: 0,
            },
        ];

        store.load_from_disk(logs);

        // Signature should be populated
        assert_eq!(store.count_signatures(), 1);

        // Insert same log should be rejected
        let data = AttendanceData {
            user_id: "U1".to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(1),
        };
        let result = store.insert("D1".to_string(), data);
        assert!(matches!(result, Err(StoreError::Duplicate)));
    }

    #[test]
    fn test_concurrent_inserts_no_duplicates() {
        use std::sync::Arc;
        use std::thread;

        let store = Arc::new(InMemoryStore::new(10000));
        let mut handles = vec![];

        // 10 threads trying to insert same log
        for _ in 0..10 {
            let store_clone = store.clone();
            let handle = thread::spawn(move || {
                let data = AttendanceData {
                    user_id: "CONCURRENT_USER".to_string(),
                    timestamp: "2026-01-29 12:00:00".to_string(),
                    status: Some(1),
                    punch: Some(0),
                };
                store_clone.insert("CONCURRENT_DEV".to_string(), data)
            });
            handles.push(handle);
        }

        let mut success_count = 0;
        let mut duplicate_count = 0;

        for handle in handles {
            match handle.join().unwrap() {
                Ok(_) => success_count += 1,
                Err(StoreError::Duplicate) => duplicate_count += 1,
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        // Exactly 1 should succeed, 9 should be duplicates
        assert_eq!(success_count, 1, "Exactly 1 insert should succeed");
        assert_eq!(duplicate_count, 9, "9 should be duplicates");
        assert_eq!(store.count_total(), 1);
        assert_eq!(store.count_signatures(), 1);
    }
}
