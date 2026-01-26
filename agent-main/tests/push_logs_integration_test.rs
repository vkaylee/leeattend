use tokio::time::{sleep, Duration, timeout};
use serde_json::json;
use tokio::net::TcpListener;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use agent::database::Database;
use agent::central_client::CentralClient;
use agent::models::AttendanceData;
use log::info;

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
                            let ack = json!({"type": "ack", "msg_id": msg_id});
                            ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                        }
                    }
                }
            }
        });

        // 3. Setup Agent with temporary DB
        let db_file = tempfile::NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", db_file.path().to_str().unwrap());
        let db = Database::new(&db_url).await.unwrap();
        
        sqlx::migrate!("./migrations")
            .run(db.get_pool())
            .await
            .expect("Failed to run migrations");

        // 4. Mock device and log
        db.upsert_device("DEV1", None, "127.0.0.1", 4370).await.unwrap();
        let log_data = AttendanceData {
            user_id: "USER1".into(),
            timestamp: "2026-01-26 12:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        db.save_raw_attendance("DEV1", log_data).await.unwrap();

        // 5. Start CentralClient
        let (client, rx) = CentralClient::new(ws_url, "AGENT1".into(), "TOKEN1".into(), db.clone());
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
