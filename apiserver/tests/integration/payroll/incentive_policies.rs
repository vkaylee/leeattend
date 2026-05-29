//! Payroll Incentive Policies API integration tests

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
async fn test_incentive_policy_list_empty() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // List incentive policies (should be empty)
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/payroll/incentive-policies"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert!(list.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_incentive_policy_create_and_list() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    println!("Starting test_incentive_policy_create_and_list");
    let create_payload = serde_json::json!({
        "name": "Test Incentive",
        "base_amount": 5000.0,
        "payout_type": "fixed",
        "effective_from": "2026-01-01",
        "tiers": [
            {
                "priority": 1,
                "payout_amount": 5000.0
            }
        ]
    });

    println!("Creating policy...");
    // Create policy
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/payroll/incentive-policies"),
            ws_id,
            admin_id,
            Some(create_payload.to_string()),
        ))
        .await
        .unwrap();

    println!("Create response status: {}", resp.status());
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    assert_eq!(created["name"], "Test Incentive");

    println!("Listing policies...");
    // List policies
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/payroll/incentive-policies"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    println!("List response status: {}", resp.status());
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["name"], "Test Incentive");
    assert_eq!(list[0]["tiers"].as_array().unwrap().len(), 1);
    println!("Test finished");
}
#[tokio::test]
async fn test_incentive_policy_delete() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Insert policy directly
    let policy_id = Uuid::new_v4();
    crate::test_execute!(
        &test_db.diesel_pools.api,
        r"INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active, created_at)
           VALUES ($1, $2, 'Test Policy', 100000, 'monthly', TRUE, NOW()::timestamp)",
        diesel::sql_types::Uuid,
        policy_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Delete
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/payroll/incentive-policies"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let list = body_json(resp).await;
    assert!(list.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_incentive_policy_clone() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Insert policy directly
    let policy_id = Uuid::new_v4();
    crate::test_execute!(
        &test_db.diesel_pools.api,
        r"INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active, created_at)
           VALUES ($1, $2, 'Original Policy', 100000, 'monthly', TRUE, NOW()::timestamp)",
        diesel::sql_types::Uuid,
        policy_id,
        diesel::sql_types::Uuid,
        ws_id
    );
    // Insert a tier for the policy
    crate::test_execute!(
        &test_db.diesel_pools.api,
        r"INSERT INTO payroll_incentive_tiers (id, policy_id, priority, payout_amount, rule_config)
           VALUES (gen_random_uuid(), $1, 1, 50000, '{}')",
        diesel::sql_types::Uuid,
        policy_id
    );

    // Clone the policy
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let cloned = body_json(resp).await;
    assert_eq!(cloned["name"], "Original Policy (Copy)");
    assert_eq!(cloned["is_active"], false); // Clone is inactive by default
    
    let new_policy_id = cloned["id"].as_str().unwrap();
    assert_ne!(new_policy_id, policy_id.to_string());
}
