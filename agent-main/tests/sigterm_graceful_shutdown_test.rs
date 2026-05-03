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
//! Tests for SIGTERM graceful shutdown behavior.
//!
//! Validates that:
//! 1. The signal handler pattern correctly cancels the CancellationToken on SIGTERM
//! 2. Workers respond to cancellation by disconnecting from devices
//! 3. Both SIGINT and SIGTERM trigger the same shutdown path

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio_util::sync::CancellationToken;

/// Test that CancellationToken propagation works: when a parent token is
/// cancelled (simulating signal handler), child tokens also cancel.
/// This is the core mechanism that makes graceful shutdown work.
#[tokio::test]
async fn test_cancellation_token_propagates_to_workers() {
    let parent_token = CancellationToken::new();
    let worker1_token = parent_token.child_token();
    let worker2_token = parent_token.child_token();

    let w1_stopped = Arc::new(AtomicBool::new(false));
    let w2_stopped = Arc::new(AtomicBool::new(false));
    let w1_clone = w1_stopped.clone();
    let w2_clone = w2_stopped.clone();

    // Simulate workers waiting for cancellation
    let h1 = tokio::spawn(async move {
        worker1_token.cancelled().await;
        w1_clone.store(true, Ordering::SeqCst);
    });
    let h2 = tokio::spawn(async move {
        worker2_token.cancelled().await;
        w2_clone.store(true, Ordering::SeqCst);
    });

    // Simulate signal handler cancelling the parent
    parent_token.cancel();

    h1.await.unwrap();
    h2.await.unwrap();

    assert!(
        w1_stopped.load(Ordering::SeqCst),
        "Worker 1 should have stopped"
    );
    assert!(
        w2_stopped.load(Ordering::SeqCst),
        "Worker 2 should have stopped"
    );
}

/// Test the exact signal handler pattern used in main.rs:
/// tokio::select! on SIGTERM triggers cancel_token.cancel().
/// We verify by sending SIGTERM to our own process.
#[tokio::test]
async fn test_sigterm_triggers_cancellation() {
    let cancel_token = CancellationToken::new();
    let cloned_token = cancel_token.clone();

    // Replicate the exact signal handler from main.rs
    tokio::spawn(async move {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = sigterm.recv() => {}
        }
        cloned_token.cancel();
    });

    // Give the handler time to register
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Send SIGTERM to ourselves
    unsafe {
        libc::kill(libc::getpid(), libc::SIGTERM);
    }

    // Wait for cancellation (with timeout to avoid hanging if broken)
    tokio::select! {
        _ = cancel_token.cancelled() => {
            // Success — SIGTERM triggered cancellation
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            panic!("SIGTERM did not trigger cancellation within 5 seconds");
        }
    }
}

/// Test that the shutdown flow: signal → cancel token → worker disconnect
/// works end-to-end with MockWorker.
#[tokio::test]
async fn test_worker_disconnects_on_cancellation() {
    use agent::mock_worker::MockWorker;
    use agent::worker::Worker;

    let cancel_token = CancellationToken::new();
    let worker_token = cancel_token.clone();
    let disconnected = Arc::new(AtomicBool::new(false));
    let dc_clone = disconnected.clone();

    // Simulate the worker loop pattern from spawn_worker()
    let handle = tokio::spawn(async move {
        let mut worker = MockWorker::new();
        worker.connect("127.0.0.1", 4370).await.unwrap();

        // Poll loop — exits when token is cancelled
        while !worker_token.is_cancelled() {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        // Graceful disconnect (same as line 246 in main.rs)
        worker.disconnect().await;
        dc_clone.store(true, Ordering::SeqCst);
    });

    // Let worker start polling
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // Simulate signal handler cancelling
    cancel_token.cancel();

    // Wait for worker to finish
    handle.await.unwrap();

    assert!(
        disconnected.load(Ordering::SeqCst),
        "Worker should have disconnected"
    );
}

/// Test that multiple workers all disconnect when the shared cancel token fires.
/// Simulates a full registry of workers being shut down simultaneously.
#[tokio::test]
async fn test_all_workers_disconnect_on_shutdown() {
    use agent::mock_worker::MockWorker;
    use agent::worker::Worker;

    let cancel_token = CancellationToken::new();
    let worker_count = 5;
    let disconnect_counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let mut handles = Vec::new();

    for i in 0..worker_count {
        let token = cancel_token.clone();
        let counter = disconnect_counter.clone();

        handles.push(tokio::spawn(async move {
            let mut worker = MockWorker::new();
            worker
                .connect("127.0.0.1", (4370 + i) as u16)
                .await
                .unwrap();

            while !token.is_cancelled() {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            worker.disconnect().await;
            counter.fetch_add(1, Ordering::SeqCst);
        }));
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    cancel_token.cancel();

    for h in handles {
        h.await.unwrap();
    }

    assert_eq!(
        disconnect_counter.load(Ordering::SeqCst),
        worker_count,
        "All {} workers should have disconnected",
        worker_count,
    );
}

/// Test that cancellation during worker connection attempt doesn't hang.
/// If a worker is blocked on connect() when SIGTERM arrives, the outer
/// loop should still exit after the failed connect returns.
#[tokio::test]
async fn test_cancellation_during_idle_does_not_hang() {
    let cancel_token = CancellationToken::new();
    let worker_token = cancel_token.clone();

    let handle = tokio::spawn(async move {
        // Simulate a worker in its poll sleep — use select! so cancellation
        // interrupts the long sleep immediately.
        loop {
            tokio::select! {
                _ = worker_token.cancelled() => break,
                _ = tokio::time::sleep(std::time::Duration::from_secs(60)) => {}
            }
        }
        // Should reach here after cancel, not hang
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    cancel_token.cancel();

    // With timeout — if this times out, the worker is hanging
    tokio::select! {
        result = handle => {
            result.unwrap();
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(3)) => {
            panic!("Worker hung after cancellation — did not exit within 3 seconds");
        }
    }
}
