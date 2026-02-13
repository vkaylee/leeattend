use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
use crate::models::{RawAttendanceLog, SignatureRow, DeviceConfig, AttendanceData};
use crate::memory_store::InMemoryStore;
use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Tz;
use serde_json;
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
    memory_store: Arc<InMemoryStore>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        // Optimize pool settings based on database type
        // For in-memory SQLite (tests), use single connection to avoid lock contention
        // and prevent connection drops that would lose in-memory data
        let is_in_memory = database_url.contains(":memory:") || database_url.contains("mode=memory");

        let pool = if is_in_memory {
            SqlitePoolOptions::new()
                .max_connections(1)  // Single connection for in-memory to avoid lock issues
                .idle_timeout(None)  // Never drop connection (in-memory DB loses data on disconnect)
                .connect(database_url)
                .await?
        } else {
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?
        };

        sqlx::migrate!("./migrations").run(&pool).await?;

        // Enable WAL mode with extended busy timeout (30s) to handle high write load
        // Skip WAL for in-memory databases (not supported)
        if !is_in_memory {
            sqlx::query("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=30000; PRAGMA cache_size=-64000; PRAGMA temp_store=MEMORY;")
                .execute(&pool)
                .await?;
        } else {
            // In-memory optimizations: temp_store and busy_timeout only
            sqlx::query("PRAGMA busy_timeout=30000; PRAGMA temp_store=MEMORY;")
                .execute(&pool)
                .await?;
        }

        // Initialize in-memory store with 10K capacity (as per plan)
        let memory_store = Arc::new(InMemoryStore::new(10_000));

        // Load existing unsynced/in-flight logs from disk into memory (startup recovery)
        let existing_logs = sqlx::query_as::<_, RawAttendanceLog>(
            "SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
             FROM zkt_raw_attendance
             WHERE is_synced IN (0, 2)
             ORDER BY id ASC"
        )
        .fetch_all(&pool)
        .await?;

        memory_store.load_from_disk(existing_logs);

        Ok(Self { pool, memory_store })
    }

    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Get device timezone from database, fallback to UTC if not found or invalid
    pub async fn get_device_timezone(&self, device_id: &str) -> Result<Tz, Error> {
        let timezone_str: Option<String> = sqlx::query_scalar(
            "SELECT timezone FROM devices WHERE id = ?"
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        let tz_str = timezone_str.unwrap_or_else(|| "UTC".to_string());
        let tz = tz_str.parse::<Tz>().unwrap_or(chrono_tz::UTC);
        Ok(tz)
    }

    /// Convert local time string to UTC Unix timestamp
    /// Returns i64 timestamp in seconds since Unix epoch
    pub fn parse_local_to_utc_timestamp(local_time_str: &str, tz: Tz) -> Result<i64, String> {
        // Try parsing with different formats
        let naive_dt = NaiveDateTime::parse_from_str(local_time_str, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(local_time_str, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|e| format!("Failed to parse timestamp: {e}"))?;

        // Convert to timezone-aware datetime
        let local_dt = tz.from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| format!("Ambiguous or non-existent local time: {local_time_str}"))?;

        // Convert to UTC and get Unix timestamp
        let utc_dt = local_dt.with_timezone(&chrono::Utc);
        Ok(utc_dt.timestamp())
    }

    pub async fn save_raw_attendance(
        &self,
        device_id: &str,
        data: AttendanceData,
    ) -> Result<i64, Error> {
        // Get device timezone
        let tz = self.get_device_timezone(device_id).await?;

        // Parse timestamp and convert to UTC Unix timestamp
        let timestamp = match Self::parse_local_to_utc_timestamp(&data.timestamp, tz) {
            Ok(ts) => ts,
            Err(e) => {
                let raw_json = serde_json::to_string(&data).unwrap_or_default();
                let error_msg = format!("Timestamp parse error: {e}");

                sqlx::query(
                    r#"
                    INSERT INTO parsing_errors (device_id, raw_data, error_msg, created_at)
                    VALUES (?, ?, ?, CURRENT_TIMESTAMP)
                    "#
                )
                .bind(device_id)
                .bind(raw_json)
                .bind(error_msg)
                .execute(&self.pool)
                .await?;

                return Ok(0);
            }
        };

        // Write to disk first
        let res = sqlx::query(
            r#"
            INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
            "#,
        )
        .bind(device_id)
        .bind(&data.user_id)
        .bind(timestamp)
        .bind(data.status)
        .bind(data.punch)
        .execute(&self.pool)
        .await?;

        let id = res.last_insert_rowid();

        // If inserted (not duplicate), also insert to memory store
        if res.rows_affected() > 0 {
            // Try in-memory insert (best effort, ignore errors for now)
            let _ = self.memory_store.insert(device_id.to_string(), data, timestamp);
        }

        Ok(id)
    }

    pub async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(String, AttendanceData)>,
    ) -> Result<usize, Error> {
        if batch.is_empty() {
            return Ok(0);
        }

        // Parse and prepare all records first
        type BatchRecord = (String, String, i64, Option<i32>, Option<i32>);
        let mut records: Vec<BatchRecord> = Vec::new();
        let mut batch_with_timestamps: Vec<(String, AttendanceData, i64)> = Vec::new();

        for (device_id, data) in batch {
            // Get device timezone
            let tz = self.get_device_timezone(&device_id).await?;

            // Parse timestamp and convert to UTC Unix timestamp
            if let Ok(timestamp) = Self::parse_local_to_utc_timestamp(&data.timestamp, tz) {
                records.push((device_id.clone(), data.user_id.clone(), timestamp, data.status, data.punch));
                batch_with_timestamps.push((device_id.clone(), data.clone(), timestamp));
            }
        }

        if records.is_empty() {
            return Ok(0);
        }

        // Write to disk first (with deduplication via CONFLICT clause)
        let mut tx = self.pool.begin().await?;
        let placeholders: Vec<&str> = records.iter().map(|_| "(?, ?, ?, ?, ?)").collect();
        let query = format!(
            "INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch) VALUES {} ON CONFLICT(device_id, user_id, timestamp) DO NOTHING",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for (device_id, user_id, timestamp, status, punch) in &records {
            q = q.bind(device_id).bind(user_id).bind(timestamp).bind(status).bind(punch);
        }

        let result = q.execute(&mut *tx).await?;
        tx.commit().await?;

        let rows_affected = result.rows_affected() as usize;

        // Also insert to memory store (best effort)
        let _ = self.memory_store.insert_batch(batch_with_timestamps);

        Ok(rows_affected)
    }

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

    /// Get unsynced logs for transmission. Returns logs WITHOUT marking them in-flight.
    /// Caller must call mark_in_flight() immediately after successful fetch.
    /// FIFO order: Logs are returned in insertion order (id ASC).
    pub async fn get_unsynced_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        // For Phase 2, read from disk to maintain test compatibility
        // Later phases will optimize to read from memory only
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            ORDER BY id ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get all logs for memory store initialization (startup only)
    /// Returns all unsynced and in-flight logs to populate memory on startup
    pub async fn get_all_logs(&self) -> Result<Vec<RawAttendanceLog>, Error> {
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch,
                   is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced IN (0, 2)  -- Only load pending/in-flight
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Load all signatures for dedup index (startup only - Phase 3)
    /// Returns minimal data: device_id, user_id, timestamp, punch, status for all logs
    pub async fn get_all_signatures(&self) -> Result<Vec<SignatureRow>, Error> {
        // Optimization: Only load signatures for ACTIVE devices to save memory.
        let rows = sqlx::query_as::<_, SignatureRow>(
            r#"
            SELECT r.device_id, r.user_id, r.timestamp, r.punch, r.status
            FROM zkt_raw_attendance r
            JOIN devices d ON r.device_id = d.id
            WHERE d.is_active = 1
            ORDER BY r.id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get signatures for a specific device (for runtime loading when enabling device)
    pub async fn get_signatures_for_device(&self, device_id: &str) -> Result<Vec<SignatureRow>, Error> {
        let rows = sqlx::query_as::<_, SignatureRow>(
            r#"
            SELECT device_id, user_id, timestamp, punch, status
            FROM zkt_raw_attendance
            WHERE device_id = ?
            ORDER BY id ASC
            "#
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Count total unsynced logs for sync coordination
    pub async fn count_unsynced_logs(&self) -> Result<i64, Error> {
        // For Phase 2, read from disk to maintain test compatibility
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Count unsynced logs for a specific device
    pub async fn count_unsynced_logs_for_device(&self, device_id: &str) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0 AND device_id = ?"
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    /// Get unsynced logs for a specific device (for runtime loading when enabling device)
    pub async fn get_unsynced_logs_for_device(&self, device_id: &str) -> Result<Vec<RawAttendanceLog>, Error> {
        let rows = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch,
                   is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0 AND device_id = ?
            ORDER BY id ASC
            "#
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Count total logs for a specific device (for polling coordination)
    pub async fn count_device_logs(&self, device_id: &str) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE device_id = ?"
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get logs that failed to sync (attempted but failed).
    /// These have is_synced=0 but updated_at is not null (marked in-flight at least once).
    pub async fn get_failed_sync_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        // For Phase 2, read from disk to maintain test compatibility
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            AND updated_at IS NOT NULL
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Count logs that failed to sync (attempted but failed).
    pub async fn count_failed_sync_logs(&self) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance
             WHERE is_synced = 0 AND updated_at IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Delete old synced logs to prevent database growth and ID overflow.
    /// Keeps audit trail for specified days. Returns number of deleted rows.
    pub async fn cleanup_old_synced_logs(&self, retention_days: i64) -> Result<u64, Error> {
        let res = sqlx::query(
            "DELETE FROM zkt_raw_attendance
             WHERE is_synced = 1
             AND created_at < datetime('now', '-' || ? || ' days')"
        )
        .bind(retention_days)
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

    /// Move expired logs to failed_logs table (dead letter queue).
    /// Returns number of logs moved to DLQ.
    pub async fn move_expired_logs_to_dlq(&self, max_retries: i64, ttl_minutes: i64) -> Result<u64, Error> {
        let mut tx = self.pool.begin().await?;

        // Find logs that exceeded retry limit OR TTL
        let expired = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            AND (
                retry_count >= ?
                OR (updated_at IS NOT NULL AND (julianday('now') - julianday(updated_at)) * 1440 >= ?)
            )
            ORDER BY id ASC
            LIMIT 1000
            "#
        )
        .bind(max_retries)
        .bind(ttl_minutes)
        .fetch_all(&mut *tx)
        .await?;

        if expired.is_empty() {
            return Ok(0);
        }

        // Insert into failed_logs table
        for log in &expired {
            let failure_reason = if log.retry_count >= max_retries {
                format!("Max retries exceeded ({max_retries})")
            } else {
                format!("TTL exceeded ({ttl_minutes}m)")
            };

            sqlx::query(
                r#"
                INSERT INTO zkt_failed_logs
                (id, device_id, user_id, timestamp, status, punch, failed_at, failure_reason, retry_attempts, first_attempt_at, last_attempt_at)
                VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?, ?, ?, ?)
                "#
            )
            .bind(log.id)
            .bind(&log.device_id)
            .bind(&log.user_id)
            .bind(log.timestamp)
            .bind(log.status)
            .bind(log.punch)
            .bind(&failure_reason)
            .bind(log.retry_count)
            .bind(log.created_at)
            .bind(log.updated_at)
            .execute(&mut *tx)
            .await?;
        }

        // Delete from zkt_raw_attendance
        let ids: Vec<i64> = expired.iter().map(|l| l.id).collect();
        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let delete_query = format!(
            "DELETE FROM zkt_raw_attendance WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&delete_query);
        for id in &ids {
            q = q.bind(id);
        }
        q.execute(&mut *tx).await?;

        let count = expired.len() as u64;
        tx.commit().await?;
        Ok(count)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_device(&self, id: &str, name: &str, ip: &str, port: i32, brand: &str, timezone: &str, is_enabled: bool) -> Result<(), Error> {
        let is_active = if is_enabled { 1 } else { 0 };
        sqlx::query(
            r#"
            INSERT INTO devices (id, name, ip, port, brand, timezone, last_seen, is_active)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                ip = excluded.ip,
                port = excluded.port,
                brand = excluded.brand,
                timezone = excluded.timezone,
                last_seen = CURRENT_TIMESTAMP,
                is_active = excluded.is_active
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(ip)
        .bind(port)
        .bind(brand)
        .bind(timezone)
        .bind(is_active)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO agent_config (key, value)
            VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET value = excluded.value
            "#
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_agent_config(&self, key: &str) -> Result<Option<String>, Error> {
        let res: Option<(String,)> = sqlx::query_as("SELECT value FROM agent_config WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;
        Ok(res.map(|r| r.0))
    }

    /// Clear all configuration on startup to ensure fresh sync from Server.
    /// Agent will use defaults until ConfigPush arrives after Handshake.
    pub async fn clear_startup_config(&self) -> Result<(), Error> {
        // Clear agent_config table (use defaults until ConfigPush)
        sqlx::query("DELETE FROM agent_config")
            .execute(&self.pool)
            .await?;

        // Deactivate all devices (wait for ConfigPush to provide fresh list)
        sqlx::query("UPDATE devices SET is_active = 0")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_active_devices(&self) -> Result<Vec<DeviceConfig>, Error> {
        sqlx::query_as::<_, DeviceConfig>(
            r#"
            SELECT id, name, ip, port, brand, timezone, last_seen, is_active
            FROM devices
            WHERE is_active = 1
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    /// Deactivate all devices whose IDs are NOT in the provided list.
    /// Used during ConfigPush to remove stale devices.
    pub async fn deactivate_devices_not_in(&self, active_ids: &[String]) -> Result<u64, Error> {
        if active_ids.is_empty() {
            // Deactivate all devices if empty list
            let res = sqlx::query("UPDATE devices SET is_active = 0 WHERE is_active = 1")
                .execute(&self.pool)
                .await?;
            return Ok(res.rows_affected());
        }

        // Build placeholders: ?,?,?
        let placeholders: Vec<&str> = active_ids.iter().map(|_| "?").collect();
        let query = format!(
            "UPDATE devices SET is_active = 0 WHERE is_active = 1 AND id NOT IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for id in active_ids {
            q = q.bind(id);
        }

        let res = q.execute(&self.pool).await?;
        Ok(res.rows_affected())
    }

    /// Batch checkpoint: Insert unpersisted logs from memory to disk
    /// Returns: (inserted_count, skipped_duplicates, composite_keys)
    pub async fn checkpoint_batch(&self, logs: &[crate::memory_store::MemoryLog]) -> Result<(usize, usize, Vec<(String, String, i64)>), Error> {
        if logs.is_empty() {
            return Ok((0, 0, vec![]));
        }

        // Collect composite keys for all logs being inserted
        let composite_keys: Vec<(String, String, i64)> = logs.iter()
            .map(|l| (l.device_id.clone(), l.user_id.clone(), l.timestamp))
            .collect();

        let mut tx = self.pool.begin().await?;
        let mut inserted = 0;
        let mut skipped = 0;

        // Batch insert with ON CONFLICT DO NOTHING
        // SQLite limit: ~500 values per statement
        for chunk in logs.chunks(100) {
            let placeholders: Vec<&str> = chunk.iter()
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?)")
                .collect();
            let query = format!(
                r#"
                INSERT INTO zkt_raw_attendance
                    (device_id, user_id, timestamp, status, punch, is_synced, created_at, retry_count)
                VALUES {}
                ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
                "#,
                placeholders.join(",")
            );

            let mut q = sqlx::query(&query);
            for log in chunk {
                q = q.bind(&log.device_id)
                    .bind(&log.user_id)
                    .bind(log.timestamp)
                    .bind(log.status)
                    .bind(log.punch)
                    .bind(log.is_synced)
                    .bind(log.created_at)
                    .bind(log.retry_count);
            }

            let result = q.execute(&mut *tx).await?;
            inserted += result.rows_affected() as usize;
            skipped += chunk.len() - result.rows_affected() as usize;
        }

        tx.commit().await?;
        Ok((inserted, skipped, composite_keys))
    }

    /// Update sync status for checkpointed logs
    pub async fn update_sync_status_batch(&self, updates: &[(i64, i64)]) -> Result<(), Error> {
        if updates.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        for chunk in updates.chunks(100) {
            for &(id, is_synced) in chunk {
                sqlx::query("UPDATE zkt_raw_attendance SET is_synced = ? WHERE id = ?")
                    .bind(is_synced)
                    .bind(id)
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

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> Database {
        let db = Database::new("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(db.get_pool()).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_save_and_retrieve_attendance() {
        let db = setup_db().await;
        let data = AttendanceData {
            user_id: "U1".into(),
            timestamp: "2026-01-01 08:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        db.save_raw_attendance("D1", data).await.unwrap();
        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "U1");
    }

    #[tokio::test]
    async fn test_save_attendance_batch() {
        let db = setup_db().await;
        let batch = vec![
            ("D1".to_string(), AttendanceData { user_id: "U1".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) }),
            ("D1".to_string(), AttendanceData { user_id: "U2".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) }),
            ("D1".to_string(), AttendanceData { user_id: "U1".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) }), // Duplicate
        ];

        let count = db.save_raw_attendance_batch(batch).await.unwrap();
        assert_eq!(count, 2, "Should return actual inserted count (2), duplicate ignored");

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 2, "Duplicate in batch should be ignored by DB constraint");
    }

    // ===== deactivate_devices_not_in tests =====

    #[tokio::test]
    async fn test_deactivate_devices_not_in_removes_stale() {
        let db = setup_db().await;

        // Insert 3 devices
        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("dev3", "Device 3", "192.168.1.3", 4370, "zkteco", "UTC", true).await.unwrap();

        // Keep only dev1 and dev3
        let active_ids = vec!["dev1".to_string(), "dev3".to_string()];
        let deactivated = db.deactivate_devices_not_in(&active_ids).await.unwrap();

        assert_eq!(deactivated, 1, "Should deactivate 1 device (dev2)");

        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|d| d.id == "dev1"));
        assert!(active.iter().any(|d| d.id == "dev3"));
        assert!(!active.iter().any(|d| d.id == "dev2"));
    }

    #[tokio::test]
    async fn test_deactivate_devices_not_in_empty_list_deactivates_all() {
        let db = setup_db().await;

        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco", "UTC", true).await.unwrap();

        // Empty list = deactivate all
        let deactivated = db.deactivate_devices_not_in(&[]).await.unwrap();

        assert_eq!(deactivated, 2, "Should deactivate all devices");
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    async fn test_deactivate_devices_not_in_no_changes_when_all_match() {
        let db = setup_db().await;

        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco", "UTC", true).await.unwrap();

        // Keep all devices
        let active_ids = vec!["dev1".to_string(), "dev2".to_string()];
        let deactivated = db.deactivate_devices_not_in(&active_ids).await.unwrap();

        assert_eq!(deactivated, 0, "Should deactivate 0 devices");
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 2);
    }

    #[tokio::test]
    async fn test_deactivate_devices_case_sensitive() {
        let db = setup_db().await;

        // Insert with lowercase
        db.upsert_device("office", "Office Device", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();

        // Keep with uppercase (different ID due to case sensitivity)
        let active_ids = vec!["OFFICE".to_string()];
        let deactivated = db.deactivate_devices_not_in(&active_ids).await.unwrap();

        // "office" should be deactivated because "OFFICE" != "office"
        assert_eq!(deactivated, 1, "Case-sensitive: 'office' should be deactivated");
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 0);
    }

    // ===== Batch UPDATE optimization tests =====

    #[tokio::test]
    async fn test_get_unsynced_logs_and_mark_in_flight() {
        let db = setup_db().await;

        // Insert 3 unsynced logs
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 10:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        // Fetch unsynced logs (read-only, doesn't modify is_synced)
        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 3);

        // Mark as in-flight (separate write operation)
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();

        // Verify in DB that logs are marked as in-flight
        let in_flight_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(in_flight_count, 3, "All 3 logs should be marked as in-flight in DB");

        // Second fetch should return 0 logs (already in-flight)
        let logs2 = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs2.len(), 0, "In-flight logs should not be fetched again");
    }

    #[tokio::test]
    async fn test_get_unsynced_logs_batch_update() {
        let db = setup_db().await;

        // Insert 150 unsynced logs (more than typical batch size)
        for i in 1..=150 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-28 10:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        // Fetch with limit=100 (read-only)
        let logs = db.get_unsynced_logs(100).await.unwrap();
        assert_eq!(logs.len(), 100, "Should fetch exactly 100 logs");

        // Mark as in-flight (separate write operation)
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();

        // Verify in DB that fetched logs are marked as in-flight
        let in_flight_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(in_flight_count, 100, "All 100 fetched logs should be in-flight in DB");

        // Remaining 50 should still be unsynced
        let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(remaining, 50, "50 logs should remain unsynced");
    }

    #[tokio::test]
    async fn test_mark_synced_batch_update() {
        let db = setup_db().await;

        // Insert 50 logs
        let mut ids = Vec::new();
        for i in 1..=50 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 10:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            let id = db.save_raw_attendance("D1", data).await.unwrap();
            ids.push(id);
        }

        // Mark as in-flight first
        let logs = db.get_unsynced_logs(50).await.unwrap();
        assert_eq!(logs.len(), 50);

        // Mark all as synced
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_synced(&log_ids).await.unwrap();

        // Verify all marked as synced
        let synced_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 1")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(synced_count, 50, "All 50 logs should be marked as synced");
    }

    #[tokio::test]
    async fn test_mark_synced_empty_array() {
        let db = setup_db().await;

        // Should not error on empty array
        let result = db.mark_synced(&[]).await;
        assert!(result.is_ok(), "mark_synced should handle empty array");
    }

    #[tokio::test]
    async fn test_concurrent_batch_operations() {
        let db = setup_db().await;

        // Insert 200 logs
        for i in 1..=200 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-28 10:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        // Simulate concurrent fetch (should not deadlock with busy_timeout)
        let db1 = db.clone();
        let db2 = db.clone();

        let handle1 = tokio::spawn(async move {
            db1.get_unsynced_logs(100).await.unwrap()
        });

        let handle2 = tokio::spawn(async move {
            // Small delay to let first task start
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            db2.get_unsynced_logs(100).await.unwrap()
        });

        let logs1 = handle1.await.unwrap();
        let logs2 = handle2.await.unwrap();

        // Both should succeed (no deadlock)
        assert!(!logs1.is_empty() || !logs2.is_empty(), "At least one fetch should succeed");

        // Total fetched should not exceed 200
        let total_fetched = logs1.len() + logs2.len();
        assert!(total_fetched <= 200, "Should not fetch more than available logs");
    }

    #[tokio::test]
    async fn test_count_unsynced_logs() {
        let db = setup_db().await;

        // Initially 0
        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 0, "Should start with 0 unsynced logs");

        // Insert 150 logs
        for i in 1..=150 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-28 11:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        // Should have 150 unsynced
        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 150, "Should have 150 unsynced logs");

        // Mark 50 as in-flight
        let logs = db.get_unsynced_logs(50).await.unwrap();
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();

        // Should still have 100 unsynced (in-flight doesn't count as synced)
        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 100, "Should have 100 unsynced after marking 50 in-flight");

        // Mark 30 as synced
        db.mark_synced(&log_ids[0..30]).await.unwrap();

        // Should have 100 unsynced (30 synced out of 50 in-flight, still 100 at is_synced=0)
        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 100, "Should have 100 unsynced after marking 30 synced");
    }

    #[tokio::test]
    async fn test_fifo_order_get_unsynced_logs() {
        let db = setup_db().await;

        // Insert logs with specific user_ids in order
        let expected_order = vec!["FIRST", "SECOND", "THIRD", "FOURTH", "FIFTH"];

        for user_id in &expected_order {
            let data = AttendanceData {
                user_id: user_id.to_string(),
                timestamp: "2026-01-28 12:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
            // Small delay to ensure distinct insertion time
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Fetch all logs
        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 5, "Should fetch all 5 logs");

        // Verify FIFO order (first inserted = first returned)
        for (i, log) in logs.iter().enumerate() {
            assert_eq!(
                log.user_id, expected_order[i],
                "Log {} should be {} (FIFO order)", i, expected_order[i]
            );
        }

        // Verify IDs are ascending (insertion order)
        for i in 1..logs.len() {
            assert!(
                logs[i].id > logs[i-1].id,
                "IDs should be ascending (id {} should be > {})", logs[i].id, logs[i-1].id
            );
        }
    }

    #[tokio::test]
    async fn test_cleanup_old_synced_logs() {
        let db = setup_db().await;

        // Insert 10 logs
        for i in 1..=10 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 13:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        // Mark first 5 as synced
        let logs = db.get_unsynced_logs(5).await.unwrap();
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();
        db.mark_synced(&log_ids).await.unwrap();

        // Manually set created_at to 400 days ago for synced logs
        sqlx::query(
            "UPDATE zkt_raw_attendance
             SET created_at = datetime('now', '-400 days')
             WHERE is_synced = 1"
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        // Try cleanup with 365 days retention (should delete old synced logs)
        let deleted = db.cleanup_old_synced_logs(365).await.unwrap();
        assert_eq!(deleted, 5, "Should delete 5 old synced logs");

        // Verify only unsynced logs remain
        let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(remaining, 5, "5 unsynced logs should remain");

        // Verify remaining are all unsynced
        let unsynced_count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(unsynced_count, 5, "All remaining logs should be unsynced");
    }

    #[tokio::test]
    async fn test_get_failed_sync_logs() {
        let db = setup_db().await;

        // Insert 10 logs with proper datetime format
        for i in 1..=10 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-01 08:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            db.save_raw_attendance("device1", data).await.unwrap();
        }

        // Initially: No failed logs (all unsynced, updated_at is NULL)
        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 0, "No failed logs initially");

        // Get 5 logs and mark in-flight
        let logs = db.get_unsynced_logs(5).await.unwrap();
        assert_eq!(logs.len(), 5, "Should fetch 5 unsynced logs");

        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&ids).await.unwrap();

        // Now they're in-flight (is_synced=2, updated_at NOT NULL)
        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 0, "In-flight logs are not failed yet");

        // Simulate 3 sync failures (reset to is_synced=0 but updated_at remains)
        // This mimics what happens in central_client when ACK status != "success"
        for id in &ids[0..3] {
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                .bind(id)
                .execute(db.get_pool())
                .await
                .unwrap();
        }

        // Now we have 3 failed logs (is_synced=0, updated_at NOT NULL)
        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 3, "3 logs should be marked as failed");

        // Get failed logs
        let failed_logs = db.get_failed_sync_logs(10).await.unwrap();
        assert_eq!(failed_logs.len(), 3, "Should return 3 failed logs");

        // Verify they have correct properties
        for log in failed_logs {
            assert_eq!(log.is_synced, 0, "Failed logs should have is_synced=0");
        }

        // Simulate 2 more failures
        for id in &ids[3..5] {
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                .bind(id)
                .execute(db.get_pool())
                .await
                .unwrap();
        }

        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 5, "5 logs should be failed now");
    }

    #[tokio::test]
    async fn test_cleanup_preserves_unsynced_and_inflight() {
        let db = setup_db().await;

        // Insert 15 logs
        for i in 1..=15 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 14:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        // Mark 5 as synced, 5 as in-flight, leave 5 unsynced
        let logs = db.get_unsynced_logs(10).await.unwrap();
        let synced_ids: Vec<i64> = logs[0..5].iter().map(|l| l.id).collect();
        let inflight_ids: Vec<i64> = logs[5..10].iter().map(|l| l.id).collect();

        db.mark_in_flight(&synced_ids).await.unwrap();
        db.mark_synced(&synced_ids).await.unwrap();

        db.mark_in_flight(&inflight_ids).await.unwrap();

        // Manually set created_at to 400 days ago for ALL logs
        sqlx::query(
            "UPDATE zkt_raw_attendance
             SET created_at = datetime('now', '-400 days')"
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        // Cleanup with 365 days (delete old synced only)
        let deleted = db.cleanup_old_synced_logs(365).await.unwrap();
        assert_eq!(deleted, 5, "Should only delete synced logs");

        // Verify 10 logs remain (5 in-flight + 5 unsynced)
        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(total, 10, "In-flight and unsynced logs should remain");

        // Verify counts by state
        let unsynced = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0"
        ).fetch_one(db.get_pool()).await.unwrap();
        assert_eq!(unsynced, 5, "5 unsynced logs should remain");

        let in_flight = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2"
        ).fetch_one(db.get_pool()).await.unwrap();
        assert_eq!(in_flight, 5, "5 in-flight logs should remain");

        let synced = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 1"
        ).fetch_one(db.get_pool()).await.unwrap();
        assert_eq!(synced, 0, "0 synced logs should remain (deleted)");
    }

    #[tokio::test]
    async fn test_checkpoint_batch_no_id_collision() {
        let db = setup_db().await;

        // Pre-populate disk with logs having IDs 1-100
        for i in 1..=100 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: "2026-01-31 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            db.save_raw_attendance(&format!("DEV{}", (i % 5) + 1), data).await.ok();
        }

        // Create memory logs with overlapping IDs (would have caused UNIQUE constraint failure before Phase 1)
        let timestamp_unix = NaiveDateTime::parse_from_str("2026-01-31 11:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp();
        let memory_logs: Vec<crate::memory_store::MemoryLog> = (50..=150)
            .map(|i| crate::memory_store::MemoryLog {
                id: i,  // IDs 50-100 overlap with disk!
                device_id: "DEV2".to_string(),
                user_id: format!("U{:03}", i + 1000),  // Different user_ids to avoid business duplicate
                timestamp: timestamp_unix,
                status: Some(1),
                punch: Some(0),
                is_synced: 0,
                created_at: chrono::Utc::now().naive_utc(),
                updated_at: None,
                retry_count: 0,
                persisted: false,
            })
            .collect();

        // This should NOT fail with UNIQUE constraint error
        let result = db.checkpoint_batch(&memory_logs).await;
        assert!(result.is_ok(), "Checkpoint should not fail with ID collision");

        let (inserted, _skipped, keys) = result.unwrap();
        assert!(inserted > 0, "Should insert logs");
        assert!(!keys.is_empty(), "Should return composite keys");
    }

    #[tokio::test]
    async fn test_checkpoint_batch_returns_composite_keys() {
        let db = setup_db().await;

        // Create memory logs
        let memory_logs: Vec<crate::memory_store::MemoryLog> = (1..=5)
            .map(|i| {
                let timestamp_unix = NaiveDateTime::parse_from_str(&format!("2026-01-31 10:{i:02}:00"), "%Y-%m-%d %H:%M:%S")
                    .unwrap()
                    .and_utc()
                    .timestamp();
                crate::memory_store::MemoryLog {
                    id: i,
                    device_id: "DEV1".to_string(),
                    user_id: format!("U{i:03}"),
                    timestamp: timestamp_unix,
                    status: Some(1),
                    punch: Some(0),
                    is_synced: 0,
                    created_at: chrono::Utc::now().naive_utc(),
                    updated_at: None,
                    retry_count: 0,
                    persisted: false,
                }
            })
            .collect();

        // Checkpoint and verify return type
        let (inserted, skipped, keys) = db.checkpoint_batch(&memory_logs).await.unwrap();

        assert_eq!(inserted, 5);
        assert_eq!(skipped, 0);
        assert_eq!(keys.len(), 5);

        // Verify composite keys are formatted correctly
        for (i, (device_id, user_id, _timestamp)) in keys.iter().enumerate() {
            assert_eq!(device_id, "DEV1");
            assert_eq!(user_id, &format!("U{:03}", i + 1));
        }
    }

    #[tokio::test]
    async fn test_count_unsynced_logs_for_device() {
        let db = setup_db().await;

        // Setup devices first (required for timezone lookup in save_raw_attendance)
        db.upsert_device("Device A", "Device A", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("Device B", "Device B", "127.0.0.2", 4370, "zkteco", "UTC", true).await.unwrap();

        // Device A: 5 unsynced, 2 synced
        for i in 1..=5 {
            let data = AttendanceData {
                user_id: format!("U_A_{i}"),
                timestamp: format!("2026-01-28 10:{i:02}:00"),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("Device A", data).await.unwrap();
        }
        // Mark 2 synced
        let logs_a = db.get_unsynced_logs(2).await.unwrap();
        let ids_a: Vec<i64> = logs_a.iter().map(|l| l.id).collect();
        db.mark_synced(&ids_a).await.unwrap();

        // Device B: 3 unsynced
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U_B_{i}"),
                timestamp: format!("2026-01-28 11:{i:02}:00"),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("Device B", data).await.unwrap();
        }

        // Verify Device A count (5 inserted - 2 synced = 3 unsynced)
        let count_a = db.count_unsynced_logs_for_device("Device A").await.unwrap();
        assert_eq!(count_a, 3, "Device A should have 3 unsynced logs");

        // Verify Device B count (3 inserted - 0 synced = 3 unsynced)
        let count_b = db.count_unsynced_logs_for_device("Device B").await.unwrap();
        assert_eq!(count_b, 3, "Device B should have 3 unsynced logs");

        // Verify Non-existent Device count
        let count_c = db.count_unsynced_logs_for_device("Device C").await.unwrap();
        assert_eq!(count_c, 0, "Non-existent device should have 0 unsynced logs");
    }
}
