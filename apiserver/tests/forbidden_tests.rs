mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::MemberRole;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
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
        jwt_secret: "test_secret_forbidden".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_forbidden_operations_for_member() {
    let (_test_db, factory, app) = setup_app().await;

    let owner_id = factory.user().create().await;
    let member_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    factory
        .add_member(ws_id, member_id, MemberRole::Member)
        .await;

    let headers = [
        ("X-Workspace-Id", ws_id.to_string()),
        ("X-User-Id", member_id.to_string()),
    ];

    // 1. Create Employee
    let emp_req = serde_json::json!({
        "full_name": "X Y",
        "employee_id_number": "X1",
        "hire_date": "2026-01-01"
    });
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees", ws_id))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from(emp_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. Create Shift
    let shift_req = serde_json::json!({
        "name": "X",
        "start_time": "09:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 0
    });
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from(shift_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 3. Create Department
    let dept_req = serde_json::json!({ "name": "X" });
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/departments", ws_id))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from(dept_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 4. Approve Leave
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/leave/requests/{}/approve",
                    ws_id,
                    uuid::Uuid::new_v4()
                ))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from("{\"status\":\"approved\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 5. Approve Adjustment
    let res = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/adjustments/{}/approve",
                    ws_id,
                    uuid::Uuid::new_v4()
                ))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from("{\"status\":\"approved\"}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}
