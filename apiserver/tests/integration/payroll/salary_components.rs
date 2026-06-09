//! Salary Components API integration tests

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
async fn test_salary_component_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create salary component definition
    let create_body = serde_json::json!({
        "name": "Housing Allowance",
        "code": "HOUSE"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/salary-components"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let comp_id = created["id"].as_str().unwrap();
    assert_eq!(created["name"].as_str().unwrap(), "Housing Allowance");
    assert_eq!(created["code"].as_str().unwrap(), "HOUSE");

    // 2. List salary component definitions
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/salary-components"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert!(list
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["code"].as_str().unwrap() == "HOUSE"));

    // 3. Create employee and assign salary component
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let assign_body = serde_json::json!({
        "component_definition_id": comp_id,
        "amount": 2000000,
        "effective_date": "2026-01-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/salary-components"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 4. Get employee salary components
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/salary-components"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let components = body_json(resp).await;
    let comps = components.as_array().unwrap();
    assert!(!comps.is_empty());
    assert!(comps.iter().any(|c| c["code"].as_str().unwrap() == "HOUSE"));
}
