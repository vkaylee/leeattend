mod common;

use axum::http::StatusCode;
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::models::{LeaveHistoryItem, SubmitLeaveRequest, ApproveLeaveRequest, AdjustBalanceRequest, LeaveRequest};
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
async fn test_leave_history_and_adjustment() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 2. Perform Manual Adjustment (Add +5 days)
    let adjust_req = AdjustBalanceRequest {
        leave_type: "vacation".to_string(),
        amount: 5.0,
        reason: "Initial Balance".to_string(),
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
                .body(axum::body::Body::from(serde_json::to_string(&adjust_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Submit & Approve Leave Request (Usage -2 days)
    let submit_req = SubmitLeaveRequest {
        employee_id: emp.id,
        leave_type: "vacation".to_string(),
        start_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
        reason: Some("Short trip".to_string()),
    };

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/submit", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&submit_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let leave_req: LeaveRequest = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 10000).await.unwrap()).unwrap();

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/requests/{}/approve", ws_id, leave_req.id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&ApproveLeaveRequest { status: "approved".to_string() }).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Fetch History
    let response = app.clone()
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
    
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let history: Vec<LeaveHistoryItem> = serde_json::from_slice(&body).unwrap();

    // Verify
    assert_eq!(history.len(), 2);
    
    // Sort logic: usage (Feb) should be first, adjustment (Jan) second
    assert_eq!(history[0].transaction_type, "usage");
    assert_eq!(history[0].amount, -2.0); // 2 days
    assert_eq!(history[0].reference_id, Some(leave_req.id));

    assert_eq!(history[1].transaction_type, "adjustment");
    assert_eq!(history[1].amount, 5.0);
    assert_eq!(history[1].reason, Some("Initial Balance".to_string()));
}
