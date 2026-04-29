use crate::redb_store::RedbStore;

/// Extract device metadata fields from a serde_json info object and persist.
/// Centralises the 10-field unwrap_or("N/A") pattern.
pub fn update_metadata_from_info(
    store: &RedbStore,
    device_id: &str,
    info: &serde_json::Value,
    now_ts: i64,
) {
    if let Some(obj) = info.as_object() {
        let _ = store.update_device_metadata(
            device_id,
            obj.get("mac").and_then(|v| v.as_str()).unwrap_or("N/A"),
            obj.get("serial").and_then(|v| v.as_str()).unwrap_or("N/A"),
            obj.get("firmware")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            obj.get("platform")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            obj.get("hw_name").and_then(|v| v.as_str()).unwrap_or("N/A"),
            obj.get("users_count").and_then(|v| v.as_i64()).unwrap_or(0),
            obj.get("logs_count").and_then(|v| v.as_i64()).unwrap_or(0),
            obj.get("users_capacity")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            obj.get("logs_capacity")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            obj.get("device_time")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            now_ts,
        );
    }
}

/// Normalize employee ID by trimming whitespace and removing leading zeros if purely numeric
pub fn normalize_employee_id(id: &str) -> String {
    let trimmed = id.trim();
    if trimmed.is_empty() {
        return "0".to_string();
    }

    // Check if the ID is purely numeric
    if trimmed.chars().all(|c| c.is_ascii_digit()) {
        let normalized = trimmed.trim_start_matches('0');
        if normalized.is_empty() {
            "0".to_string()
        } else {
            normalized.to_string()
        }
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_employee_id_removes_leading_zeros() {
        assert_eq!(normalize_employee_id("0058"), "58");
        assert_eq!(normalize_employee_id("0026"), "26");
        assert_eq!(normalize_employee_id("0001"), "1");
        assert_eq!(normalize_employee_id("000123"), "123");
    }

    #[test]
    fn test_normalize_employee_id_no_leading_zeros() {
        assert_eq!(normalize_employee_id("1234"), "1234");
        assert_eq!(normalize_employee_id("58"), "58");
        assert_eq!(normalize_employee_id("9"), "9");
    }

    #[test]
    fn test_normalize_employee_id_all_zeros() {
        assert_eq!(normalize_employee_id("0000"), "0");
        assert_eq!(normalize_employee_id("00"), "0");
        assert_eq!(normalize_employee_id("0"), "0");
    }

    #[test]
    fn test_normalize_employee_id_single_digit() {
        assert_eq!(normalize_employee_id("5"), "5");
        assert_eq!(normalize_employee_id("0"), "0");
    }

    #[test]
    fn test_normalize_employee_id_mixed() {
        assert_eq!(normalize_employee_id("0100"), "100");
        assert_eq!(normalize_employee_id("00500"), "500");
        assert_eq!(normalize_employee_id("00050"), "50");
    }

    #[test]
    fn test_normalize_employee_id_whitespace() {
        assert_eq!(normalize_employee_id("  123  "), "123");
        assert_eq!(normalize_employee_id("\t456\n"), "456");
        assert_eq!(normalize_employee_id("  00055  "), "55");
    }

    #[test]
    fn test_normalize_employee_id_alphanumeric_preserved() {
        // Alphanumeric IDs are preserved as-is (not normalized)
        assert_eq!(normalize_employee_id("EMP001"), "EMP001");
        assert_eq!(normalize_employee_id("00A1"), "00A1");
        assert_eq!(normalize_employee_id("123-456"), "123-456");
        assert_eq!(normalize_employee_id("  EMP001  "), "EMP001");
    }

    #[test]
    fn test_normalize_employee_id_empty() {
        assert_eq!(normalize_employee_id(""), "0");
        assert_eq!(normalize_employee_id("   "), "0");
    }

    // === update_metadata_from_info tests ===

    fn make_store() -> RedbStore {
        RedbStore::new(":memory:", 90).expect("in-memory RedbStore")
    }

    #[test]
    fn test_update_metadata_full_info() {
        let store = make_store();
        store
            .upsert_device(
                "dev1",
                "Office",
                "192.168.1.1",
                4370,
                "zkteco",
                "UTC",
                None,
                true,
                123456,
            )
            .unwrap();

        let info = serde_json::json!({
            "mac": "AA:BB:CC:DD:EE:FF",
            "serial": "SN12345",
            "firmware": "v1.2.3",
            "platform": "ZEM600",
            "hw_name": "TFT800",
            "users_count": 150,
            "logs_count": 5000,
            "users_capacity": 3000,
            "logs_capacity": 100000,
            "device_time": "2026-03-08 09:00:00"
        });

        update_metadata_from_info(&store, "dev1", &info, 123456);

        let meta = store.get_device_metadata("dev1").unwrap().unwrap();
        assert_eq!(meta.mac.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
        assert_eq!(meta.serial.as_deref(), Some("SN12345"));
        assert_eq!(meta.firmware.as_deref(), Some("v1.2.3"));
        assert_eq!(meta.platform.as_deref(), Some("ZEM600"));
        assert_eq!(meta.hw_name.as_deref(), Some("TFT800"));
        assert_eq!(meta.users_count, Some(150));
        assert_eq!(meta.logs_count, Some(5000));
        assert_eq!(meta.users_capacity, Some(3000));
        assert_eq!(meta.logs_capacity, Some(100000));
        assert_eq!(meta.device_time.as_deref(), Some("2026-03-08 09:00:00"));
    }

    #[test]
    fn test_update_metadata_partial_info_defaults_to_na() {
        let store = make_store();
        store
            .upsert_device(
                "dev2", "Lobby", "10.0.0.1", 4370, "zkteco", "UTC", None, true, 123456,
            )
            .unwrap();

        // Only mac and serial present — other fields should default to N/A / 0
        let info = serde_json::json!({
            "mac": "11:22:33:44:55:66",
            "serial": "SN999"
        });

        update_metadata_from_info(&store, "dev2", &info, 123456);

        let meta = store.get_device_metadata("dev2").unwrap().unwrap();
        assert_eq!(meta.mac.as_deref(), Some("11:22:33:44:55:66"));
        assert_eq!(meta.serial.as_deref(), Some("SN999"));
        assert_eq!(meta.firmware.as_deref(), Some("N/A"));
        assert_eq!(meta.platform.as_deref(), Some("N/A"));
        assert_eq!(meta.users_count, Some(0));
        assert_eq!(meta.logs_count, Some(0));
        assert_eq!(meta.device_time.as_deref(), Some("N/A"));
    }

    #[test]
    fn test_update_metadata_empty_object_sets_defaults() {
        let store = make_store();
        store
            .upsert_device(
                "dev3", "Gate", "10.0.0.2", 4370, "zkteco", "UTC", None, true, 123456,
            )
            .unwrap();

        let info = serde_json::json!({});
        update_metadata_from_info(&store, "dev3", &info, 123456);

        let meta = store.get_device_metadata("dev3").unwrap().unwrap();
        assert_eq!(meta.mac.as_deref(), Some("N/A"));
        assert_eq!(meta.serial.as_deref(), Some("N/A"));
        assert_eq!(meta.users_count, Some(0));
    }

    #[test]
    fn test_update_metadata_non_object_is_noop() {
        let store = make_store();
        store
            .upsert_device(
                "dev4", "Door", "10.0.0.3", 4370, "zkteco", "UTC", None, true, 123456,
            )
            .unwrap();

        // Array, string, null — none are objects, so nothing should be updated
        update_metadata_from_info(&store, "dev4", &serde_json::json!([1, 2, 3]), 123456);
        update_metadata_from_info(&store, "dev4", &serde_json::json!("just a string"), 123456);
        update_metadata_from_info(&store, "dev4", &serde_json::Value::Null, 123456);

        let meta = store.get_device_metadata("dev4").unwrap().unwrap();
        // Device exists but metadata fields were never set (only upsert_device fields)
        assert_eq!(meta.mac, None);
        assert_eq!(meta.serial, None);
    }

    #[test]
    fn test_update_metadata_nonexistent_device_is_noop() {
        let store = make_store();
        // Device "ghost" was never upserted
        let info = serde_json::json!({ "mac": "FF:FF:FF:FF:FF:FF" });
        update_metadata_from_info(&store, "ghost", &info, 123456);

        // Should not crash, device simply doesn't exist
        assert!(store.get_device_metadata("ghost").unwrap().is_none());
    }
}
