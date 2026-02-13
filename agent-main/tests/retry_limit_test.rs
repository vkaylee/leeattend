use agent::database::Database;
use agent::models::AttendanceData;

/// Test retry count increments on each mark_in_flight call
#[tokio::test]
async fn test_retry_count_increments() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert a log
    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-28 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("device1", data).await.unwrap();

    // Simulate 3 retry attempts
    for expected_count in 1..=3 {
        let logs = db.get_unsynced_logs(1).await.unwrap();
        assert_eq!(logs.len(), 1);
        let log_id = logs[0].id;

        // Mark in-flight (increments retry_count)
        db.mark_in_flight(&[log_id]).await.unwrap();

        // Reset to unsynced (simulate failure)
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
            .bind(log_id)
            .execute(db.get_pool())
            .await
            .unwrap();

        // Verify retry_count incremented
        let logs = db.get_unsynced_logs(1).await.unwrap();
        assert_eq!(logs[0].retry_count, expected_count);
    }

    println!("✅ Retry count increments correctly on each attempt");
}

/// Test logs exceeding max retry count are moved to DLQ
#[tokio::test]
async fn test_max_retry_limit_moves_to_dlq() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert a log
    let data = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-28 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("device1", data).await.unwrap();

    // Simulate 10 failed attempts
    for _ in 0..10 {
        let logs = db.get_unsynced_logs(1).await.unwrap();
        let log_id = logs[0].id;
        db.mark_in_flight(&[log_id]).await.unwrap();

        // Reset to unsynced
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
            .bind(log_id)
            .execute(db.get_pool())
            .await
            .unwrap();
    }

    // Verify retry_count = 10
    let logs = db.get_unsynced_logs(1).await.unwrap();
    assert_eq!(logs[0].retry_count, 10);

    // Move expired logs (max_retries=10, ttl=60)
    let moved = db.move_expired_logs_to_dlq(10, 60).await.unwrap();
    assert_eq!(moved, 1, "Should move 1 log to DLQ");

    // Verify log removed from main table
    let remaining = db.get_unsynced_logs(1).await.unwrap();
    assert_eq!(remaining.len(), 0, "Should have no unsynced logs");

    // Verify log exists in failed_logs table
    let failed: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, failure_reason FROM zkt_failed_logs WHERE device_id = 'device1'"
    )
    .fetch_all(db.get_pool())
    .await
    .unwrap();

    assert_eq!(failed.len(), 1);
    assert!(failed[0].1.contains("Max retries exceeded"));

    println!("✅ Logs exceeding max retries moved to DLQ");
}

/// Test TTL-based expiration moves logs to DLQ
#[tokio::test]
async fn test_ttl_expiration_moves_to_dlq() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert a log
    let data = AttendanceData {
        user_id: "U002".to_string(),
        timestamp: "2026-01-28 09:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("device2", data).await.unwrap();

    // Mark in-flight (sets updated_at)
    let logs = db.get_unsynced_logs(1).await.unwrap();
    let log_id = logs[0].id;
    db.mark_in_flight(&[log_id]).await.unwrap();

    // Reset to unsynced
    sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
        .bind(log_id)
        .execute(db.get_pool())
        .await
        .unwrap();

    // Simulate TTL expired by setting updated_at to 2 hours ago
    sqlx::query("UPDATE zkt_raw_attendance SET updated_at = datetime('now', '-2 hours') WHERE id = ?")
        .bind(log_id)
        .execute(db.get_pool())
        .await
        .unwrap();

    // Move expired logs (max_retries=10, ttl=60 minutes)
    let moved = db.move_expired_logs_to_dlq(10, 60).await.unwrap();
    assert_eq!(moved, 1, "Should move 1 log to DLQ");

    // Verify log removed from main table
    let remaining = db.get_unsynced_logs(1).await.unwrap();
    assert_eq!(remaining.len(), 0);

    // Verify log exists in failed_logs table
    let failed: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, failure_reason FROM zkt_failed_logs WHERE device_id = 'device2'"
    )
    .fetch_all(db.get_pool())
    .await
    .unwrap();

    assert_eq!(failed.len(), 1);
    assert!(failed[0].1.contains("TTL exceeded"));

    println!("✅ Logs exceeding TTL moved to DLQ");
}

/// Test logs within limits are NOT moved to DLQ
#[tokio::test]
async fn test_valid_retries_not_moved_to_dlq() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert 3 logs with different retry counts
    for i in 1..=3 {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-28 10:0{i}:00"),
            status: Some(1),
            punch: Some(0),
        };
        db.save_raw_attendance("device1", data).await.unwrap();
    }

    // Simulate different retry counts (1, 5, 9 - all below limit)
    let retry_counts = [1, 5, 9];
    for (idx, &count) in retry_counts.iter().enumerate() {
        for _ in 0..count {
            let logs = db.get_unsynced_logs(10).await.unwrap();
            let log_id = logs[idx].id;
            db.mark_in_flight(&[log_id]).await.unwrap();
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                .bind(log_id)
                .execute(db.get_pool())
                .await
                .unwrap();
        }
    }

    // Move expired logs (max_retries=10, ttl=60)
    let moved = db.move_expired_logs_to_dlq(10, 60).await.unwrap();
    assert_eq!(moved, 0, "Should NOT move any logs (all within limits)");

    // Verify all logs still in main table
    let remaining = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(remaining.len(), 3);

    println!("✅ Logs within retry limits remain in queue");
}

/// Test batch DLQ move (multiple expired logs)
#[tokio::test]
async fn test_batch_dlq_move() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert 20 logs
    for i in 1..=20 {
        let data = AttendanceData {
            user_id: format!("U{i:03}"),
            timestamp: format!("2026-01-28 11:{:02}:00", i % 60),
            status: Some(1),
            punch: Some(0),
        };
        db.save_raw_attendance("device_batch", data).await.unwrap();
    }

    // Simulate 15 logs exceeding max retries
    for i in 0..15 {
        for _ in 0..10 {
            let logs = db.get_unsynced_logs(20).await.unwrap();
            let log_id = logs[i].id;
            db.mark_in_flight(&[log_id]).await.unwrap();
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                .bind(log_id)
                .execute(db.get_pool())
                .await
                .unwrap();
        }
    }

    // Move expired logs
    let moved = db.move_expired_logs_to_dlq(10, 60).await.unwrap();
    assert_eq!(moved, 15, "Should move 15 logs to DLQ");

    // Verify remaining logs
    let remaining = db.get_unsynced_logs(20).await.unwrap();
    assert_eq!(remaining.len(), 5, "Should have 5 logs remaining");

    // Verify DLQ count
    let dlq_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM zkt_failed_logs WHERE device_id = 'device_batch'"
    )
    .fetch_one(db.get_pool())
    .await
    .unwrap();
    assert_eq!(dlq_count, 15);

    println!("✅ Batch DLQ move works correctly");
}

/// Test DLQ preserves log metadata
#[tokio::test]
async fn test_dlq_preserves_metadata() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    let data = AttendanceData {
        user_id: "U999".to_string(),
        timestamp: "2026-01-28 12:30:00".to_string(),
        status: Some(1),
        punch: Some(3),
    };
    db.save_raw_attendance("metadata_test", data).await.unwrap();

    // Simulate 10 retries
    for _ in 0..10 {
        let logs = db.get_unsynced_logs(1).await.unwrap();
        let log_id = logs[0].id;
        db.mark_in_flight(&[log_id]).await.unwrap();
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
            .bind(log_id)
            .execute(db.get_pool())
            .await
            .unwrap();
    }

    // Move to DLQ
    db.move_expired_logs_to_dlq(10, 60).await.unwrap();

    // Verify metadata preserved in DLQ
    let failed: Vec<(String, String, i32, i32, i64)> = sqlx::query_as(
        "SELECT device_id, user_id, status, punch, retry_attempts FROM zkt_failed_logs WHERE user_id = 'U999'"
    )
    .fetch_all(db.get_pool())
    .await
    .unwrap();

    assert_eq!(failed.len(), 1);
    let (device_id, user_id, status, punch, retry_attempts) = &failed[0];
    assert_eq!(device_id, "metadata_test");
    assert_eq!(user_id, "U999");
    assert_eq!(*status, 1);
    assert_eq!(*punch, 3);
    assert_eq!(*retry_attempts, 10);

    println!("✅ DLQ preserves all log metadata");
}

/// Test configurable retry limits
#[tokio::test]
async fn test_configurable_retry_limits() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Set custom limits via agent_config
    db.upsert_agent_config("max_retry_count", "5").await.unwrap();
    db.upsert_agent_config("retry_ttl_minutes", "30").await.unwrap();

    // Verify config stored
    let max_retries = db.get_agent_config("max_retry_count").await.unwrap();
    let ttl = db.get_agent_config("retry_ttl_minutes").await.unwrap();

    assert_eq!(max_retries, Some("5".to_string()));
    assert_eq!(ttl, Some("30".to_string()));

    // Insert and retry log 5 times
    let data = AttendanceData {
        user_id: "UCONFIG".to_string(),
        timestamp: "2026-01-28 13:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("config_device", data).await.unwrap();

    for _ in 0..5 {
        let logs = db.get_unsynced_logs(1).await.unwrap();
        let log_id = logs[0].id;
        db.mark_in_flight(&[log_id]).await.unwrap();
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
            .bind(log_id)
            .execute(db.get_pool())
            .await
            .unwrap();
    }

    // Move with custom limit
    let moved = db.move_expired_logs_to_dlq(5, 30).await.unwrap();
    assert_eq!(moved, 1, "Should move log with custom limit of 5");

    println!("✅ Configurable retry limits work correctly");
}
