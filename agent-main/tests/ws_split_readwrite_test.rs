use tokio::time::{sleep, Duration, timeout};
use serde_json::json;
use tokio::net::TcpListener;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use agent::database::Database;
use agent::hybrid_database::HybridDatabase;
use agent::central_client::CentralClient;
use agent::models::AttendanceData;
use uuid::Uuid;
use std::sync::Arc;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Create an in-memory SQLite URL with shared cache for parallel test isolation
fn in_memory_db_url() -> String {
    format!("sqlite:file:{}?mode=memory&cache=shared", Uuid::new_v4())
}

/// Verify ACKs are processed concurrently with bulk sends (the main bug fix).
/// Before the fix, the select! loop starved the read branch when many outgoing
/// messages were queued, causing ACKs to never be received.
#[tokio::test]
async fn test_concurrent_ack_processing_during_bulk_send() {
    let _ = env_logger::builder().is_test(true).try_init();

    timeout(Duration::from_secs(30), async {
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_url = format!("ws://{}", ws_listener.local_addr().unwrap());

        let ack_count = Arc::new(AtomicUsize::new(0));
        let ack_count_server = ack_count.clone();

        // Mock server: immediately ACK every attendance log
        tokio::spawn(async move {
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                let _ = ws_stream.next().await; // Handshake

                while let Some(Ok(msg)) = ws_stream.next().await {
                    if let Message::Text(text) = msg
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
                        && val["type"] == "attendance" {
                                let ack = json!({
                                    "type": "ack",
                                    "msg_id": val["msg_id"],
                                    "status": "success"
                                });
                                ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                                ack_count_server.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        // Setup agent with 20 logs (enough to test concurrent behavior)
        let db_url = in_memory_db_url();
        let db = Database::new(&db_url).await.unwrap();
        db.upsert_device("DEV_BULK", "Bulk Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();

        for i in 0..20 {
            db.save_raw_attendance("DEV_BULK", AttendanceData {
                user_id: format!("USER_{i}"),
                timestamp: format!("2026-01-26 12:{i:02}:00"),
                status: Some(1),
                punch: Some(1),
            }).await.unwrap();
        }

        let hybrid_db = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();
        hybrid_db.load_device_memory("DEV_BULK").await.unwrap();
        let hybrid_check = hybrid_db.clone();

        let (client, rx, _reload_rx) = CentralClient::new(ws_url, "AGENT_BULK".into(), "TOKEN".into(), hybrid_db, 300, Arc::new(DashMap::new()));
        client.start(rx).await;

        // Wait for all ACKs to be processed (server counter confirms delivery)
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(500)).await;
            if ack_count.load(Ordering::Relaxed) >= 20 { break; }
            attempts += 1;
            assert!(attempts < 30, "Timed out waiting for ACKs: server only received {}", ack_count.load(Ordering::Relaxed));
        }

        // Give ACKs time to propagate back to agent memory
        sleep(Duration::from_secs(1)).await;
        let unsynced = hybrid_check.count_unsynced_logs();
        assert_eq!(unsynced, 0, "All logs should be marked synced in memory, got {unsynced} unsynced");
    }).await.expect("Concurrent ACK test timed out");
}

/// Verify agent reconnects after server closes the WebSocket.
/// The rx receiver must be preserved across reconnections via graceful shutdown.
#[tokio::test]
async fn test_reconnect_after_server_close() {
    let _ = env_logger::builder().is_test(true).try_init();

    timeout(Duration::from_secs(30), async {
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_url = format!("ws://{}", ws_listener.local_addr().unwrap());

        let connection_count = Arc::new(AtomicUsize::new(0));
        let conn_count_server = connection_count.clone();
        let ack_sent = Arc::new(AtomicUsize::new(0));
        let ack_sent_clone = ack_sent.clone();

        // Mock server: accept first connection, close it, then accept second and ACK
        tokio::spawn(async move {
            // First connection: accept handshake then close
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                let _ = ws_stream.next().await; // Handshake
                conn_count_server.fetch_add(1, Ordering::Relaxed);
                // Close immediately to simulate server restart
                ws_stream.close(None).await.ok();
            }

            // Second connection: full operation
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                let _ = ws_stream.next().await; // Handshake
                conn_count_server.fetch_add(1, Ordering::Relaxed);

                while let Some(Ok(msg)) = ws_stream.next().await {
                    if let Message::Text(text) = msg
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
                        && val["type"] == "attendance" {
                                let ack = json!({
                                    "type": "ack",
                                    "msg_id": val["msg_id"],
                                    "status": "success"
                                });
                                ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                                ack_sent_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        // Setup agent with 1 log
        let db_url = in_memory_db_url();
        let db = Database::new(&db_url).await.unwrap();
        db.upsert_device("DEV_RC", "Reconnect Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.save_raw_attendance("DEV_RC", AttendanceData {
            user_id: "USER_RC".into(),
            timestamp: "2026-01-26 12:00:00".into(),
            status: Some(1),
            punch: Some(1),
        }).await.unwrap();

        let hybrid_db = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();
        hybrid_db.load_device_memory("DEV_RC").await.unwrap();

        let (client, rx, _reload_rx) = CentralClient::new(ws_url, "AGENT_RC".into(), "TOKEN".into(), hybrid_db, 300, Arc::new(DashMap::new()));
        client.start(rx).await;

        // Wait for the log to be ACKed on second connection
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(500)).await;
            if ack_sent.load(Ordering::Relaxed) >= 1 { break; }
            attempts += 1;
            assert!(attempts < 40, "Timed out waiting for reconnect sync");
        }

        // Agent should have connected twice (first closed, second successful)
        assert!(connection_count.load(Ordering::Relaxed) >= 2, "Agent should reconnect after server close");
    }).await.expect("Reconnect test timed out");
}

/// Verify connection failure preserves rx for retry.
#[tokio::test]
async fn test_connection_failure_preserves_receiver() {
    let _ = env_logger::builder().is_test(true).try_init();

    timeout(Duration::from_secs(30), async {
        // Bind then close so the port is unused and connection will fail
        let failing_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let failing_addr = failing_listener.local_addr().unwrap();
        drop(failing_listener);

        let db_url = in_memory_db_url();
        let db = Database::new(&db_url).await.unwrap();
        db.upsert_device("DEV_FAIL", "Fail Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();
        db.save_raw_attendance("DEV_FAIL", AttendanceData {
            user_id: "USER_FAIL".into(),
            timestamp: "2026-01-26 12:00:00".into(),
            status: Some(1),
            punch: Some(1),
        }).await.unwrap();

        let hybrid_db = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();
        hybrid_db.load_device_memory("DEV_FAIL").await.unwrap();
        let hybrid_check = hybrid_db.clone();

        let ws_url = format!("ws://{failing_addr}");
        let (client, rx, _reload_rx) = CentralClient::new(ws_url, "AGENT_FAIL".into(), "TOKEN".into(), hybrid_db, 300, Arc::new(DashMap::new()));
        client.start(rx).await;

        // Let it fail a few connection attempts
        sleep(Duration::from_secs(3)).await;

        // Logs should still be pending (rx not lost, agent still running)
        let unsynced = hybrid_check.count_unsynced_logs();
        assert!(unsynced > 0, "Logs should still be pending after connection failure");
    }).await.expect("Connection failure test timed out");
}

/// Verify multiple messages are sent AND acknowledged without starvation.
/// Uses server-side counter to confirm all logs are processed.
#[tokio::test]
async fn test_no_ack_starvation_under_load() {
    let _ = env_logger::builder().is_test(true).try_init();

    timeout(Duration::from_secs(30), async {
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_url = format!("ws://{}", ws_listener.local_addr().unwrap());

        let total_acked = Arc::new(AtomicUsize::new(0));
        let total_acked_server = total_acked.clone();

        // Mock server: add a small delay per ACK to simulate real server processing
        tokio::spawn(async move {
            if let Ok((stream, _)) = ws_listener.accept().await {
                let mut ws_stream = accept_async(stream).await.unwrap();
                let _ = ws_stream.next().await; // Handshake

                while let Some(Ok(msg)) = ws_stream.next().await {
                    if let Message::Text(text) = msg
                        && let Ok(val) = serde_json::from_str::<serde_json::Value>(&text)
                        && val["type"] == "attendance" {
                                // Simulate server processing time
                                sleep(Duration::from_millis(2)).await;
                                let ack = json!({
                                    "type": "ack",
                                    "msg_id": val["msg_id"],
                                    "status": "success"
                                });
                                ws_stream.send(Message::Text(ack.to_string().into())).await.unwrap();
                                total_acked_server.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

        // Setup agent with 50 logs for load testing
        let db_url = in_memory_db_url();
        let db = Database::new(&db_url).await.unwrap();
        db.upsert_device("DEV_LOAD", "Load Device", "127.0.0.1", 4370, "zkteco", "UTC", true).await.unwrap();

        for i in 0..50 {
            db.save_raw_attendance("DEV_LOAD", AttendanceData {
                user_id: format!("LOAD_USER_{i}"),
                timestamp: format!("2026-01-26 {}:{:02}:00", 10 + i / 60, i % 60),
                status: Some(1),
                punch: Some(1),
            }).await.unwrap();
        }

        let hybrid_db = HybridDatabase::new(&db_url, 1000, 90).await.unwrap();
        hybrid_db.load_device_memory("DEV_LOAD").await.unwrap();

        let (client, rx, _reload_rx) = CentralClient::new(ws_url, "AGENT_LOAD".into(), "TOKEN".into(), hybrid_db, 300, Arc::new(DashMap::new()));
        client.start(rx).await;

        // Wait for server to ACK all 50 logs
        let mut attempts = 0;
        loop {
            sleep(Duration::from_millis(500)).await;
            let acked = total_acked.load(Ordering::Relaxed);
            if acked >= 50 { break; }
            attempts += 1;
            assert!(attempts < 40, "ACK starvation detected: server only processed {acked}/50 after 20s");
        }

        assert_eq!(total_acked.load(Ordering::Relaxed), 50, "All 50 logs should be processed");
    }).await.expect("ACK starvation test timed out");
}
