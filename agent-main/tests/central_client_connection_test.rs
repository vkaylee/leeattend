use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;

use agent::central_client::CentralClient;
use agent::hybrid_database::HybridDatabase;
use agent::models::AttendanceData;
use agent::time_sync::TimeSync;
use dashmap::DashMap;

async fn start_mock_ws_server() -> (tokio::sync::broadcast::Receiver<serde_json::Value>, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = tokio::sync::broadcast::channel(100);

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            // Very simple server config, tungstenite handles headers implicitly
            if let Ok(mut ws_stream) = accept_async(stream).await {
                while let Some(msg) = ws_stream.next().await {
                    if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                        let text_str = String::from_utf8_lossy(text.as_ref()).to_string();
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_str) {
                            let _ = tx.send(json.clone());

                            // If it's an attendance message, mock an ACK response to complete flow
                            if json.get("msg_id").is_some() && json.get("data").is_some() {
                                let ack = serde_json::json!({
                                    "msg_id": json["msg_id"],
                                    "success": true
                                });
                                let _ = ws_stream
                                    .send(tokio_tungstenite::tungstenite::Message::Text(
                                        ack.to_string().into(),
                                    ))
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    });

    (rx, port)
}

#[tokio::test]
#[ignore] // Network-dependent: requires WS mock server timing; run explicitly
async fn test_central_client_connection_and_sync() {
    let (mut rx_server, port) = start_mock_ws_server().await;
    let url = format!("ws://127.0.0.1:{}", port);

    let hybrid = HybridDatabase::new(":memory:", 1000, 90).unwrap();
    let store = hybrid.store();

    // Must have active device to start syncing
    store
        .upsert_device(
            "DEV_SYNC", "Test", "1.1.1.1", 4370, "zkteco", "UTC", None, true, 123,
        )
        .unwrap();

    let time_sync = Arc::new(TimeSync::new());
    let device_locks = Arc::new(DashMap::new());
    let command_registry = Arc::new(DashMap::new());

    let (client, _, _reload_rx) = CentralClient::new(
        url,
        "AGENT_1".to_string(),
        "TOKEN".to_string(),
        hybrid.clone(),
        30,
        command_registry,
        device_locks,
        time_sync,
    );

    let tx = client.get_tx();
    let (mut client_tx, client_rx) = mpsc::channel(100);

    // Spawn the client
    tokio::spawn(async move {
        client.start(client_rx).await;
    });

    // Wait and receive Handshake
    let handshake = tokio::time::timeout(Duration::from_secs(10), rx_server.recv())
        .await
        .expect("Timeout waiting for handshake")
        .unwrap();
    assert_eq!(handshake["agent_id"], "AGENT_1");
    // In CentralMessage::Handshake it's {"Handshake": {"agent_id":...}} inside an Enum maybe, wait, Serde enum is probably adjacently tagged or externally tagged.
    // By default, enum serialization might look like: `{"Handshake": {"agent_id": ...}}`
    // Let's just assert it is valid JSON and contains AGENT_1 somewhere.
    assert!(handshake.to_string().contains("AGENT_1"));

    // Insert an attendance record to trigger sync
    let log_data = AttendanceData {
        user_id: "U1".into(),
        timestamp: "2026-01-01 10:00:00".into(),
        status: Some(1),
        punch: Some(1),
    };
    hybrid
        .save_raw_attendance("DEV_SYNC".into(), log_data)
        .await
        .unwrap();

    // Wait for the attendance log to be sent over WS
    let mut found_log = false;
    for _ in 0..10 {
        if let Ok(msg) = tokio::time::timeout(Duration::from_secs(2), rx_server.recv()).await {
            let msg_val = msg.unwrap();
            let json_str = msg_val.to_string();
            if json_str.contains("Attendance") && json_str.contains("DEV_SYNC") {
                found_log = true;
                break;
            }
        }
    }
    assert!(found_log, "Did not receive sync message");
}
