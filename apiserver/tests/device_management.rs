use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::AppState;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Clean up
    sqlx::query("DELETE FROM workspace_members").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM devices").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM authorized_agents").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM users").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM workspaces").execute(&db_pool).await.unwrap();

    // Create a test user
    let user_id = sqlx::query_scalar("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id")
        .bind("device-test@example.com")
        .bind("hash")
        .fetch_one(&db_pool)
        .await
        .unwrap();

    // Create a test workspace
    let workspace_id = sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ($1) RETURNING id")
        .bind("Device Test Workspace")
        .fetch_one(&db_pool)
        .await
        .unwrap();

    // Add user to workspace as owner
    sqlx::query("INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(workspace_id)
        .bind(user_id)
        .bind("owner")
        .execute(&db_pool)
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

    (format!("http://127.0.0.1:{}", port), workspace_id, user_id)
}

#[tokio::test]
#[serial]
async fn test_create_and_get_device() {
    let (address, workspace_id, user_id) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    let body = serde_json::json!({
        "agent_id": "TEST_AGENT",
        "name": "Test Device",
        "ip": "192.168.1.50",
        "port": 4370
    });

    let response = client
        .post(&format!("{}/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201);
    let json: serde_json::Value = response.json().await.unwrap();
    let device_id = json["id"].as_str().unwrap();
    assert!(Uuid::parse_str(device_id).is_ok());

    let response = client
        .get(&format!("{}/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);
    let devices: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(devices.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_delete_device() {
    let (address, workspace_id, user_id) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    let response = client
        .post(&format!("{}/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({
            "agent_id": "TEST_AGENT",
            "name": "To Delete",
            "ip": "1.1.1.1"
        }))
        .send()
        .await
        .unwrap();

    let json: serde_json::Value = response.json().await.unwrap();
    let id = json["id"].as_str().unwrap();

    let response = client
        .delete(&format!("{}/devices/{}", address, id))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 204);

    let response = client
        .get(&format!("{}/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let devices: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(devices.len(), 0);
}
