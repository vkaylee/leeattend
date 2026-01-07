use apiserver::create_app;
use std::sync::Arc;
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use sqlx::postgres::PgPoolOptions;
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use serial_test::serial;
use tokio_tungstenite::tungstenite::Message;

async fn spawn_app() -> (String, sqlx::PgPool) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Clean DB
    sqlx::query("DELETE FROM authorized_agents").execute(&pool).await.unwrap();

    let state = Arc::new(AppState {
        agent_manager: AgentManager::new(),
        db: pool.clone(),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("127.0.0.1:{}", port), pool)
}

#[tokio::test]
#[serial]
async fn test_websocket_auth_success() {
    let (addr, pool) = spawn_app().await;
    
    sqlx::query("INSERT INTO authorized_agents (id, name, token, is_trusted) VALUES ($1, $2, $3, $4)")
        .bind("GOOD_AGENT")
        .bind("Test")
        .bind("secret123")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

    let url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    
    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": "GOOD_AGENT",
        "token": "secret123"
    });
    ws_stream.send(Message::Text(handshake.to_string().into())).await.unwrap();

    if let Some(Ok(msg)) = ws_stream.next().await {
        assert!(msg.is_text());
        assert!(msg.to_text().unwrap().contains("config_push"));
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_auth_wrong_token() {
    let (addr, pool) = spawn_app().await;
    
    sqlx::query("INSERT INTO authorized_agents (id, name, token, is_trusted) VALUES ($1, $2, $3, $4)")
        .bind("AGENT_X")
        .bind("Test")
        .bind("real_password")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

    let url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    
    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": "AGENT_X",
        "token": "wrong_password"
    });
    ws_stream.send(Message::Text(handshake.to_string().into())).await.unwrap();

    // Phải nhận được lỗi INVALID_TOKEN
    if let Some(Ok(msg)) = ws_stream.next().await {
        let resp: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        assert_eq!(resp["code"], "INVALID_TOKEN");
    }
}

#[tokio::test]
#[serial]
async fn test_websocket_auth_untrusted() {
    let (addr, pool) = spawn_app().await;
    
    sqlx::query("INSERT INTO authorized_agents (id, name, token, is_trusted) VALUES ($1, $2, $3, $4)")
        .bind("NEW_AGENT")
        .bind("Test")
        .bind("pass")
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

    let url = format!("ws://{}/ws", addr);
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    
    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": "NEW_AGENT",
        "token": "pass"
    });
    ws_stream.send(Message::Text(handshake.to_string().into())).await.unwrap();

    if let Some(Ok(msg)) = ws_stream.next().await {
        let resp: serde_json::Value = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        assert_eq!(resp["code"], "AGENT_NOT_TRUSTED");
    }
}
