//! Leave Policy API integration tests

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
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder = builder.header("Content-Type", "application/json");
        builder.body(axum::body::Body::from(b.to_string())).unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

async fn body_json(resp: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn test_leave_policy_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Get a leave type
    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Create Policy
    let create_req = serde_json::json!({
        "name": "Standard Vacation Policy",
        "description": "Standard 15 days",
        "leave_type_id": vacation_id,
        "rule_config": {
            "frequency": "monthly",
            "accrual_amount": 1.25,
            "accrual_timing": "start_of_period"
        }
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies"),
            ws_id,
            admin_id,
            Some(create_req),
        ))
        .await
        .unwrap();

    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    if status != StatusCode::CREATED {
        panic!("Failed to create policy. Status: {}, body: {}", status, body_str);
    }
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let policy_id = created["id"].as_str().unwrap().to_string();

    // 2. Update Policy
    let update_req = serde_json::json!({
        "name": "Updated Vacation Policy"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/{policy_id}"),
            ws_id,
            admin_id,
            Some(update_req),
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["name"], "Updated Vacation Policy");

    // 3. Delete Policy
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/leave/policies/{policy_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
