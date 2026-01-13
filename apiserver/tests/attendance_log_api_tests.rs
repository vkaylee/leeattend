mod common;

use axum::http::StatusCode;
use chrono::{Utc, Duration};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use serde_json::Value;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("apiserver=debug,sqlx=info")
        .with_test_writer()
        .try_init();

    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());
    
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = apiserver::agent_manager::AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(agent_repo.clone(), agent_manager.clone()));
    
    let app_state = Arc::new(apiserver::AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
    });
    
    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_get_attendance_logs_filtering_and_pagination() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup: User, Workspace, 2 Employees
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp1_id = factory.employee().with_workspace(ws_id).create().await;
    let emp2_id = factory.employee().with_workspace(ws_id).create().await;

    let now = Utc::now();
    
    // Employee 1 entries
    factory.time_entry().with_employee(emp1_id).with_punch_in(now - Duration::days(1)).with_punch_out(now - Duration::days(1) + Duration::hours(8)).create().await;
    factory.time_entry().with_employee(emp1_id).with_punch_in(now - Duration::days(2)).with_punch_out(now - Duration::days(2) + Duration::hours(8)).create().await;
    
    // Employee 2 entries
    factory.time_entry().with_employee(emp2_id).with_punch_in(now - Duration::days(3)).with_punch_out(now - Duration::days(3) + Duration::hours(8)).create().await;
    factory.time_entry().with_employee(emp2_id).with_punch_in(now - Duration::days(4)).with_punch_out(now - Duration::days(4) + Duration::hours(8)).create().await;
    factory.time_entry().with_employee(emp2_id).with_punch_in(now - Duration::days(5)).with_punch_out(now - Duration::days(5) + Duration::hours(8)).create().await;

    // --- Test 1: Get all logs (limit 20) ---
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/attendance/logs", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["total"], 5);
    assert_eq!(json["data"].as_array().unwrap().len(), 5);
    assert_eq!(json["page"], 1);
    assert_eq!(json["limit"], 20);

    // --- Test 2: Filter by employee_id ---
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/attendance/logs?employee_id={}", ws_id, emp1_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 2);
    assert_eq!(json["data"].as_array().unwrap().len(), 2);

    // --- Test 3: Filter by date range ---
    let from = (now - Duration::days(3) - Duration::hours(1)).to_rfc3339().replace("+", "%2B");
    let to = (now - Duration::days(2) + Duration::hours(1)).to_rfc3339().replace("+", "%2B");
    
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/attendance/logs?from={}&to={}", ws_id, from, to))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    // Days 2 and 3 should match
    assert_eq!(json["total"], 2);

    // --- Test 4: Pagination ---
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/attendance/logs?limit=2&page=2", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 5);
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(json["page"], 2);
    assert_eq!(json["limit"], 2);
}

#[tokio::test]
async fn test_get_attendance_logs_unauthorized() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // No headers
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/attendance/logs", ws_id))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Wrong workspace header (but valid user)
    let ws_id_2 = factory.workspace().create().await; // Not owner/member
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/attendance/logs", ws_id_2))
                .header("X-Workspace-Id", ws_id_2.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
