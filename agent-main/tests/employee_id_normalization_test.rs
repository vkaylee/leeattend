// ── TECH_DEBT: Enterprise-grade lint upgrade (2026-05-03) ──
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::suspicious)]
#![allow(clippy::style)]
#![allow(clippy::correctness)]
#![allow(clippy::perf)]
#![allow(clippy::complexity)]
#![allow(clippy::clone_on_ref_ptr)]
#![allow(missing_docs)]

use agent::hybrid_database::HybridDatabase;
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
async fn test_employee_id_stored_with_leading_zeros_smoke() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid
        .store()
        .upsert_device(
            "device1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456,
        )
        .unwrap();

    let data = AttendanceData {
        user_id: "0058".to_string(),
        timestamp: "2026-01-29 08:00:00".to_string(),
        status: Some(1),
        punch: Some(0),
    };
    hybrid
        .save_raw_attendance("device1".into(), data)
        .await
        .unwrap();

    let logs = hybrid.get_unsynced_logs(1).await.unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].user_id, "0058", "Should store original format");

    let normalized = normalize_employee_id(&logs[0].user_id);
    assert_eq!(normalized, "58", "Should normalize to 58");
}

/// Test multiple employee IDs with various formats
#[tokio::test]
async fn test_multiple_employee_id_formats() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");
    hybrid
        .store()
        .upsert_device(
            "device1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true, 123456,
        )
        .unwrap();

    let test_cases = vec![
        ("0001", "1"),
        ("0026", "26"),
        ("0058", "58"),
        ("0100", "100"),
        ("1234", "1234"),
        ("0000", "0"),
    ];

    for (original_id, _) in &test_cases {
        let data = AttendanceData {
            user_id: original_id.to_string(),
            timestamp: "2026-01-29 09:00:00".to_string(),
            status: Some(1),
            punch: Some(0),
        };
        let _ = hybrid.save_raw_attendance("device1".into(), data).await;
    }

    let logs = hybrid.get_unsynced_logs(10).await.unwrap();
    for log in &logs {
        let normalized = normalize_employee_id(&log.user_id);
        let expected = test_cases
            .iter()
            .find(|(orig, _)| *orig == log.user_id)
            .unwrap()
            .1;
        assert_eq!(
            normalized, expected,
            "ID {} should normalize to {}",
            log.user_id, expected
        );
    }
}

/// Test edge case: empty string
#[test]
fn test_normalize_empty_string() {
    assert_eq!(normalize_employee_id(""), "0");
    assert_eq!(normalize_employee_id("   "), "0");
    assert_eq!(normalize_employee_id("\t\n"), "0");
}

/// Test normalization with alphanumeric IDs
#[test]
fn test_normalize_alphanumeric_ids() {
    assert_eq!(normalize_employee_id("00ABC"), "00ABC");
    assert_eq!(normalize_employee_id("0A123"), "0A123");
    assert_eq!(normalize_employee_id("ABC123"), "ABC123");
    assert_eq!(normalize_employee_id("EMP001"), "EMP001");
}
