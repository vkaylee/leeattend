//! Working pattern API integration tests

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
async fn test_working_pattern_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Need a shift first for the pattern details
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Morning Shift")
        .create()
        .await;

    // 1. Create working pattern
    let create_body = serde_json::json!({
        "name": "Mon-Fri Standard",
        "details": [
            { "day_of_week": 1, "shift_id": shift_id },
            { "day_of_week": 2, "shift_id": shift_id },
            { "day_of_week": 3, "shift_id": shift_id },
            { "day_of_week": 4, "shift_id": shift_id },
            { "day_of_week": 5, "shift_id": shift_id }
        ]
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let pattern = body_json(resp).await;
    let pattern_id = pattern["id"].as_str().unwrap();
    assert_eq!(pattern["name"].as_str().unwrap(), "Mon-Fri Standard");

    // 2. List patterns
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert!(!list.as_array().unwrap().is_empty());

    // 3. Get pattern details
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/working-patterns/{pattern_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. Delete pattern
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/working-patterns/{pattern_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_working_pattern_assign_to_employee() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Day Shift")
        .create()
        .await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create pattern
    let create_body = serde_json::json!({
        "name": "Weekday Pattern",
        "details": [
            { "day_of_week": 1, "shift_id": shift_id },
            { "day_of_week": 2, "shift_id": shift_id },
            { "day_of_week": 3, "shift_id": shift_id }
        ]
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let pattern = body_json(resp).await;
    let pattern_id = pattern["id"].as_str().unwrap();

    // Assign pattern to employee
    let assign_body = serde_json::json!({
        "pattern_id": pattern_id,
        "effective_from": "2026-01-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/employees/{emp_id}/working-patterns"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Get assignment history
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/employees/{emp_id}/working-patterns"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let history = body_json(resp).await;
    assert!(!history.as_array().unwrap().is_empty());
}
