mod common;

use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use apiserver::repositories::WorkspaceRepository;
use std::sync::Arc;
use tower::ServiceExt;
use chrono::{Utc, Duration};
use uuid::Uuid;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
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
        jwt_secret: "test_secret_ws".to_string(),
    });
    
    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_member_management_lifecycle() {
    let (_test_db, factory, app) = setup_app().await;

    let owner_id = factory.user().create().await;
    let member_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    factory.add_member(ws_id, member_id, "member").await;

    // 1. Update member role
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{}/members/{}", ws_id, member_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(serde_json::json!({ "role": "admin" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Assign extra role
    let expires_at = Utc::now() + Duration::hours(24);
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/members/{}/roles", ws_id, member_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(serde_json::json!({ "role": "manager", "expires_at": expires_at }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let extra_role: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 10000).await.unwrap()).unwrap();
    let role_id = extra_role["id"].as_str().unwrap();

    // 3. Get member roles
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/members/{}/roles", ws_id, member_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Revoke extra role
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{}/members/{}/roles/{}", ws_id, member_id, role_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. Remove member
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{}/members/{}", ws_id, member_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_remove_self_forbidden() {
    let (_test_db, factory, app) = setup_app().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{}/members/{}", ws_id, owner_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_user_profile_update() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri("/api/me")
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::json!({
                    "full_name": "New Name",
                    "bio": "New Bio"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_and_update_workspace() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Get workspace
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}", ws_id))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Update workspace
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{}", ws_id))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::json!({
                    "name": "Renamed Workspace",
                    "work_unit_minutes": 15
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated: serde_json::Value = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 10000).await.unwrap()).unwrap();
    assert_eq!(updated["name"], "Renamed Workspace");
    assert_eq!(updated["work_unit_minutes"], 15);
}

#[tokio::test]
async fn test_get_users_list() {
    let (_test_db, _factory, app) = setup_app().await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/users")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_workspace_repo_member_ops() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let repo = apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() };
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws = repo.create_workspace("Repo Test").await.unwrap();

    // 1. Add member
    repo.add_member(ws.id, user_id, "admin").await.unwrap();

    // 2. Find primary role
    let role = repo.find_primary_role(ws.id, user_id).await.unwrap();
    assert_eq!(role.unwrap(), "admin");

    // 3. Update role
    repo.update_member_role(ws.id, user_id, "member").await.unwrap();
    let role = repo.find_primary_role(ws.id, user_id).await.unwrap();
    assert_eq!(role.unwrap(), "member");

    // 4. Remove member
    repo.remove_member(ws.id, user_id).await.unwrap();
    let role = repo.find_primary_role(ws.id, user_id).await.unwrap();
    assert!(role.is_none());
}

#[tokio::test]
async fn test_workspace_not_found() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}", Uuid::new_v4()))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_workspace_no_fields() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{}", ws_id))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from("{}")) // No fields
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR); // Based on current implementation returning Err
}



