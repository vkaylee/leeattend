//! Integration tests for salary bases endpoints.
//!
//! Validates that:
//! - GET /workspaces/:ws_id/salary-bases returns groups and members
//! - POST /workspaces/:ws_id/salary-bases/:base_id/members assigns a component to a base
//! - DELETE /workspaces/:ws_id/salary-bases/:base_id/members/:component_id unassigns it

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let req = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json");

    if let Some(b) = body {
        req.body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        req.body(axum::body::Body::empty()).unwrap()
    }
}

async fn create_salary_base_group(pool: &sqlx::PgPool, ws_id: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO salary_base_groups (id, workspace_id, name, slug, is_system) \
         VALUES ($1, $2, $3, $4, false)",
    )
    .bind(id)
    .bind(ws_id)
    .bind(name)
    .bind(name.to_lowercase().replace(' ', "_"))
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn create_payroll_component(pool: &sqlx::PgPool, ws_id: Uuid, name: &str) -> Uuid {
    let id = Uuid::new_v4();
    // Insert a dummy payroll component definition
    sqlx::query(
        "INSERT INTO salary_component_definitions \
         (id, workspace_id, name, code, component_type) \
         VALUES ($1, $2, $3, $4, 'basic_salary')",
    )
    .bind(id)
    .bind(ws_id)
    .bind(name)
    .bind(name.to_lowercase().replace(' ', "_"))
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn test_salary_bases_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create a base group
    let base_id = create_salary_base_group(&factory.pool, ws_id, "Standard Base").await;

    // Create a component
    let comp_id = create_payroll_component(&factory.pool, ws_id, "Basic Salary").await;

    // 1. Get salary bases (empty members)
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let mut body = body_json(res).await;
    let groups = body["groups"].as_array().unwrap();
    assert!(groups.iter().any(|g| g["id"] == base_id.to_string()));

    // 2. Assign component to base
    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/salary-bases/{base_id}/members"),
            ws_id,
            user_id,
            Some(json!({ "component_definition_id": comp_id })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Get salary bases (should contain the member)
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    body = body_json(res).await;
    let members = body["members"].as_array().unwrap();
    assert!(members
        .iter()
        .any(|m| m["component_definition_id"] == comp_id.to_string()
            && m["base_group_id"] == base_id.to_string()));

    // 4. Unassign component
    let res = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/salary-bases/{base_id}/members/{comp_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Get salary bases (should be empty again)
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    body = body_json(res).await;
    let members = body["members"].as_array().unwrap();
    assert!(!members
        .iter()
        .any(|m| m["component_definition_id"] == comp_id.to_string()
            && m["base_group_id"] == base_id.to_string()));
}

#[tokio::test]
async fn test_create_custom_salary_base() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let payload = json!({
        "name": "Custom Base",
        "slug": "custom_base",
        "description": "My custom base"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body_json(res).await;
    assert_eq!(body["name"], "Custom Base");
    assert_eq!(body["slug"], "custom_base");
    assert_eq!(body["is_system"], false);

    // Verify it appears in GET
    let res = app
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    let groups = body["groups"].as_array().unwrap();
    assert!(groups
        .iter()
        .any(|g| g["name"] == "Custom Base" && g["slug"] == "custom_base"));
}

#[tokio::test]
async fn test_system_salary_bases_seeded() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Fetch salary bases
    let res = app
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    let groups = body["groups"].as_array().unwrap();

    // Verify the 4 seeded system bases are present
    let system_slugs = vec!["gross", "net", "bhxh", "taxable"];
    for slug in system_slugs {
        let found = groups
            .iter()
            .find(|g| g["slug"] == slug && g["is_system"] == true);
        assert!(
            found.is_some(),
            "System base with slug '{}' should be seeded and returned",
            slug
        );
    }
}
