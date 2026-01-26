use apiserver::services::AttendanceLogService;
use apiserver::{
    agent_manager::AgentManager,
    create_app,
    models::Shift,
    repositories::{SqlxAgentRepository, SqlxWorkspaceRepository},
    services::{auth_service::AuthService, AgentService},
    AppState,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::NaiveDate;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

mod common;
use common::factory::TestFactory;

async fn setup_test_db() -> (PgPool, common::db::TestDatabase) {
    let test_db = common::db::TestDatabase::new().await;
    (test_db.pool(), test_db)
}

async fn setup_test_app(pool: PgPool) -> axum::Router {
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo = Arc::new(SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(AgentService::new(agent_repo.clone(), agent_manager.clone()));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
    });

    create_app(state)
}

#[tokio::test]
async fn test_shift_versioning_update() {
    let (pool, _tdb) = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    // 1. Setup Data
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Create Shift and Assign
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .assign_to_employee(employee_id, start_date)
        .await;

    // Login to get token
    // Login to get token
    let token = AuthService::generate_token(user_id, "test_secret", 3600).unwrap();

    // 2. Perform Versioning Update (New Version)
    let effective_date = NaiveDate::from_ymd_opt(2025, 2, 1).unwrap();
    let new_name = "Shift V2";

    let update_payload = json!({
        "name": new_name,
        "start_time": "10:00:00",
        "end_time": "18:00:00",
        "break_duration_minutes": 60,
        "break_is_overtime": false,
        "is_break_auto_deduct": true,
        "early_punch_in_grace_period_minutes": 15,
        "is_correction": false,
        "effective_date": effective_date.to_string(), // "2025-02-01"
        "days_of_week": [1, 2, 3, 4, 5]
    });

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/workspaces/{}/shifts/{}", ws_id, shift_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    if status != StatusCode::OK {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        println!("Response body: {:?}", String::from_utf8_lossy(&bytes));
    }
    assert_eq!(status, StatusCode::OK);

    // 3. Verify Database State

    // A. Old Shift should be inactive
    let old_shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = $1")
        .bind(shift_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!old_shift.is_active);

    // B. New Shift should exist and be active
    // If logic isn't strictly parent=old_id, let's just search by name
    let active_shifts = sqlx::query_as::<_, Shift>(
        "SELECT * FROM shifts WHERE workspace_id = $1 AND is_active = true",
    )
    .bind(ws_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(active_shifts.len(), 1);
    let new_shift_record = &active_shifts[0];
    assert_eq!(new_shift_record.name, new_name);
    assert_ne!(new_shift_record.id, shift_id);
    assert_eq!(new_shift_record.parent_shift_id, Some(shift_id));

    // C. Verify Assignments
    // Old assignment should end on effective_date - 1 day
    let old_assignment = sqlx::query!(
        "SELECT end_date FROM employee_shifts WHERE shift_id = $1 AND employee_id = $2",
        shift_id,
        employee_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let expected_end_date = effective_date.pred_opt().unwrap();
    assert_eq!(old_assignment.end_date, Some(expected_end_date));

    // New assignment should start on effective_date
    let new_assignment = sqlx::query!(
        "SELECT start_date, end_date FROM employee_shifts WHERE shift_id = $1 AND employee_id = $2",
        new_shift_record.id,
        employee_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(new_assignment.start_date, effective_date);
    assert_eq!(new_assignment.end_date, None);

    // factory.cleanup().await;
}

#[tokio::test]
async fn test_shift_correction_update() {
    let (pool, _tdb) = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    // 1. Setup Data
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Create Shift and Assign
    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Wrong Name")
        .assign_to_employee(employee_id, start_date)
        .await;

    // Login to get token
    let token = AuthService::generate_token(user_id, "test_secret", 3600).unwrap();

    // 2. Perform Correction Update
    let corrected_name = "Correct Name";
    let update_payload = json!({
        "name": corrected_name,
        "start_time": "09:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "break_is_overtime": false,
        "is_break_auto_deduct": true,
        "early_punch_in_grace_period_minutes": 0,
        "is_correction": true,
        // effective_date ignored for correction
        "days_of_week": [1, 2, 3, 4, 5]
    });

    let req = Request::builder()
        .method("PUT")
        .uri(format!("/api/workspaces/{}/shifts/{}", ws_id, shift_id))
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&update_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    if status != StatusCode::OK {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        println!("Response body: {:?}", String::from_utf8_lossy(&bytes));
    }
    assert_eq!(status, StatusCode::OK);

    // 3. Verify Database State
    // ID should be same, name updated, is_active true
    let shift = sqlx::query_as::<_, Shift>("SELECT * FROM shifts WHERE id = $1")
        .bind(shift_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(shift.name, corrected_name);
    assert!(shift.is_active);

    // Assignment should be unchanged
    let assignment = sqlx::query!(
        "SELECT shift_id, start_date, end_date FROM employee_shifts WHERE employee_id = $1",
        employee_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(assignment.shift_id, shift_id);
    assert_eq!(assignment.start_date, start_date);
    assert_eq!(assignment.end_date, None);

    // factory.cleanup().await;
}
