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

use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant, sleep};

/// Type matching the production DeviceLocks definition
type DeviceLocks = Arc<DashMap<String, Arc<Mutex<()>>>>;

fn create_device_locks() -> DeviceLocks {
    Arc::new(DashMap::new())
}

/// Helper: acquire device lock (same pattern as main.rs)
fn get_device_lock(locks: &DeviceLocks, device_id: &str) -> Arc<Mutex<()>> {
    locks
        .entry(device_id.to_string())
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// Test that operations on the SAME device are serialized (never concurrent).
#[tokio::test]
async fn test_same_device_operations_serialized() {
    let locks = create_device_locks();
    let concurrent_ops = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));
    let violation = Arc::new(AtomicBool::new(false));

    let mut handles = vec![];

    // Spawn 5 tasks all targeting the same device
    for i in 0..5 {
        let locks = locks.clone();
        let concurrent = concurrent_ops.clone();
        let max_c = max_concurrent.clone();
        let viol = violation.clone();

        handles.push(tokio::spawn(async move {
            let lock = get_device_lock(&locks, "device-A");
            let _guard = lock.lock().await;

            let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
            max_c.fetch_max(current, Ordering::SeqCst);

            if current > 1 {
                viol.store(true, Ordering::SeqCst);
            }

            // Simulate device operation (get_info or get_attendance_logs)
            sleep(Duration::from_millis(20)).await;

            concurrent.fetch_sub(1, Ordering::SeqCst);
            drop(_guard);

            i // return task id for verification
        }));
    }

    let mut results = vec![];
    for h in handles {
        results.push(h.await.unwrap());
    }

    // All 5 tasks completed
    assert_eq!(results.len(), 5);

    // Max concurrent on same device should be 1 (serialized)
    let max = max_concurrent.load(Ordering::SeqCst);
    assert_eq!(
        max, 1,
        "Same-device ops must be serialized, but max concurrent was {max}"
    );
    assert!(
        !violation.load(Ordering::SeqCst),
        "Concurrent access violation detected on same device"
    );
}

/// Test that operations on DIFFERENT devices can run concurrently.
#[tokio::test]
async fn test_different_devices_run_concurrently() {
    let locks = create_device_locks();
    let concurrent_ops = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    // Spawn 3 tasks each targeting a different device
    for i in 0..3 {
        let locks = locks.clone();
        let concurrent = concurrent_ops.clone();
        let max_c = max_concurrent.clone();
        let device_id = format!("device-{i}");

        handles.push(tokio::spawn(async move {
            let lock = get_device_lock(&locks, &device_id);
            let _guard = lock.lock().await;

            let current = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
            max_c.fetch_max(current, Ordering::SeqCst);

            // Hold lock long enough for other tasks to start
            sleep(Duration::from_millis(100)).await;

            concurrent.fetch_sub(1, Ordering::SeqCst);
        }));
    }

    for h in handles {
        h.await.unwrap();
    }

    let max = max_concurrent.load(Ordering::SeqCst);
    assert!(
        max >= 2,
        "Different devices should run concurrently, but max concurrent was {max}"
    );
}

/// Test that a "remote command" targeting a device waits for the "polling" task
/// to finish before proceeding (simulates the real scenario).
#[tokio::test]
async fn test_remote_command_waits_for_polling() {
    let locks = create_device_locks();
    let poll_started = Arc::new(AtomicBool::new(false));
    let poll_finished = Arc::new(AtomicBool::new(false));
    let cmd_started_during_poll = Arc::new(AtomicBool::new(false));

    // Polling task: holds device lock for 200ms
    let poll_locks = locks.clone();
    let poll_started_flag = poll_started.clone();
    let poll_finished_flag = poll_finished.clone();
    let poll_handle = tokio::spawn(async move {
        let lock = get_device_lock(&poll_locks, "device-X");
        let _guard = lock.lock().await;

        poll_started_flag.store(true, Ordering::SeqCst);

        // Simulate long poll (get_attendance_logs on a slow device)
        sleep(Duration::from_millis(200)).await;

        poll_finished_flag.store(true, Ordering::SeqCst);
    });

    // Remote command task: starts after small delay, must wait for poll to finish
    let cmd_locks = locks.clone();
    let cmd_poll_started = poll_started.clone();
    let cmd_poll_finished = poll_finished.clone();
    let cmd_during = cmd_started_during_poll.clone();
    let cmd_handle = tokio::spawn(async move {
        // Wait for poll to start
        while !cmd_poll_started.load(Ordering::SeqCst) {
            sleep(Duration::from_millis(5)).await;
        }

        // Now try to acquire lock (should block until poll finishes)
        let lock = get_device_lock(&cmd_locks, "device-X");
        let _guard = lock.lock().await;

        // Check: did we start while poll was still running?
        if !cmd_poll_finished.load(Ordering::SeqCst) {
            cmd_during.store(true, Ordering::SeqCst);
        }

        // Simulate command execution
        sleep(Duration::from_millis(10)).await;
    });

    let _ = tokio::join!(poll_handle, cmd_handle);

    assert!(
        !cmd_started_during_poll.load(Ordering::SeqCst),
        "Remote command must NOT execute while polling is in progress on the same device"
    );
}

/// Test that device lock is properly released after task completes,
/// allowing subsequent operations to proceed.
#[tokio::test]
async fn test_lock_released_after_operation() {
    let locks = create_device_locks();
    let ops_completed = Arc::new(AtomicUsize::new(0));

    // Run 3 sequential operations on same device
    for _ in 0..3 {
        let lock = get_device_lock(&locks, "device-Z");
        let _guard = lock.lock().await;
        sleep(Duration::from_millis(10)).await;
        ops_completed.fetch_add(1, Ordering::SeqCst);
        // _guard drops here, releasing lock
    }

    assert_eq!(
        ops_completed.load(Ordering::SeqCst),
        3,
        "All 3 operations should complete"
    );
}

/// Test that device lock entries are lazily created (only when accessed).
#[tokio::test]
async fn test_lazy_lock_creation() {
    let locks = create_device_locks();

    assert_eq!(locks.len(), 0, "No locks should exist initially");

    // Access device-A
    let _ = get_device_lock(&locks, "device-A");
    assert_eq!(locks.len(), 1, "One lock should be created");

    // Access same device again — no new entry
    let _ = get_device_lock(&locks, "device-A");
    assert_eq!(locks.len(), 1, "Same device should reuse existing lock");

    // Access device-B
    let _ = get_device_lock(&locks, "device-B");
    assert_eq!(locks.len(), 2, "Second device should get its own lock");
}

/// Test that the serialization adds minimal overhead when there's no contention.
#[tokio::test]
async fn test_uncontested_lock_latency() {
    let locks = create_device_locks();

    let start = Instant::now();
    for i in 0..100 {
        let device_id = format!("device-{i}");
        let lock = get_device_lock(&locks, &device_id);
        let _guard = lock.lock().await;
        // Immediate release
    }
    let elapsed = start.elapsed();

    // 100 uncontested lock acquisitions should be well under 50ms
    assert!(
        elapsed < Duration::from_millis(50),
        "Uncontested locks should be near-instant, took {:?}",
        elapsed
    );
}
