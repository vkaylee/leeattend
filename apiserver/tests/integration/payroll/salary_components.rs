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

#[tokio::test]
async fn test_compensation_history_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create definitions for Basic and House
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/salary-components"),
            ws_id,
            admin_id,
            Some(serde_json::json!({"name": "Basic Salary", "code": "BASIC"}).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let basic_def = body_json(resp).await;
    let basic_id = basic_def["id"].as_str().unwrap();

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/salary-components"),
            ws_id,
            admin_id,
            Some(serde_json::json!({"name": "Housing Allowance", "code": "HOUSE"}).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let house_def = body_json(resp).await;
    let house_id = house_def["id"].as_str().unwrap();

    // 2. Create employee
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // 3. Assign Basic = 10M, House = 2M on 2026-01-01
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/salary-components"),
            ws_id,
            admin_id,
            Some(serde_json::json!({
                "component_definition_id": basic_id,
                "amount": 10000000,
                "effective_date": "2026-01-01"
            }).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/salary-components"),
            ws_id,
            admin_id,
            Some(serde_json::json!({
                "component_definition_id": house_id,
                "amount": 2000000,
                "effective_date": "2026-01-01"
            }).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 4. Assign Basic = 12M on 2026-06-01 (simulating salary change)
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employments/{employment_id}/salary-components"),
            ws_id,
            admin_id,
            Some(serde_json::json!({
                "component_definition_id": basic_id,
                "amount": 12000000,
                "effective_date": "2026-06-01"
            }).to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // 5. Query compensation history endpoint
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/compensation-history"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let history = body_json(resp).await;
    let timeline = history["timeline"].as_array().unwrap();

    assert_eq!(timeline.len(), 2);

    // Event 1 (newest, 2026-06-01)
    let event1 = &timeline[0];
    assert_eq!(event1["effective_date"].as_str().unwrap(), "2026-06-01");
    assert_eq!(event1["total_compensation"].as_str().unwrap(), "14000000.00");
    assert_eq!(event1["change_type"].as_str().unwrap(), "adjustment");
    assert_eq!(event1["change_details"]["delta"].as_str().unwrap(), "2000000.00");
    assert_eq!(event1["change_details"]["previous_total"].as_str().unwrap(), "12000000.00");

    // Event 2 (initial, 2026-01-01)
    let event2 = &timeline[1];
    assert_eq!(event2["effective_date"].as_str().unwrap(), "2026-01-01");
    assert_eq!(event2["total_compensation"].as_str().unwrap(), "12000000.00");
    assert_eq!(event2["change_type"].as_str().unwrap(), "initial");
    assert_eq!(event2["change_details"]["delta"].as_str().unwrap(), "12000000.00");
    assert!(event2["change_details"]["previous_total"].is_null());
}
