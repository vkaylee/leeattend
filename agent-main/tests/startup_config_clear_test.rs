use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;

#[test]
fn test_clear_startup_config_clears_agent_config() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store.upsert_agent_config("max_retry_count", "10").unwrap();
    store.upsert_agent_config("retry_ttl_minutes", "60").unwrap();
    store.upsert_agent_config("max_concurrent_workers", "4").unwrap();

    assert_eq!(store.get_agent_config("max_retry_count").unwrap(), Some("10".to_string()));

    store.clear_all_config().unwrap();

    assert_eq!(store.get_agent_config("max_retry_count").unwrap(), None);
    assert_eq!(store.get_agent_config("retry_ttl_minutes").unwrap(), None);
    assert_eq!(store.get_agent_config("max_concurrent_workers").unwrap(), None);
}

#[test]
fn test_clear_startup_config_deactivates_devices() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store.upsert_device("device1", "Office", "192.168.1.100", 4370, "zkteco", "UTC", None, true, 123456).unwrap();
    store.upsert_device("device2", "Warehouse", "192.168.1.101", 4370, "zkteco", "UTC", None, true, 123456).unwrap();
    store.upsert_device("device3", "Factory", "192.168.1.102", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

    assert_eq!(store.get_active_devices().unwrap().len(), 3);

    store.deactivate_all_devices().unwrap();

    assert_eq!(store.get_active_devices().unwrap().len(), 0);
}

#[test]
fn test_clear_startup_config_on_empty_database() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    assert!(store.clear_all_config().is_ok());
    assert_eq!(store.get_agent_config("any_key").unwrap(), None);
    assert_eq!(store.get_active_devices().unwrap().len(), 0);
}

#[test]
fn test_clear_startup_config_preserves_logs() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store.upsert_device("device1", "Dev 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456).unwrap();
    store.upsert_device("device2", "Dev 2", "1.2.3.5", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

    let data1 = AttendanceData { user_id: "U001".to_string(), timestamp: "2026-01-29 08:00:00".to_string(), status: Some(1), punch: Some(0) };
    let data2 = AttendanceData { user_id: "U002".to_string(), timestamp: "2026-01-29 09:00:00".to_string(), status: Some(1), punch: Some(0) };

    hybrid.save_raw_attendance("device1", data1).unwrap();
    hybrid.save_raw_attendance("device2", data2).unwrap();
    assert_eq!(hybrid.count_unsynced_logs(), 2);

    store.clear_all_config().unwrap();

    assert_eq!(hybrid.count_unsynced_logs(), 2);
}

#[test]
fn test_startup_simulation_with_config_push() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    // Stale config from previous session
    store.upsert_agent_config("max_concurrent_workers", "2").unwrap();
    store.upsert_device("old_device", "Old Office", "192.168.1.50", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

    assert_eq!(store.get_agent_config("max_concurrent_workers").unwrap(), Some("2".to_string()));
    assert_eq!(store.get_active_devices().unwrap().len(), 1);

    // Agent restarts — clear
    store.clear_all_config().unwrap();
    store.deactivate_all_devices().unwrap();

    assert_eq!(store.get_agent_config("max_concurrent_workers").unwrap(), None);
    assert_eq!(store.get_active_devices().unwrap().len(), 0);

    // Fresh config push
    store.upsert_agent_config("max_concurrent_workers", "4").unwrap();
    store.upsert_device("device1", "New Office", "192.168.1.100", 4370, "zkteco", "UTC", None, true, 123456).unwrap();
    store.upsert_device("device2", "New WH", "192.168.1.101", 4370, "zkteco", "UTC", None, true, 123456).unwrap();

    assert_eq!(store.get_agent_config("max_concurrent_workers").unwrap(), Some("4".to_string()));
    assert_eq!(store.get_active_devices().unwrap().len(), 2);
}

#[test]
fn test_clear_startup_config_with_retry_settings() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    let store = hybrid.store();

    store.upsert_agent_config("max_retry_count", "20").unwrap();
    store.upsert_agent_config("retry_ttl_minutes", "120").unwrap();

    assert_eq!(store.get_agent_config("max_retry_count").unwrap(), Some("20".to_string()));
    assert_eq!(store.get_agent_config("retry_ttl_minutes").unwrap(), Some("120".to_string()));

    store.clear_all_config().unwrap();

    assert_eq!(store.get_agent_config("max_retry_count").unwrap(), None);
    assert_eq!(store.get_agent_config("retry_ttl_minutes").unwrap(), None);
}
