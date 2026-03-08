//! Overtime policy API integration tests

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
async fn test_overtime_policy_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create policy
    let create_body = serde_json::json!({
        "name": "Standard OT Policy",
        "description": "1.5x weekdays, 2x weekends"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/overtime/policies"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let policy = body_json(resp).await;
    let policy_id = policy["id"].as_str().unwrap();
    assert_eq!(policy["name"].as_str().unwrap(), "Standard OT Policy");

    // 2. Add a rule to the policy
    let rule_body = serde_json::json!({
        "priority": 1,
        "multiplier": "1.5",
        "time_start": null,
        "time_end": null,
        "days_of_week": 31,
        "is_holiday": false
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/overtime/policies/{policy_id}/rules"),
            ws_id,
            admin_id,
            Some(rule_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let rule = body_json(resp).await;
    assert_eq!(rule["priority"].as_i64().unwrap(), 1);

    // 3. Get policy with rules
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/overtime/policies/{policy_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 4. List all policies
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/overtime/policies"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let policies = body_json(resp).await;
    assert!(!policies.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_overtime_policy_assignment() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create policy
    let create_body = serde_json::json!({
        "name": "Weekend OT",
        "description": null
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/overtime/policies"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let policy = body_json(resp).await;
    let policy_id = policy["id"].as_str().unwrap();

    // Assign policy to workspace level
    let assign_body = serde_json::json!({
        "assignee_type": "workspace",
        "assignee_id": null,
        "policy_id": policy_id,
        "effective_from": "2026-01-01"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/workspaces/{ws_id}/overtime/policies/assign"),
            ws_id,
            admin_id,
            Some(assign_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
