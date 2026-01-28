mod common;

use apiserver::services::{AttendanceLogService, AgentLogEntry};
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_attendance_service_ingestion_synchronous() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    // Use Builder pattern from factory
    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Fetch employee details to get id_number
    let employee = sqlx::query!("SELECT employee_id_number FROM employees WHERE id = $1", emp_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    // Initialize service
    let (service, _worker_handle) = AttendanceLogService::new(
        pool.clone(),
        100, // capacity
        1,   // batch size
        100, // flush interval 100ms
    );

    let entry = AgentLogEntry {
        workspace_id: ws_id,
        employee_id_number: employee.employee_id_number.clone(),
        punch_time_raw: "2026-01-26 09:00:00".to_string(),
        device_id: Some("TEST_DEVICE".to_string()),
        raw_data: None,
        timezone: "Asia/Ho_Chi_Minh".to_string(),
        date_fmt: "YYYY-MM-DD".to_string(),
        time_fmt: "24h".to_string(),
    };

    // Act: Ingest log synchronously
    let result = service.ingest_agent_log(entry).await;

    // Assert: Should succeed immediately
    assert!(result.is_ok(), "Ingestion should succeed");

    // Assert: Log should be in DB immediately (no sleep needed)
    let logs = sqlx::query!("SELECT count(*) as count FROM raw_attendance_logs WHERE employee_id = $1", emp_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(logs.count, Some(1), "Log should be immediately visible in DB");
}

#[tokio::test]
async fn test_attendance_service_ingestion_employee_not_found() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let ws_id = factory.workspace().create().await;

    // Initialize service
    let (service, _worker_handle) = AttendanceLogService::new(
        pool.clone(),
        100,
        1,
        100,
    );

    let entry = AgentLogEntry {
        workspace_id: ws_id,
        employee_id_number: "NONEXISTENT".to_string(),
        punch_time_raw: "2026-01-26 09:00:00".to_string(),
        device_id: Some("TEST_DEVICE".to_string()),
        raw_data: None,
        timezone: "Asia/Ho_Chi_Minh".to_string(),
        date_fmt: "YYYY-MM-DD".to_string(),
        time_fmt: "24h".to_string(),
    };

    // Act: Ingest log with nonexistent employee
    let result = service.ingest_agent_log(entry).await;

    // Assert: Should fail immediately with employee not found error
    assert!(result.is_err(), "Ingestion should fail for nonexistent employee");
    assert!(
        result.unwrap_err().contains("Employee not found"),
        "Error should indicate employee not found"
    );

    // Assert: No log should be created
    let logs = sqlx::query!("SELECT count(*) as count FROM raw_attendance_logs WHERE workspace_id = $1", ws_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(logs.count, Some(0), "No log should be created for invalid employee");
}

#[tokio::test]
async fn test_attendance_service_ingestion_invalid_timestamp() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employee = sqlx::query!("SELECT employee_id_number FROM employees WHERE id = $1", emp_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    let (service, _worker_handle) = AttendanceLogService::new(
        pool.clone(),
        100,
        1,
        100,
    );

    let entry = AgentLogEntry {
        workspace_id: ws_id,
        employee_id_number: employee.employee_id_number,
        punch_time_raw: "INVALID_TIMESTAMP".to_string(),
        device_id: Some("TEST_DEVICE".to_string()),
        raw_data: None,
        timezone: "Asia/Ho_Chi_Minh".to_string(),
        date_fmt: "YYYY-MM-DD".to_string(),
        time_fmt: "24h".to_string(),
    };

    // Act: Ingest log with invalid timestamp
    let result = service.ingest_agent_log(entry).await;

    // Assert: Should fail immediately with parse error
    assert!(result.is_err(), "Ingestion should fail for invalid timestamp");
    assert!(
        result.unwrap_err().contains("Failed to parse timestamp"),
        "Error should indicate timestamp parse failure"
    );

    // Assert: No log should be created
    let logs = sqlx::query!("SELECT count(*) as count FROM raw_attendance_logs WHERE employee_id = $1", emp_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(logs.count, Some(0), "No log should be created for invalid timestamp");
}
