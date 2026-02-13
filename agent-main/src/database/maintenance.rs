use sqlx::Error;
use crate::models::RawAttendanceLog;

use super::Database;

impl Database {
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
}

#[cfg(test)]
mod tests {
    use super::super::Database;
    use crate::models::AttendanceData;
    use chrono::NaiveDateTime;

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

    #[tokio::test]
    async fn test_deactivate_devices_not_in_removes_stale() {
        let db = setup_db().await;
        db.upsert_device("dev1", "Device 1", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("dev2", "Device 2", "192.168.1.2", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("dev3", "Device 3", "192.168.1.3", 4370, "zkteco", "UTC", true).await.unwrap();

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

        let active_ids = vec!["dev1".to_string(), "dev2".to_string()];
        let deactivated = db.deactivate_devices_not_in(&active_ids).await.unwrap();
        assert_eq!(deactivated, 0, "Should deactivate 0 devices");
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 2);
    }

    #[tokio::test]
    async fn test_deactivate_devices_case_sensitive() {
        let db = setup_db().await;
        db.upsert_device("office", "Office Device", "192.168.1.1", 4370, "zkteco", "UTC", true).await.unwrap();

        let active_ids = vec!["OFFICE".to_string()];
        let deactivated = db.deactivate_devices_not_in(&active_ids).await.unwrap();
        assert_eq!(deactivated, 1, "Case-sensitive: 'office' should be deactivated");
        let active = db.get_active_devices().await.unwrap();
        assert_eq!(active.len(), 0);
    }

    #[tokio::test]
    async fn test_get_unsynced_logs_and_mark_in_flight() {
        let db = setup_db().await;
        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 10:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 3);

        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();

        let in_flight_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(in_flight_count, 3, "All 3 logs should be marked as in-flight in DB");

        let logs2 = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs2.len(), 0, "In-flight logs should not be fetched again");
    }

    #[tokio::test]
    async fn test_get_unsynced_logs_batch_update() {
        let db = setup_db().await;
        for i in 1..=150 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-28 10:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let logs = db.get_unsynced_logs(100).await.unwrap();
        assert_eq!(logs.len(), 100, "Should fetch exactly 100 logs");

        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();

        let in_flight_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 2")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(in_flight_count, 100, "All 100 fetched logs should be in-flight in DB");

        let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(remaining, 50, "50 logs should remain unsynced");
    }

    #[tokio::test]
    async fn test_mark_synced_batch_update() {
        let db = setup_db().await;
        for i in 1..=50 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 10:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let logs = db.get_unsynced_logs(50).await.unwrap();
        assert_eq!(logs.len(), 50);

        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_synced(&log_ids).await.unwrap();

        let synced_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 1")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(synced_count, 50, "All 50 logs should be marked as synced");
    }

    #[tokio::test]
    async fn test_mark_synced_empty_array() {
        let db = setup_db().await;
        let result = db.mark_synced(&[]).await;
        assert!(result.is_ok(), "mark_synced should handle empty array");
    }

    #[tokio::test]
    async fn test_concurrent_batch_operations() {
        let db = setup_db().await;
        for i in 1..=200 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-28 10:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let db1 = db.clone();
        let db2 = db.clone();

        let handle1 = tokio::spawn(async move {
            db1.get_unsynced_logs(100).await.unwrap()
        });

        let handle2 = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            db2.get_unsynced_logs(100).await.unwrap()
        });

        let logs1 = handle1.await.unwrap();
        let logs2 = handle2.await.unwrap();

        assert!(!logs1.is_empty() || !logs2.is_empty(), "At least one fetch should succeed");
        let total_fetched = logs1.len() + logs2.len();
        assert!(total_fetched <= 200, "Should not fetch more than available logs");
    }

    #[tokio::test]
    async fn test_count_unsynced_logs() {
        let db = setup_db().await;
        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 0, "Should start with 0 unsynced logs");

        for i in 1..=150 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-28 11:{:02}:00", i % 60),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 150, "Should have 150 unsynced logs");

        let logs = db.get_unsynced_logs(50).await.unwrap();
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();

        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 100, "Should have 100 unsynced after marking 50 in-flight");

        db.mark_synced(&log_ids[0..30]).await.unwrap();
        let count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(count, 100, "Should have 100 unsynced after marking 30 synced");
    }

    #[tokio::test]
    async fn test_fifo_order_get_unsynced_logs() {
        let db = setup_db().await;
        let expected_order = vec!["FIRST", "SECOND", "THIRD", "FOURTH", "FIFTH"];

        for user_id in &expected_order {
            let data = AttendanceData {
                user_id: user_id.to_string(),
                timestamp: "2026-01-28 12:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 5, "Should fetch all 5 logs");

        for (i, log) in logs.iter().enumerate() {
            assert_eq!(
                log.user_id, expected_order[i],
                "Log {} should be {} (FIFO order)", i, expected_order[i]
            );
        }

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
        for i in 1..=10 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 13:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let logs = db.get_unsynced_logs(5).await.unwrap();
        let log_ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&log_ids).await.unwrap();
        db.mark_synced(&log_ids).await.unwrap();

        sqlx::query(
            "UPDATE zkt_raw_attendance
             SET created_at = datetime('now', '-400 days')
             WHERE is_synced = 1"
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        let deleted = db.cleanup_old_synced_logs(365).await.unwrap();
        assert_eq!(deleted, 5, "Should delete 5 old synced logs");

        let remaining = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(remaining, 5, "5 unsynced logs should remain");

        let unsynced_count = db.count_unsynced_logs().await.unwrap();
        assert_eq!(unsynced_count, 5, "All remaining logs should be unsynced");
    }

    #[tokio::test]
    async fn test_get_failed_sync_logs() {
        let db = setup_db().await;
        for i in 1..=10 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: format!("2026-01-01 08:{i:02}:00"),
                status: Some(1),
                punch: Some(0),
            };
            db.save_raw_attendance("device1", data).await.unwrap();
        }

        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 0, "No failed logs initially");

        let logs = db.get_unsynced_logs(5).await.unwrap();
        assert_eq!(logs.len(), 5, "Should fetch 5 unsynced logs");

        let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
        db.mark_in_flight(&ids).await.unwrap();

        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 0, "In-flight logs are not failed yet");

        for id in &ids[0..3] {
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                .bind(id)
                .execute(db.get_pool())
                .await
                .unwrap();
        }

        let failed_count = db.count_failed_sync_logs().await.unwrap();
        assert_eq!(failed_count, 3, "3 logs should be marked as failed");

        let failed_logs = db.get_failed_sync_logs(10).await.unwrap();
        assert_eq!(failed_logs.len(), 3, "Should return 3 failed logs");

        for log in failed_logs {
            assert_eq!(log.is_synced, 0, "Failed logs should have is_synced=0");
        }

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
        for i in 1..=15 {
            let data = AttendanceData {
                user_id: format!("U{i}"),
                timestamp: "2026-01-28 14:00:00".into(),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("D1", data).await.unwrap();
        }

        let logs = db.get_unsynced_logs(10).await.unwrap();
        let synced_ids: Vec<i64> = logs[0..5].iter().map(|l| l.id).collect();
        let inflight_ids: Vec<i64> = logs[5..10].iter().map(|l| l.id).collect();

        db.mark_in_flight(&synced_ids).await.unwrap();
        db.mark_synced(&synced_ids).await.unwrap();
        db.mark_in_flight(&inflight_ids).await.unwrap();

        sqlx::query(
            "UPDATE zkt_raw_attendance
             SET created_at = datetime('now', '-400 days')"
        )
        .execute(db.get_pool())
        .await
        .unwrap();

        let deleted = db.cleanup_old_synced_logs(365).await.unwrap();
        assert_eq!(deleted, 5, "Should only delete synced logs");

        let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM zkt_raw_attendance")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(total, 10, "In-flight and unsynced logs should remain");

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
        for i in 1..=100 {
            let data = AttendanceData {
                user_id: format!("U{i:03}"),
                timestamp: "2026-01-31 10:00:00".to_string(),
                status: Some(1),
                punch: Some(0),
            };
            db.save_raw_attendance(&format!("DEV{}", (i % 5) + 1), data).await.ok();
        }

        let timestamp_unix = NaiveDateTime::parse_from_str("2026-01-31 11:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp();
        let memory_logs: Vec<crate::memory_store::MemoryLog> = (50..=150)
            .map(|i| crate::memory_store::MemoryLog {
                id: i,
                device_id: "DEV2".to_string(),
                user_id: format!("U{:03}", i + 1000),
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

        let result = db.checkpoint_batch(&memory_logs).await;
        assert!(result.is_ok(), "Checkpoint should not fail with ID collision");

        let (inserted, _skipped, keys) = result.unwrap();
        assert!(inserted > 0, "Should insert logs");
        assert!(!keys.is_empty(), "Should return composite keys");
    }

    #[tokio::test]
    async fn test_checkpoint_batch_returns_composite_keys() {
        let db = setup_db().await;
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

        let (inserted, skipped, keys) = db.checkpoint_batch(&memory_logs).await.unwrap();
        assert_eq!(inserted, 5);
        assert_eq!(skipped, 0);
        assert_eq!(keys.len(), 5);

        for (i, (device_id, user_id, _timestamp)) in keys.iter().enumerate() {
            assert_eq!(device_id, "DEV1");
            assert_eq!(user_id, &format!("U{:03}", i + 1));
        }
    }

    #[tokio::test]
    async fn test_count_unsynced_logs_for_device() {
        let db = setup_db().await;
        db.upsert_device("Device A", "Device A", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.upsert_device("Device B", "Device B", "127.0.0.2", 4370, "zkteco", "UTC", true).await.unwrap();

        for i in 1..=5 {
            let data = AttendanceData {
                user_id: format!("U_A_{i}"),
                timestamp: format!("2026-01-28 10:{i:02}:00"),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("Device A", data).await.unwrap();
        }
        let logs_a = db.get_unsynced_logs(2).await.unwrap();
        let ids_a: Vec<i64> = logs_a.iter().map(|l| l.id).collect();
        db.mark_synced(&ids_a).await.unwrap();

        for i in 1..=3 {
            let data = AttendanceData {
                user_id: format!("U_B_{i}"),
                timestamp: format!("2026-01-28 11:{i:02}:00"),
                status: Some(1),
                punch: Some(1),
            };
            db.save_raw_attendance("Device B", data).await.unwrap();
        }

        let count_a = db.count_unsynced_logs_for_device("Device A").await.unwrap();
        assert_eq!(count_a, 3, "Device A should have 3 unsynced logs");

        let count_b = db.count_unsynced_logs_for_device("Device B").await.unwrap();
        assert_eq!(count_b, 3, "Device B should have 3 unsynced logs");

        let count_c = db.count_unsynced_logs_for_device("Device C").await.unwrap();
        assert_eq!(count_c, 0, "Non-existent device should have 0 unsynced logs");
    }
}
