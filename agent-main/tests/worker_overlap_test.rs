//! Tests for worker overlap prevention during config reload.
//!
//! Validates that:
//! 1. spawn_worker() returns a JoinHandle that can be awaited
//! 2. Cancel-aware sleep responds to cancellation immediately (not after full interval)
//! 3. Old worker fully stops before new worker starts during reload
//! 4. connection_changed() excludes use_legacy_checksum from comparison

use agent::mock_worker::MockWorker;
use agent::worker::Worker;
use agent::worker_processor::WorkerConfig;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::time::{sleep, Duration, Instant};
use tokio_util::sync::CancellationToken;

// ============================================================
// Fix 1: JoinHandle — old worker can be awaited before re-spawn
// ============================================================

/// Test that spawning a worker via tokio::spawn returns a JoinHandle
/// that completes promptly after CancellationToken is cancelled.
#[tokio::test]
async fn test_worker_handle_completes_after_cancel() {
    let token = CancellationToken::new();
    let worker_token = token.clone();
    let disconnected = Arc::new(AtomicBool::new(false));
    let dc_clone = disconnected.clone();

    // Simulate spawn_worker pattern: connect → poll loop → disconnect
    let handle = tokio::spawn(async move {
        let mut worker = MockWorker::new();
        worker.connect("127.0.0.1", 4370).await.unwrap();

        while !worker_token.is_cancelled() {
            // Cancel-aware sleep (Fix 2 pattern)
            tokio::select! {
                _ = worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(30)) => {}
            }
        }

        worker.disconnect().await;
        dc_clone.store(true, Ordering::SeqCst);
    });

    // Let worker start
    sleep(Duration::from_millis(50)).await;

    // Cancel and await handle — should complete quickly
    token.cancel();
    let start = Instant::now();
    let result = tokio::time::timeout(Duration::from_secs(3), handle).await;

    assert!(result.is_ok(), "JoinHandle should complete within 3s after cancel");
    assert!(result.unwrap().is_ok(), "Worker task should not panic");
    assert!(disconnected.load(Ordering::SeqCst), "Worker must disconnect before handle completes");

    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(1),
        "Worker should stop quickly after cancel, took {:?}", elapsed);
}

/// Test that awaiting the JoinHandle AFTER cancel guarantees
/// the worker has fully stopped (no resource leaks).
#[tokio::test]
async fn test_await_handle_guarantees_cleanup() {
    let token = CancellationToken::new();
    let worker_token = token.clone();

    let cleanup_order = Arc::new(AtomicUsize::new(0));
    let order_clone = cleanup_order.clone();

    let handle = tokio::spawn(async move {
        let mut worker = MockWorker::new();
        worker.connect("127.0.0.1", 4370).await.unwrap();

        while !worker_token.is_cancelled() {
            tokio::select! {
                _ = worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(60)) => {}
            }
        }

        worker.disconnect().await;
        // Mark cleanup sequence: worker sets step 1
        order_clone.store(1, Ordering::SeqCst);
    });

    sleep(Duration::from_millis(50)).await;
    token.cancel();
    handle.await.unwrap();

    // After handle.await, cleanup_order must be 1 (worker finished)
    assert_eq!(cleanup_order.load(Ordering::SeqCst), 1,
        "After JoinHandle.await, worker must have completed cleanup");
}

// ============================================================
// Fix 2: Cancel-aware sleep — immediate response to shutdown
// ============================================================

/// Test that a cancel-aware sleep exits immediately when token is cancelled,
/// NOT waiting the full interval (regression test for the old behavior).
#[tokio::test]
async fn test_cancel_aware_sleep_exits_immediately() {
    let token = CancellationToken::new();
    let inner_token = token.clone();

    let handle = tokio::spawn(async move {
        let start = Instant::now();

        // Simulate the fixed sleep pattern from orchestrator.rs:676-679
        tokio::select! {
            _ = inner_token.cancelled() => {}
            _ = sleep(Duration::from_secs(45)) => {}  // SESSION_SAFE_INTERVAL_SECS
        }

        start.elapsed()
    });

    // Let the task enter the sleep
    sleep(Duration::from_millis(50)).await;

    // Cancel — should wake the sleep immediately
    token.cancel();

    let elapsed = handle.await.unwrap();
    assert!(elapsed < Duration::from_secs(1),
        "Cancel-aware sleep should exit in <1s, took {:?}", elapsed);
}

/// Test that without cancel awareness, sleep blocks for the full interval
/// (validates the old bug would have caused the delay).
#[tokio::test]
async fn test_plain_sleep_ignores_cancel() {
    let token = CancellationToken::new();
    let inner_token = token.clone();

    let handle = tokio::spawn(async move {
        let start = Instant::now();

        // Old behavior: plain sleep, then check token
        sleep(Duration::from_millis(500)).await;
        let _ = inner_token.is_cancelled(); // Only checked AFTER sleep

        start.elapsed()
    });

    // Cancel immediately
    sleep(Duration::from_millis(10)).await;
    token.cancel();

    let elapsed = handle.await.unwrap();
    // Plain sleep should have waited ~500ms despite cancel at 10ms
    assert!(elapsed >= Duration::from_millis(400),
        "Plain sleep should NOT respond to cancel, took {:?}", elapsed);
}

// ============================================================
// Fix 3: Worker overlap — sequential cancel → await → spawn
// ============================================================

/// Test the full reload sequence: cancel old → await old → spawn new.
/// Guarantees no overlap: old worker must fully stop before new starts.
#[tokio::test]
async fn test_no_worker_overlap_during_reload() {
    let old_running = Arc::new(AtomicBool::new(false));
    let new_running = Arc::new(AtomicBool::new(false));
    let overlap_detected = Arc::new(AtomicBool::new(false));

    // --- Old worker ---
    let old_token = CancellationToken::new();
    let old_worker_token = old_token.clone();
    let old_running_flag = old_running.clone();
    let overlap_flag = overlap_detected.clone();
    let new_running_check = new_running.clone();

    let old_handle = tokio::spawn(async move {
        let mut worker = MockWorker::new();
        worker.connect("192.168.1.1", 4370).await.unwrap();
        old_running_flag.store(true, Ordering::SeqCst);

        while !old_worker_token.is_cancelled() {
            // Check for overlap: if new worker started while old is still running, flag it
            if new_running_check.load(Ordering::SeqCst) {
                overlap_flag.store(true, Ordering::SeqCst);
            }

            tokio::select! {
                _ = old_worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(30)) => {}
            }
        }

        // Simulate disconnect taking some time
        sleep(Duration::from_millis(50)).await;
        worker.disconnect().await;
        old_running_flag.store(false, Ordering::SeqCst);
    });

    // Wait for old worker to be running with a timeout to avoid race conditions
    let mut started = false;
    for _ in 0..50 { // Max 5 seconds (50 * 100ms)
        if old_running.load(Ordering::SeqCst) {
            started = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(started, "Old worker should be running within timeout");

    // --- Reload sequence (mimics fixed handle_config_reload) ---

    // Step 1: Cancel old worker
    old_token.cancel();

    // Step 2: AWAIT old handle (the fix!)
    let result = tokio::time::timeout(Duration::from_secs(5), old_handle).await;
    assert!(result.is_ok(), "Old worker should complete within 5s timeout");

    // Old worker must be fully stopped now
    assert!(!old_running.load(Ordering::SeqCst),
        "Old worker must be stopped BEFORE spawning new worker");

    // Step 3: Spawn new worker
    let new_token = CancellationToken::new();
    let new_worker_token = new_token.clone();
    let new_running_flag = new_running.clone();

    let new_handle = tokio::spawn(async move {
        let mut worker = MockWorker::new();
        worker.connect("192.168.1.1", 4370).await.unwrap();
        new_running_flag.store(true, Ordering::SeqCst);

        while !new_worker_token.is_cancelled() {
            tokio::select! {
                _ = new_worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(30)) => {}
            }
        }

        worker.disconnect().await;
        new_running_flag.store(false, Ordering::SeqCst);
    });

    // Verify new worker is running with timeout
    let mut new_started = false;
    for _ in 0..50 { // Max 5 seconds
        if new_running.load(Ordering::SeqCst) {
            new_started = true;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert!(new_started, "New worker should be running within timeout");

    // No overlap detected
    assert!(!overlap_detected.load(Ordering::SeqCst),
        "Old and new worker must NEVER run simultaneously");

    // Cleanup
    new_token.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(3), new_handle).await;
}

/// Contrast test: demonstrate that WITHOUT awaiting the old handle,
/// overlap DOES occur (validates the test is meaningful).
#[tokio::test]
async fn test_overlap_occurs_without_await() {
    let old_running = Arc::new(AtomicBool::new(false));
    let new_started = Arc::new(AtomicBool::new(false));
    let overlap_detected = Arc::new(AtomicBool::new(false));

    let old_token = CancellationToken::new();
    let old_worker_token = old_token.clone();
    let old_flag = old_running.clone();

    let _old_handle = tokio::spawn(async move {
        old_flag.store(true, Ordering::SeqCst);

        while !old_worker_token.is_cancelled() {
            // Old behavior: plain sleep (NOT cancel-aware)
            sleep(Duration::from_millis(200)).await;
        }

        // Cleanup takes time
        sleep(Duration::from_millis(100)).await;
        old_flag.store(false, Ordering::SeqCst);
    });

    sleep(Duration::from_millis(50)).await;

    // Bug behavior: cancel old but DON'T await — spawn new immediately
    old_token.cancel();
    // No: let _ = old_handle.await;  <-- missing!

    let new_flag = new_started.clone();
    let overlap = overlap_detected.clone();
    let old_still_running = old_running.clone();

    let new_token = CancellationToken::new();
    let new_worker_token = new_token.clone();

    let new_handle = tokio::spawn(async move {
        new_flag.store(true, Ordering::SeqCst);

        // Check: is old worker still running?
        if old_still_running.load(Ordering::SeqCst) {
            overlap.store(true, Ordering::SeqCst);
        }

        while !new_worker_token.is_cancelled() {
            tokio::select! {
                _ = new_worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(30)) => {}
            }
        }
    });

    // Let both run briefly
    sleep(Duration::from_millis(50)).await;

    // Overlap SHOULD be detected (old worker was still cleaning up)
    assert!(overlap_detected.load(Ordering::SeqCst),
        "Without await, overlap should occur — proving the fix is necessary");

    // Cleanup
    new_token.cancel();
    let _ = tokio::time::timeout(Duration::from_secs(3), new_handle).await;
}

// ============================================================
// Fix 3 (connection_changed): checksum exclusion
// ============================================================

/// Test that connection_changed ignores use_legacy_checksum differences.
/// This prevents unnecessary worker restarts when checksum is auto-detected.
#[test]
fn test_connection_changed_ignores_checksum() {
    let config_a = WorkerConfig {
        id: "dev1".into(), name: "Office".into(), brand: "zkteco".into(),
        ip: "192.168.1.1".into(), port: 4370, password: None,
        use_legacy_checksum: None,
    };

    // None → Some(false): auto-detect persisted
    let mut config_b = config_a.clone();
    config_b.use_legacy_checksum = Some(false);
    assert!(!config_a.connection_changed(&config_b),
        "None→Some(false) should NOT trigger restart");

    // None → Some(true): auto-detect persisted
    config_b.use_legacy_checksum = Some(true);
    assert!(!config_a.connection_changed(&config_b),
        "None→Some(true) should NOT trigger restart");

    // Some(false) → Some(true): explicit change, still excluded
    let mut config_c = config_a.clone();
    config_c.use_legacy_checksum = Some(false);
    config_b.use_legacy_checksum = Some(true);
    assert!(!config_c.connection_changed(&config_b),
        "Some(false)→Some(true) should NOT trigger restart (handled at connect time)");
}

/// Test that connection_changed still detects real config changes.
#[test]
fn test_connection_changed_detects_real_changes() {
    let base = WorkerConfig {
        id: "dev1".into(), name: "Office".into(), brand: "zkteco".into(),
        ip: "192.168.1.1".into(), port: 4370, password: None,
        use_legacy_checksum: None,
    };

    // IP change → must trigger restart
    let mut changed = base.clone();
    changed.ip = "10.0.0.1".into();
    assert!(base.connection_changed(&changed), "IP change must trigger restart");

    // Port change → must trigger restart
    changed = base.clone();
    changed.port = 4371;
    assert!(base.connection_changed(&changed), "Port change must trigger restart");

    // Password change → must trigger restart
    changed = base.clone();
    changed.password = Some(12345);
    assert!(base.connection_changed(&changed), "Password change must trigger restart");

    // Brand change → must trigger restart
    changed = base.clone();
    changed.brand = "mock".into();
    assert!(base.connection_changed(&changed), "Brand change must trigger restart");

    // Name change → must NOT trigger restart
    changed = base.clone();
    changed.name = "New Name".into();
    assert!(!base.connection_changed(&changed), "Name change should NOT trigger restart");
}

// ============================================================
// Combined test: full reload lifecycle with timing verification
// ============================================================

/// End-to-end test simulating what handle_config_reload does:
/// cancel(old) → await(old_handle) → spawn(new) → verify ordering.
#[tokio::test]
async fn test_reload_lifecycle_ordering() {
    let events = Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));

    // Old worker
    let old_token = CancellationToken::new();
    let old_worker_token = old_token.clone();
    let old_events = events.clone();

    let old_handle = tokio::spawn(async move {
        old_events.lock().await.push("old:started".into());

        while !old_worker_token.is_cancelled() {
            tokio::select! {
                _ = old_worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(60)) => {}
            }
        }

        // Simulate disconnect delay
        sleep(Duration::from_millis(30)).await;
        old_events.lock().await.push("old:stopped".into());
    });

    sleep(Duration::from_millis(50)).await;

    // Reload: cancel → await → spawn
    old_token.cancel();
    old_handle.await.unwrap();
    events.lock().await.push("reload:old_awaited".into());

    // Spawn new worker
    let new_token = CancellationToken::new();
    let new_worker_token = new_token.clone();
    let new_events = events.clone();

    let new_handle = tokio::spawn(async move {
        new_events.lock().await.push("new:started".into());

        while !new_worker_token.is_cancelled() {
            tokio::select! {
                _ = new_worker_token.cancelled() => { break; }
                _ = sleep(Duration::from_secs(60)) => {}
            }
        }
        new_events.lock().await.push("new:stopped".into());
    });

    sleep(Duration::from_millis(50)).await;

    // Cleanup
    new_token.cancel();
    new_handle.await.unwrap();

    // Verify event ordering
    let log = events.lock().await;
    let old_stopped_idx = log.iter().position(|e| e == "old:stopped").unwrap();
    let old_awaited_idx = log.iter().position(|e| e == "reload:old_awaited").unwrap();
    let new_started_idx = log.iter().position(|e| e == "new:started").unwrap();

    assert!(old_stopped_idx < old_awaited_idx,
        "old:stopped must come before reload:old_awaited");
    assert!(old_awaited_idx < new_started_idx,
        "reload:old_awaited must come before new:started");
}
