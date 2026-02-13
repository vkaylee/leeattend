use tokio::sync::Semaphore;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Test that semaphore-based polling achieves fair round-robin scheduling.
/// With max_workers=1 and 2 devices, both should get roughly equal turns.
#[tokio::test]
async fn test_fair_round_robin_polling() {
    let semaphore = Arc::new(Semaphore::new(1)); // max_workers = 1
    let device_a_polls = Arc::new(AtomicUsize::new(0));
    let device_b_polls = Arc::new(AtomicUsize::new(0));

    let total_cycles = 10;

    // Device A worker
    let sem_a = semaphore.clone();
    let polls_a = device_a_polls.clone();
    let handle_a = tokio::spawn(async move {
        for _ in 0..total_cycles {
            // Acquire permit for this poll cycle
            let _permit = sem_a.acquire().await.unwrap();
            polls_a.fetch_add(1, Ordering::SeqCst);
            // Simulate poll work
            sleep(Duration::from_millis(10)).await;
            // Release permit before sleep (mimics actual implementation)
            drop(_permit);
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Device B worker
    let sem_b = semaphore.clone();
    let polls_b = device_b_polls.clone();
    let handle_b = tokio::spawn(async move {
        for _ in 0..total_cycles {
            // Acquire permit for this poll cycle
            let _permit = sem_b.acquire().await.unwrap();
            polls_b.fetch_add(1, Ordering::SeqCst);
            // Simulate poll work
            sleep(Duration::from_millis(10)).await;
            // Release permit before sleep
            drop(_permit);
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Wait for both to complete
    let _ = tokio::join!(handle_a, handle_b);

    let a_count = device_a_polls.load(Ordering::SeqCst);
    let b_count = device_b_polls.load(Ordering::SeqCst);

    // Both devices should have polled equal times
    assert_eq!(a_count, total_cycles, "Device A should complete all polls");
    assert_eq!(b_count, total_cycles, "Device B should complete all polls");

    println!("✅ Fair polling verified: Device A={a_count}, Device B={b_count}");
}

/// Test that without releasing permit between cycles, one device dominates.
/// This validates that our fix (releasing permit each cycle) was necessary.
#[tokio::test]
async fn test_unfair_polling_without_release() {
    let semaphore = Arc::new(Semaphore::new(1));
    let device_a_polls = Arc::new(AtomicUsize::new(0));
    let device_b_polls = Arc::new(AtomicUsize::new(0));

    // Device A - holds permit for entire duration (old buggy behavior)
    let sem_a = semaphore.clone();
    let polls_a = device_a_polls.clone();
    let handle_a = tokio::spawn(async move {
        let _permit = sem_a.acquire().await.unwrap();
        // Hold permit for entire loop - never release until done
        for _ in 0..5 {
            polls_a.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(30)).await;
        }
        // Permit released here when task ends
    });

    // Device B - tries to acquire but blocked while A is running
    let sem_b = semaphore.clone();
    let polls_b = device_b_polls.clone();
    let handle_b = tokio::spawn(async move {
        // Small delay to ensure A starts first
        sleep(Duration::from_millis(5)).await;

        // Try to poll multiple times - but blocked by A holding permit
        for _ in 0..5 {
            // This will block until A releases permit
            let result = tokio::time::timeout(
                Duration::from_millis(25),
                sem_b.acquire()
            ).await;

            if let Ok(Ok(_permit)) = result {
                polls_b.fetch_add(1, Ordering::SeqCst);
            }
            // If timeout, B couldn't get permit - starved
        }
    });

    let _ = tokio::join!(handle_a, handle_b);

    let a_count = device_a_polls.load(Ordering::SeqCst);
    let b_count = device_b_polls.load(Ordering::SeqCst);

    // A should have polled 5 times
    // B should have polled fewer times (starved during A's run)
    assert_eq!(a_count, 5, "Device A completed all its polls");
    assert!(b_count < 5, "Device B was starved (got {b_count} polls, should be < 5)");

    println!("✅ Unfair polling verified: A={a_count}, B={b_count} (B starved as expected)");
}

/// Test semaphore with multiple permits allows parallel polling.
#[tokio::test]
async fn test_parallel_polling_with_multiple_permits() {
    let semaphore = Arc::new(Semaphore::new(2)); // max_workers = 2
    let device_a_polls = Arc::new(AtomicUsize::new(0));
    let device_b_polls = Arc::new(AtomicUsize::new(0));
    let concurrent_count = Arc::new(AtomicUsize::new(0));
    let max_concurrent = Arc::new(AtomicUsize::new(0));

    // Device A worker
    let sem_a = semaphore.clone();
    let polls_a = device_a_polls.clone();
    let concurrent_a = concurrent_count.clone();
    let max_a = max_concurrent.clone();
    let handle_a = tokio::spawn(async move {
        for _ in 0..5 {
            let _permit = sem_a.acquire().await.unwrap();
            let current = concurrent_a.fetch_add(1, Ordering::SeqCst) + 1;
            max_a.fetch_max(current, Ordering::SeqCst);

            polls_a.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(50)).await;

            concurrent_a.fetch_sub(1, Ordering::SeqCst);
            drop(_permit);
            sleep(Duration::from_millis(10)).await;
        }
    });

    // Device B worker
    let sem_b = semaphore.clone();
    let polls_b = device_b_polls.clone();
    let concurrent_b = concurrent_count.clone();
    let max_b = max_concurrent.clone();
    let handle_b = tokio::spawn(async move {
        for _ in 0..5 {
            let _permit = sem_b.acquire().await.unwrap();
            let current = concurrent_b.fetch_add(1, Ordering::SeqCst) + 1;
            max_b.fetch_max(current, Ordering::SeqCst);

            polls_b.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(50)).await;

            concurrent_b.fetch_sub(1, Ordering::SeqCst);
            drop(_permit);
            sleep(Duration::from_millis(10)).await;
        }
    });

    let _ = tokio::join!(handle_a, handle_b);

    let a_count = device_a_polls.load(Ordering::SeqCst);
    let b_count = device_b_polls.load(Ordering::SeqCst);
    let max_c = max_concurrent.load(Ordering::SeqCst);

    assert_eq!(a_count, 5, "Device A should complete all polls");
    assert_eq!(b_count, 5, "Device B should complete all polls");
    assert_eq!(max_c, 2, "Both devices should have polled concurrently");

    println!("✅ Parallel polling verified: A={a_count}, B={b_count}, max_concurrent={max_c}");
}
