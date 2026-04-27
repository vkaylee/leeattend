//! Workspace Roles API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;

use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => axum::body::Body::from(b),
            None => axum::body::Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_system_roles_are_seeded_with_permissions() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Get workspace roles to find the 'owner' role
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/roles"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    
    // Find owner role
    let owner_role = roles.as_array().unwrap().iter().find(|r| r["name"] == "owner").expect("Owner role must exist");
    assert_eq!(owner_role["is_system"], true);
    
    let role_id = owner_role["id"].as_str().unwrap();

    // 2. Get permissions for the owner role
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/roles/{role_id}/permissions"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let permissions = body_json(resp).await;
    let perms_array = permissions.as_array().unwrap();
    
    // 3. Verify it has ess:profile and feature_flags:read (added in the latest migration)
    let perm_names: Vec<&str> = perms_array.iter().map(|p| p.as_str().unwrap()).collect();
    
    assert!(perm_names.contains(&"ess:profile"), "Owner should have ess:profile");
    assert!(perm_names.contains(&"feature_flags:read"), "Owner should have feature_flags:read");
    assert!(perm_names.len() >= 67, "Owner should have all core permissions");
}

#[tokio::test]
async fn test_extra_role_assign_and_revoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create member
    let member_id = factory.user().with_email("staff@example.com").create().await;
    factory.add_member(ws_id, member_id, Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    // 1. Get member roles (should have primary only)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    assert!(roles["primary"].as_str().is_some());

    // 2. Assign extra role (TeamLead, expires in 1 hour)
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    let assign_body = serde_json::json!({
        "role_id": "a0000000-0000-0000-0000-000000000005",
        "expires_at": expires_at.to_rfc3339()
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let extra_role = body_json(resp).await;
    let role_id = extra_role["id"].as_str().unwrap();

    // 3. Verify roles now include extra
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    assert!(!roles["extra"].as_array().unwrap().is_empty());

    // 4. Revoke extra role
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles/{role_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_assign_role_expired_returns_validation_error() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory.user().with_email("staff2@example.com").create().await;
    factory.add_member(ws_id, member_id, Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    // Try to assign with expired time
    let expired = chrono::Utc::now() - chrono::Duration::hours(1);
    let body = serde_json::json!({
        "role_id": "a0000000-0000-0000-0000-000000000003",
        "expires_at": expired.to_rfc3339()
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            admin_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    // Should return validation error (400 or 422)
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_assign_role_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory.user().with_email("member@example.com").create().await;
    factory.add_member(ws_id, member_id, Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    let target_id = factory.user().with_email("target@example.com").create().await;
    factory.add_member(ws_id, target_id, Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap()).await;

    let body = serde_json::json!({
        "role_id": "a0000000-0000-0000-0000-000000000003",
        "expires_at": (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/members/{target_id}/roles"),
            ws_id,
            member_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_custom_role_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create a custom role
    let create_body = serde_json::json!({
        "name": "Finance Manager",
        "description": "Can manage payroll",
        "color": "emerald"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/roles"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let role = body_json(resp).await;
    assert_eq!(role["name"], "Finance Manager");
    assert_eq!(role["color"], "emerald");
    assert_eq!(role["is_system"], false);

    let role_id = role["id"].as_str().unwrap();

    // 2. Update the custom role
    let update_body = serde_json::json!({
        "name": "Senior Finance Manager",
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/workspaces/{ws_id}/roles/{role_id}"),
            ws_id,
            admin_id,
            Some(update_body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let updated_role = body_json(resp).await;
    assert_eq!(updated_role["name"], "Senior Finance Manager");

    // 3. Delete the custom role
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/roles/{role_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_custom_role_permissions() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create a custom role
    let create_body = serde_json::json!({
        "name": "Payroll Manager",
        "description": "Can run payroll",
        "color": "cyan"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/roles"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    
    let role = body_json(resp).await;
    let role_id = role["id"].as_str().unwrap();

    // 1. Initially it has no permissions
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/roles/{role_id}/permissions"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let perms = body_json(resp).await;
    assert!(perms.as_array().unwrap().is_empty());

    // 2. Update permissions
    let update_body = serde_json::json!({
        "permissions": ["payroll:manage", "reports:read"]
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/workspaces/{ws_id}/roles/{role_id}/permissions"),
            ws_id,
            admin_id,
            Some(update_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Verify permissions were updated
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/roles/{role_id}/permissions"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    let perms = body_json(resp).await;
    let perms_array = perms.as_array().unwrap();
    assert_eq!(perms_array.len(), 2);
    
    let perm_names: Vec<&str> = perms_array.iter().map(|p| p.as_str().unwrap()).collect();
    assert!(perm_names.contains(&"payroll:manage"));
    assert!(perm_names.contains(&"reports:read"));
}
