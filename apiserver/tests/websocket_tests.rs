use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::AppState;
use futures_util::{SinkExt, StreamExt};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Arc<AppState>) {
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
    let app = create_app(state.clone());

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("ws://127.0.0.1:{}/ws", port), workspace_id, state)
}

#[tokio::test]
#[serial]
async fn test_websocket_auth_success() {
    let (address, workspace_id, state) = spawn_app().await;

    // 1. Prepare agent in DB
    let agent_id = "WS_AGENT_OK";
    let token = "ws_token_123";
    sqlx::query("INSERT INTO authorized_agents (id, name, token, is_trusted, workspace_id) VALUES ($1, $2, $3, true, $4)")
        .bind(agent_id)
        .bind("WS Agent")
        .bind(token)
        .bind(workspace_id)
        .execute(&state.db)
        .await
        .unwrap();

    // 2. Connect
    let (ws_stream, _) = connect_async(&address).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    // 3. Send handshake
    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": agent_id,
        "token": token
    });
    write
        .send(Message::Text(handshake.to_string().into()))
        .await
        .unwrap();

    // 4. Check if agent is added to manager
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let active_agents = state.agent_manager.agents.read().await;
    assert!(active_agents.contains_key(agent_id));
}

#[tokio::test]
#[serial]
async fn test_websocket_auth_wrong_token() {
    let (address, workspace_id, state) = spawn_app().await;

    // 1. Prepare agent in DB
    let agent_id = "WS_AGENT_WRONG_TOKEN";
    sqlx::query("INSERT INTO authorized_agents (id, name, token, is_trusted, workspace_id) VALUES ($1, $2, $3, true, $4)")
        .bind(agent_id)
        .bind("WS Agent")
        .bind("correct_token")
        .bind(workspace_id)
        .execute(&state.db)
        .await
        .unwrap();

    let (ws_stream, _) = connect_async(&address).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": agent_id,
        "token": "WRONG_TOKEN"
    });
    write
        .send(Message::Text(handshake.to_string().into()))
        .await
        .unwrap();

    // 2. Expect error message and close
    if let Some(Ok(Message::Text(text))) = read.next().await {
        assert!(text.contains("INVALID_TOKEN"));
    } else {
        panic!("Expected error message");
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_auth_untrusted() {
    let (address, workspace_id, state) = spawn_app().await;

    // 1. Prepare agent in DB (is_trusted = false)
    let agent_id = "WS_AGENT_UNTRUSTED";
    let token = "ws_token_untrusted";
    sqlx::query("INSERT INTO authorized_agents (id, name, token, is_trusted, workspace_id) VALUES ($1, $2, $3, false, $4)")
        .bind(agent_id)
        .bind("WS Agent")
        .bind(token)
        .bind(workspace_id)
        .execute(&state.db)
        .await
        .unwrap();

    let (ws_stream, _) = connect_async(&address).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();

    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": agent_id,
        "token": token
    });
    write
        .send(Message::Text(handshake.to_string().into()))
        .await
        .unwrap();

    // 2. Expect error message
    if let Some(Ok(Message::Text(text))) = read.next().await {
        assert!(text.contains("AGENT_NOT_TRUSTED"));
    } else {
        panic!("Expected error message");
    }
}