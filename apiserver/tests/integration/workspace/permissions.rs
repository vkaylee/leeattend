//! Integration tests for Extra Permissions CRUD endpoints

use crate::common::app::spawn_app_with_member;
use crate::common::db::TestDatabase;

#[tokio::test]
async fn test_grant_and_list_extra_permission() {
    let (address, ws_id, owner_id, member_id, _factory, _db): (
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
        _,
        TestDatabase,
    ) = spawn_app_with_member().await;
    let client = reqwest::Client::new();

    // 1. Grant a permission to the member
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions"
        ))
        .header("X-User-Id", owner_id.to_string())
        .json(&serde_json::json!({
            "permission": "employees:create",
            "expires_at": "2099-12-31T23:59:59Z"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::CREATED);
    let perm: serde_json::Value = res.json().await.unwrap();
    assert_eq!(perm["permission"], "employees:create");
    assert_eq!(perm["user_id"], member_id.to_string());
    let perm_id = perm["id"].as_str().unwrap().to_string();

    // 2. List the member's extra permissions
    let res = client
        .get(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions"
        ))
        .header("X-User-Id", owner_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let perms: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(perms.len(), 1);
    assert_eq!(perms[0]["id"], perm_id);

    // 3. Revoke the permission
    let res = client
        .delete(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions/{perm_id}"
        ))
        .header("X-User-Id", owner_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::NO_CONTENT);

    // 4. Verify it's gone
    let res = client
        .get(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions"
        ))
        .header("X-User-Id", owner_id.to_string())
        .send()
        .await
        .unwrap();

    let perms: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(perms.is_empty(), "Permissions should be empty after revoke");
}

#[tokio::test]
async fn test_grant_invalid_permission_returns_validation_error() {
    let (address, ws_id, owner_id, member_id, _factory, _db): (
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
        _,
        TestDatabase,
    ) = spawn_app_with_member().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions"
        ))
        .header("X-User-Id", owner_id.to_string())
        .json(&serde_json::json!({
            "permission": "nonexistent:permission",
            "expires_at": "2099-12-31T23:59:59Z"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_grant_expired_permission_returns_validation_error() {
    let (address, ws_id, owner_id, member_id, _factory, _db): (
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
        _,
        TestDatabase,
    ) = spawn_app_with_member().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions"
        ))
        .header("X-User-Id", owner_id.to_string())
        .json(&serde_json::json!({
            "permission": "employees:create",
            "expires_at": "2020-01-01T00:00:00Z"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_member_cannot_grant_permissions() {
    let (address, ws_id, _owner_id, member_id, _factory, _db): (
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
        _,
        TestDatabase,
    ) = spawn_app_with_member().await;
    let client = reqwest::Client::new();

    // Member tries to grant themselves a permission (Forbidden)
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions"
        ))
        .header("X-User-Id", member_id.to_string())
        .json(&serde_json::json!({
            "permission": "employees:create",
            "expires_at": "2099-12-31T23:59:59Z"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_available_permissions() {
    let (address, ws_id, owner_id, _member_id, _factory, _db): (
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
        _,
        TestDatabase,
    ) = spawn_app_with_member().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
            "{address}/api/v1/workspaces/{ws_id}/permissions/available"
        ))
        .header("X-User-Id", owner_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let perms: Vec<String> = res.json().await.unwrap();
    assert!(perms.len() >= 60, "Should have 60+ permissions");
    assert!(perms.contains(&"employees:create".to_string()));
    assert!(perms.contains(&"members:roles:manage".to_string()));
}

#[tokio::test]
async fn test_revoke_nonexistent_permission_returns_not_found() {
    let (address, ws_id, owner_id, member_id, _factory, _db): (
        String,
        uuid::Uuid,
        uuid::Uuid,
        uuid::Uuid,
        _,
        TestDatabase,
    ) = spawn_app_with_member().await;
    let client = reqwest::Client::new();

    let fake_id = uuid::Uuid::new_v4();
    let res = client
        .delete(format!(
            "{address}/api/v1/workspaces/{ws_id}/members/{member_id}/permissions/{fake_id}"
        ))
        .header("X-User-Id", owner_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::NOT_FOUND);
}
