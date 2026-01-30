mod common;
use apiserver::services::AttendanceLogService;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use common::factory::TestFactory;
use reqwest::StatusCode;
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid, TestFactory, common::db::TestDatabase) {
    let test_db = common::db::TestDatabase::new().await;
    let db_pool = test_db.pool();

    let factory = TestFactory::new(db_pool.clone());
    // factory.cleanup().await; // TestDatabase is already empty, no need to cleanup manually and risk main DB

    let user_id = factory
        .user()
        .with_email("token_test@example.com")
        .create()
        .await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository {
        pool: db_pool.clone(),
    });
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository {
        pool: db_pool.clone(),
    });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: db_pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "token_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(db_pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, create_app(state)).await.unwrap();
    });

    (url, workspace_id, user_id, factory, test_db)
}

#[tokio::test]
#[serial]
async fn test_agent_token_update() {
    let (address, workspace_id, user_id, _, _tdb) = spawn_app().await;
    let client = reqwest::Client::new();

    let agent_id = "TOKEN_UPDATE_AGENT";
    let initial_token = "initial_secret";
    let new_token = "updated_secret_2026";

    // 1. Khai báo agent
    client
        .post(format!("{}/api/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({
            "id": agent_id,
            "name": "Test Agent",
            "token": initial_token
        }))
        .send()
        .await
        .unwrap();

    // 2. Cập nhật token
    let response = client
        .patch(format!("{}/api/authorized-agents/{}", address, agent_id))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({
            "token": new_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated_agent["token"], new_token);

    // 3. Kiểm tra lại bằng GET
    let response = client
        .get(format!("{}/api/authorized-agents/{}", address, agent_id))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    let fetched_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent["token"], new_token);
}
