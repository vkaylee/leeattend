mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;
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

    // Setup test storage

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_policy".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_leave_policy_api_lifecycle() {
    let (test_db, factory, app) = setup_app().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create Policy
    let vacation_id: Uuid = sqlx::query_scalar!("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'", ws_id)
        .fetch_one(&test_db.pool())
        .await
        .unwrap();

    let policy_req = serde_json::json!({
        "name": "Standard Annual Leave",
        "description": "Standard policy for all employees",
        "leave_type_id": vacation_id,
        "rule_config": {
            "frequency": "monthly",
            "accrual_amount": 1.25,
            "accrual_timing": "start_of_period"
        }
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/policies", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(policy_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let policy: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    let policy_id = policy["id"].as_str().unwrap();

    // 2. List Policies
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/policies", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Assign Policy to Workspace (Global)
    let assign_req = serde_json::json!({
        "policy_id": policy_id,
        "assignee_type": "workspace",
        "assignee_id": null,
        "effective_from": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/policies/assign", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(assign_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // 4. Trigger Accrual Run
    let trigger_req = serde_json::json!({
        "date": "2026-01-10"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/accrual/run", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(trigger_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Update Policy
    let update_req = serde_json::json!({
        "name": "Updated Annual Leave",
        "description": "Updated description",
        "rule_config": {
            "frequency": "monthly",
            "accrual_amount": 2.0,
            "accrual_timing": "start_of_period"
        }
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/workspaces/{}/leave/policies/{}",
                    ws_id, policy_id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(update_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated_policy: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(updated_policy["name"], "Updated Annual Leave");

    // 6. Delete Policy
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{}/leave/policies/{}",
                    ws_id, policy_id
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 7. Verify deletion
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/policies", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let policies: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(policies.iter().all(|p| p["id"] != policy_id));
}
