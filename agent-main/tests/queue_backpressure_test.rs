use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[tokio::test]
async fn test_queue_backpressure_behavior() {
    // 1. Setup a channel with buffer size 1
    // In Tokio, send() blocks when the buffer is full AND no one is ready to receive.
    let (tx, mut rx) = mpsc::channel::<usize>(1);
    let processed_count = Arc::new(AtomicUsize::new(0));
    let processed_count_clone = processed_count.clone();

    // 2. Spawn a slow consumer
    tokio::spawn(async move {
        while (rx.recv().await).is_some() {
            sleep(Duration::from_millis(200)).await;
            processed_count_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    let start = std::time::Instant::now();
    
    // Send 1st item: immediately accepted and taken by consumer (async task)
    tx.send(1).await.unwrap(); 
    
    // Send 2nd item: fits in the 1-slot buffer
    tx.send(2).await.unwrap(); 
    
    // Send 3rd item: BUFFER FULL. Must wait for consumer to finish 1st item.
    tx.send(3).await.unwrap();
    let elapsed = start.elapsed();
    
    assert!(elapsed >= Duration::from_millis(200), "Producer should have blocked until consumer finished 1st item. Elapsed: {elapsed:?}");
    
    assert_eq!(processed_count.load(Ordering::SeqCst), 1);
    
    println!("✅ Backpressure verified: Producer blocked correctly at slot limit.");
}
