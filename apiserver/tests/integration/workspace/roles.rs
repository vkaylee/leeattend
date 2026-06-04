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
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;

    // Find owner role
    let owner_role = roles["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "owner")
        .expect("Owner role must exist");
    assert_eq!(owner_role["is_system"], true);

    let role_id = owner_role["id"].as_str().unwrap();

    // 2. Get permissions for the owner role
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}/permissions"),
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

    assert!(
        perm_names.contains(&"ess:profile"),
        "Owner should have ess:profile"
    );
    assert!(
        perm_names.contains(&"feature_flags:read"),
        "Owner should have feature_flags:read"
    );
    assert!(
        perm_names.len() >= 67,
        "Owner should have all core permissions"
    );
}

#[tokio::test]
async fn test_extra_role_assign_and_revoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create member
    let member_id = factory
        .user()
        .with_email("staff@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // 1. Get member roles (should have primary only)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles"),
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
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles"),
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
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles"),
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
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles/{role_id}"),
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

    let member_id = factory
        .user()
        .with_email("staff2@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

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
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles"),
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

    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let target_id = factory
        .user()
        .with_email("target@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            target_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let body = serde_json::json!({
        "role_id": "a0000000-0000-0000-0000-000000000003",
        "expires_at": (chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339()
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/members/{target_id}/roles"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}/permissions"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}/permissions"),
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
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}/permissions"),
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

// ─── RBAC Cleanup Tests ─────────────────────────────────────────────────────
// Verify that get_workspace_roles is accessible without MembersRolesManage
// while write operations remain gated.

#[tokio::test]
async fn test_regular_member_can_list_workspace_roles() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Create a regular employee member (has NO members_roles:manage permission)
    let employee_id = factory
        .user()
        .with_email("employee@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            employee_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(), // employee role
        )
        .await;

    // Employee should be able to list workspace roles (no permission gate)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            employee_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Regular member must be able to list workspace roles"
    );

    let roles = body_json(resp).await;
    let roles_array = roles["data"].as_array().unwrap();

    // Should include all system roles
    assert!(
        roles_array.len() >= 6,
        "Should have at least 6 system roles, got {}",
        roles_array.len()
    );

    // Verify expected system roles exist
    let role_names: Vec<&str> = roles_array
        .iter()
        .map(|r| r["name"].as_str().unwrap())
        .collect();
    assert!(role_names.contains(&"owner"), "Must include owner role");
    assert!(role_names.contains(&"admin"), "Must include admin role");
    assert!(
        role_names.contains(&"employee"),
        "Must include employee role"
    );
}

#[tokio::test]
async fn test_workspace_roles_response_includes_scope_type() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    let roles_array = roles["data"].as_array().unwrap();

    for role in roles_array {
        let scope = role["scope_type"].as_str();
        assert!(
            scope.is_some(),
            "Every role must have scope_type field: {:?}",
            role["name"]
        );
        assert!(
            scope.unwrap() == "global" || scope.unwrap() == "scoped",
            "scope_type must be 'global' or 'scoped', got '{}'",
            scope.unwrap()
        );
    }

    // Owner and admin should be global
    let owner_role = roles_array.iter().find(|r| r["name"] == "owner").unwrap();
    assert_eq!(owner_role["scope_type"], "global");

    let admin_role = roles_array.iter().find(|r| r["name"] == "admin").unwrap();
    assert_eq!(admin_role["scope_type"], "global");

    // Employee should be scoped
    let employee_role = roles_array
        .iter()
        .find(|r| r["name"] == "employee")
        .unwrap();
    assert_eq!(employee_role["scope_type"], "scoped");
}

#[tokio::test]
async fn test_regular_member_cannot_create_role() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let member_id = factory
        .user()
        .with_email("basic@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let create_body = serde_json::json!({
        "name": "Hacker Role",
        "description": "Should be denied",
        "color": "red"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            member_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Regular member must not create roles"
    );
}

#[tokio::test]
async fn test_regular_member_cannot_update_or_delete_role() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Owner creates a custom role first
    let create_body = serde_json::json!({
        "name": "Target Role",
        "description": "For testing",
        "color": "cyan"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            owner_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let role = body_json(resp).await;
    let role_id = role["id"].as_str().unwrap();

    // Add employee member
    let member_id = factory
        .user()
        .with_email("sneaky@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // Member tries to UPDATE the role → Forbidden
    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}"),
            ws_id,
            member_id,
            Some(serde_json::json!({ "name": "Pwned Role" }).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Regular member must not update roles"
    );

    // Member tries to DELETE the role → Forbidden
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/roles/{role_id}"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Regular member must not delete roles"
    );
}

#[tokio::test]
async fn test_cannot_delete_system_role() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Get the system owner role ID
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();
    let roles = body_json(resp).await;
    let owner_role = roles["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["name"] == "owner" && r["is_system"] == true)
        .expect("Owner system role must exist");
    let system_role_id = owner_role["id"].as_str().unwrap();

    // Even the owner cannot delete a system role
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/roles/{system_role_id}"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    // Should return NotFound because the DELETE query has `is_system = false` guard
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "System roles must not be deletable"
    );
}

#[tokio::test]
async fn test_cannot_update_system_role() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Use the well-known admin system role UUID
    let admin_role_id = "a0000000-0000-0000-0000-000000000002";

    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/v1/workspaces/{ws_id}/roles/{admin_role_id}"),
            ws_id,
            owner_id,
            Some(serde_json::json!({ "name": "Renamed Admin" }).to_string()),
        ))
        .await
        .unwrap();

    // Should return NotFound because the UPDATE query has `is_system = false` guard
    assert_eq!(
        resp.status(),
        StatusCode::NOT_FOUND,
        "System roles must not be updatable"
    );
}

#[tokio::test]
async fn test_update_primary_role_via_patch_members() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let member_id = factory
        .user()
        .with_email("promote@example.com")
        .create()
        .await;
    let employee_role = Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    let admin_role = Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap();
    factory.add_member(ws_id, member_id, employee_role).await;

    // 1. Verify initial role is employee
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    assert_eq!(roles["primary"], employee_role.to_string());

    // 2. Owner promotes member to admin via PATCH /members/:id
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}"),
            ws_id,
            owner_id,
            Some(serde_json::json!({ "role_id": admin_role }).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Owner should be able to promote member"
    );

    // 3. Verify role was updated
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/members/{member_id}/roles"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let roles = body_json(resp).await;
    assert_eq!(
        roles["primary"],
        admin_role.to_string(),
        "Primary role should be updated to admin"
    );
}

#[tokio::test]
async fn test_custom_role_with_scope_type() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Create a global-scope custom role
    let create_body = serde_json::json!({
        "name": "VP Engineering",
        "description": "Vice President of Engineering",
        "color": "indigo",
        "scope_type": "global"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            owner_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let role = body_json(resp).await;
    assert_eq!(role["name"], "VP Engineering");
    assert_eq!(
        role["scope_type"], "global",
        "Custom role should respect scope_type parameter"
    );
    assert_eq!(role["is_system"], false);

    // Create a scoped custom role (default)
    let create_body = serde_json::json!({
        "name": "Shift Supervisor",
        "color": "teal"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/roles"),
            ws_id,
            owner_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let role = body_json(resp).await;
    assert_eq!(
        role["scope_type"], "scoped",
        "Default scope_type should be 'scoped'"
    );
}
