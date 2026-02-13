use agent::database::Database;
use agent::models::AttendanceData;

/// Test that new logs (updated_at IS NULL) are correctly identified vs retries.
#[tokio::test]
async fn test_sync_log_classification_new_vs_retry() {
    let db = Database::new("sqlite::memory:").await.expect("Failed to create in-memory DB");

    // Insert 5 logs without updated_at (new logs)
    for i in 1..=5 {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-28 08:0{i}:00"),
            status: Some(1),
            punch: Some(0),
        };
        db.save_raw_attendance("device1", data).await.unwrap();
    }

    // Fetch and mark 3 as in-flight (sets updated_at)
    let logs = db.get_unsynced_logs(3).await.unwrap();
    assert_eq!(logs.len(), 3);
    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
    db.mark_in_flight(&ids).await.unwrap();

    // Simulate sync failure - reset to unsynced but updated_at remains
    for id in &ids {
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
            .bind(id)
            .execute(db.get_pool())
            .await
            .unwrap();
    }

    // Now fetch all unsynced logs
    let all_unsynced = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(all_unsynced.len(), 5, "Should have 5 unsynced logs total");

    // Count new vs retry
    let new_count = all_unsynced.iter().filter(|l| l.updated_at.is_none()).count();
    let retry_count = all_unsynced.iter().filter(|l| l.updated_at.is_some()).count();

    assert_eq!(new_count, 2, "Should have 2 new logs (never attempted)");
    assert_eq!(retry_count, 3, "Should have 3 retry logs (previously failed)");

    println!("✅ Sync log classification test passed:");
    println!("   Total unsynced: {}", all_unsynced.len());
    println!("   New logs: {new_count}");
    println!("   Retry logs: {retry_count}");
}

/// Test that log classification appears in sync messages.
#[tokio::test]
async fn test_sync_message_includes_classification() {
    let db = Database::new("sqlite::memory:").await.expect("Failed to create in-memory DB");

    // Insert 10 new logs
    for i in 1..=10 {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-28 09:{i}:00"),
            status: Some(1),
            punch: Some(0),
        };
        db.save_raw_attendance("device1", data).await.unwrap();
    }

    // Fetch logs
    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 10);

    // Classify logs (simulates central_client.rs logic)
    let new_count = logs.iter().filter(|l| l.updated_at.is_none()).count();
    let retry_count = logs.len() - new_count;

    assert_eq!(new_count, 10, "All 10 should be new");
    assert_eq!(retry_count, 0, "None should be retries");

    // Simulate message format
    let message = if retry_count > 0 {
        format!("📤 Syncing {} logs to central server ({} new, {} retries)...",
                logs.len(), new_count, retry_count)
    } else {
        format!("📤 Syncing {} logs to central server...", logs.len())
    };

    assert_eq!(message, "📤 Syncing 10 logs to central server...");

    println!("✅ Sync message format test passed:");
    println!("   Message: {message}");
}

/// Test high retry count scenario (system health indicator).
#[tokio::test]
async fn test_high_retry_count_detection() {
    let db = Database::new("sqlite::memory:").await.expect("Failed to create in-memory DB");

    // Insert 100 logs
    for i in 1..=100 {
        let data = AttendanceData {
            user_id: format!("U{i}"),
            timestamp: format!("2026-01-28 10:{:02}:{:02}", i / 60, i % 60),
            status: Some(1),
            punch: Some(0),
        };
        db.save_raw_attendance("device1", data).await.unwrap();
    }

    // Mark 80 as in-flight then reset (simulate failures)
    let logs = db.get_unsynced_logs(80).await.unwrap();
    let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
    db.mark_in_flight(&ids).await.unwrap();

    for id in &ids {
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
            .bind(id)
            .execute(db.get_pool())
            .await
            .unwrap();
    }

    // Fetch all unsynced
    let all_unsynced = db.get_unsynced_logs(100).await.unwrap();
    let new_count = all_unsynced.iter().filter(|l| l.updated_at.is_none()).count();
    let retry_count = all_unsynced.iter().filter(|l| l.updated_at.is_some()).count();

    assert_eq!(new_count, 20, "Should have 20 new logs");
    assert_eq!(retry_count, 80, "Should have 80 retry logs");

    // Health check: High retry rate indicates problems
    let retry_rate = (retry_count as f64 / all_unsynced.len() as f64) * 100.0;
    assert!(retry_rate > 50.0, "Retry rate should be > 50% (system unhealthy)");

    println!("✅ High retry count detection test passed:");
    println!("   Total: {}", all_unsynced.len());
    println!("   New: {} ({}%)", new_count, (new_count as f64 / all_unsynced.len() as f64) * 100.0);
    println!("   Retry: {retry_count} ({retry_rate:.1}%)");
    println!("   ⚠️ Health: UNHEALTHY (high retry rate)");
}
