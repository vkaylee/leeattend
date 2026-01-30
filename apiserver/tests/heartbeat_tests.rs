mod common;
use apiserver::services::AttendanceLogService;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use chrono::{Duration, Utc};
use common::factory::TestFactory;
use futures_util::{SinkExt, StreamExt};
use serial_test::serial;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid, TestFactory, common::db::TestDatabase) {
    let test_db = common::db::TestDatabase::new().await;
    let pool = test_db.pool();

    let factory = TestFactory::new(pool.clone());

    let user_id = factory
        .user()
        .with_email("hb_ws_test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("HB WS Workspace")
        .create_with_owner(user_id)
        .await;

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "hb_ws_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
    });

    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("ws://{}/api/ws", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (url, user_id, workspace_id, factory, test_db)
}

#[tokio::test]
#[serial]
async fn test_agent_heartbeat_via_websocket() {
    let (url, _, workspace_id, factory, _tdb) = spawn_app().await;

    let agent_id = "HB_WS_AGENT_01".to_string();
    let token = "hb_ws_token_123".to_string();

    // 1. Khai báo agent và phê duyệt
    factory
        .agent()
        .with_id(&agent_id)
        .with_token(&token)
        .with_workspace(workspace_id)
        .with_trust(true)
        .create()
        .await;

    // 2. Kết nối WebSocket
    let (mut ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

    // 3. Thực hiện Handshake
    let handshake = serde_json::json!({
        "type": "handshake",
        "agent_id": agent_id,
        "token": token,
        "timestamp": Utc::now().to_rfc3339()
    });
    ws_stream
        .send(Message::Text(handshake.to_string().into()))
        .await
        .unwrap();

    // Chờ config push để chắc chắn handshake xong
    if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["type"], "config_push");
    } else {
        panic!("Did not receive config_push after handshake");
    }

    // 4. Gửi Heartbeat message
    let heartbeat = serde_json::json!({
        "type": "heartbeat",
        "agent_id": agent_id,
        "timestamp": Utc::now().to_rfc3339()
    });
    ws_stream
        .send(Message::Text(heartbeat.to_string().into()))
        .await
        .unwrap();

    // Chờ một chút để server xử lý
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // 5. Kiểm tra DB xem last_seen_at có được cập nhật không
    let last_seen: chrono::DateTime<Utc> =
        sqlx::query_scalar("SELECT last_seen_at FROM authorized_agents WHERE id = $1")
            .bind(&agent_id)
            .fetch_one(&factory.pool)
            .await
            .unwrap();

    let now = Utc::now();
    let diff = if now > last_seen {
        now - last_seen
    } else {
        last_seen - now
    };

    assert!(diff < Duration::seconds(10));

    let _ = ws_stream.close(None).await;
}
