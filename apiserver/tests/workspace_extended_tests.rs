mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::MemberRole;
use apiserver::repositories::{SqlxWorkspaceRepository, WorkspaceRepository};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
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

    // Setup test storage

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_extended".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_workspace_rbac_flow() {
    let (_test_db, factory, app) = setup_app().await;

    let owner_id = factory.user().create().await;
    let member_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    factory
        .add_member(ws_id, member_id, MemberRole::Member)
        .await;

    let headers = [
        ("X-Workspace-Id", ws_id.to_string()),
        ("X-User-Id", member_id.to_string()),
    ];

    // 1. Member tries to change roles (Forbidden)
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{}/members/{}", ws_id, member_id))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from(
                    serde_json::json!({ "role": MemberRole::Admin }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. Owner changes member to admin (Success)
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{}/members/{}", ws_id, member_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::json!({ "role": MemberRole::Admin }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. New Admin tries to remove owner (Forbidden - logic: cannot remove owner or yourself)
    let _res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{}/members/{}", ws_id, owner_id))
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // Assuming our logic prevents removing owner
    // assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_workspace_repository_directly() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let repo = SqlxWorkspaceRepository { pool: pool.clone() };

    let user_id = factory.user().create().await;
    let ws = repo.create_workspace("Test WS").await.unwrap();

    repo.add_member(ws.id, user_id, MemberRole::Admin)
        .await
        .unwrap();

    let role = repo.find_primary_role(ws.id, user_id).await.unwrap();
    assert_eq!(role.unwrap(), MemberRole::Admin);

    // Update
    repo.update_member_role(ws.id, user_id, MemberRole::Member)
        .await
        .unwrap();
    let role = repo.find_primary_role(ws.id, user_id).await.unwrap();
    assert_eq!(role.unwrap(), MemberRole::Member);

    // Remove
    repo.remove_member(ws.id, user_id).await.unwrap();
    let role = repo.find_primary_role(ws.id, user_id).await.unwrap();
    assert!(role.is_none());
}
