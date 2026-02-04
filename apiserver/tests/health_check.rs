use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use std::sync::Arc;

mod common;
use common::db::TestDatabase;

async fn spawn_app() -> (String, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    // Setup test storage

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), test_db)
}

#[tokio::test]
async fn health_check_works() {
    let (address, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/health", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
}
