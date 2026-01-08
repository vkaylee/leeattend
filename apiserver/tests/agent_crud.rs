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

    // Run migrations before testing
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let factory = TestFactory::new(db_pool.clone());
    factory.cleanup().await;

    // Create a test user and workspace using factory
    let user_id = factory.user().with_email("test@example.com").create().await;
    let workspace_id = factory
        .workspace()
        .with_name("Test Workspace")
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
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (
        format!("http://127.0.0.1:{}", port),
        workspace_id,
        user_id,
        factory,
    )
}

#[tokio::test]
#[serial]
async fn test_agent_crud_operations() {
    let (address, workspace_id, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    // 1. Khai báo Agent mới
    let agent_id = "AGENT_CRUD_001";
    let create_body = serde_json::json!({
        "id": agent_id,
        "name": "Initial Name",
        "token": "initial_token"
    });

    let response = client
        .post(&format!("{}/authorized-agents", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&create_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Lấy Agent theo ID
    let response = client
        .get(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent["id"], agent_id);

    // 3. Cập nhật Agent
    let update_body = serde_json::json!({
        "name": "Updated Name"
    });

    let response = client
        .patch(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&update_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Soft Delete
    let response = client
        .delete(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
#[serial]
async fn test_workspace_isolation() {
    let (address, workspace_id_1, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let user_header = ("X-User-Id", user_id.to_string());

    // Create second workspace
    let ws_2_resp = client
        .post(&format!("{}/workspaces", address))
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "name": "Second Workspace" }))
        .send()
        .await
        .unwrap();
    let ws_2: serde_json::Value = ws_2_resp.json().await.unwrap();
    let workspace_id_2 = Uuid::parse_str(ws_2["id"].as_str().unwrap()).unwrap();

    // 1. Create agent in WS 1
    client
        .post(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_1.to_string())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "id": "A1", "name": "Agent 1", "token": "T1" }))
        .send()
        .await
        .unwrap();

    // 2. Create agent in WS 2
    client
        .post(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_2.to_string())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "id": "A2", "name": "Agent 2", "token": "T2" }))
        .send()
        .await
        .unwrap();

    // 3. List agents from WS 1
    let resp_1 = client
        .get(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_1.to_string())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let agents_1: Vec<serde_json::Value> = resp_1.json().await.unwrap();
    assert_eq!(agents_1.len(), 1);
    assert_eq!(agents_1[0]["id"], "A1");

    // 4. List agents from WS 2
    let resp_2 = client
        .get(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_2.to_string())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let agents_2: Vec<serde_json::Value> = resp_2.json().await.unwrap();
    assert_eq!(agents_2.len(), 1);
    assert_eq!(agents_2[0]["id"], "A2");
}

#[tokio::test]
#[serial]
async fn test_rbac_member_cannot_invite() {
    let (address, workspace_id, _owner_id, factory) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Create a "Member" user using factory
    let member_user_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;

    // 2. Add them to workspace as "member" using factory
    factory
        .add_member(workspace_id, member_user_id, "member")
        .await;

    // 3. Try to invite someone as this member
    let response = client
        .post(&format!("{}/workspaces/{}/members", address, workspace_id))
        .header("X-User-Id", member_user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&serde_json::json!({ "email": "other@test.com", "role": "admin" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
