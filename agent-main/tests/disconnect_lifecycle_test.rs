use agent::mock_worker::MockWorker;
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
async fn test_worker_lifecycle_connect_operations_disconnect() {
    let mut worker = MockWorker::new();
    
    // Connect
    let connected = worker.connect("127.0.0.1", 4370).await.unwrap();
    assert!(connected);
    
    // Operations
    let info = worker.get_info().await.unwrap();
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
    let result = worker.get_info().await;
    assert!(result.is_ok(), "MockWorker should still return data (stateless)");
}
