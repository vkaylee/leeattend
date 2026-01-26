mod common;

use apiserver::services::{AttendanceLogService, AgentLogEntry};
use common::db::TestDatabase;
use common::factory::TestFactory;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_attendance_service_ingestion_flow() {
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

    // Initialize service with small batch size and fast flush for testing
    let (service, _worker_handle) = AttendanceLogService::new(
        pool.clone(),
        100, // capacity
        1,   // batch size (flush immediately)
        100, // flush interval 100ms
    );

    let entry = AgentLogEntry {
        workspace_id: ws_id,
        employee_id_number: employee.employee_id_number,
        punch_time_raw: "2026-01-26 09:00:00".to_string(),
        device_id: Some("TEST_DEVICE".to_string()),
        raw_data: None,
        timezone: "Asia/Ho_Chi_Minh".to_string(),
        date_fmt: "YYYY-MM-DD".to_string(),
        time_fmt: "24h".to_string(),
    };

    // Act: Ingest log
    service.ingest_agent_log(entry).await.expect("Ingestion failed");

    // Wait for worker to process
    sleep(Duration::from_millis(1500)).await;

    // Assert: Check if log is in DB
    let logs = sqlx::query!("SELECT count(*) as count FROM raw_attendance_logs WHERE employee_id = $1", emp_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(logs.count, Some(1));
}
