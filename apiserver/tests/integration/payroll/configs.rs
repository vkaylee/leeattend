//! Integration tests for payroll config CRUD endpoints
//! Tests GET/POST for department and employee payroll configs

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_set_and_get_department_payroll_config() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let dept_id: Uuid = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;

    // POST: Create a department payroll config
    let body = json!({
        "std_working_days": 22,
        "hours_per_day": 8,
        "effective_date": "2024-01-01"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/departments/{dept_id}/payroll-config"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // GET: Retrieve department payroll configs
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/departments/{dept_id}/payroll-config"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let configs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(configs.len(), 1);
    assert!(configs[0]["std_working_days"] == json!("22") || configs[0]["std_working_days"] == json!("22.00"));
    assert_eq!(configs[0]["department_id"], json!(dept_id.to_string()));
}

#[tokio::test]
async fn test_set_and_get_employee_payroll_config() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // POST: Create an employee payroll config
    let body = json!({
        "std_working_days": 20,
        "hours_per_day": 8,
        "effective_date": "2024-06-01"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/employees/{emp_id}/payroll-config"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // GET: Retrieve employee payroll configs
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/employees/{emp_id}/payroll-config"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let configs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(configs.len(), 1);
    assert!(configs[0]["std_working_days"] == json!("20") || configs[0]["std_working_days"] == json!("20.00"));
    assert_eq!(configs[0]["employee_id"], json!(emp_id.to_string()));
}

#[tokio::test]
async fn test_get_empty_payroll_configs() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let dept_id: Uuid = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // GET department configs when none exist
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/departments/{dept_id}/payroll-config"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let configs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(configs.is_empty());

    // GET employee configs when none exist
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/workspaces/{ws_id}/employees/{emp_id}/payroll-config"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let configs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(configs.is_empty());
}
