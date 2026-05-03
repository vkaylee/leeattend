// ── TECH_DEBT: Enterprise-grade lint upgrade (2026-05-03) ──
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]
#![allow(clippy::todo)]
#![allow(clippy::unimplemented)]
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::suspicious)]
#![allow(clippy::style)]
#![allow(clippy::correctness)]
#![allow(clippy::perf)]
#![allow(clippy::complexity)]
#![allow(clippy::clone_on_ref_ptr)]
#![allow(missing_docs)]
//! Integration tests for server time synchronization.
//!
//! These tests verify the end-to-end flow:
//!   1. heartbeat_ack message → TimeSync offset update
//!   2. corrected timestamps in device operations
//!   3. multi-sync convergence behavior
//!   4. thread-safety under concurrent access

use agent::central_client::message_handler::handle_incoming_text;
use agent::hybrid_database::HybridDatabase;
use agent::models::CentralMessage;
use agent::time_sync::TimeSync;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

type DeviceLocks = Arc<DashMap<String, std::sync::Arc<tokio::sync::Mutex<()>>>>;

fn setup_db() -> HybridDatabase {
    HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed")
}

fn setup_locks() -> DeviceLocks {
    Arc::new(DashMap::new())
}

// ─── heartbeat_ack → TimeSync offset ────────────────────────────

#[tokio::test]
async fn test_heartbeat_ack_updates_time_sync_offset() {
    let hybrid_db = setup_db();
    let locks = setup_locks();
    let (reload_tx, _reload_rx) = mpsc::channel(1);
    let (central_tx, _central_rx) = mpsc::channel(1);
    let time_sync = Arc::new(TimeSync::new());

    // Before sync: offset should be 0
    assert_eq!(time_sync.offset_ms(), 0);

    // Simulate server being 60 seconds ahead
    let server_time = (Utc::now() + Duration::seconds(60)).to_rfc3339();
    let ack_json = serde_json::json!({
        "type": "heartbeat_ack",
        "server_time": server_time
    })
    .to_string();

    handle_incoming_text(
        &ack_json,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    // After sync: offset should be ~60000ms (allow 1s tolerance)
    let offset = time_sync.offset_ms();
    assert!(
        offset > 59000 && offset < 61000,
        "Expected offset ~60000ms, got {offset}ms"
    );
}

#[tokio::test]
async fn test_heartbeat_ack_negative_offset() {
    let hybrid_db = setup_db();
    let locks = setup_locks();
    let (reload_tx, _reload_rx) = mpsc::channel(1);
    let (central_tx, _central_rx) = mpsc::channel(1);
    let time_sync = Arc::new(TimeSync::new());

    // Server is 30 seconds behind local clock
    let server_time = (Utc::now() - Duration::seconds(30)).to_rfc3339();
    let ack_json = serde_json::json!({
        "type": "heartbeat_ack",
        "server_time": server_time
    })
    .to_string();

    handle_incoming_text(
        &ack_json,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    let offset = time_sync.offset_ms();
    assert!(
        offset < -29000 && offset > -31000,
        "Expected offset ~-30000ms, got {offset}ms"
    );
}

#[tokio::test]
async fn test_heartbeat_ack_with_invalid_time_preserves_offset() {
    let hybrid_db = setup_db();
    let locks = setup_locks();
    let (reload_tx, _reload_rx) = mpsc::channel(1);
    let (central_tx, _central_rx) = mpsc::channel(1);
    let time_sync = Arc::new(TimeSync::new());

    // First: set a valid offset
    let server_time = (Utc::now() + Duration::seconds(10)).to_rfc3339();
    let ack_json = serde_json::json!({
        "type": "heartbeat_ack",
        "server_time": server_time
    })
    .to_string();
    handle_incoming_text(
        &ack_json,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    let offset_before = time_sync.offset_ms();
    assert!(
        offset_before > 9000,
        "Setup failed: offset was {offset_before}ms"
    );

    // Second: send invalid server_time → offset should NOT change
    let bad_ack = serde_json::json!({
        "type": "heartbeat_ack",
        "server_time": "not-a-valid-timestamp"
    })
    .to_string();
    handle_incoming_text(
        &bad_ack,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    let offset_after = time_sync.offset_ms();
    assert_eq!(
        offset_before, offset_after,
        "Invalid server_time should not change offset"
    );
}

// ─── corrected timestamps in device ops ─────────────────────────

#[tokio::test]
async fn test_device_upsert_uses_corrected_timestamp() {
    let hybrid_db = setup_db();
    let time_sync = Arc::new(TimeSync::new());

    // Simulate server 5 minutes ahead
    let server_time = (Utc::now() + Duration::minutes(5)).to_rfc3339();
    time_sync.update_offset(&server_time);

    let corrected_ts = time_sync.corrected_timestamp();
    let raw_ts = Utc::now().timestamp();

    // corrected should be ~300s ahead of raw
    let diff = corrected_ts - raw_ts;
    assert!(
        (299..=301).contains(&diff),
        "Expected corrected_timestamp ~300s ahead, got {diff}s"
    );

    // Use corrected timestamp in upsert
    hybrid_db
        .store()
        .upsert_device(
            "dev_ts_test",
            "TS Test Device",
            "1.2.3.4",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            corrected_ts,
        )
        .unwrap();

    let devices = hybrid_db.store().get_active_devices().unwrap();
    let dev = devices.iter().find(|d| d.id == "dev_ts_test").unwrap();
    assert_eq!(dev.last_seen, Some(corrected_ts));
}

#[tokio::test]
async fn test_device_metadata_update_uses_corrected_timestamp() {
    let hybrid_db = setup_db();
    let time_sync = Arc::new(TimeSync::new());

    // Simulate server 2 minutes behind
    let server_time = (Utc::now() - Duration::minutes(2)).to_rfc3339();
    time_sync.update_offset(&server_time);

    let corrected_ts = time_sync.corrected_timestamp();

    hybrid_db
        .store()
        .upsert_device(
            "dev_meta_ts",
            "Meta TS Device",
            "1.2.3.4",
            4370,
            "zkteco",
            "UTC",
            None,
            true,
            corrected_ts,
        )
        .unwrap();

    hybrid_db
        .store()
        .update_device_metadata(
            "dev_meta_ts",
            "AA:BB:CC:DD:EE:FF",
            "SN001",
            "v2.0",
            "Linux",
            "xFace200",
            200,
            8000,
            2000,
            50000,
            "2026-03-08 12:00:00",
            corrected_ts,
        )
        .unwrap();

    let meta = hybrid_db
        .store()
        .get_device_metadata("dev_meta_ts")
        .unwrap()
        .unwrap();
    assert_eq!(meta.last_info_update, Some(corrected_ts));

    // Verify that corrected_ts is indeed ~120s behind raw Utc::now()
    let raw_ts = Utc::now().timestamp();
    let diff = raw_ts - corrected_ts;
    assert!(
        (119..=121).contains(&diff),
        "Expected corrected_ts ~120s behind raw, got {diff}s"
    );
}

// ─── config_push uses corrected time for upsert ─────────────────

#[tokio::test]
async fn test_config_push_uses_time_sync_for_device_upsert() {
    let hybrid_db = setup_db();
    let locks = setup_locks();
    let (reload_tx, _reload_rx) = mpsc::channel(1);
    let (central_tx, _central_rx) = mpsc::channel(1);
    let time_sync = Arc::new(TimeSync::new());

    // Simulate server 3 minutes ahead
    let server_time = (Utc::now() + Duration::minutes(3)).to_rfc3339();
    time_sync.update_offset(&server_time);

    let config_json = serde_json::json!({
        "type": "config_push",
        "data": {
            "devices": [{
                "device_id": "dev_push_ts",
                "name": "Push TS Device",
                "ip": "10.0.0.1",
                "port": 4370,
                "brand": "zkteco",
                "timezone": "UTC",
                "is_enabled": true
            }]
        }
    })
    .to_string();

    handle_incoming_text(
        &config_json,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    let devices = hybrid_db.store().get_active_devices().unwrap();
    let dev = devices.iter().find(|d| d.id == "dev_push_ts").unwrap();

    // The device's last_seen should use corrected time (~180s ahead of raw)
    let raw_ts = Utc::now().timestamp();
    let diff = dev.last_seen.unwrap() - raw_ts;
    assert!(
        (179..=181).contains(&diff),
        "Expected config_push device last_seen ~180s ahead, got {diff}s"
    );
}

// ─── multi-sync convergence ─────────────────────────────────────

#[tokio::test]
async fn test_multiple_heartbeat_acks_converge_offset() {
    let hybrid_db = setup_db();
    let locks = setup_locks();
    let (reload_tx, _reload_rx) = mpsc::channel(1);
    let (central_tx, _central_rx) = mpsc::channel(1);
    let time_sync = Arc::new(TimeSync::new());

    // First sync: server 5 minutes ahead
    let ack1 = serde_json::json!({
        "type": "heartbeat_ack",
        "server_time": (Utc::now() + Duration::minutes(5)).to_rfc3339()
    })
    .to_string();
    handle_incoming_text(
        &ack1,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    let offset1 = time_sync.offset_ms();
    assert!(
        offset1 > 290000,
        "First offset should be ~300000ms, got {offset1}ms"
    );

    // Second sync: server corrects to only 10 seconds ahead
    let ack2 = serde_json::json!({
        "type": "heartbeat_ack",
        "server_time": (Utc::now() + Duration::seconds(10)).to_rfc3339()
    })
    .to_string();
    handle_incoming_text(
        &ack2,
        &hybrid_db,
        &reload_tx,
        &central_tx,
        &locks,
        &time_sync,
    )
    .await;

    let offset2 = time_sync.offset_ms();
    assert!(
        offset2 > 9000 && offset2 < 11000,
        "Second offset should converge to ~10000ms, got {offset2}ms"
    );

    // Verify corrected_now is close to simulated server time
    let corrected = time_sync.corrected_timestamp();
    let expected = Utc::now().timestamp() + 10;
    let diff = (corrected - expected).abs();
    assert!(
        diff <= 1,
        "corrected_timestamp should be ~10s ahead, diff was {diff}s"
    );
}

// ─── thread-safety ──────────────────────────────────────────────

#[tokio::test]
async fn test_concurrent_heartbeat_acks_are_safe() {
    let time_sync = Arc::new(TimeSync::new());

    let mut handles = vec![];
    for i in 0..10 {
        let ts = time_sync.clone();
        let handle = tokio::spawn(async move {
            let server_time = (Utc::now() + Duration::seconds(i * 10)).to_rfc3339();
            ts.update_offset(&server_time);
            ts.corrected_now();
            ts.corrected_timestamp();
        });
        handles.push(handle);
    }

    for h in handles {
        h.await.unwrap();
    }

    // Just verify no panics — the final offset will be from whichever task ran last
    let offset = time_sync.offset_ms();
    assert!(
        (0..=91000).contains(&offset),
        "Offset should be within expected range, got {offset}ms"
    );
}

// ─── CentralMessage deserialization ─────────────────────────────

#[test]
fn test_heartbeat_ack_deserialization() {
    let json = r#"{"type":"heartbeat_ack","server_time":"2026-03-08T12:00:00Z"}"#;
    let msg: CentralMessage = serde_json::from_str(json).unwrap();
    match msg {
        CentralMessage::HeartbeatAck { server_time } => {
            assert_eq!(server_time, "2026-03-08T12:00:00Z");
        }
        other => panic!("Expected HeartbeatAck, got {:?}", other),
    }
}

#[test]
fn test_heartbeat_ack_with_timezone_offset() {
    let json = r#"{"type":"heartbeat_ack","server_time":"2026-03-08T19:00:00+07:00"}"#;
    let msg: CentralMessage = serde_json::from_str(json).unwrap();
    match msg {
        CentralMessage::HeartbeatAck { server_time } => {
            assert_eq!(server_time, "2026-03-08T19:00:00+07:00");
        }
        other => panic!("Expected HeartbeatAck, got {:?}", other),
    }
}

// ─── edge case: zero offset still works ─────────────────────────

#[tokio::test]
async fn test_zero_offset_passthrough() {
    let time_sync = Arc::new(TimeSync::new());
    assert_eq!(time_sync.offset_ms(), 0);

    // With zero offset, corrected times should match raw times (±1s)
    let raw_ts = Utc::now().timestamp();
    let corrected_ts = time_sync.corrected_timestamp();
    let diff = (corrected_ts - raw_ts).abs();
    assert!(
        diff <= 1,
        "Zero-offset corrected should match raw, diff was {diff}s"
    );
}
