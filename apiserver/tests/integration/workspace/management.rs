//! Workspace management and RBAC integration tests

use crate::common::app::{spawn_app_with_factory, setup_app_tower};
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::MemberRole;
use apiserver::repositories::{SqlxWorkspaceRepository, WorkspaceRepository};
use apiserver::AppState;
use axum::http::StatusCode;
use reqwest;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_workspace_creation_and_membership() {
    let (address, _, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Create new workspace
    let response = client
        .post(format!("{address}/api/workspaces"))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "name": "New Ventures" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    let ws_id = workspace["id"].as_str().unwrap();

    // 2. Check member list
    let response = client
        .get(format!("{address}/api/workspaces/{ws_id}/members"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let members: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(members
        .iter()
        .any(|m| m["user_id"] == user_id.to_string() && m["role"] == "owner"));
}

#[tokio::test]
async fn test_create_workspace_with_non_existent_user() {
    let (address, _, _, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let fake_user_id = Uuid::new_v4();

    let response = client
        .post(format!("{address}/api/workspaces"))
        .header("X-User-Id", fake_user_id.to_string())
        .json(&serde_json::json!({ "name": "Ghost Workspace" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_workspace_rbac_flow() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let member_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;
    factory
        .add_member(ws_id, member_id, MemberRole::Member)
        .await;

    // 1. Member tries to change roles (Forbidden)
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{ws_id}/members/{member_id}"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", member_id.to_string())
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
                .uri(format!("/api/workspaces/{ws_id}/members/{member_id}"))
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
}

#[tokio::test]
async fn test_workspace_repository_directly() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let repo = SqlxWorkspaceRepository { pool: pool.clone() };

    let user_id: Uuid = factory.user().create().await;
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
