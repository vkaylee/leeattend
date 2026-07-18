//! Integration tests for Condition Type Catalog API

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_condition_type_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory
        .user()
        .with_email("condtype-owner@test.local")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. List — should have seeded system types
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/condition-types"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let types: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        types.is_empty(),
        "Catalog should start empty — no system types are pre-seeded"
    );
    // No pre-seeded types — workspace starts with an empty catalog.

    // 2. Create a custom type
    let create_payload = json!({
        "code": "REMOTE_WORK",
        "name_i18n": {"en": "Remote Work", "vi": "Làm việc từ xa"},
        "category": "working_hours",
        "default_minute_allowance": 45,
        "default_is_paid": true,
        "affects_rules": {
            "authorized_early_leave_minutes": true,
            "is_early_leave_paid": true,
            "grace_period_minutes": true,
            "is_break_auto_deduct": false,
            "is_break_overtime": true,
            "allow_early_ot": false,
            "early_arrival_strategy": "Block"
        },
        "icon": "Clock",
        "color": "green",
        "sort_order": 10
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/condition-types"),
            ws_id,
            user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let type_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["code"], "REMOTE_WORK");
    assert_eq!(created["name_i18n"]["en"], "Remote Work");
    assert_eq!(created["name_i18n"]["vi"], "Làm việc từ xa");
    assert_eq!(created["is_system"], false);
    assert_eq!(created["affects_rules"]["grace_period_minutes"], true);
    assert_eq!(created["affects_rules"]["is_break_auto_deduct"], false);
    assert_eq!(created["affects_rules"]["early_arrival_strategy"], "Block");

    // 3. Get by ID
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/condition-types/{type_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let fetched: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(fetched["id"], type_id);
    assert_eq!(fetched["code"], "REMOTE_WORK");

    // 4. Update the type
    let update_payload = json!({
        "name_i18n": {"en": "Remote Work Updated", "vi": "Làm việc từ xa (cập nhật)"},
        "default_is_paid": false,
        "sort_order": 20
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/condition-types/{type_id}"),
            ws_id,
            user_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["name_i18n"]["en"], "Remote Work Updated");
    assert_eq!(updated["default_is_paid"], false);
    assert_eq!(updated["sort_order"], 20);
    // code should remain unchanged
    assert_eq!(updated["code"], "REMOTE_WORK");

    // 5. Delete the custom type
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/condition-types/{type_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Verify deleted
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/condition-types/{type_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_cannot_delete_system_type() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory
        .user()
        .with_email("sys-type-test@test.local")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Insert a system type directly (the API always creates non-system types)
    let system_type_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO condition_type_definitions (id, workspace_id, code, name_i18n, category, is_system, sort_order) \
         VALUES ($1, $2, 'SYSTEM_TEST', '{\"en\":\"Test\"}', 'general', true, 0)",
        diesel::sql_types::Uuid,
        system_type_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Try to delete a system type
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/condition-types/{system_type_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "System types must not be deletable"
    );
}

#[tokio::test]
async fn test_duplicate_code_returns_conflict() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory
        .user()
        .with_email("dup-code-test@test.local")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let payload = json!({
        "code": "UNIQUE_TYPE",
        "name_i18n": {"en": "Unique Type"},
        "category": "general",
        "default_minute_allowance": 15,
        "affects_rules": {"grace_period_minutes": true}
    });

    // First creation succeeds
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/condition-types"),
            ws_id,
            user_id,
            Some(payload.clone()),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Second creation with same code fails
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/condition-types"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CONFLICT,
        "Duplicate code must return 409 Conflict"
    );
}

#[tokio::test]
async fn test_invalid_code_format_returns_validation_error() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory
        .user()
        .with_email("bad-code-test@test.local")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Lowercase code
    let payload = json!({
        "code": "lowercase_code",
        "name_i18n": {"en": "Bad Code"}
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/condition-types"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .unwrap();

    assert!(
        response.status().is_client_error(),
        "Invalid code format should return 4xx"
    );
}

#[tokio::test]
async fn test_active_only_filter() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory
        .user()
        .with_email("active-filter@test.local")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // List with active_only=true — all seeded types are active
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/condition-types?active_only=true"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let types: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    for t in &types {
        assert_eq!(
            t["is_active"], true,
            "All returned types should be active when active_only=true"
        );
    }
}
