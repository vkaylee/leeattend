use redb::{ReadableTable, ReadableDatabase, ReadableTableMetadata};
use super::{RedbStore, StoredLog, StoreError, ATTENDANCE_LOGS, COMPOSITE_INDEX, SYNC_INDEX, SIGNATURES};

impl RedbStore {
    /// Mark logs as in-flight (sync state = 2).
    pub fn mark_in_flight(&self, ids: &[i64]) -> Result<(), StoreError> {
        if ids.is_empty() { return Ok(()); }

        let now = chrono::Utc::now().timestamp();

        // Phase 1: Read all logs to update
        let mut updates: Vec<(i64, Vec<u8>)> = Vec::with_capacity(ids.len());
        {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            for &id in ids {
                if let Some(bytes_guard) = log_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))? {
                    let mut log = StoredLog::from_bytes(bytes_guard.value())?;
                    log.is_synced = 2;
                    log.retry_count += 1;
                    log.updated_at = Some(now);
                    updates.push((id, log.to_bytes()));
                }
            }
        }

        // Phase 2: Write updates
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                log_table.insert(*id, bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table.insert(*id, 2u8).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Mark logs as synced (sync state = 1) and remove from active storage.
    /// Keeps signature for dedup protection.
    pub fn mark_synced(&self, ids: &[i64]) -> Result<(), StoreError> {
        if ids.is_empty() { return Ok(()); }

        // Phase 1: Read log data to compute composite hashes
        let mut removals: Vec<(i64, u64)> = Vec::with_capacity(ids.len());
        {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            for &id in ids {
                if let Some(bytes_guard) = log_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))? {
                    let log = StoredLog::from_bytes(bytes_guard.value())?;
                    let composite_hash = Self::compute_composite_hash(
                        &log.device_id, &log.user_id, log.timestamp, log.punch, log.status
                    );
                    removals.push((id, composite_hash));
                }
            }
        }

        // Phase 2: Remove from active storage
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut comp_table = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, composite_hash) in &removals {
                let _ = log_table.remove(*id);
                let _ = sync_table.remove(*id);
                let _ = comp_table.remove(*composite_hash);
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Reset log from in-flight back to unsynced (sync state 2 → 0).
    pub fn reset_to_unsynced(&self, id: i64) -> Result<(), StoreError> {
        // Phase 1: Read log
        let log_data: Option<Vec<u8>> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            log_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))?
                .map(|g| {
                    let mut log = StoredLog::from_bytes(g.value()).unwrap();
                    log.is_synced = 0;
                    log.to_bytes()
                })
        };

        if let Some(bytes) = log_data {
            let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
                let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
                log_table.insert(id, bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table.insert(id, 0u8).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    /// Mark synced by composite key (for ACK handling).
    pub fn mark_synced_by_key(&self, composite_key: &str) -> bool {
        if let Some(id) = self.get_id_by_composite_key(composite_key) {
            self.mark_synced(&[id]).is_ok()
        } else {
            false
        }
    }

    /// Reset to unsynced by composite key (for failed ACK).
    pub fn reset_to_unsynced_by_key(&self, composite_key: &str) -> bool {
        if let Some(id) = self.get_id_by_composite_key(composite_key) {
            self.reset_to_unsynced(id).is_ok()
        } else {
            false
        }
    }

    /// Reset all in-flight logs to unsynced (crash recovery).
    pub fn reset_syncing_status(&self) -> Result<u64, StoreError> {
        // Phase 1: Read — find in-flight logs
        let ids_to_reset: Vec<i64> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in sync_table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
                let (id_guard, state_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                if state_guard.value() == 2 {
                    result.push(id_guard.value());
                }
            }
            result
        };

        let count = ids_to_reset.len() as u64;
        if count == 0 { return Ok(0); }

        // Phase 2: Read existing logs and prepare updates
        let mut updates: Vec<(i64, Vec<u8>)> = Vec::new();
        {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            for &id in &ids_to_reset {
                if let Some(bytes_guard) = log_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))? {
                    let mut log = StoredLog::from_bytes(bytes_guard.value())?;
                    log.is_synced = 0;
                    updates.push((id, log.to_bytes()));
                }
            }
        }

        // Phase 3: Write updates
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                log_table.insert(*id, bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table.insert(*id, 0u8).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    /// Reset stale in-flight logs older than timeout to unsynced.
    pub fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().timestamp();

        // Phase 1: Read — find stale in-flight logs
        let ids_to_reset: Vec<i64> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;

            let mut result = Vec::new();
            for entry in sync_table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
                let (id_guard, state_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                if state_guard.value() == 2 {
                    let id = id_guard.value();
                    if let Some(bytes_guard) = log_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))? {
                        let log = StoredLog::from_bytes(bytes_guard.value())?;
                        if let Some(updated_at) = log.updated_at
                            && now - updated_at > timeout_secs {
                                result.push(id);
                            }
                    }
                }
            }
            result
        };

        if ids_to_reset.is_empty() { return Ok(0); }

        // Phase 2: Reset each (uses own read+write txns internally)
        for id in &ids_to_reset {
            self.reset_to_unsynced(*id)?;
        }
        Ok(ids_to_reset.len() as u64)
    }

    /// Get max log ID.
    pub fn get_max_id(&self) -> Result<i64, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(table.last().map_err(|e| StoreError::DbError(e.to_string()))?.map(|(k, _)| k.value()).unwrap_or(0))
    }

    /// Evict signatures older than TTL. Returns (evicted, before, after).
    pub fn evict_old_signatures(&self) -> Result<(usize, usize, usize), StoreError> {
        let threshold = chrono::Utc::now().timestamp() - (self.signature_ttl_days * 86400);

        // Phase 1: Read — find expired signatures
        let (before, to_remove) = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let sig_table = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;
            let before = sig_table.len().map_err(|e| StoreError::DbError(format!("{e}")))? as usize;
            let mut expired = Vec::new();
            for entry in sig_table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
                let (hash_guard, ts_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                if ts_guard.value() < threshold {
                    expired.push(hash_guard.value());
                }
            }
            (before, expired)
        };

        // Phase 2: Write — remove expired
        if !to_remove.is_empty() {
            let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut sig_table = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;
                for hash in &to_remove {
                    let _ = sig_table.remove(*hash);
                }
            }
            txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        }

        let evicted = to_remove.len();
        let after = before - evicted;
        Ok((evicted, before, after))
    }
}
