use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
use crate::models::{RawAttendanceLog, DeviceConfig, AttendanceData};
use chrono::NaiveDateTime;
use serde_json;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        // Enable WAL mode with extended busy timeout (30s) to handle high write load
        sqlx::query("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=30000; PRAGMA cache_size=-64000; PRAGMA temp_store=MEMORY;")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn save_raw_attendance(
        &self,
        device_id: &str,
        data: AttendanceData,
    ) -> Result<i64, Error> {
        let timestamp_res = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"));

        let timestamp = match timestamp_res {
            Ok(t) => t,
            Err(e) => {
                let raw_json = serde_json::to_string(&data).unwrap_or_default();
                let error_msg = format!("Timestamp parse error: {}", e);
                
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

        let res = sqlx::query(
            r#"
            INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
            "#,
        )
        .bind(device_id)
        .bind(data.user_id)
        .bind(timestamp)
        .bind(data.status)
        .bind(data.punch)
        .execute(&self.pool)
        .await?;

        Ok(res.last_insert_rowid())
    }

    pub async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(String, AttendanceData)>,
    ) -> Result<usize, Error> {
        if batch.is_empty() {
            return Ok(0);
        }

        // Parse and prepare all records first
        let mut records: Vec<(String, String, NaiveDateTime, Option<i32>, Option<i32>)> = Vec::new();
        for (device_id, data) in batch {
            let timestamp_res = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"));

            if let Ok(timestamp) = timestamp_res {
                records.push((device_id, data.user_id, timestamp, data.status, data.punch));
            }
        }

        if records.is_empty() {
            return Ok(0);
        }

        // Use single INSERT with multiple VALUES for better performance
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

        Ok(result.rows_affected() as usize)
    }

    pub async fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, Error> {
        let res = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2 AND (strftime(\"%s\", \"now\") - strftime(\"%s\", updated_at)) > ?")
            .bind(timeout_secs)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    pub async fn reset_syncing_status(&self) -> Result<(), Error> {
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get unsynced logs for transmission. Returns logs WITHOUT marking them in-flight.
    /// Caller must call mark_in_flight() immediately after successful fetch.
    /// FIFO order: Logs are returned in insertion order (id ASC).
    pub async fn get_unsynced_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        // Use read-only query to avoid lock contention with batch writer
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

    /// Count total unsynced logs for sync coordination
    pub async fn count_unsynced_logs(&self) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
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
        let mut tx = self.pool.begin().await?;

        // Batch update using IN clause instead of loop
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
                format!("Max retries exceeded ({})", max_retries)
            } else {
                format!("TTL exceeded ({}m)", ttl_minutes)
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
            .bind(&log.timestamp)
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

    pub async fn upsert_device(&self, id: &str, name: &str, ip: &str, port: i32, brand: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO devices (id, name, ip, port, brand, last_seen, is_active)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP, 1)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                ip = excluded.ip,
                port = excluded.port, brand = excluded.brand,
                last_seen = CURRENT_TIMESTAMP,
                is_active = 1
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(ip)
        .bind(port).bind(brand)
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

    pub async fn get_active_devices(&self) -> Result<Vec<DeviceConfig>, Error> {
        sqlx::query_as::<_, DeviceConfig>(
            r#"
            SELECT id, name, ip, port, brand, last_seen, is_active
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
        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco").await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco").await.unwrap();
        db.upsert_device("dev3", "Device 3", "192.168.1.3", 4370, "zkteco").await.unwrap();

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

        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco").await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco").await.unwrap();

        // Empty list = deactivate all
        let deactivated = db.deactivate_devices_not_in(&[]).await.unwrap();

        assert_eq!(deactivated, 2, "Should deactivate all devices");
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    async fn test_deactivate_devices_not_in_no_changes_when_all_match() {
        let db = setup_db().await;

        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco").await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco").await.unwrap();

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
        db.upsert_device("office", "Office Device", "192.168.1.1", 4370, "zkteco").await.unwrap();

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
                user_id: format!("U{}", i),
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
                user_id: format!("U{}", i),
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
                user_id: format!("U{}", i),
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
                user_id: format!("U{}", i),
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
        assert!(logs1.len() > 0 || logs2.len() > 0, "At least one fetch should succeed");

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
                user_id: format!("U{}", i),
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
                user_id: format!("U{}", i),
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
                user_id: format!("U{}", i),
                timestamp: format!("2026-01-01 08:{:02}:00", i),
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
                user_id: format!("U{}", i),
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
}
