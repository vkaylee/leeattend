use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicI64, AtomicUsize, AtomicBool, Ordering};
use std::time::Instant;
use std::sync::Arc;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;
use crate::models::{RawAttendanceLog, AttendanceData, SignatureRow};

use super::types::{MemoryLog, StoreError};
use super::backoff::calculate_backoff_with_jitter;

/// Pending sync update entry: (device_id, user_id, timestamp, punch, status, is_synced)
type PendingSyncUpdates = Arc<RwLock<Vec<(Arc<str>, Arc<str>, i64, Option<i32>, Option<i32>, i64)>>>;

/// In-memory store for attendance logs with FIFO ordering
pub struct InMemoryStore {
    logs: DashMap<i64, MemoryLog>,
    signatures: DashMap<u64, i64>,  // Hash signature -> timestamp_utc for dedup + TTL eviction
    composite_index: DashMap<u64, i64>,  // composite_key_hash -> memory_id for ACK correlation
    unsynced_index: Arc<RwLock<BTreeSet<i64>>>,
    unpersisted_index: Arc<RwLock<BTreeSet<i64>>>,
    inflight_index: Arc<RwLock<HashMap<i64, Instant>>>,
    pending_sync_updates: PendingSyncUpdates,
    next_id: AtomicI64,
    dirty: AtomicBool,
    count: AtomicUsize,
    max_capacity: usize,
}

impl InMemoryStore {
    /// Create a new in-memory store with specified capacity
    pub fn new(max_capacity: usize) -> Self {
        Self {
            logs: DashMap::with_capacity(max_capacity / 2),
            signatures: DashMap::with_capacity(max_capacity),
            composite_index: DashMap::with_capacity(max_capacity),
            unsynced_index: Arc::new(RwLock::new(BTreeSet::new())),
            unpersisted_index: Arc::new(RwLock::new(BTreeSet::new())),
            inflight_index: Arc::new(RwLock::new(HashMap::new())),
            pending_sync_updates: Arc::new(RwLock::new(Vec::new())),
            next_id: AtomicI64::new(1),
            dirty: AtomicBool::new(false),
            count: AtomicUsize::new(0),
            max_capacity,
        }
    }

    /// Compute 64-bit signature for deduplication.
    /// Signature = hash(device_id, user_id, timestamp_unix, punch, status)
    fn compute_signature(device_id: &str, user_id: &str, timestamp: i64, punch: Option<i32>, status: Option<i32>) -> u64 {
        let mut hasher = FxHasher::default();
        device_id.hash(&mut hasher);
        user_id.hash(&mut hasher);
        // Use Unix timestamp for deterministic hashing
        timestamp.hash(&mut hasher);
        punch.hash(&mut hasher);
        status.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if a signature already exists (public for testing)
    pub fn has_signature(&self, device_id: &str, user_id: &str, timestamp: i64, punch: Option<i32>, status: Option<i32>) -> bool {
        let sig = Self::compute_signature(device_id, user_id, timestamp, punch, status);
        self.signatures.contains_key(&sig)
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
            self.signatures.insert(sig, row.timestamp);
        }
    }

    /// Remove a batch of signatures (Runtime optimization: unload disabled devices)
    pub fn remove_signatures(&self, rows: Vec<SignatureRow>) {
        for row in rows {
            let sig = Self::compute_signature(&row.device_id, &row.user_id, row.timestamp, row.punch, row.status);
            self.signatures.remove(&sig);
        }
    }

    /// Remove all logs and indices for a specific device (Runtime optimization)
    /// Cleans up: logs, composite_index, and updates counts.
    /// Does NOT remove signatures from the independent set (caller must use remove_signatures for that)
    pub fn remove_logs_for_device(&self, device_id: &str) {
        let mut to_remove = Vec::new();
        
        // 1. Identify logs to remove
        for entry in self.logs.iter() {
            if &*entry.device_id == device_id {
                to_remove.push((*entry.key(), entry.composite_key_hash()));
            }
        }

        // 2. Remove them
        let count = to_remove.len();
        let mut unsynced = self.unsynced_index.write();
        let mut unpersisted = self.unpersisted_index.write();
        let mut inflight = self.inflight_index.write();

        for (id, composite_hash) in to_remove {
            self.logs.remove(&id);
            self.composite_index.remove(&composite_hash);
            unsynced.remove(&id);
            unpersisted.remove(&id);
            inflight.remove(&id);
        }

        self.count.fetch_sub(count, Ordering::SeqCst);
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
            self.signatures.insert(signature, log.timestamp);

            let memory_log = MemoryLog {
                id: log.id,
                device_id: Arc::from(log.device_id.as_str()),
                user_id: Arc::from(log.user_id.as_str()),
                timestamp: log.timestamp,
                status: log.status,
                punch: log.punch,
                is_synced: log.is_synced,
                created_at: log.created_at.unwrap_or_else(|| chrono::Utc::now().naive_utc()),
                updated_at: log.updated_at,
                retry_count: log.retry_count,
                persisted: true, // Loaded from disk = already persisted
            };

            // Compute composite key hash for ACK correlation
            let composite_hash = MemoryLog::compute_composite_hash(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);

            // Update indices based on sync status
            if log.is_synced == 0 {
                log::debug!("load_from_disk: adding log ID {} to unsynced_index", log.id);
                unsynced.insert(log.id);
            } else if log.is_synced == 2 {
                log::debug!("load_from_disk: adding log ID {} to inflight_index", log.id);
                inflight.insert(log.id, Instant::now());
            }

            self.logs.insert(log.id, memory_log);
            self.composite_index.insert(composite_hash, log.id);
        }

        self.next_id.store(max_id + 1, Ordering::SeqCst);
        self.count.store(self.logs.len(), Ordering::SeqCst);
    }

    /// Insert a new attendance log. Returns assigned ID.
    /// Accepts pre-converted UTC Unix timestamp (i64) to avoid re-parsing
    pub fn insert(&self, device_id: String, data: AttendanceData, timestamp_utc: i64) -> Result<i64, StoreError> {
        // Check capacity first (before inserting signature)
        if self.count.load(Ordering::Relaxed) >= self.max_capacity {
            return Err(StoreError::CapacityExceeded);
        }

        // Atomic check+insert signature to prevent TOCTOU race on duplicates
        let signature = Self::compute_signature(&device_id, &data.user_id, timestamp_utc, data.punch, data.status);
        use dashmap::mapref::entry::Entry;
        match self.signatures.entry(signature) {
            Entry::Occupied(_) => return Err(StoreError::Duplicate),
            Entry::Vacant(e) => { e.insert(timestamp_utc); }
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let now = chrono::Utc::now().naive_utc();

        // Compute composite hash BEFORE moving strings into Arc
        let composite_hash = MemoryLog::compute_composite_hash(&device_id, &data.user_id, timestamp_utc, data.punch, data.status);

        let log = MemoryLog {
            id,
            device_id: Arc::from(device_id.as_str()),
            user_id: Arc::from(data.user_id.as_str()),
            timestamp: timestamp_utc,
            status: data.status,
            punch: data.punch,
            is_synced: 0,
            created_at: now,
            updated_at: None,
            retry_count: 0,
            persisted: false,
        };

        self.logs.insert(id, log);
        self.composite_index.insert(composite_hash, id);
        self.unsynced_index.write().insert(id);
        self.unpersisted_index.write().insert(id);
        self.count.fetch_add(1, Ordering::SeqCst);
        self.dirty.store(true, Ordering::SeqCst);

        Ok(id)
    }

    /// Batch insert with automatic capacity check
    /// Accepts pre-converted UTC Unix timestamps (i64) to avoid re-parsing
    pub fn insert_batch(&self, batch: Vec<(Arc<String>, AttendanceData, i64)>) -> Result<usize, StoreError> {
        if batch.is_empty() {
            return Err(StoreError::CapacityExceeded); // Maintain original behavior for empty batch
        }

        let available = self.max_capacity.saturating_sub(self.count.load(Ordering::Relaxed));
        if available == 0 {
            return Err(StoreError::CapacityExceeded);
        }

        let mut inserted = 0;
        let mut unsynced = self.unsynced_index.write();
        let mut unpersisted = self.unpersisted_index.write();

        for (device_id, data, timestamp_utc) in batch.into_iter() {
            if inserted >= available {
                break; // Capacity limit reached
            }

            // Check for duplicate
            let signature = Self::compute_signature(&device_id, &data.user_id, timestamp_utc, data.punch, data.status);
            if self.signatures.contains_key(&signature) {
                continue; // Skip duplicate
            }

            // Add signature FIRST
            self.signatures.insert(signature, timestamp_utc);

            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let now = chrono::Utc::now().naive_utc();

            // Compute composite hash for ACK correlation
            let composite_hash = MemoryLog::compute_composite_hash(&device_id, &data.user_id, timestamp_utc, data.punch, data.status);

            let log = MemoryLog {
                id,
                device_id: Arc::from(device_id.as_str()), // Optimized: convert Arc<String> to Arc<str> (still alloc if not reusing, but better to keep Arc<String> in MemoryLog if possible? MemoryLog uses Arc<str>)
                // Actually MemoryLog uses Arc<str>. Arc<String> is not directly Arc<str>. 
                // But we can just use device_id.as_str() and existing logic. 
                // Wait, if we want to avoid allocation, MemoryLog should probably use Arc<String> or we pass Arc<str>.
                // For now, let's keep Arc<str> but at least we saved cloning in the channel.
                // device_id here is Arc<String>. device_id.as_str() gets &str. Arc::from(&str) allocates.
                // IMPORTANT: To fully avoid alloc, MemoryLog should be generic or use Arc<String>.
                // Taking a step back: Arc<String> -> Arc<str> is not zero-cost conversion without realloc.
                // However, the massive benefit is avoiding String clone in main.rs loop and channel.
                // Here we still alloc for MemoryLog, which is acceptable for now.
                user_id: Arc::from(data.user_id.as_str()),
                timestamp: timestamp_utc,
                status: data.status,
                punch: data.punch,
                is_synced: 0,
                created_at: now,
                updated_at: None,
                retry_count: 0,
                persisted: false,
            };

            self.logs.insert(id, log);
            self.composite_index.insert(composite_hash, id);
            unsynced.insert(id);
            unpersisted.insert(id);
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
        let count = unsynced.len();
        if count > 0 {
            log::debug!("get_unsynced: found {count} IDs in unsynced_index");
        }
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
            } else {
                log::warn!("⚠️ get_unsynced: log ID {id} found in unsynced_index but NOT in logs map");
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

    /// Mark logs as synced and remove from memory (is_synced=1)
    /// Keeps signature for dedup protection, removes log entry
    /// Rationale: Once synced, disk has authoritative record
    /// Safety: Only removes logs that are already persisted to disk
    pub fn mark_synced(&self, ids: &[i64]) {
        let mut inflight = self.inflight_index.write();
        let mut pending_updates = self.pending_sync_updates.write();

        for &id in ids {
            // Safety check: only remove if persisted to disk
            let should_remove = self.logs.get(&id)
                .map(|log| log.persisted)
                .unwrap_or(false);

            if should_remove && let Some(log) = self.logs.get(&id) {
                    pending_updates.push((
                        log.device_id.clone(),
                        log.user_id.clone(),
                        log.timestamp,
                        log.punch,
                        log.status,
                        1,
                    ));
                }

            if !should_remove {
                // Log not persisted yet - mark as synced so checkpoint persists it correctly
                if let Some(mut log) = self.logs.get_mut(&id) {
                    log.is_synced = 1;
                    log.updated_at = Some(chrono::Utc::now().naive_utc());
                }
                // Remove from inflight so it's not retried/reset
                inflight.remove(&id);
                continue;
            }

            // Get composite hash and log data before removing
            let composite_hash = self.logs.get(&id).map(|log| log.composite_key_hash());
            let _signature_to_remove = self.logs.get(&id).map(|log| {
                Self::compute_signature(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status)
            });

            // Remove from memory (log no longer needed)
            self.logs.remove(&id);

            // Remove from composite index
            if let Some(hash) = composite_hash {
                self.composite_index.remove(&hash);
            }

            // Remove from inflight
            inflight.remove(&id);

            // Note: Signature is NOT removed - preserved for dedup protection
            // This allows the system to reject re-insertions of the same data even after sync

            // Decrement count
            self.count.fetch_sub(1, Ordering::SeqCst);
        }

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Reset failed sync back to unsynced (is_synced=2 -> 0)
    pub fn reset_to_unsynced(&self, id: i64) {
        let mut unsynced = self.unsynced_index.write();
        let mut inflight = self.inflight_index.write();

        if let Some(mut log) = self.logs.get_mut(&id) {
            log.is_synced = 0;
            if log.persisted {
                self.pending_sync_updates.write().push((
                    log.device_id.clone(),
                    log.user_id.clone(),
                    log.timestamp,
                    log.punch,
                    log.status,
                    0,
                ));
            }
        }
        unsynced.insert(id);
        inflight.remove(&id);

        self.dirty.store(true, Ordering::SeqCst);
    }

    /// Get memory ID by composite key string (parses and computes hash)
    /// Format: "device_id:user_id:timestamp_unix:punch:status"
    /// Uses rsplit to safely handle device_id containing colons
    pub fn get_id_by_composite_key(&self, composite_key: &str) -> Option<i64> {
        let mut parts = composite_key.rsplitn(4, ':');
        
        // 1. Extract status, punch, timestamp
        let status_str = parts.next()?;
        let punch_str = parts.next()?;
        let timestamp_str = parts.next()?;
        let rest = parts.next()?; // device_id:user_id

        let status: Option<i32> = status_str.parse().ok().filter(|&v| v != -1);
        let punch: Option<i32> = punch_str.parse().ok().filter(|&v| v != -1);
        let timestamp_unix: i64 = timestamp_str.parse().ok()?;

        // 2. Extract Device ID and User ID
        // Strategy: If device_id looks like a UUID (36 chars), split from left.
        // Otherwise, assume strict "no colons in user_id" rule (legacy MAC address support).
        let (device_id, user_id) = if rest.len() > 36 && rest.as_bytes()[36] == b':' {
            // Assume UUID device_id
            (&rest[..36], &rest[37..]) // Skip the colon
        } else {
            // Fallback: Split from right (preserves colons in device_id, assumes none in user_id)
            match rest.rsplit_once(':') {
                Some((d, u)) => (d, u),
                None => return None,
            }
        };

        let hash = MemoryLog::compute_composite_hash(device_id, user_id, timestamp_unix, punch, status);
        self.composite_index.get(&hash).map(|entry| *entry)
    }

    /// Mark synced by composite key (for ACK handling)
    pub fn mark_synced_by_key(&self, composite_key: &str) -> bool {
        if let Some(id) = self.get_id_by_composite_key(composite_key) {
            self.mark_synced(&[id]);
            true
        } else {
            false
        }
    }

    /// Reset to unsynced by composite key (for failed ACK)
    pub fn reset_to_unsynced_by_key(&self, composite_key: &str) -> bool {
        if let Some(id) = self.get_id_by_composite_key(composite_key) {
            self.reset_to_unsynced(id);
            true
        } else {
            false
        }
    }

    /// Get all unpersisted logs for checkpointing (O(k) via index, not O(n) full scan)
    pub fn get_unpersisted(&self) -> Vec<MemoryLog> {
        let unpersisted = self.unpersisted_index.read();
        unpersisted.iter()
            .filter_map(|id| self.logs.get(id).map(|entry| entry.clone()))
            .collect()
    }

    /// Mark logs as persisted after successful checkpoint
    pub fn mark_persisted(&self, ids: &[i64]) {
        let mut unpersisted = self.unpersisted_index.write();
        for &id in ids {
            if let Some(mut log) = self.logs.get_mut(&id) {
                log.persisted = true;
            }
            unpersisted.remove(&id);
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

    /// Get and clear pending sync status updates for disk persistence
    pub fn get_and_clear_pending_sync_updates(&self) -> Vec<(Arc<str>, Arc<str>, i64, Option<i32>, Option<i32>, i64)> {
        self.pending_sync_updates.write().drain(..).collect()
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

    /// Evict signatures older than TTL. Returns (evicted, before, after).
    pub fn evict_old_signatures(&self, ttl_secs: i64) -> (usize, usize, usize) {
        let threshold = chrono::Utc::now().timestamp() - ttl_secs;
        let before = self.signatures.len();
        self.signatures.retain(|_hash, ts| *ts >= threshold);
        let after = self.signatures.len();
        (before - after, before, after)
    }

    /// Remove synced logs older than threshold (memory cleanup)
    /// IMPORTANT: Signatures are NOT removed - they provide ongoing dedup protection
    pub fn cleanup_synced(&self, max_age_secs: u64) -> usize {
        let threshold = chrono::Utc::now().naive_utc() - chrono::Duration::seconds(max_age_secs as i64);
        let mut to_remove = Vec::new();

        for entry in self.logs.iter() {
            if entry.is_synced == 1 && entry.created_at < threshold {
                to_remove.push((*entry.key(), entry.composite_key_hash()));
            }
        }

        let count = to_remove.len();
        for (id, composite_hash) in to_remove {
            self.logs.remove(&id);
            self.composite_index.remove(&composite_hash);
            // NOTE: DO NOT remove signature - keep for dedup protection
        }
        self.count.fetch_sub(count, Ordering::SeqCst);

        count
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;

