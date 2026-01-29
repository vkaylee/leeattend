use agent::database::Database;
use agent::models::AttendanceData;

/// Test clear_startup_config clears agent_config table
#[tokio::test]
async fn test_clear_startup_config_clears_agent_config() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert some config values
    db.upsert_agent_config("max_retry_count", "10").await.unwrap();
    db.upsert_agent_config("retry_ttl_minutes", "60").await.unwrap();
    db.upsert_agent_config("max_concurrent_workers", "4").await.unwrap();

    // Verify config exists
    let config = db.get_agent_config("max_retry_count").await.unwrap();
    assert_eq!(config, Some("10".to_string()));

    // Clear startup config
    db.clear_startup_config().await.unwrap();

    // Verify all config cleared
    let config1 = db.get_agent_config("max_retry_count").await.unwrap();
    let config2 = db.get_agent_config("retry_ttl_minutes").await.unwrap();
    let config3 = db.get_agent_config("max_concurrent_workers").await.unwrap();

    assert_eq!(config1, None, "max_retry_count should be cleared");
    assert_eq!(config2, None, "retry_ttl_minutes should be cleared");
    assert_eq!(config3, None, "max_concurrent_workers should be cleared");

    println!("✅ Config cleared successfully on startup");
}

/// Test clear_startup_config deactivates all devices
#[tokio::test]
async fn test_clear_startup_config_deactivates_devices() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert some active devices
    db.upsert_device("device1", "Office", "192.168.1.100", 4370, "zkteco").await.unwrap();
    db.upsert_device("device2", "Warehouse", "192.168.1.101", 4370, "zkteco").await.unwrap();
    db.upsert_device("device3", "Factory", "192.168.1.102", 4370, "zkteco").await.unwrap();

    // Verify devices are active
    let active_devices = db.get_active_devices().await.unwrap();
    assert_eq!(active_devices.len(), 3, "Should have 3 active devices");

    // Clear startup config
    db.clear_startup_config().await.unwrap();

    // Verify all devices deactivated
    let active_devices = db.get_active_devices().await.unwrap();
    assert_eq!(active_devices.len(), 0, "Should have 0 active devices after clear");

    println!("✅ All devices deactivated on startup");
}

/// Test clear_startup_config works on empty database
#[tokio::test]
async fn test_clear_startup_config_on_empty_database() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Clear on empty database should not error
    let result = db.clear_startup_config().await;
    assert!(result.is_ok(), "Should succeed on empty database");

    // Verify still empty
    let config = db.get_agent_config("any_key").await.unwrap();
    assert_eq!(config, None);

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 0);

    println!("✅ Clear on empty database works without errors");
}

/// Test clear_startup_config preserves attendance logs
#[tokio::test]
async fn test_clear_startup_config_preserves_logs() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert attendance logs
    let data1 = AttendanceData {
        user_id: "U001".to_string(),
        timestamp: "2026-01-29 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    let data2 = AttendanceData {
        user_id: "U002".to_string(),
        timestamp: "2026-01-29 09:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    db.save_raw_attendance("device1", data1).await.unwrap();
    db.save_raw_attendance("device2", data2).await.unwrap();

    // Verify logs exist
    let logs_before = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs_before.len(), 2, "Should have 2 logs before clear");

    // Clear startup config
    db.clear_startup_config().await.unwrap();

    // Verify logs still exist (should NOT be deleted)
    let logs_after = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs_after.len(), 2, "Should still have 2 logs after clear");

    println!("✅ Attendance logs preserved after startup config clear");
}

/// Test startup simulation: clear config then apply ConfigPush
#[tokio::test]
async fn test_startup_simulation_with_config_push() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Step 1: Simulate previous session with stale config
    db.upsert_agent_config("max_concurrent_workers", "2").await.unwrap();
    db.upsert_device("old_device", "Old Office", "192.168.1.50", 4370, "zkteco").await.unwrap();

    // Verify stale config exists
    let stale_config = db.get_agent_config("max_concurrent_workers").await.unwrap();
    assert_eq!(stale_config, Some("2".to_string()));

    let stale_devices = db.get_active_devices().await.unwrap();
    assert_eq!(stale_devices.len(), 1);

    // Step 2: Agent restarts - clear startup config
    db.clear_startup_config().await.unwrap();

    // Verify config cleared
    let cleared_config = db.get_agent_config("max_concurrent_workers").await.unwrap();
    assert_eq!(cleared_config, None, "Config should be cleared");

    let cleared_devices = db.get_active_devices().await.unwrap();
    assert_eq!(cleared_devices.len(), 0, "Devices should be deactivated");

    // Step 3: Simulate ConfigPush from server (fresh config)
    db.upsert_agent_config("max_concurrent_workers", "4").await.unwrap();
    db.upsert_device("device1", "New Office", "192.168.1.100", 4370, "zkteco").await.unwrap();
    db.upsert_device("device2", "New Warehouse", "192.168.1.101", 4370, "zkteco").await.unwrap();

    // Verify fresh config applied
    let fresh_config = db.get_agent_config("max_concurrent_workers").await.unwrap();
    assert_eq!(fresh_config, Some("4".to_string()), "Should have fresh config");

    let fresh_devices = db.get_active_devices().await.unwrap();
    assert_eq!(fresh_devices.len(), 2, "Should have 2 fresh devices");

    println!("✅ Startup simulation: stale config cleared, fresh config applied");
}

/// Test multiple consecutive startups
#[tokio::test]
async fn test_multiple_consecutive_startups() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // First startup cycle
    db.upsert_agent_config("key1", "value1").await.unwrap();
    db.upsert_device("d1", "Device 1", "10.0.0.1", 4370, "zkteco").await.unwrap();
    db.clear_startup_config().await.unwrap();

    let config1 = db.get_agent_config("key1").await.unwrap();
    assert_eq!(config1, None);

    // Second startup cycle
    db.upsert_agent_config("key2", "value2").await.unwrap();
    db.upsert_device("d2", "Device 2", "10.0.0.2", 4370, "zkteco").await.unwrap();
    db.clear_startup_config().await.unwrap();

    let config2 = db.get_agent_config("key2").await.unwrap();
    assert_eq!(config2, None);

    // Third startup cycle
    db.upsert_agent_config("key3", "value3").await.unwrap();
    db.upsert_device("d3", "Device 3", "10.0.0.3", 4370, "zkteco").await.unwrap();
    db.clear_startup_config().await.unwrap();

    let config3 = db.get_agent_config("key3").await.unwrap();
    assert_eq!(config3, None);

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 0);

    println!("✅ Multiple consecutive startups work correctly");
}

/// Test clear_startup_config preserves DLQ entries
#[tokio::test]
async fn test_clear_startup_config_preserves_dlq() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert logs and simulate DLQ move
    let data = AttendanceData {
        user_id: "U999".to_string(),
        timestamp: "2026-01-29 10:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    db.save_raw_attendance("device1", data).await.unwrap();

    // Simulate retry exhaustion and DLQ move
    for _ in 0..10 {
        let logs = db.get_unsynced_logs(1).await.unwrap();
        if !logs.is_empty() {
            let log_id = logs[0].id;
            db.mark_in_flight(&[log_id]).await.unwrap();
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE id = ?")
                .bind(log_id)
                .execute(db.get_pool())
                .await
                .unwrap();
        }
    }

    db.move_expired_logs_to_dlq(10, 60).await.unwrap();

    // Verify DLQ entry exists
    let dlq_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await
        .unwrap();
    assert_eq!(dlq_before, 1, "Should have 1 DLQ entry");

    // Clear startup config
    db.clear_startup_config().await.unwrap();

    // Verify DLQ preserved
    let dlq_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM zkt_failed_logs")
        .fetch_one(db.get_pool())
        .await
        .unwrap();
    assert_eq!(dlq_after, 1, "DLQ should still have 1 entry after clear");

    println!("✅ DLQ entries preserved after startup config clear");
}

/// Test config clear with retry TTL and max retry settings
#[tokio::test]
async fn test_clear_startup_config_with_retry_settings() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Set retry configuration
    db.upsert_agent_config("max_retry_count", "20").await.unwrap();
    db.upsert_agent_config("retry_ttl_minutes", "120").await.unwrap();

    // Verify config set
    let max_retries = db.get_agent_config("max_retry_count").await.unwrap();
    let ttl = db.get_agent_config("retry_ttl_minutes").await.unwrap();
    assert_eq!(max_retries, Some("20".to_string()));
    assert_eq!(ttl, Some("120".to_string()));

    // Clear startup config
    db.clear_startup_config().await.unwrap();

    // Verify retry settings cleared (will use defaults)
    let max_retries_after = db.get_agent_config("max_retry_count").await.unwrap();
    let ttl_after = db.get_agent_config("retry_ttl_minutes").await.unwrap();
    assert_eq!(max_retries_after, None, "max_retry_count should be cleared");
    assert_eq!(ttl_after, None, "retry_ttl_minutes should be cleared");

    println!("✅ Retry configuration cleared correctly on startup");
}
