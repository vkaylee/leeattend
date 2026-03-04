use redb::{ReadableTable, ReadableDatabase, ReadableTableMetadata};
use super::{RedbStore, StoredLog, StoredFailedLog, StoreError, ATTENDANCE_LOGS, SIGNATURES, COMPOSITE_INDEX, SYNC_INDEX, FAILED_LOGS, PARSING_ERRORS};

impl RedbStore {
    /// Clean up old synced logs beyond retention period (days).
    /// In Redb, synced logs are already removed from active storage by mark_synced().
    /// This cleans up any that might remain (e.g., from older logic).
    pub fn cleanup_old_synced_logs(&self, retention_days: i64) -> Result<u64, StoreError> {
        let cutoff = chrono::Utc::now().timestamp() - (retention_days * 86400);

        // Phase 1: Read — find old synced logs
        let to_remove: Vec<(i64, StoredLog)> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let sync_table = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;

            let mut result = Vec::new();
            for entry in log_table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
                let (id_guard, bytes_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                let id = id_guard.value();
                if let Some(state_guard) = sync_table.get(id).map_err(|e| StoreError::DbError(e.to_string()))?
                    && state_guard.value() == 1 {
                        let log = StoredLog::from_bytes(bytes_guard.value())?;
                        if log.created_at < cutoff {
                            result.push((id, log));
                        }
                    }
            }
            result
        };

        let count = to_remove.len() as u64;
        if count == 0 { return Ok(0); }

        // Phase 2: Write — remove old logs and their indices
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut lt = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut st = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut ct = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sigt = txn.open_table(SIGNATURES).map_err(|e| StoreError::DbError(e.to_string()))?;

            for (id, log) in &to_remove {
                let sig = Self::compute_signature(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
                let comp = Self::compute_composite_hash(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);

                let _ = lt.remove(*id);
                let _ = st.remove(*id);
                let _ = ct.remove(comp);
                let _ = sigt.remove(sig);
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    /// Move expired logs (max retry exceeded) to dead letter queue.
    pub fn move_expired_logs_to_dlq(&self, max_retries: i64) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().timestamp();

        // Phase 1: Read — find expired logs
        let to_move: Vec<StoredLog> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in log_table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
                let (_, bytes_guard) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                let log = StoredLog::from_bytes(bytes_guard.value())?;
                if log.retry_count >= max_retries && log.is_synced != 1 {
                    result.push(log);
                }
            }
            result
        };

        let count = to_move.len() as u64;
        if count == 0 { return Ok(0); }

        // Phase 2: Write — move to DLQ and remove from active
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut lt = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut st = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut ct = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut ft = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;

            for log in &to_move {
                let failed = StoredFailedLog {
                    id: log.id,
                    device_id: log.device_id.clone(),
                    user_id: log.user_id.clone(),
                    timestamp: log.timestamp,
                    status: log.status,
                    punch: log.punch,
                    failed_at: now,
                    failure_reason: format!("Max retries ({max_retries}) exceeded"),
                    retry_attempts: log.retry_count,
                    first_attempt_at: Some(log.created_at),
                    last_attempt_at: log.updated_at,
                };

                ft.insert(log.id, failed.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
                let comp = Self::compute_composite_hash(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
                let _ = lt.remove(log.id);
                let _ = st.remove(log.id);
                let _ = ct.remove(comp);
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    /// Get all failed logs for CLI display.
    pub fn get_failed_logs(&self) -> Result<Vec<StoredFailedLog>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut logs = Vec::new();
        for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
            let (_, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            logs.push(StoredFailedLog::from_bytes(g.value())?);
        }
        Ok(logs)
    }

    /// Count failed logs.
    pub fn count_failed_logs(&self) -> Result<usize, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(format!("{e}")))?;
        let table = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(format!("{e}")))?;
        Ok(table.len().map_err(|e| StoreError::DbError(format!("{e}")))? as usize)
    }

    /// Retry a failed log by moving it back to active queue.
    pub fn retry_failed_log(&self, id: i64) -> Result<bool, StoreError> {
        // Phase 1: Read failed log data
        let failed_data: Option<StoredFailedLog> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let ft = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            ft.get(id).map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredFailedLog::from_bytes(g.value()).ok())
        };

        let failed = match failed_data {
            Some(f) => f,
            None => return Ok(false),
        };

        // Phase 2: Write — create new log and remove from failed
        let now = chrono::Utc::now().timestamp();
        let log = StoredLog {
            id: self.next_id(),
            device_id: failed.device_id.clone(),
            user_id: failed.user_id.clone(),
            timestamp: failed.timestamp,
            status: failed.status,
            punch: failed.punch,
            is_synced: 0,
            created_at: now,
            updated_at: None,
            retry_count: 0,
        };

        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut lt = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut st = txn.open_table(SYNC_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut ct = txn.open_table(COMPOSITE_INDEX).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut ft = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;

            let comp = Self::compute_composite_hash(&log.device_id, &log.user_id, log.timestamp, log.punch, log.status);
            lt.insert(log.id, log.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
            st.insert(log.id, 0u8).map_err(|e| StoreError::DbError(e.to_string()))?;
            ct.insert(comp, log.id).map_err(|e| StoreError::DbError(e.to_string()))?;
            ft.remove(id).map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(true)
    }

    /// Delete a failed log permanently.
    pub fn delete_failed_log(&self, id: i64) -> Result<bool, StoreError> {
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        let removed;
        {
            let mut ft = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            removed = ft.remove(id).map_err(|e| StoreError::DbError(e.to_string()))?.is_some();
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(removed)
    }

    /// Purge all failed logs.
    pub fn purge_failed_logs(&self) -> Result<u64, StoreError> {
        // Phase 1: Read all IDs
        let ids: Vec<i64> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.iter().map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| e.ok().map(|(k, _)| k.value())).collect()
        };

        let count = ids.len() as u64;
        if count == 0 { return Ok(0); }

        // Phase 2: Write — remove all
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut ft = txn.open_table(FAILED_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
            for id in &ids { let _ = ft.remove(*id); }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    /// Record a parsing error.
    pub fn record_parsing_error(&self, device_id: &str, raw_timestamp: &str, error: &str) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();
        let id = self.next_id();

        #[derive(serde::Serialize)]
        struct ParseError<'a> { device_id: &'a str, raw_timestamp: &'a str, error: &'a str, created_at: i64 }

        let record = ParseError { device_id, raw_timestamp, error, created_at: now };
        let bytes = bincode::serialize(&record).map_err(|e| StoreError::DbError(e.to_string()))?;

        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn.open_table(PARSING_ERRORS).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.insert(id, bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Get all attendance logs (for CLI status display).
    pub fn get_all_logs(&self) -> Result<Vec<StoredLog>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(ATTENDANCE_LOGS).map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut logs = Vec::new();
        for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
            let (_, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            logs.push(StoredLog::from_bytes(g.value())?);
        }
        Ok(logs)
    }
}
