use super::{
    ATTENDANCE_LOGS, COMPOSITE_INDEX, RedbStore, SIGNATURES, SYNC_INDEX, StoreError, StoredLog,
};
use redb::{ReadableDatabase, ReadableTable, ReadableTableMetadata};

impl RedbStore {
    /// Mark logs as in-flight (sync state = 2).
    pub fn mark_in_flight(&self, ids: &[i64]) -> Result<(), StoreError> {
        if ids.is_empty() {
            return Ok(());
        }

        let now = chrono::Utc::now().timestamp();

        // Phase 1: Read all logs to update
        let mut updates: Vec<(i64, Vec<u8>)> = Vec::with_capacity(ids.len());
        {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for &id in ids {
                if let Some(bytes_guard) = log_table
                    .get(id)
                    .map_err(|e| StoreError::DbError(e.to_string()))?
                {
                    let mut log = StoredLog::from_bytes(bytes_guard.value())?;
                    log.is_synced = 2;
                    log.retry_count += 1;
                    log.updated_at = Some(now);
                    updates.push((id, log.to_bytes()));
                }
            }
        }

        // Phase 2: Write updates
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn
                .open_table(SYNC_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                log_table
                    .insert(*id, bytes.as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table
                    .insert(*id, 2u8)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Mark logs as synced (sync state = 1) and remove from active storage.
    /// Keeps signature for dedup protection.
    pub fn mark_synced(&self, ids: &[i64]) -> Result<(), StoreError> {
        if ids.is_empty() {
            return Ok(());
        }

        // Phase 1: Read log data to compute composite hashes
        let mut removals: Vec<(i64, u64)> = Vec::with_capacity(ids.len());
        {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for &id in ids {
                if let Some(bytes_guard) = log_table
                    .get(id)
                    .map_err(|e| StoreError::DbError(e.to_string()))?
                {
                    let log = StoredLog::from_bytes(bytes_guard.value())?;
                    let composite_hash = Self::compute_composite_hash(
                        &log.device_id,
                        &log.user_id,
                        log.timestamp,
                        log.punch,
                        log.status,
                    );
                    removals.push((id, composite_hash));
                }
            }
        }

        // Phase 2: Remove from active storage
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn
                .open_table(SYNC_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut comp_table = txn
                .open_table(COMPOSITE_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, composite_hash) in &removals {
                let _ = log_table.remove(*id);
                let _ = sync_table.remove(*id);
                let _ = comp_table.remove(*composite_hash);
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Reset log from in-flight back to unsynced (sync state 2 → 0).
    pub fn reset_to_unsynced(&self, id: i64) -> Result<(), StoreError> {
        // Phase 1: Read log
        let log_data: Option<Vec<u8>> = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            if let Some(g) = log_table
                .get(id)
                .map_err(|e| StoreError::DbError(e.to_string()))?
            {
                let mut log = StoredLog::from_bytes(g.value())?;
                log.is_synced = 0;
                Some(log.to_bytes())
            } else {
                None
            }
        };

        if let Some(bytes) = log_data {
            let txn = self
                .db
                .read()
                .begin_write()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut log_table = txn
                    .open_table(ATTENDANCE_LOGS)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                let mut sync_table = txn
                    .open_table(SYNC_INDEX)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                log_table
                    .insert(id, bytes.as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table
                    .insert(id, 0u8)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
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
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let sync_table = txn
                .open_table(SYNC_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in sync_table
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
            {
                let (id_guard, state_guard) =
                    entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                if state_guard.value() == 2 {
                    result.push(id_guard.value());
                }
            }
            result
        };

        let count = ids_to_reset.len() as u64;
        if count == 0 {
            return Ok(0);
        }

        // Phase 2: Read existing logs and prepare updates
        let mut updates: Vec<(i64, Vec<u8>)> = Vec::new();
        {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for &id in &ids_to_reset {
                if let Some(bytes_guard) = log_table
                    .get(id)
                    .map_err(|e| StoreError::DbError(e.to_string()))?
                {
                    let mut log = StoredLog::from_bytes(bytes_guard.value())?;
                    log.is_synced = 0;
                    updates.push((id, log.to_bytes()));
                }
            }
        }

        // Phase 3: Write updates
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn
                .open_table(SYNC_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                log_table
                    .insert(*id, bytes.as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table
                    .insert(*id, 0u8)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    /// Reset stale in-flight logs older than timeout to unsynced.
    pub fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, StoreError> {
        let now = chrono::Utc::now().timestamp();

        // Phase 1: Read — find stale in-flight logs and prepare updates
        let mut updates: Vec<(i64, Vec<u8>)> = Vec::new();
        {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let sync_table = txn
                .open_table(SYNC_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;

            for entry in sync_table
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
            {
                let (id_guard, state_guard) =
                    entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                if state_guard.value() == 2 {
                    let id = id_guard.value();
                    if let Some(bytes_guard) = log_table
                        .get(id)
                        .map_err(|e| StoreError::DbError(e.to_string()))?
                    {
                        let mut log = StoredLog::from_bytes(bytes_guard.value())?;
                        if let Some(updated_at) = log.updated_at
                            && now - updated_at > timeout_secs
                        {
                            log.is_synced = 0;
                            updates.push((id, log.to_bytes()));
                        }
                    }
                }
            }
        }

        let count = updates.len() as u64;
        if count == 0 {
            return Ok(0);
        }

        // Phase 2: Write updates in a single transaction
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut log_table = txn
                .open_table(ATTENDANCE_LOGS)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut sync_table = txn
                .open_table(SYNC_INDEX)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                log_table
                    .insert(*id, bytes.as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                sync_table
                    .insert(*id, 0u8)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;

        Ok(count)
    }

    /// Get max log ID.
    pub fn get_max_id(&self) -> Result<i64, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(ATTENDANCE_LOGS)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(table
            .last()
            .map_err(|e| StoreError::DbError(e.to_string()))?
            .map(|(k, _)| k.value())
            .unwrap_or(0))
    }

    /// Evict signatures older than TTL. Returns (evicted, before, after).
    pub fn evict_old_signatures(&self) -> Result<(usize, usize, usize), StoreError> {
        let threshold = chrono::Utc::now().timestamp() - (self.signature_ttl_days * 86400);

        // Phase 1: Read — find expired signatures
        let (before, to_remove) = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let sig_table = txn
                .open_table(SIGNATURES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let len = sig_table
                .len()
                .map_err(|e| StoreError::DbError(format!("{e}")))?;
            let before = usize::try_from(len)
                .map_err(|e| StoreError::DbError(format!("Signatures table size overflow: {e}")))?;
            let mut expired = Vec::new();
            for entry in sig_table
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
            {
                let (hash_guard, ts_guard) =
                    entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                if ts_guard.value() < threshold {
                    expired.push(hash_guard.value());
                }
            }
            (before, expired)
        };

        // Phase 2: Write — remove expired
        if !to_remove.is_empty() {
            let txn = self
                .db
                .read()
                .begin_write()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut sig_table = txn
                    .open_table(SIGNATURES)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                for hash in &to_remove {
                    let _ = sig_table.remove(*hash);
                }
            }
            txn.commit()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
        }

        let evicted = to_remove.len();
        let after = before - evicted;
        Ok((evicted, before, after))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store() -> RedbStore {
        RedbStore::new(":memory:", 90).expect("in-memory store")
    }

    fn insert_log(store: &RedbStore, device_id: &str, user_id: &str, ts: i64) -> i64 {
        use crate::models::AttendanceData;
        let data = AttendanceData {
            user_id: user_id.into(),
            timestamp: "2026-01-01 10:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        store.insert(device_id, &data, ts).unwrap()
    }

    #[test]
    fn test_mark_in_flight_empty_ids() {
        let store = make_store();
        store.mark_in_flight(&[]).unwrap();
    }

    #[test]
    fn test_mark_in_flight_updates_sync_state() {
        let store = make_store();
        let id = insert_log(&store, "DEV1", "U1", 1000);

        store.mark_in_flight(&[id]).unwrap();

        let logs = store.get_all_logs().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].is_synced, 2);
        assert_eq!(logs[0].retry_count, 1);
        assert!(logs[0].updated_at.is_some());
    }

    #[test]
    fn test_mark_synced_removes_from_active() {
        let store = make_store();
        let id = insert_log(&store, "DEV1", "U1", 1000);

        store.mark_in_flight(&[id]).unwrap();
        store.mark_synced(&[id]).unwrap();

        let logs = store.get_all_logs().unwrap();
        assert!(
            logs.is_empty(),
            "Synced logs should be removed from active storage"
        );
    }

    #[test]
    fn test_mark_synced_empty_ids() {
        let store = make_store();
        store.mark_synced(&[]).unwrap();
    }

    #[test]
    fn test_reset_to_unsynced() {
        let store = make_store();
        let id = insert_log(&store, "DEV1", "U1", 1000);

        store.mark_in_flight(&[id]).unwrap();
        assert_eq!(store.get_all_logs().unwrap()[0].is_synced, 2);

        store.reset_to_unsynced(id).unwrap();
        assert_eq!(store.get_all_logs().unwrap()[0].is_synced, 0);
    }

    #[test]
    fn test_reset_to_unsynced_nonexistent() {
        let store = make_store();
        // Should not panic on nonexistent ID
        store.reset_to_unsynced(99999).unwrap();
    }

    #[test]
    fn test_mark_synced_by_key() {
        let store = make_store();
        let id = insert_log(&store, "DEV1", "U1", 1000);
        store.mark_in_flight(&[id]).unwrap();

        let log = &store.get_all_logs().unwrap()[0];
        let key = log.composite_key();

        let result = store.mark_synced_by_key(&key);
        assert!(result);
        assert!(store.get_all_logs().unwrap().is_empty());
    }

    #[test]
    fn test_mark_synced_by_key_not_found() {
        let store = make_store();
        let result = store.mark_synced_by_key("invalid:key:0:1:1");
        assert!(!result);
    }

    #[test]
    fn test_reset_to_unsynced_by_key() {
        let store = make_store();
        let id = insert_log(&store, "DEV1", "U1", 1000);
        store.mark_in_flight(&[id]).unwrap();

        let log = &store.get_all_logs().unwrap()[0];
        let key = log.composite_key();

        let result = store.reset_to_unsynced_by_key(&key);
        assert!(result);
        assert_eq!(store.get_all_logs().unwrap()[0].is_synced, 0);
    }

    #[test]
    fn test_reset_to_unsynced_by_key_not_found() {
        let store = make_store();
        let result = store.reset_to_unsynced_by_key("invalid:key:0:1:1");
        assert!(!result);
    }

    #[test]
    fn test_reset_syncing_status() {
        let store = make_store();
        let id1 = insert_log(&store, "DEV1", "U1", 1000);
        let id2 = insert_log(&store, "DEV1", "U2", 2000);

        store.mark_in_flight(&[id1, id2]).unwrap();

        let reset = store.reset_syncing_status().unwrap();
        assert_eq!(reset, 2);

        let logs = store.get_all_logs().unwrap();
        assert!(logs.iter().all(|l| l.is_synced == 0));
    }

    #[test]
    fn test_reset_syncing_status_no_inflight() {
        let store = make_store();
        insert_log(&store, "DEV1", "U1", 1000);
        let reset = store.reset_syncing_status().unwrap();
        assert_eq!(reset, 0);
    }

    #[test]
    fn test_get_max_id_empty() {
        let store = make_store();
        let max = store.get_max_id().unwrap();
        assert_eq!(max, 0);
    }

    #[test]
    fn test_get_max_id_with_data() {
        let store = make_store();
        let id1 = insert_log(&store, "DEV1", "U1", 1000);
        let id2 = insert_log(&store, "DEV1", "U2", 2000);
        let max = store.get_max_id().unwrap();
        assert_eq!(max, std::cmp::max(id1, id2));
    }

    #[test]
    fn test_evict_old_signatures_none_expired() {
        let store = make_store();
        insert_log(&store, "DEV1", "U1", chrono::Utc::now().timestamp());
        let (evicted, before, after) = store.evict_old_signatures().unwrap();
        assert_eq!(evicted, 0);
        assert_eq!(before, 1);
        assert_eq!(after, 1);
    }

    #[test]
    fn test_evict_old_signatures_empty() {
        let store = make_store();
        let (evicted, before, after) = store.evict_old_signatures().unwrap();
        assert_eq!(evicted, 0);
        assert_eq!(before, 0);
        assert_eq!(after, 0);
    }

    #[test]
    fn test_mark_in_flight_increments_retry_count() {
        let store = make_store();
        let id = insert_log(&store, "DEV1", "U1", 1000);

        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
        store.mark_in_flight(&[id]).unwrap();
        store.reset_to_unsynced(id).unwrap();
        store.mark_in_flight(&[id]).unwrap();

        let log = &store.get_all_logs().unwrap()[0];
        assert_eq!(log.retry_count, 3);
    }
}
