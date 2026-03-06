use redb::{ReadableTable, ReadableDatabase};
use crate::models::AttendanceData;
use super::{RedbStore, StoredLog, StoreError, ATTENDANCE_LOGS, SIGNATURES, COMPOSITE_INDEX, SYNC_INDEX};

impl RedbStore {
    /// Insert a single attendance log with dedup check.
    /// Returns assigned ID on success.
    pub fn insert(&self, device_id: &str, data: &AttendanceData, timestamp_utc: i64) -> Result<i64, StoreError> {
        let sig = Self::compute_signature(device_id, &data.user_id, timestamp_utc, data.punch, data.status);
        let composite_hash = Self::compute_composite_hash(device_id, &data.user_id, timestamp_utc, data.punch, data.status);

        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            // Check for duplicate
            let sig_table = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;
            if sig_table.get(sig).map_err(|e| StoreError::DbError(e.to_string()))?.is_some() {
                return Err(StoreError::Duplicate);
            }
        }

        let id = self.next_id();
        let now = chrono::Utc::now().timestamp();

        let log = StoredLog {
            id,
            device_id: device_id.to_string(),
            user_id: data.user_id.clone(),
            timestamp: timestamp_utc,
            status: data.status,
            punch: data.punch,
            is_synced: 0,
            created_at: now,
            updated_at: None,
            retry_count: 0,
        };

        let bytes = log.to_bytes();

        {
            let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            log_table.insert(id, bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;

            let mut sig_table = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;
            sig_table.insert(sig, timestamp_utc).map_err(|e| StoreError::DbError(e.to_string()))?;

            let mut comp_table = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            comp_table.insert(composite_hash, id).map_err(|e| StoreError::DbError(e.to_string()))?;

            let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            sync_table.insert(id, 0u8).map_err(|e| StoreError::DbError(e.to_string()))?;
        }

        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(id)
    }

    /// Batch insert with automatic dedup. Returns count of successfully inserted logs.
    pub fn insert_batch(&self, batch: &[(impl AsRef<str>, AttendanceData, i64)]) -> Result<usize, StoreError> {
        if batch.is_empty() {
            return Ok(0);
        }

        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut inserted = 0;

        {
            let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sig_table = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut comp_table = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;

            for (device_id, data, timestamp_utc) in batch {
                let device_id = device_id.as_ref();
                let sig = Self::compute_signature(device_id, &data.user_id, *timestamp_utc, data.punch, data.status);

                // Skip duplicate
                if sig_table.get(sig).map_err(|e| StoreError::DbError(e.to_string()))?.is_some() {
                    continue;
                }

                let id = self.next_id();
                let now = chrono::Utc::now().timestamp();
                let composite_hash = Self::compute_composite_hash(device_id, &data.user_id, *timestamp_utc, data.punch, data.status);

                let log = StoredLog {
                    id,
                    device_id: device_id.to_string(),
                    user_id: data.user_id.clone(),
                    timestamp: *timestamp_utc,
                    status: data.status,
                    punch: data.punch,
                    is_synced: 0,
                    created_at: now,
                    updated_at: None,
                    retry_count: 0,
                };

                let bytes = log.to_bytes();
                log_table.insert(id, bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
                sig_table.insert(sig, *timestamp_utc).map_err(|e| StoreError::DbError(e.to_string()))?;
                comp_table.insert(composite_hash, id).map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table.insert(id, 0u8).map_err(|e| StoreError::DbError(e.to_string()))?;
                inserted += 1;
            }
        }

        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(inserted)
    }

    /// Get unsynced logs in FIFO order (by ID) with exponential backoff filtering.
    pub fn get_unsynced(&self, limit: usize) -> Result<Vec<StoredLog>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
        let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;

        let now = chrono::Utc::now().timestamp();
        let mut result = Vec::with_capacity(limit);

        // Range scan in key order (i64 ascending = FIFO)
        let iter = sync_table.iter().map_err(|e| StoreError::DbError(e.to_string()))?;
        for entry in iter {
            if result.len() >= limit {
                break;
            }
            let (id_guard, state_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            let state = state_guard.value();
            if state != 0 {
                continue; // Only unsynced
            }

            let id = id_guard.value();
            if let Some(bytes_guard) = log_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))? {
                let log = StoredLog::from_bytes(bytes_guard.value())?;

                // Backoff check
                if let Some(updated_at) = log.updated_at {
                    let delay = Self::calculate_backoff(log.retry_count);
                    if now < updated_at + delay {
                        continue; // Still in backoff
                    }
                }

                result.push(log);
            }
        }

        Ok(result)
    }

    /// Count unsynced logs.
    pub fn count_unsynced(&self) -> Result<usize, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;

        let mut count = 0;
        let iter = sync_table.iter().map_err(|e| StoreError::DbError(e.to_string()))?;
        for entry in iter {
            let (_, state_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            if state_guard.value() == 0 {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Remove signatures for a specific device (used when disabling device).
    pub fn remove_signatures_for_device(&self, device_id: &str) -> Result<usize, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;

        let mut sigs_to_remove = Vec::new();
        let mut logs_to_remove = Vec::new();

        let iter = log_table.iter().map_err(|e| StoreError::DbError(e.to_string()))?;
        for entry in iter {
            let (id_guard, bytes_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            let log = StoredLog::from_bytes(bytes_guard.value())?;
            if log.device_id == device_id {
                let sig = Self::compute_signature(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
                sigs_to_remove.push(sig);
                let comp = Self::compute_composite_hash(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
                logs_to_remove.push((id_guard.value(), comp));
            }
        }
        drop(txn);

        let count = sigs_to_remove.len();
        if count == 0 {
            return Ok(0);
        }

        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut sig_table = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;
            for sig in &sigs_to_remove {
                sig_table.remove(*sig).map_err(|e| StoreError::DbError(e.to_string()))?;
            }

            let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut comp_table = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;

            for (id, comp) in &logs_to_remove {
                log_table.remove(*id).map_err(|e| StoreError::DbError(e.to_string()))?;
                comp_table.remove(*comp).map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table.remove(*id).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    /// Backoff calculation: exponential with cap at 300 seconds.
    fn calculate_backoff(retry_count: i64) -> i64 {
        let base: i64 = 5; // 5 seconds base
        let max: i64 = 300; // 5 minute cap
        std::cmp::min(base * (1 << std::cmp::min(retry_count, 6)), max)
    }
}
