mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{
    AdjustBalanceRequest, LeaveHistoryItem,
};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
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
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_leave_history_filtering_by_type() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let vacation_id: uuid::Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        ws_id
    )
    .fetch_one(&_test_db.pool())
    .await
    .unwrap();

    let sick_id: uuid::Uuid = sqlx::query_scalar!(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'sick'",
        ws_id
    )
    .fetch_one(&_test_db.pool())
    .await
    .unwrap();

    // 2. Adjust Vacation (+5)
    let adjust_vax = AdjustBalanceRequest {
        leave_type_id: vacation_id,
        amount: 5.0,
        reason: "Vacation Credit".to_string(),
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    };
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees/{}/leave/adjust", ws_id, emp_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&adjust_vax).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Adjust Sick (+3)
    let adjust_sick = AdjustBalanceRequest {
        leave_type_id: sick_id,
        amount: 3.0,
        reason: "Sick Credit".to_string(),
        transaction_date: NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
    };
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees/{}/leave/adjust", ws_id, emp_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&adjust_sick).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Fetch All History
    let resp = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees/{}/leave/history", ws_id, emp_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    let history: Vec<LeaveHistoryItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(history.len(), 2);

    // 5. Fetch Filtered (Vacation Only)
    let resp = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees/{}/leave/history?leave_type_id={}", ws_id, emp_id, vacation_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    let history: Vec<LeaveHistoryItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].reason, Some("Vacation Credit".to_string()));

    // 6. Fetch Filtered (Sick Only)
    let resp = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees/{}/leave/history?leave_type_id={}", ws_id, emp_id, sick_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    let history: Vec<LeaveHistoryItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].reason, Some("Sick Credit".to_string()));
}
