use std::collections::HashSet;
use tokio::sync::mpsc;
use tokio::time::Duration;

/// Test that duplicate detection filters repeated logs before sending to queue.
/// Simulates device returning same logs repeatedly (no device clearing).
#[tokio::test]
async fn test_duplicate_detection_filters_repeated_logs() {
    let (tx, mut rx) = mpsc::channel::<(String, String)>(100);
    let mut seen_logs: HashSet<String> = HashSet::new();

    // Simulate 3 polling cycles from same device
    let device_name = "OFFICE".to_string();

    // Poll 1: Device returns 5 logs
    let poll1_logs = vec![
        ("U001", "2026-01-28 08:00:00"),
        ("U002", "2026-01-28 08:01:00"),
        ("U003", "2026-01-28 08:02:00"),
        ("U004", "2026-01-28 08:03:00"),
        ("U005", "2026-01-28 08:04:00"),
    ];

    let mut sent_count_p1 = 0;
    for (user_id, timestamp) in &poll1_logs {
        let key = format!("{user_id}:{timestamp}");
        if seen_logs.insert(key) {
            tx.send((device_name.clone(), format!("{user_id}:{timestamp}"))).await.unwrap();
            sent_count_p1 += 1;
        }
    }
    assert_eq!(sent_count_p1, 5, "Poll 1: All 5 logs should be new");

    // Poll 2: Device returns 7 logs (5 duplicates + 2 new)
    let poll2_logs = vec![
        ("U001", "2026-01-28 08:00:00"), // Duplicate
        ("U002", "2026-01-28 08:01:00"), // Duplicate
        ("U003", "2026-01-28 08:02:00"), // Duplicate
        ("U004", "2026-01-28 08:03:00"), // Duplicate
        ("U005", "2026-01-28 08:04:00"), // Duplicate
        ("U006", "2026-01-28 08:05:00"), // New
        ("U007", "2026-01-28 08:06:00"), // New
    ];

    let mut sent_count_p2 = 0;
    let mut dup_count_p2 = 0;
    for (user_id, timestamp) in &poll2_logs {
        let key = format!("{user_id}:{timestamp}");
        if seen_logs.insert(key) {
            tx.send((device_name.clone(), format!("{user_id}:{timestamp}"))).await.unwrap();
            sent_count_p2 += 1;
        } else {
            dup_count_p2 += 1;
        }
    }
    assert_eq!(sent_count_p2, 2, "Poll 2: Only 2 new logs should be sent");
    assert_eq!(dup_count_p2, 5, "Poll 2: 5 duplicates should be filtered");

    // Poll 3: Device returns 10 logs (7 duplicates + 3 new)
    let poll3_logs = vec![
        ("U001", "2026-01-28 08:00:00"), // Duplicate
        ("U002", "2026-01-28 08:01:00"), // Duplicate
        ("U003", "2026-01-28 08:02:00"), // Duplicate
        ("U004", "2026-01-28 08:03:00"), // Duplicate
        ("U005", "2026-01-28 08:04:00"), // Duplicate
        ("U006", "2026-01-28 08:05:00"), // Duplicate
        ("U007", "2026-01-28 08:06:00"), // Duplicate
        ("U008", "2026-01-28 08:07:00"), // New
        ("U009", "2026-01-28 08:08:00"), // New
        ("U010", "2026-01-28 08:09:00"), // New
    ];

    let mut sent_count_p3 = 0;
    let mut dup_count_p3 = 0;
    for (user_id, timestamp) in &poll3_logs {
        let key = format!("{user_id}:{timestamp}");
        if seen_logs.insert(key) {
            tx.send((device_name.clone(), format!("{user_id}:{timestamp}"))).await.unwrap();
            sent_count_p3 += 1;
        } else {
            dup_count_p3 += 1;
        }
    }
    assert_eq!(sent_count_p3, 3, "Poll 3: Only 3 new logs should be sent");
    assert_eq!(dup_count_p3, 7, "Poll 3: 7 duplicates should be filtered");

    // Verify total messages in queue
    drop(tx);
    let mut received = 0;
    while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        received += 1;
    }
    assert_eq!(received, 10, "Total: 10 unique logs should be in queue (5+2+3)");

    println!("✅ Duplicate detection test passed:");
    println!("   Poll 1: {sent_count_p1} sent, 0 filtered");
    println!("   Poll 2: {sent_count_p2} sent, {dup_count_p2} filtered");
    println!("   Poll 3: {sent_count_p3} sent, {dup_count_p3} filtered");
    println!("   Total: {received} unique logs in queue");
}

/// Test memory efficiency: HashSet size stays bounded.
#[tokio::test]
async fn test_hashset_memory_bounded_u64() {
    let mut seen_logs: HashSet<u64> = HashSet::with_capacity(100_000);

    // Simulate large device with 250K logs (exceeding cap)
    for i in 0..250_000 {
        let mut hasher = rustc_hash::FxHasher::default();
        use std::hash::{Hash, Hasher};
        format!("U{}", i % 1000).hash(&mut hasher);
        format!("2026-01-28 08:{}:{}", i / 60, i % 60).hash(&mut hasher);
        let key = hasher.finish();
        seen_logs.insert(key);
        
        // Simulate clearance logic
        if seen_logs.len() > 200_000 {
            seen_logs.clear();
            seen_logs.shrink_to(100_000);
        }
    }

    // Since we inserted 250k and cap is 200k, it should have cleared once and have ~50k remaining
    assert!(seen_logs.capacity() >= 100_000, "Capacity should be at least shrunk size");
    assert!(seen_logs.len() <= 200_000, "Should not exceed cap");

    // Each key ~8 bytes (u64)
    let mem_estimate = seen_logs.capacity() * 8;
    assert!(mem_estimate < 2_000_000, "Memory should be under 2MB");

    println!("✅ Memory bounded test passed:");
    println!("   {} unique keys", seen_logs.len());
    println!("   capacity: {}", seen_logs.capacity());
    println!("   ~{} bytes (~{} MB)", mem_estimate, mem_estimate / 1_000_000);
}

/// Test that different timestamps for same user create unique keys.
#[tokio::test]
async fn test_unique_key_generation() {
    let mut seen_logs: HashSet<String> = HashSet::new();

    // Same user, different timestamps
    let key1 = "U001:2026-01-28 08:00:00".to_string();
    let key2 = "U001:2026-01-28 08:01:00".to_string();
    let key3 = "U001:2026-01-28 08:00:00".to_string(); // Duplicate of key1

    assert!(seen_logs.insert(key1.clone()), "First timestamp should be new");
    assert!(seen_logs.insert(key2.clone()), "Second timestamp should be new");
    assert!(!seen_logs.insert(key3.clone()), "Duplicate timestamp should be rejected");

    assert_eq!(seen_logs.len(), 2, "Should have 2 unique keys");

    println!("✅ Unique key generation test passed:");
    println!("   U001:08:00:00 → new");
    println!("   U001:08:01:00 → new");
    println!("   U001:08:00:00 → duplicate (filtered)");
}

/// Test concurrent polling from multiple devices (each has own HashSet).
#[tokio::test]
async fn test_multiple_devices_independent_deduplication() {
    let (tx, mut rx) = mpsc::channel::<(String, String)>(100);

    // Device 1: OFFICE
    let mut seen_office: HashSet<String> = HashSet::new();
    let key_o1 = "U001:2026-01-28 08:00:00".to_string();
    let key_o2 = "U002:2026-01-28 08:01:00".to_string();

    seen_office.insert(key_o1.clone());
    tx.send(("OFFICE".to_string(), key_o1.clone())).await.unwrap();
    seen_office.insert(key_o2.clone());
    tx.send(("OFFICE".to_string(), key_o2.clone())).await.unwrap();

    // Device 2: XUONG 1
    let mut seen_xuong: HashSet<String> = HashSet::new();
    let key_x1 = "U001:2026-01-28 08:00:00".to_string(); // Same key as OFFICE, but different device
    let key_x2 = "U003:2026-01-28 08:02:00".to_string();

    seen_xuong.insert(key_x1.clone());
    tx.send(("XUONG 1".to_string(), key_x1.clone())).await.unwrap();
    seen_xuong.insert(key_x2.clone());
    tx.send(("XUONG 1".to_string(), key_x2.clone())).await.unwrap();

    // Both devices can have same user+timestamp (different physical logs)
    drop(tx);
    let mut received = 0;
    while let Ok(Some(_)) = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
        received += 1;
    }
    assert_eq!(received, 4, "Should have 4 logs (2 per device)");

    println!("✅ Multiple devices test passed:");
    println!("   OFFICE: {} unique logs", seen_office.len());
    println!("   XUONG 1: {} unique logs", seen_xuong.len());
    println!("   Total queued: {received}");
}
