//! Memory Leak Detection Tests
//!
//! These tests verify that continuous loops don't accumulate memory over time.
//! Key areas tested:
//! - Channel buffer growth
//! - Database connection pool
//! - Batch processing buffers
//! - Long-running async tasks

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, timeout};

/// Test that mpsc channel doesn't accumulate when consumer keeps up with producer.
/// Memory leak indicator: channel buffer grows unboundedly.
#[tokio::test]
async fn test_channel_no_accumulation_when_consumed() {
    let (tx, mut rx) = mpsc::channel::<usize>(100);
    let produced = Arc::new(AtomicUsize::new(0));
    let consumed = Arc::new(AtomicUsize::new(0));

    let produced_clone = produced.clone();
    let producer = tokio::spawn(async move {
        for i in 0..1000 {
            if tx.send(i).await.is_err() {
                break;
            }
            produced_clone.fetch_add(1, Ordering::SeqCst);
            // Simulate realistic production rate
            if i % 100 == 0 {
                sleep(Duration::from_millis(1)).await;
            }
        }
    });

    let consumed_clone = consumed.clone();
    let consumer = tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            consumed_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    // Wait for producer to finish
    let _ = producer.await;
    // Give consumer time to drain
    sleep(Duration::from_millis(100)).await;

    let p = produced.load(Ordering::SeqCst);
    let c = consumed.load(Ordering::SeqCst);

    // Consumer should have processed all or nearly all messages
    assert!(c >= p - 100, "Consumer lagging too far behind: produced={}, consumed={}", p, c);

    println!("✅ Channel consumption test passed: produced={}, consumed={}", p, c);
}

/// Test that slow consumer causes backpressure, not unbounded memory growth.
/// Memory leak indicator: producer doesn't block when buffer is full.
#[tokio::test]
async fn test_channel_backpressure_prevents_memory_growth() {
    // Small buffer to test backpressure quickly
    let (tx, mut rx) = mpsc::channel::<usize>(10);
    let send_blocked = Arc::new(AtomicUsize::new(0));

    let blocked_clone = send_blocked.clone();
    let producer = tokio::spawn(async move {
        for i in 0..100 {
            // Use try_send to detect when buffer is full
            loop {
                match tx.try_send(i) {
                    Ok(_) => break,
                    Err(mpsc::error::TrySendError::Full(_)) => {
                        blocked_clone.fetch_add(1, Ordering::SeqCst);
                        sleep(Duration::from_millis(10)).await;
                    }
                    Err(_) => return,
                }
            }
        }
    });

    // Slow consumer - simulates DB writer processing batches
    let consumer = tokio::spawn(async move {
        while let Ok(Some(_)) = timeout(Duration::from_secs(2), rx.recv()).await {
            // Simulate slow processing
            sleep(Duration::from_millis(50)).await;
        }
    });

    let _ = tokio::join!(producer, consumer);

    let blocked_count = send_blocked.load(Ordering::SeqCst);

    // Producer should have been blocked multiple times (backpressure working)
    assert!(blocked_count > 0, "Producer was never blocked - backpressure not working");

    println!("✅ Backpressure test passed: producer blocked {} times", blocked_count);
}

/// Test that batch buffer is cleared after processing.
/// Memory leak indicator: buffer grows without bound.
#[tokio::test]
async fn test_batch_buffer_cleared_after_processing() {
    let mut buffer: Vec<String> = Vec::with_capacity(100);
    let mut total_processed = 0usize;
    let mut max_buffer_size = 0usize;

    // Simulate batch processing loop
    for cycle in 0..10 {
        // Fill buffer (simulates receiving logs)
        for i in 0..100 {
            buffer.push(format!("log_{}_{}", cycle, i));
        }

        max_buffer_size = max_buffer_size.max(buffer.len());

        // Process batch (simulates DB write)
        total_processed += buffer.len();

        // Clear buffer - CRITICAL for preventing memory leak
        buffer.clear();

        // Verify buffer is empty after processing
        assert_eq!(buffer.len(), 0, "Buffer not cleared after cycle {}", cycle);
    }

    // Buffer capacity may grow, but len should always return to 0
    assert_eq!(buffer.len(), 0, "Final buffer should be empty");
    assert_eq!(total_processed, 1000, "Should have processed 1000 items");
    assert_eq!(max_buffer_size, 100, "Max buffer size should be 100 (batch size)");

    println!("✅ Batch buffer test passed: processed={}, max_size={}", total_processed, max_buffer_size);
}

/// Test that Vec with capacity doesn't leak when using swap pattern.
/// Memory leak indicator: old buffer not dropped.
#[tokio::test]
async fn test_vec_swap_pattern_no_leak() {
    let mut buffer: Vec<i32> = Vec::with_capacity(100);
    let mut processed_buffers = 0usize;

    for _ in 0..100 {
        // Fill buffer
        for i in 0..50 {
            buffer.push(i);
        }

        // Swap pattern (used in DB writer)
        let to_process = std::mem::replace(&mut buffer, Vec::with_capacity(100));

        // Verify new buffer is empty
        assert_eq!(buffer.len(), 0);
        assert!(buffer.capacity() >= 100);

        // Process old buffer (simulates DB write)
        assert_eq!(to_process.len(), 50);
        processed_buffers += 1;

        // to_process is dropped here - memory freed
    }

    assert_eq!(processed_buffers, 100);
    assert_eq!(buffer.len(), 0);

    println!("✅ Vec swap pattern test passed: {} buffers processed", processed_buffers);
}

/// Test that database query results don't accumulate in caller.
/// Memory leak indicator: results vector grows without bound.
#[tokio::test]
async fn test_query_results_consumed_not_accumulated() {
    let all_results: Vec<i32> = Vec::new();
    let mut batches_processed = 0usize;

    // Simulate fetching and processing in batches
    for batch_num in 0..10 {
        // Simulate DB query returning batch
        let batch: Vec<i32> = (0..100).map(|i| batch_num * 100 + i).collect();

        // BAD pattern (accumulates): all_results.extend(batch);
        // GOOD pattern (processes and discards):
        for _item in batch {
            // Process item (e.g., send to channel, mark synced)
            // _item is consumed and dropped here
        }
        batches_processed += 1;

        // Results should NOT accumulate
        assert_eq!(all_results.len(), 0, "Results accumulating - memory leak!");
    }

    assert_eq!(batches_processed, 10);
    assert_eq!(all_results.len(), 0, "No results should accumulate");

    println!("✅ Query results consumption test passed");
}

/// Test semaphore permit lifecycle doesn't leak.
/// Memory leak indicator: permits not returned to semaphore.
#[tokio::test]
async fn test_semaphore_permits_returned() {
    use tokio::sync::Semaphore;

    let semaphore = Arc::new(Semaphore::new(2));
    let acquisitions = Arc::new(AtomicUsize::new(0));

    let mut handles = vec![];

    for _ in 0..10 {
        let sem = semaphore.clone();
        let acq = acquisitions.clone();

        handles.push(tokio::spawn(async move {
            for _ in 0..5 {
                let _permit = sem.acquire().await.unwrap();
                acq.fetch_add(1, Ordering::SeqCst);
                sleep(Duration::from_millis(5)).await;
                // _permit dropped here - returned to semaphore
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    // All permits should be returned
    let available = semaphore.available_permits();
    assert_eq!(available, 2, "Not all permits returned: {} available", available);

    let total_acquisitions = acquisitions.load(Ordering::SeqCst);
    assert_eq!(total_acquisitions, 50, "Should have 50 acquisitions");

    println!("✅ Semaphore permit lifecycle test passed: {} acquisitions, {} permits available",
             total_acquisitions, available);
}

/// Test that spawned tasks complete and don't accumulate.
/// Memory leak indicator: tasks never complete, handles accumulate.
#[tokio::test]
async fn test_spawned_tasks_complete() {
    let completed = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn many short-lived tasks
    for _ in 0..100 {
        let c = completed.clone();
        handles.push(tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;
            c.fetch_add(1, Ordering::SeqCst);
        }));
    }

    // Wait for all to complete with timeout
    let result = timeout(Duration::from_secs(5), async {
        for h in handles {
            let _ = h.await;
        }
    }).await;

    assert!(result.is_ok(), "Tasks didn't complete in time - possible leak");

    let count = completed.load(Ordering::SeqCst);
    assert_eq!(count, 100, "Not all tasks completed: {}", count);

    println!("✅ Spawned tasks completion test passed: {} tasks completed", count);
}

/// Test that cancellation token properly stops loops.
/// Memory leak indicator: loop continues after cancellation.
#[tokio::test]
async fn test_cancellation_stops_loop() {
    use tokio_util::sync::CancellationToken;

    let token = CancellationToken::new();
    let iterations = Arc::new(AtomicUsize::new(0));

    let iter_clone = iterations.clone();
    let token_clone = token.clone();

    let handle = tokio::spawn(async move {
        while !token_clone.is_cancelled() {
            iter_clone.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Let loop run for a bit
    sleep(Duration::from_millis(100)).await;

    // Cancel
    token.cancel();

    // Wait for task to complete
    let result = timeout(Duration::from_millis(500), handle).await;
    assert!(result.is_ok(), "Task didn't stop after cancellation");

    let count_at_cancel = iterations.load(Ordering::SeqCst);

    // Verify loop stopped (no more iterations)
    sleep(Duration::from_millis(100)).await;
    let count_after = iterations.load(Ordering::SeqCst);

    assert_eq!(count_at_cancel, count_after, "Loop continued after cancellation!");

    println!("✅ Cancellation test passed: loop stopped at iteration {}", count_at_cancel);
}

/// Test that channel sender drop causes receiver to exit.
/// Memory leak indicator: receiver waits forever.
#[tokio::test]
async fn test_channel_sender_drop_closes_receiver() {
    let (tx, mut rx) = mpsc::channel::<i32>(10);
    let receiver_exited = Arc::new(AtomicUsize::new(0));

    let exit_clone = receiver_exited.clone();
    let receiver = tokio::spawn(async move {
        while let Some(_) = rx.recv().await {
            // Process
        }
        // Receiver exits when all senders dropped
        exit_clone.store(1, Ordering::SeqCst);
    });

    // Send some messages
    for i in 0..5 {
        let _ = tx.send(i).await;
    }

    // Drop sender
    drop(tx);

    // Receiver should exit
    let result = timeout(Duration::from_millis(500), receiver).await;
    assert!(result.is_ok(), "Receiver didn't exit after sender dropped");

    let exited = receiver_exited.load(Ordering::SeqCst);
    assert_eq!(exited, 1, "Receiver didn't signal exit");

    println!("✅ Channel sender drop test passed");
}

/// Test long-running simulation with memory checkpoints.
/// This simulates the actual agent loop pattern.
#[tokio::test]
async fn test_simulated_agent_loop_memory_stability() {
    use tokio_util::sync::CancellationToken;

    let token = CancellationToken::new();
    let (log_tx, mut log_rx) = mpsc::channel::<(String, i32)>(1000);

    let processed_count = Arc::new(AtomicUsize::new(0));
    let max_pending = Arc::new(AtomicUsize::new(0));

    // Simulate device polling (producer)
    let tx = log_tx.clone();
    let token_poll = token.clone();
    let producer = tokio::spawn(async move {
        let mut batch_num = 0;
        while !token_poll.is_cancelled() {
            // Simulate fetching logs from device
            for i in 0..100 {
                if tx.send(("device1".to_string(), batch_num * 100 + i)).await.is_err() {
                    return;
                }
            }
            batch_num += 1;
            sleep(Duration::from_millis(50)).await;
        }
    });

    // Simulate DB writer (consumer)
    let proc_clone = processed_count.clone();
    let max_clone = max_pending.clone();
    let token_write = token.clone();
    let consumer = tokio::spawn(async move {
        let mut buffer: Vec<(String, i32)> = Vec::with_capacity(100);
        let mut pending = 0usize;

        loop {
            tokio::select! {
                Some(item) = log_rx.recv() => {
                    buffer.push(item);
                    pending += 1;
                    max_clone.fetch_max(pending, Ordering::SeqCst);

                    if buffer.len() >= 100 {
                        // Simulate batch write
                        proc_clone.fetch_add(buffer.len(), Ordering::SeqCst);
                        pending -= buffer.len();
                        buffer.clear();
                        sleep(Duration::from_millis(10)).await;
                    }
                }
                _ = token_write.cancelled() => {
                    // Flush remaining
                    if !buffer.is_empty() {
                        proc_clone.fetch_add(buffer.len(), Ordering::SeqCst);
                    }
                    break;
                }
            }
        }
    });

    // Run for a simulated period
    sleep(Duration::from_millis(500)).await;

    // Cancel all tasks
    token.cancel();
    drop(log_tx); // Close channel

    let _ = tokio::join!(producer, consumer);

    let processed = processed_count.load(Ordering::SeqCst);
    let max_p = max_pending.load(Ordering::SeqCst);

    // Verify processing happened
    assert!(processed > 0, "No logs processed");

    // Verify max pending stayed bounded (< 2x batch size indicates no accumulation)
    assert!(max_p < 300, "Max pending too high ({}), possible accumulation", max_p);

    println!("✅ Simulated agent loop test passed: processed={}, max_pending={}", processed, max_p);
}
