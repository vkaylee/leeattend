mod common;

use axum::http::StatusCode;
use chrono::{Utc, Duration};
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::models::{AttendanceAdjustmentRequest, CreateAttendanceAdjustmentRequest, ApproveAttendanceAdjustmentRequest, TimeEntry};
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("apiserver=debug,sqlx=info")
        .with_test_writer()
        .try_init();

    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());
    
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(agent_repo.clone(), agent_manager.clone()));
    
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

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/adjustments", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&submit_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let adjustment: AttendanceAdjustmentRequest = serde_json::from_slice(&body).unwrap();
    assert_eq!(adjustment.status, "pending");

    // 3. Get My Adjustments
    let response = app.clone()
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let adjustments: Vec<AttendanceAdjustmentRequest> = serde_json::from_slice(&body).unwrap();
    assert!(!adjustments.is_empty());
    assert_eq!(adjustments[0].id, adjustment.id);

    // 4. Approve Adjustment
    let approve_req = ApproveAttendanceAdjustmentRequest {
        status: "approved".to_string(),
    };

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/adjustments/{}/approve", ws_id, adjustment.id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&approve_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify Time Entry Creation
    let start_query = punch_in - Duration::days(1);
    let end_query = punch_out + Duration::days(1);
    
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{}/time/history/{}?start={}&end={}", 
                    ws_id, emp_id, 
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
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let history: Vec<TimeEntry> = serde_json::from_slice(&body).unwrap();
    assert!(!history.is_empty());
    assert_eq!(history[0].entry_type, "work");
}