use apiserver::services::AttendanceLogService;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

mod common;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

async fn setup_test_app(pool: PgPool) -> axum::Router {
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
    });

    create_app(state)
}

#[tokio::test]
async fn test_unique_shift_assignment_constraint() {
    let (test_db, factory) = setup_test_db().await;
    let pool = test_db.pool();
    let app = setup_test_app(pool.clone()).await;

    // 1. Setup
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let shift_1_id = factory.shift().with_workspace(workspace_id).create().await;
    let shift_2_id = factory.shift().with_workspace(workspace_id).create().await;

    // 2. Assign Shift 1 - Should Succeed
    let assign_req_1 = json!({
        "employee_id": employee_id,
        "shift_id": shift_1_id,
        "start_date": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts/assign", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_vec(&assign_req_1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 3. Assign Shift 2 on SAME DATE - Should Fail (Conflict)
    let assign_req_2 = json!({
        "employee_id": employee_id,
        "shift_id": shift_2_id,
        "start_date": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts/assign", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_vec(&assign_req_2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    // 4. Assign Shift 2 on DIFFERENT DATE - Should Succeed
    let assign_req_3 = json!({
        "employee_id": employee_id,
        "shift_id": shift_2_id,
        "start_date": "2026-01-02"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts/assign", workspace_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", workspace_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_vec(&assign_req_3).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
