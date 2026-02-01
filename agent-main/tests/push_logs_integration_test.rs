use tokio::time::{sleep, Duration, timeout};
use serde_json::json;
use tokio::net::TcpListener;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use agent::database::Database;
use agent::hybrid_database::HybridDatabase;
use agent::central_client::CentralClient;
use agent::models::AttendanceData;
use log::info;
use uuid::Uuid;

/// Create an in-memory SQLite URL with shared cache for parallel test isolation
fn in_memory_db_url() -> String {
    format!("sqlite:file:{}?mode=memory&cache=shared", Uuid::new_v4())
}

#[tokio::test]
async fn test_end_to_end_log_push() {
    let _ = env_logger::builder().is_test(true).try_init();

    timeout(Duration::from_secs(30), async {
        // 1. Setup Mock Central Server (WebSocket)
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_addr = ws_listener.local_addr().unwrap();
        let ws_url = format!("ws://{}", ws_addr);

        // 2. Start Mock WS Handler
        let (log_tx, mut log_rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.expect("Failed to accept WS");
                // Wait for Handshake (first message)
                let _ = ws_stream.next().await;

                while let Some(Ok(msg)) = ws_stream.next().await {
                    if let Message::Text(text) = msg {
                        let val: serde_json::Value = serde_json::from_str(&text).unwrap();
                        if val["type"] == "attendance" {
                            let msg_id = val["msg_id"].clone();
                            let _ = log_tx.send(val).await;
                            // Send ACK back
                            let ack = json!({"type": "ack", "msg_id": msg_id, "status": "success"});
                            ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                        }
                    }
                }
            }
        });

        // 3. Setup Agent with in-memory DB (shared cache for connection pool)
        let db_url = in_memory_db_url();
        let db = Database::new(&db_url).await.unwrap();

        // 4. Mock device and log
        db.upsert_device("DEV1", "Test Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        let log_data = AttendanceData {
            user_id: "USER1".into(),
            timestamp: "2026-01-26 12:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        db.save_raw_attendance("DEV1", log_data).await.unwrap();

        // 5. Start CentralClient with HybridDatabase
        let hybrid_db = HybridDatabase::new(&db_url, 1000).await.unwrap();
        let (client, rx, _reload_rx) = CentralClient::new(ws_url, "AGENT1".into(), "TOKEN1".into(), hybrid_db);
        client.start(rx).await;

        // 6. Verify Mock Server received the log
        info!("Waiting for log on mock server...");
        let received_log = log_rx.recv().await.expect("Did not receive log on mock server");
        assert_eq!(received_log["agent_id"], "AGENT1");
        assert_eq!(received_log["device_id"], "DEV1");
        assert_eq!(received_log["data"]["user_id"], "USER1");

        // 7. Verify DB is marked as synced
        // Wait for potential sync loop (10s) + ACK processing
        // Actually, we can wait up to 12s
        sleep(Duration::from_secs(12)).await;
        let unsynced = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(unsynced.len(), 0, "Log should be marked as synced in local DB");

    }).await.expect("E2E Log Push Test Timed Out");
}

#[tokio::test]
async fn test_reliable_sync_with_server_error() {
    let _ = env_logger::builder().is_test(true).try_init();

    timeout(Duration::from_secs(30), async {
        // 1. Setup Mock Central Server
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_addr = ws_listener.local_addr().unwrap();
        let ws_url = format!("ws://{}", ws_addr);

        let (log_tx, mut log_rx) = tokio::sync::mpsc::channel(10);
        tokio::spawn(async move {
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.expect("Failed to accept WS");
                let _ = ws_stream.next().await; // Handshake

                let mut attempt = 0;
                while let Some(Ok(msg)) = ws_stream.next().await {
                    if let Message::Text(text) = msg {
                        let val: serde_json::Value = serde_json::from_str(&text).unwrap();
                        if val["type"] == "attendance" {
                            attempt += 1;
                            let msg_id = val["msg_id"].clone();
                            let _ = log_tx.send(val).await;

                            if attempt == 1 {
                                // First attempt: Return error
                                let ack = json!({"type": "ack", "msg_id": msg_id, "status": "error", "error": "buffer full"});
                                ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                            } else {
                                // Subsequent attempts: Return success
                                let ack = json!({"type": "ack", "msg_id": msg_id, "status": "success"});
                                ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                            }
                        }
                    }
                }
            }
        });

        // 2. Setup Agent with in-memory DB
        let db_url = in_memory_db_url();
        let db = Database::new(&db_url).await.unwrap();

        db.upsert_device("DEV_RETRY", "Retry Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.save_raw_attendance("DEV_RETRY", AttendanceData {
            user_id: "RETRY_USER".into(),
            timestamp: "2026-01-26 12:00:00".into(),
            status: Some(1),
            punch: Some(1),
        }).await.unwrap();

        let hybrid_db = HybridDatabase::new(&db_url, 1000).await.unwrap();
        let (client, rx, _reload_rx) = CentralClient::new(ws_url, "AGENT_RETRY".into(), "TOKEN_RETRY".into(), hybrid_db);
        client.start(rx).await;

        // 3. Verify Mock Server received the log TWICE (once for failed attempt, once for retry)
        info!("Waiting for first attempt...");
        let _ = log_rx.recv().await.expect("Did not receive first attempt");

        info!("Waiting for retry attempt...");
        let second = log_rx.recv().await.expect("Did not receive second attempt");
        assert_eq!(second["data"]["user_id"], "RETRY_USER");

        // 4. Verify DB is finally marked as synced
        // Wait for retry loop (continuous loop has 50ms delay if limit hit, but here we wait for next fetch)
        // With current logic count < limit (1 < 500) -> 10s sleep
        sleep(Duration::from_secs(12)).await;
        let unsynced = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(unsynced.len(), 0, "Log should be finally synced");

    }).await.expect("Reliable Sync Test Timed Out");
}
