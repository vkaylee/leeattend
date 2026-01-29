mod common;

use apiserver::models::{OtStatus, AttendanceDailySummary, BulkOtUploadResult};
use apiserver::{agent_manager::AgentManager, services::AttendanceLogService, AppState};
use chrono::NaiveDate;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_ot_claim_approval_workflow() {
    let db = common::db::TestDatabase::new().await;
    let pool = &db.pool();
    let factory = common::factory::TestFactory::new(pool.clone());

    // 1. Setup: Create Workspace, Employee
    let ws_id = factory.workspace().create().await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("John Doe")
        .with_employee_id_number("E001")
        .create()
        .await;

    let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();

    // 2. Scenario 1: System Detects OT
    // Simulate AttendanceCalculator inserting a summary with OT
    sqlx::query!(
        r#"
        INSERT INTO attendance_daily_summaries (workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status)
        VALUES ($1, $2, $3, 480, 120, 'pending_employee')
        "#,
        ws_id, employee_id, date
    ).execute(pool).await.unwrap();

    // Verify initial state
    let summary = sqlx::query_as!(
        AttendanceDailySummary,
        "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        employee_id, date
    ).fetch_one(pool).await.unwrap();

    assert_eq!(summary.system_detected_ot_minutes, 120);
    assert_eq!(summary.ot_status, OtStatus::PendingEmployee);

    // 3. Scenario 2: Employee Claims OT
    // Call service directly (Integration Test style)
    apiserver::services::attendance_summary_service::AttendanceSummaryService::claim_ot(
        pool,
        employee_id,
        date,
        120, // Claim full amount
        Some("Urgent fix".to_string())
    ).await.unwrap();

    let summary_claimed = sqlx::query_as!(
        AttendanceDailySummary,
        "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        employee_id, date
    ).fetch_one(pool).await.unwrap();

    assert_eq!(summary_claimed.claimed_ot_minutes, 120);
    assert_eq!(summary_claimed.ot_status, OtStatus::PendingManager);
    assert_eq!(summary_claimed.ot_reason, Some("Urgent fix".to_string()));

    // 4. Scenario 3: Manager Approves OT
    apiserver::services::attendance_summary_service::AttendanceSummaryService::approve_ot(
        pool,
        employee_id,
        date,
        90, // Approve partial
        Some("Good job but strict budget".to_string()),
        OtStatus::Approved
    ).await.unwrap();

    let summary_approved = sqlx::query_as!(
        AttendanceDailySummary,
        "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        employee_id, date
    ).fetch_one(pool).await.unwrap();

    assert_eq!(summary_approved.approved_ot_minutes, 90);
    assert_eq!(summary_approved.ot_status, OtStatus::Approved);
    assert_eq!(summary_approved.manager_note, Some("Good job but strict budget".to_string()));
}

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_bulk_csv_ot_upload() {
    let (_test_db, factory, app) = setup_app().await;
    let pool = &_test_db.pool();

    // Setup
    let admin_user = factory.user().with_email("admin@test.com").create().await;
    let ws_id = factory.workspace().create_with_owner(admin_user).await;

    // Create 3 employees
    let emp1_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice")
        .with_employee_id_number("E001")
        .create()
        .await;

    let emp2_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob")
        .with_employee_id_number("E002")
        .create()
        .await;

    let _emp3_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Charlie")
        .with_employee_id_number("E003")
        .create()
        .await;

    // Pre-insert summaries with system_detected_ot
    let date1 = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let date2 = NaiveDate::from_ymd_opt(2024, 1, 16).unwrap();

    sqlx::query!(
        "INSERT INTO attendance_daily_summaries (workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status) VALUES ($1, $2, $3, 480, 120, 'pending_employee')",
        ws_id, emp1_id, date1
    ).execute(pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO attendance_daily_summaries (workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, ot_status) VALUES ($1, $2, $3, 480, 90, 'pending_employee')",
        ws_id, emp2_id, date2
    ).execute(pool).await.unwrap();

    // Prepare CSV
    let csv_content = format!(
        "employee_id,date,minutes,reason
E001,2024-01-15,120,Production line overtime - paper form #123
E002,2024-01-16,90,Urgent order completion
E003,2024-01-17,60,Maintenance work"
    );

    // Create multipart form data
    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"ot_claims.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--{}--\r\n",
        boundary, csv_content, boundary
    );

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{}/attendance/bulk-upload-paper-ot", ws_id))
        .header("Content-Type", format!("multipart/form-data; boundary={}", boundary))
        .header("X-User-Id", admin_user.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .body(axum::body::Body::from(body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: BulkOtUploadResult = serde_json::from_slice(&body_bytes).unwrap();

    // Assertions
    assert_eq!(result.total_rows, 3);
    assert_eq!(result.success_count, 3); // All should succeed because validation passes
    assert_eq!(result.failed_count, 0); // No failures

    // Verify E001
    let summary1 = sqlx::query_as!(
        AttendanceDailySummary,
        "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        emp1_id, date1
    ).fetch_one(pool).await.unwrap();

    assert_eq!(summary1.claimed_ot_minutes, 120);
    assert_eq!(summary1.ot_status, OtStatus::PendingManager);
    assert_eq!(summary1.entered_via, Some("paper_form".to_string()));

    // Verify E002
    let summary2 = sqlx::query_as!(
        AttendanceDailySummary,
        "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, created_at, updated_at FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        emp2_id, date2
    ).fetch_one(pool).await.unwrap();

    assert_eq!(summary2.claimed_ot_minutes, 90);
    assert_eq!(summary2.ot_status, OtStatus::PendingManager);

    // Verify error details
    assert!(result.errors.is_empty(), "Expected no errors but got: {:?}", result.errors);
}

#[tokio::test]
async fn test_bulk_csv_upload_validation_errors() {
    let (_test_db, factory, app) = setup_app().await;
    let _pool = &_test_db.pool();

    let admin_user = factory.user().with_email("admin@test.com").create().await;
    let ws_id = factory.workspace().create_with_owner(admin_user).await;

    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("E001")
        .create()
        .await;

    // CSV with various errors
    let csv_content = "employee_id,date,minutes,reason
,2024-01-15,120,Missing employee ID
E001,invalid-date,120,Bad date format
E001,2024-01-15,-50,Negative minutes
E001,2024-01-15,120,
E999,2024-01-15,120,Non-existent employee
E001,2099-01-15,120,Future date";

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"errors.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--{}--\r\n",
        boundary, csv_content, boundary
    );

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{}/attendance/bulk-upload-paper-ot", ws_id))
        .header("Content-Type", format!("multipart/form-data; boundary={}", boundary))
        .header("X-User-Id", admin_user.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .body(axum::body::Body::from(body))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: BulkOtUploadResult = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(result.total_rows, 6);
    assert_eq!(result.success_count, 0);
    assert_eq!(result.failed_count, 6);

    // Verify error messages
    assert!(result.errors.iter().any(|e| e.error_message.contains("Employee ID is required")));
    assert!(result.errors.iter().any(|e| e.error_message.contains("Invalid date format")));
    assert!(result.errors.iter().any(|e| e.error_message.contains("must be greater than 0")));
    assert!(result.errors.iter().any(|e| e.error_message.contains("Reason is required")));
    assert!(result.errors.iter().any(|e| e.error_message.contains("not found")));
    assert!(result.errors.iter().any(|e| e.error_message.contains("cannot be in the future")));
}

