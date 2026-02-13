use agent::database::Database;
use agent::models::AttendanceData;
use agent::utils::normalize_employee_id;

/// Test normalize_employee_id function directly
#[test]
fn test_normalize_employee_id_direct() {
    assert_eq!(normalize_employee_id("0058"), "58");
    assert_eq!(normalize_employee_id("0026"), "26");
    assert_eq!(normalize_employee_id("1234"), "1234");
    assert_eq!(normalize_employee_id("0000"), "0");
}

/// Test employee ID normalization in database flow
#[tokio::test]
async fn test_employee_id_stored_with_leading_zeros() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert log with leading zeros (as received from device)
    let data = AttendanceData {
        user_id: "0058".to_string(), // Device sends with leading zeros
        timestamp: "2026-01-29 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    db.save_raw_attendance("device1", data).await.unwrap();

    // Retrieve log
    let logs = db.get_unsynced_logs(1).await.unwrap();
    assert_eq!(logs.len(), 1);

    // Verify stored with leading zeros (as received)
    assert_eq!(logs[0].user_id, "0058", "Should store original format");

    // Verify normalization removes leading zeros
    let normalized = normalize_employee_id(&logs[0].user_id);
    assert_eq!(normalized, "58", "Should normalize to 58");

    println!("✅ Employee ID stored as received, normalized on sync");
}

/// Test multiple employee IDs with various formats
#[tokio::test]
async fn test_multiple_employee_id_formats() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert logs with different ID formats
    let test_cases = vec![
        ("0001", "1"),
        ("0026", "26"),
        ("0058", "58"),
        ("0100", "100"),
        ("1234", "1234"),
        ("0000", "0"),
    ];

    for (original_id, _expected_normalized) in &test_cases {
        let data = AttendanceData {
            user_id: original_id.to_string(),
            timestamp: "2026-01-29 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        db.save_raw_attendance("device1", data).await.unwrap();
    }

    // Retrieve all logs
    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 6);

    // Verify each normalization
    for (i, (original_id, expected_normalized)) in test_cases.iter().enumerate() {
        assert_eq!(logs[i].user_id, *original_id, "Original ID should be stored as-is");

        let normalized = normalize_employee_id(&logs[i].user_id);
        assert_eq!(
            normalized, *expected_normalized,
            "ID {original_id} should normalize to {expected_normalized}"
        );
    }

    println!("✅ All employee ID formats normalized correctly");
}

/// Test normalization preserves non-zero-padded IDs
#[tokio::test]
async fn test_normalization_preserves_regular_ids() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Insert logs with regular IDs (no leading zeros)
    let regular_ids = vec!["123", "456", "789", "1", "99"];

    for id in &regular_ids {
        let data = AttendanceData {
            user_id: id.to_string(),
            timestamp: "2026-01-29 10:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };

        db.save_raw_attendance("device1", data).await.unwrap();
    }

    // Retrieve logs
    let logs = db.get_unsynced_logs(10).await.unwrap();
    assert_eq!(logs.len(), 5);

    // Verify normalization doesn't change regular IDs
    for (i, expected_id) in regular_ids.iter().enumerate() {
        let normalized = normalize_employee_id(&logs[i].user_id);
        assert_eq!(
            normalized, *expected_id,
            "Regular ID {expected_id} should remain unchanged"
        );
    }

    println!("✅ Regular IDs preserved without modification");
}

/// Test edge case: empty string
#[test]
fn test_normalize_empty_string() {
    // Empty string normalizes to "0" (matching API server behavior)
    let result = normalize_employee_id("");
    assert_eq!(result, "0", "Empty string should normalize to 0");

    // Whitespace-only also normalizes to "0"
    assert_eq!(normalize_employee_id("   "), "0");
    assert_eq!(normalize_employee_id("\t\n"), "0");
}

/// Test normalization with alphanumeric IDs (preserved as-is, matching API server logic)
#[test]
fn test_normalize_alphanumeric_ids() {
    // Alphanumeric IDs are preserved as-is (matching API server logic)
    // Only purely numeric IDs have leading zeros removed
    assert_eq!(normalize_employee_id("00ABC"), "00ABC"); // Not purely numeric
    assert_eq!(normalize_employee_id("0A123"), "0A123"); // Not purely numeric
    assert_eq!(normalize_employee_id("ABC123"), "ABC123"); // Not purely numeric
    assert_eq!(normalize_employee_id("EMP001"), "EMP001"); // Not purely numeric
}

/// Test the actual sync scenario that was failing
#[tokio::test]
async fn test_sync_scenario_employee_0058() {
    let db = Database::new("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Simulate the exact scenario from the error log
    let data = AttendanceData {
        user_id: "0058".to_string(), // This was causing "Employee not found"
        timestamp: "2026-01-29 04:59:17".to_string(),
        status: Some(1),
        punch: Some(0),
    };

    db.save_raw_attendance("device1", data).await.unwrap();

    // Retrieve log
    let logs = db.get_unsynced_logs(1).await.unwrap();
    assert_eq!(logs[0].user_id, "0058");

    // This is what gets sent to server (normalized)
    let normalized_id = normalize_employee_id(&logs[0].user_id);
    assert_eq!(normalized_id, "58");

    println!("✅ Employee ID 0058 → 58 (server expects normalized format)");
}
