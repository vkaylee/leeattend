mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, Arc<AppState>, axum::Router) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_shift".to_string(),
    });

    let app = apiserver::create_app(app_state.clone());
    (test_db, factory, app_state, app)
}

#[tokio::test]
async fn test_shift_api_lifecycle() {
    let (_test_db, factory, _state, app) = setup_app().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create Shift
    let shift_req = serde_json::json!({
        "name": "Night Shift",
        "start_time": "22:00:00",
        "end_time": "06:00:00",
        "break_duration_minutes": 30,
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 15
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let shift: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    let shift_id = shift["id"].as_str().unwrap();

    // 2. List Shifts
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let shifts: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(!shifts.is_empty());

    // 3. Assign Shift
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let assign_req = serde_json::json!({
        "employee_id": emp_id,
        "shift_id": shift_id,
        "start_date": Utc::now().date_naive().to_string()
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts/assign", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(assign_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Update Shift
    let update_req = serde_json::json!({
        "name": "Updated Night Shift",
        "start_time": "22:00:00",
        "end_time": "06:00:00",
        "break_duration_minutes": 45,
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 15,
        "is_correction": true
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/shifts/{}", ws_id, shift_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(update_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Update Shift with Versioning (is_correction = false)
    let version_req = serde_json::json!({
        "name": "New Version Night Shift",
        "start_time": "23:00:00",
        "end_time": "07:00:00",
        "break_duration_minutes": 30,
        "is_break_auto_deduct": true,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 15,
        "is_correction": false,
        "effective_date": "2026-02-01"
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/shifts/{}", ws_id, shift_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(version_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 6. List Shifts again
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 7. Create Shift with explicit breaks
    let shift_req_2 = serde_json::json!({
        "name": "Mid Shift",
        "start_time": "12:00:00",
        "end_time": "20:00:00",
        "break_duration_minutes": 30,
        "break_start": "16:00:00",
        "break_end": "16:30:00",
        "is_break_auto_deduct": false,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 0
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(shift_req_2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}
