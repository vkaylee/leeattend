use apiserver::create_app;
use std::sync::Arc;
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use sqlx::postgres::PgPoolOptions;
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;

async fn spawn_app() -> String {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    let state = Arc::new(AppState {
        agent_manager: AgentManager::new(),
        db: pool,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("127.0.0.1:{}", port)
}

#[tokio::test]
async fn test_websocket_handshake() {
    let addr = spawn_app().await;
    let url = format!("ws://{}/ws", addr);

    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    
    // Send handshake
    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": "TEST_AGENT_01",
        "token": "test_token"
    });
    
    use tokio_tungstenite::tungstenite::Message;
    ws_stream.send(Message::Text(handshake.to_string().into())).await.expect("Failed to send message");

    // We should be able to receive something or at least stay connected
    // (Server currently sends config_update on handshake)
    if let Some(Ok(msg)) = ws_stream.next().await {
        assert!(msg.is_text());
        assert!(msg.to_text().unwrap().contains("config_update"));
    }
}

use futures_util::SinkExt;