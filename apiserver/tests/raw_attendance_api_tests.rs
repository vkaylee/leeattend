mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;
use chrono::{Utc, Duration, TimeZone};
use axum::http::{StatusCode, Request};
use tower::ServiceExt;
use apiserver::models::RawAttendanceLog;
use apiserver::AppState;
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_context() -> (TestDatabase, sqlx::PgPool, TestFactory, Arc<AppState>, Uuid, Uuid, Uuid, chrono::DateTime<Utc>) {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Initialize AppState
    let agent_manager = apiserver::agent_manager::AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));
    let jwt_secret = "dev_secret_key".to_string();

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret,
    });

    let base_time = Utc.with_ymd_and_hms(2026, 1, 14, 8, 0, 0).unwrap();
    
    for i in 0..3 {
        let log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            employee_id: emp_id,
            punch_time: base_time + Duration::hours(i),
            source: "test".to_string(),
            device_id: None,
            raw_data: None,
            created_at: Utc::now(),
        };
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &log).await.unwrap();
    }

    (_test_db, pool, factory, state, user_id, ws_id, emp_id, base_time)
}

#[tokio::test]
async fn test_get_raw_logs_filter_employee() {
    let (_db, _pool, _factory, state, user_id, ws_id, emp_id, _base_time) = setup_test_context().await;
    let app = apiserver::create_app(state);
    
    let uri = format!("/api/workspaces/{}/attendance/raw-logs?employee_id={}", ws_id, emp_id);
    let response = app.oneshot(
        Request::builder()
            .uri(uri)
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", user_id.to_string()) 
            .body(axum::body::Body::empty())
            .unwrap()
    ).await.map_err(|e| format!("{:?}", e)).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let res_obj: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res_obj["data"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_get_raw_logs_filter_date() {
    let (_db, _pool, _factory, state, user_id, ws_id, _emp_id, base_time) = setup_test_context().await;
    let app = apiserver::create_app(state);
    
    let from_time = base_time + Duration::minutes(30); // 08:30
    // Use ISO format with 'Z' which is safer for query params
    let from_str = from_time.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let uri = format!("/api/workspaces/{}/attendance/raw-logs?from={}", ws_id, from_str);
    let response = app.oneshot(
        Request::builder()
            .uri(uri)
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", user_id.to_string())
            .body(axum::body::Body::empty())
            .unwrap()
    ).await.map_err(|e| format!("{:?}", e)).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let res_obj: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Should return 09:00 and 10:00 logs (2 logs)
    assert_eq!(res_obj["data"].as_array().unwrap().len(), 2);
}