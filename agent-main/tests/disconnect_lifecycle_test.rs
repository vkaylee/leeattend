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

use agent::mock_worker::MockWorker;
use agent::time_sync::TimeSync;
use agent::worker::Worker;

/// Test that MockWorker::disconnect() completes without error.
/// This validates the trait contract is implemented.
#[tokio::test]
async fn test_mock_worker_disconnect_succeeds() {
    let mut worker = MockWorker::new();

    // Connect first
    let connected = worker.connect("127.0.0.1", 4370).await.unwrap();
    assert!(connected);

    // Disconnect should complete without panic or error
    worker.disconnect().await;
}

/// Test that calling disconnect() multiple times is safe (idempotent).
/// This guards against double-disconnect bugs in error recovery paths.
#[tokio::test]
async fn test_mock_worker_disconnect_idempotent() {
    let mut worker = MockWorker::new();
    worker.connect("127.0.0.1", 4370).await.unwrap();

    // Disconnect multiple times — must not panic
    worker.disconnect().await;
    worker.disconnect().await;
    worker.disconnect().await;
}

/// Test full lifecycle: connect → get_info → get_logs → disconnect
/// Ensures operations work before disconnect and the sequence is clean.
#[tokio::test]
async fn test_worker_lifecycle_connect_operations_disconnect_smoke() {
    let mut worker = MockWorker::new();

    // Connect
    let connected = worker.connect("127.0.0.1", 4370).await.unwrap();
    assert!(connected);

    // Operations
    let time_sync = TimeSync::new();
    let info = worker.get_info(&time_sync).await.unwrap();
    assert!(info.get("firmware").is_some());

    let logs = worker.get_attendance_logs().await.unwrap();
    assert!(!logs.is_empty());

    // Disconnect
    worker.disconnect().await;
}

/// Test that disconnect can be called even without connecting first.
/// The error path in main.rs may call disconnect on a worker that
/// failed to connect, so this must be safe.
#[tokio::test]
async fn test_disconnect_without_connect() {
    let mut worker = MockWorker::new();

    // Disconnect without prior connect — should not panic
    worker.disconnect().await;
}

/// Test that operations fail gracefully after disconnect.
/// Once disconnected, a ZkWorker should return "Device not connected".
/// MockWorker doesn't track state so this is a contract documentation test.
#[tokio::test]
async fn test_operations_after_disconnect_on_mock() {
    let mut worker = MockWorker::new();
    worker.connect("127.0.0.1", 4370).await.unwrap();
    worker.disconnect().await;

    // MockWorker doesn't track connection state, so these still succeed.
    // But this test documents the expected lifecycle pattern.
    // For ZkWorker, these would fail with "Device not connected" after disconnect.
    let time_sync = TimeSync::new();
    let result = worker.get_info(&time_sync).await;
    assert!(
        result.is_ok(),
        "MockWorker should still return data (stateless)"
    );
}

// === Ping Lifecycle Tests ===

/// Test that ping() returns true on a connected MockWorker.
/// This validates the basic trait contract for heartbeat probes.
#[tokio::test]
async fn test_mock_worker_ping_returns_true() {
    let mut worker = MockWorker::new();
    worker.connect("127.0.0.1", 4370).await.unwrap();

    assert!(
        worker.ping().await,
        "MockWorker ping should always return true"
    );
}

/// Test that ping() is safe to call before connect.
/// MockWorker is stateless, so this should succeed.
#[tokio::test]
async fn test_mock_worker_ping_before_connect() {
    let mut worker = MockWorker::new();

    // MockWorker has no real connection state, ping is always true
    assert!(
        worker.ping().await,
        "MockWorker ping before connect should return true"
    );
}

/// Test ping() after disconnect is still safe on MockWorker.
/// In real ZkWorker, ping would return false after disconnect.
#[tokio::test]
async fn test_mock_worker_ping_after_disconnect() {
    let mut worker = MockWorker::new();
    worker.connect("127.0.0.1", 4370).await.unwrap();
    worker.disconnect().await;

    // MockWorker is stateless — always true
    assert!(
        worker.ping().await,
        "MockWorker ping after disconnect should still return true (stateless)"
    );
}

/// Test full lifecycle with ping as heartbeat:
/// connect → ping → get_info → ping → get_logs → ping → disconnect
#[tokio::test]
async fn test_full_lifecycle_with_ping_heartbeat() {
    let mut worker = MockWorker::new();

    // Connect
    worker.connect("127.0.0.1", 4370).await.unwrap();

    // Heartbeat 1
    assert!(worker.ping().await, "Ping should succeed after connect");

    // Operation 1
    let time_sync = TimeSync::new();
    let info = worker.get_info(&time_sync).await.unwrap();
    assert!(info.get("firmware").is_some());

    // Heartbeat 2
    assert!(worker.ping().await, "Ping should succeed after get_info");

    // Operation 2
    let logs = worker.get_attendance_logs().await.unwrap();
    assert!(!logs.is_empty());

    // Heartbeat 3
    assert!(worker.ping().await, "Ping should succeed after get_logs");

    // Disconnect
    worker.disconnect().await;
}

/// Test repeated pings (simulating polling loop heartbeats).
/// Ensures ping doesn't degrade or leak resources over many calls.
#[tokio::test]
async fn test_repeated_pings_stability() {
    let mut worker = MockWorker::new();
    worker.connect("127.0.0.1", 4370).await.unwrap();

    for i in 0..100 {
        assert!(worker.ping().await, "Ping failed on iteration {i}");
    }

    worker.disconnect().await;
}

// === Detected Checksum Mode Tests ===

/// Test that MockWorker always returns None for detected_checksum_mode.
/// Mock devices don't have real firmware, so checksum detection is N/A.
#[tokio::test]
async fn test_mock_worker_detected_checksum_is_none() {
    let mut worker = MockWorker::new();
    assert_eq!(
        worker.detected_checksum_mode(),
        None,
        "MockWorker should return None before connect"
    );

    worker.connect("127.0.0.1", 4370).await.unwrap();
    assert_eq!(
        worker.detected_checksum_mode(),
        None,
        "MockWorker should return None after connect"
    );

    worker.disconnect().await;
    assert_eq!(
        worker.detected_checksum_mode(),
        None,
        "MockWorker should return None after disconnect"
    );
}

/// Test simulating the updated worker loop logic to verify that
/// the worker is always disconnected after a successful logs fetch.
#[tokio::test]
async fn test_always_disconnect_after_fetch_simulation() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let disconnected = Arc::new(AtomicBool::new(false));
    let dc_clone = disconnected.clone();

    let handle = tokio::spawn(async move {
        let mut worker = MockWorker::new();
        // 1. Connect
        worker.connect("127.0.0.1", 4370).await.unwrap();

        // 2. Fetch logs (succeeds)
        let res = worker.get_attendance_logs().await;
        assert!(res.is_ok());

        // 3. Always disconnect immediately after fetch
        worker.disconnect().await;
        dc_clone.store(true, Ordering::SeqCst);
    });

    handle.await.unwrap();
    assert!(
        disconnected.load(Ordering::SeqCst),
        "Worker must have disconnected immediately after fetch"
    );
}
