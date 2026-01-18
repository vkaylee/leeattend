mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{
    AdjustmentStatus, ApproveAttendanceAdjustmentRequest, AttendanceAdjustmentRequest,
    CreateAttendanceAdjustmentRequest, TimeEntry,
};
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("apiserver=debug,sqlx=info")
        .with_test_writer()
        .try_init();

    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
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
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_attendance_adjustment_lifecycle() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup: User, Workspace, Employee
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 2. Submit Attendance Adjustment (without existing time entry)
    let punch_in = Utc::now() - Duration::hours(8);
    let punch_out = Utc::now();

    let submit_req = CreateAttendanceAdjustmentRequest {
        employee_id: emp_id,
        time_entry_id: None,
        requested_punch_in_time: punch_in,
        requested_punch_out_time: Some(punch_out),
        is_break_overtime: false,
        reason: Some("Forgot to punch in".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/adjustments", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&submit_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let adjustment: AttendanceAdjustmentRequest = serde_json::from_slice(&body).unwrap();
    assert_eq!(adjustment.status, AdjustmentStatus::Pending);

    // 3. Get My Adjustments
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/employees/{}/attendance/adjustments", emp_id))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let adjustments: Vec<AttendanceAdjustmentRequest> = serde_json::from_slice(&body).unwrap();
    assert!(!adjustments.is_empty());
    assert_eq!(adjustments[0].id, adjustment.id);

    // 4. Approve Adjustment
    let approve_req = ApproveAttendanceAdjustmentRequest {
        status: AdjustmentStatus::Approved,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/adjustments/{}/approve",
                    ws_id, adjustment.id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&approve_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify Time Entry Creation
    let start_query = punch_in - Duration::days(1);
    let end_query = punch_out + Duration::days(1);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/time/history/{}?start={}&end={}",
                    ws_id,
                    emp_id,
                    start_query.to_rfc3339().replace("+", "%2B"),
                    end_query.to_rfc3339().replace("+", "%2B")
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let history: Vec<TimeEntry> = serde_json::from_slice(&body).unwrap();
    assert!(!history.is_empty());
    assert_eq!(history[0].entry_type, "work");
}

#[tokio::test]
async fn test_attendance_adjustment_with_existing_entry() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Create an entry
    let entry_id = factory.time_entry().with_employee(emp_id).create().await;

    // 2. Submit adjustment for this entry
    let submit_req = CreateAttendanceAdjustmentRequest {
        employee_id: emp_id,
        time_entry_id: Some(entry_id),
        requested_punch_in_time: Utc::now() - Duration::hours(2),
        requested_punch_out_time: Some(Utc::now() - Duration::hours(1)),
        is_break_overtime: true,
        reason: Some("Correction".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/adjustments", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&submit_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let adjustment: AttendanceAdjustmentRequest = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();

    // 3. Approve
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/adjustments/{}/approve",
                    ws_id, adjustment.id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&serde_json::json!({ "status": "approved" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Verify entry updated
    let entry = apiserver::repositories::time_entry_repository::get_time_entry_by_id(
        &factory.pool,
        entry_id,
    )
    .await
    .unwrap()
    .unwrap();
    assert!(entry.is_break_overtime);
    assert_eq!(entry.duration_minutes, Some(60));
}

#[tokio::test]
async fn test_get_workspace_adjustments_filter() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create one pending and one approved (manually in DB for speed)
    sqlx::query("INSERT INTO attendance_adjustment_requests (id, employee_id, requested_punch_in_time, status, created_at, updated_at) VALUES ($1, $2, NOW(), 'pending', NOW(), NOW())")
        .bind(Uuid::new_v4()).bind(emp_id).execute(&factory.pool).await.unwrap();
    sqlx::query("INSERT INTO attendance_adjustment_requests (id, employee_id, requested_punch_in_time, status, created_at, updated_at) VALUES ($1, $2, NOW(), 'rejected', NOW(), NOW())")
        .bind(Uuid::new_v4()).bind(emp_id).execute(&factory.pool).await.unwrap();

    // List pending only
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/attendance/adjustments?status=pending",
                    ws_id
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let adjustments: Vec<AttendanceAdjustmentRequest> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(adjustments.len(), 1);
    assert_eq!(adjustments[0].status, AdjustmentStatus::Pending);
}

#[tokio::test]
async fn test_reject_attendance_adjustment() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let submit_req = CreateAttendanceAdjustmentRequest {
        employee_id: emp_id,
        time_entry_id: None,
        requested_punch_in_time: Utc::now(),
        requested_punch_out_time: None,
        is_break_overtime: false,
        reason: Some("Reject me".to_string()),
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/adjustments", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&submit_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let adjustment: AttendanceAdjustmentRequest = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();

    // Reject
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/adjustments/{}/approve",
                    ws_id, adjustment.id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&serde_json::json!({ "status": "rejected" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify rejected in DB
    let fetched = apiserver::repositories::attendance_adjustment_repository::get_request_by_id(
        &factory.pool,
        adjustment.id,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(fetched.status, AdjustmentStatus::Rejected);
}

#[tokio::test]
async fn test_approve_non_existent_adjustment() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/adjustments/{}/approve",
                    ws_id,
                    Uuid::new_v4()
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&serde_json::json!({ "status": "approved" })).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
