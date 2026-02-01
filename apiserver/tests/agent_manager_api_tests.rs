mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
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

    let storage_provider = apiserver::services::create_storage_provider(&storage)


    // Setup test storage



    let storage = apiserver::StorageBackend::Local {



        base_path: std::path::PathBuf::from("./test_storage"),



    };



    let storage_provider = apiserver::services::create_storage_provider(&storage)



        .await



        .expect("Failed to create test storage");




    // Setup test storage
    let storage = apiserver::StorageBackend::Local {
        base_path: std::path::PathBuf::from("./test_storage"),
    };
    let storage_provider = apiserver::services::create_storage_provider(&storage)
        .await
        .expect("Failed to create test storage");

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_agent".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        storage,
        storage_provider,
    });

    let app = apiserver::create_app(app_state.clone());
    (test_db, factory, app_state, app)
}

#[tokio::test]
async fn test_agent_commands_api() {
    let (_test_db, factory, state, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let agent_id = factory
        .agent()
        .with_workspace(ws_id)
        .with_trust(true)
        .create()
        .await;

    // 1. Get agents (none online yet)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/agents")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Send command to offline agent (should fail with 404)
    let command_req = serde_json::json!({
        "method": "reboot",
        "data": null
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/agent/{}/command", agent_id))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(command_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // 3. Mock agent being online
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    state.agent_manager.add_agent(agent_id.clone(), tx).await;

    // 4. Send command to online agent
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/agent/{}/command", agent_id))
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(command_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
