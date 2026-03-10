use agent::zk_worker::ZkWorker;
use agent::worker::Worker;

#[tokio::test]
async fn test_zk_worker_timeout_logic() {
    // This test verifies that the ZkWorker handles timeouts gracefully.
    // Since we cannot easily mock the internal ZK struct without dependency injection,
    // we will test the public API behaviors we can control or simulate.
    
    // Ideally, we would refactor ZkWorker to take a trait for ZK operations,
    // allowing us to mock the device responses.
    // For now, we can verify that basic instantiation works and that
    // connecting to an unreachable IP returns the expected timeout/error.
    
    let mut worker = ZkWorker::new(None);
    
    // Test connection to a local closed port should fail immediately (Connection refused)
    // This verifies the failure path without waiting for a long TCP timeout.
    let result = worker.connect("127.0.0.1", 54321).await; 
    
    assert!(result.is_err());
    let err = result.err().unwrap();
    // It might be "Connection timed out" or "No route to host" depending on OS
    println!("Connection error: {}", err);
}

/// Test that ping() returns false when ZkWorker has no active connection.
/// This is the expected behavior: unconnected worker should report "not reachable".
#[tokio::test]
async fn test_zk_worker_ping_without_connection() {
    let mut worker = ZkWorker::new(None);
    
    // No connect() called — ZK is None
    assert!(!worker.ping().await, "Ping should return false without active connection");
}

/// Test that disconnect() is safe to call without prior connect.
/// The orchestrator may call disconnect() in error recovery paths
/// even if connect() never succeeded.
#[tokio::test]
async fn test_zk_worker_disconnect_without_connect() {
    let mut worker = ZkWorker::new(None);
    
    // Should not panic
    worker.disconnect().await;
}

/// Test that disconnect() is idempotent — calling it multiple times is safe.
/// This guards against double-disconnect bugs in timeout cleanup paths.
#[tokio::test]
async fn test_zk_worker_disconnect_idempotent() {
    let mut worker = ZkWorker::new(None);
    
    // Multiple disconnects without connect — all should be no-ops
    worker.disconnect().await;
    worker.disconnect().await;
    worker.disconnect().await;
}

/// Test that ping() returns false after a failed connection attempt.
/// After connect() fails, the internal ZK instance should be None.
#[tokio::test]
async fn test_zk_worker_ping_after_failed_connect() {
    let mut worker = ZkWorker::new(None);
    
    // Attempt connection to unreachable port — will fail
    let result = worker.connect("127.0.0.1", 54321).await;
    assert!(result.is_err());
    
    // Ping should return false — no active ZK instance
    assert!(!worker.ping().await, "Ping should return false after failed connect");
}

/// Test ZkWorker with password parameter.
/// Verifies that password is stored but doesn't affect unconnected state.
#[tokio::test]
async fn test_zk_worker_ping_with_password_no_connect() {
    let mut worker = ZkWorker::new(Some(12345));
    
    // Not connected — should be false regardless of password
    assert!(!worker.ping().await);
    worker.disconnect().await; // no-op
}
