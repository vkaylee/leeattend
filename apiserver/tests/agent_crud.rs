use apiserver::{agent_manager::AgentManager, create_app, AppState};
use reqwest::StatusCode;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid) {
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

    // Dọn dẹp dữ liệu cũ trước khi test
    sqlx::query("DELETE FROM workspace_members").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM devices").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM authorized_agents").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM workspaces").execute(&db_pool).await.unwrap();

    // Create a test workspace
    let workspace_id = sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ($1) RETURNING id")
        .bind("Test Workspace")
        .fetch_one(&db_pool)
        .await
        .unwrap();

    let state = Arc::new(AppState {
        agent_manager: AgentManager::new(),
        db: db_pool,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), workspace_id)
}

#[tokio::test]
#[serial]
async fn test_agent_crud_operations() {
    let (address, workspace_id) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());

    // 1. Khai báo Agent mới (để test các thao tác CRUD lên nó)
    let agent_id = "AGENT_CRUD_001";
    let create_body = serde_json::json!({
        "id": agent_id,
        "name": "Initial Name",
        "token": "initial_token"
    });

    let response = client
        .post(&format!("{}/authorized-agents", address))
        .header(ws_header.0, ws_header.1.clone())
        .json(&create_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Lấy Agent theo ID
    let response = client
        .get(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent["id"], agent_id);
    assert_eq!(fetched_agent["name"], "Initial Name");
    assert_eq!(fetched_agent["is_active"], true);

    // 3. Cập nhật Agent (PATCH)
    let update_body = serde_json::json!({
        "name": "Updated Name",
        "is_trusted": true,
        "is_active": true
    });

    let response = client
        .patch(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .json(&update_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated_agent["name"], "Updated Name");
    assert_eq!(updated_agent["is_trusted"], true);
    assert_eq!(updated_agent["is_active"], true);

    // 4. Lấy Agent theo ID lần nữa để xác nhận cập nhật
    let response = client
        .get(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched_agent_after_update: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent_after_update["name"], "Updated Name");
    assert_eq!(fetched_agent_after_update["is_trusted"], true);
    assert_eq!(fetched_agent_after_update["is_active"], true);

    // 5. Soft Delete Agent
    let response = client
        .delete(&format!("{}/authorized-agents/{}", address, agent_id))
        .header(ws_header.0, ws_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 6. Kiểm tra danh sách Agent (mặc định không bao gồm inactive)
    let response = client
        .get(&format!("{}/authorized-agents", address))
        .header(ws_header.0, ws_header.1.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(!agents.iter().any(|a| a["id"] == agent_id)); // Agent đã xóa không có trong danh sách

    // 7. Kiểm tra danh sách Agent (bao gồm inactive)
    let response = client
        .get(&format!(
            "{}/authorized-agents?include_inactive=true",
            address
        ))
        .header(ws_header.0, ws_header.1.clone())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let agents_with_inactive: Vec<serde_json::Value> = response.json().await.unwrap();
    let deleted_agent = agents_with_inactive
        .iter()
        .find(|a| a["id"] == agent_id)
        .expect("Deleted agent not found in inactive list");
    assert_eq!(deleted_agent["is_active"], false);
}

#[tokio::test]
#[serial]
async fn test_workspace_isolation() {
    let (address, workspace_id_1) = spawn_app().await;
    let client = reqwest::Client::new();
    
    // Create second workspace manually
    let ws_2_resp = client.post(&format!("{}/workspaces", address))
        .json(&serde_json::json!({ "name": "Second Workspace" }))
        .send().await.unwrap();
    let ws_2: serde_json::Value = ws_2_resp.json().await.unwrap();
    let workspace_id_2 = Uuid::parse_str(ws_2["id"].as_str().unwrap()).unwrap();

    // 1. Create agent in WS 1
    client.post(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_1.to_string())
        .json(&serde_json::json!({ "id": "A1", "name": "Agent 1", "token": "T1" }))
        .send().await.unwrap();

    // 2. Create agent in WS 2
    client.post(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_2.to_string())
        .json(&serde_json::json!({ "id": "A2", "name": "Agent 2", "token": "T2" }))
        .send().await.unwrap();

    // 3. List agents from WS 1 - should only see A1
    let resp_1 = client.get(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_1.to_string())
        .send().await.unwrap();
    let agents_1: Vec<serde_json::Value> = resp_1.json().await.unwrap();
    assert_eq!(agents_1.len(), 1);
    assert_eq!(agents_1[0]["id"], "A1");

    // 4. List agents from WS 2 - should only see A2
    let resp_2 = client.get(&format!("{}/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id_2.to_string())
        .send().await.unwrap();
    let agents_2: Vec<serde_json::Value> = resp_2.json().await.unwrap();
    assert_eq!(agents_2.len(), 1);
    assert_eq!(agents_2[0]["id"], "A2");
}