mod common;

use apiserver::models::EarlyArrivalStrategy;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use apiserver::repositories::{SqlxAgentRepository, SqlxWorkspaceRepository};
use apiserver::services::AgentService;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid, TestFactory, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let db_pool = test_db.pool();

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(SqlxAgentRepository {
        pool: db_pool.clone(),
    });
    let workspace_repo = Arc::new(SqlxWorkspaceRepository {
        pool: db_pool.clone(),
    });
    let agent_service = Arc::new(AgentService::new(agent_repo.clone(), agent_manager.clone()));

    let state = Arc::new(AppState {
        db: db_pool.clone(),
        agent_manager,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(db_pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let app = apiserver::create_app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}", listener.local_addr().unwrap());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let factory = TestFactory::new(db_pool.clone());
    let owner_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(owner_id).await;

    (address, workspace_id, owner_id, factory, test_db)
}

#[tokio::test]
async fn test_workspace_early_arrival_strategy_crud() {
    let (address, workspace_id, owner_id, _, _db) = spawn_app().await;
    let client = reqwest::Client::new();
    let user_header = ("X-User-Id", owner_id.to_string());

    // 1. Update to CLAMP
    let resp = client.patch(format!("{}/api/workspaces/{}", address, workspace_id))
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "early_arrival_strategy": "clamp" }))
        .send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    
    let ws: apiserver::models::Workspace = resp.json().await.unwrap();
    assert_eq!(ws.early_arrival_strategy, EarlyArrivalStrategy::Clamp);

    // 2. Verify GET
    let resp = client.get(format!("{}/api/workspaces/{}", address, workspace_id))
        .header(user_header.0, user_header.1.clone())
        .send().await.unwrap();
    let ws: apiserver::models::Workspace = resp.json().await.unwrap();
    assert_eq!(ws.early_arrival_strategy, EarlyArrivalStrategy::Clamp);
}
