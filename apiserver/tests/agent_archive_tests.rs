mod common;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use common::factory::TestFactory;
use reqwest::StatusCode;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid, TestFactory) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let factory = TestFactory::new(db_pool.clone());
    factory.cleanup().await;

    let user_id = factory
        .user()
        .with_email("archive_test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("Archive Workspace")
        .create_with_owner(user_id)
        .await;

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
        db: db_pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "archive_secret".to_string(),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    let app = create_app(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (url, workspace_id, user_id, factory)
}

#[tokio::test]
#[serial]
async fn test_agent_archive_and_restore() {
    let (address, workspace_id, user_id, factory) = spawn_app().await;
    let client = reqwest::Client::new();

    let agent_id = "AGENT_ARCHIVE_01";
    factory
        .agent()
        .with_id(agent_id)
        .with_workspace(workspace_id)
        .create()
        .await;

    // 1. Soft Delete (Archive)
    let response = client
        .delete(&format!("{}/api/authorized-agents/{}", address, agent_id))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 2. List agents (should be empty by default)
    let response = client
        .get(&format!("{}/api/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(agents.len(), 0);

    // 3. List agents with include_inactive=true
    let response = client
        .get(&format!(
            "{}/api/authorized-agents?include_inactive=true",
            address
        ))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["id"], agent_id);
    assert_eq!(agents[0]["is_active"], false);

    // 4. Restore Agent
    let response = client
        .patch(&format!("{}/api/authorized-agents/{}", address, agent_id))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "is_active": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. List agents again (should now show up)
    let response = client
        .get(&format!("{}/api/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["is_active"], true);
}
