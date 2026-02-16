use std::sync::Arc;
use sqlx::Error;

use super::Database;

impl Database {
    pub async fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, Error> {
        let res = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2 AND (strftime(\"%s\", \"now\") - strftime(\"%s\", updated_at)) > ?")
            .bind(timeout_secs)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn reset_syncing_status(&self) -> Result<u64, Error> {
        let res = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2")
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    /// Mark fetched logs as in-flight (is_synced=2) and increment retry_count. Call after get_unsynced_logs.
    pub async fn mark_in_flight(&self, ids: &[i64]) -> Result<(), Error> {
        if ids.is_empty() {
            return Ok(());
        }

        // Update in-memory store
        self.memory_store.mark_in_flight(ids);

        // Also update disk for consistency with tests and recovery
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "UPDATE zkt_raw_attendance SET is_synced = 2, updated_at = CURRENT_TIMESTAMP, retry_count = retry_count + 1 WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.execute(&self.pool).await?;

        Ok(())
    }

    pub async fn mark_synced(&self, ids: &[i64]) -> Result<(), Error> {
        if ids.is_empty() {
            return Ok(());
        }

        // Update in-memory store
        self.memory_store.mark_synced(ids);

        // Also update disk for consistency
        let mut tx = self.pool.begin().await?;

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "UPDATE zkt_raw_attendance SET is_synced = 1 WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for id in ids {
            q = q.bind(id);
        }
        q.execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }

    /// Checkpoint batch of memory logs to disk (Phase 1: ID-independent insert)
    pub async fn checkpoint_batch(&self, logs: &[crate::memory_store::MemoryLog]) -> Result<(usize, usize, Vec<(String, String, i64)>), Error> {
        if logs.is_empty() {
            return Ok((0, 0, vec![]));
        }

        let mut tx = self.pool.begin().await?;
        let mut inserted = 0;
        let mut skipped = 0;
        let mut composite_keys = Vec::new();

        for log in logs {
            // Use INSERT OR IGNORE to handle duplicates gracefully
            // Do NOT insert memory_id — let SQLite auto-increment to avoid ID collision
            let result = sqlx::query(
                r#"
                INSERT OR IGNORE INTO zkt_raw_attendance
                (device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(&*log.device_id)
            .bind(&*log.user_id)
            .bind(log.timestamp)
            .bind(log.status)
            .bind(log.punch)
            .bind(log.is_synced)
            .bind(log.created_at)
            .bind(log.updated_at)
            .bind(log.retry_count)
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() > 0 {
                inserted += 1;
                composite_keys.push((log.device_id.to_string(), log.user_id.to_string(), log.timestamp));
            } else {
                skipped += 1;
            }
        }

        tx.commit().await?;
        Ok((inserted, skipped, composite_keys))
    }

    /// Batch update sync status for multiple logs using composite keys
    /// Tuple: (device_id, user_id, timestamp, punch, status, is_synced)
    pub async fn update_sync_status_batch(&self, updates: &[(Arc<str>, Arc<str>, i64, Option<i32>, Option<i32>, i64)]) -> Result<(), Error> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for chunk in updates.chunks(100) {
            for (device_id, user_id, timestamp, punch, status, is_synced) in chunk {
                sqlx::query("UPDATE zkt_raw_attendance SET is_synced = ? WHERE device_id = ? AND user_id = ? AND timestamp = ? AND punch IS ? AND status IS ?")
                    .bind(is_synced)
                    .bind(&**device_id)
                    .bind(&**user_id)
                    .bind(timestamp)
                    .bind(punch)
                    .bind(status)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Get max ID from disk (for memory store ID seeding)
    pub async fn get_max_id(&self) -> Result<i64, Error> {
        let max_id: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(id) FROM zkt_raw_attendance"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(max_id.unwrap_or(0))
    }
}
