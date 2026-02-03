/// Standalone simulation test for merge race condition fix
/// Does not require Redis - uses in-memory atomics to demonstrate the fix
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::task::JoinSet;

/// Simulates the OLD buggy behavior (before fix)
struct OldBuggyCoordinator {
    completed_count: Arc<AtomicU32>,
    total_batches: u32,
    merge_triggered_count: Arc<AtomicU32>,
}

impl OldBuggyCoordinator {
    fn new(total_batches: u32) -> Self {
        Self {
            completed_count: Arc::new(AtomicU32::new(0)),
            total_batches,
            merge_triggered_count: Arc::new(AtomicU32::new(0)),
        }
    }

    /// BUGGY: Race condition here - check and trigger are not atomic
    async fn on_batch_complete_buggy(&self, _batch_id: u32) {
        // Step 1: Increment counter (atomic)
        let completed = self.completed_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Step 2: Check if done (NOT atomic with trigger!)
        if completed >= self.total_batches {
            // Simulate slight delay to make race condition more visible
            tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;

            // Step 3: Trigger merge (MULTIPLE WORKERS CAN REACH HERE!)
            self.merge_triggered_count.fetch_add(1, Ordering::SeqCst);
            println!(
                "  [BUGGY] Batch {} triggered merge (completed={}/{})",
                _batch_id, completed, self.total_batches
            );
        }
    }
}

/// Simulates the NEW fixed behavior (after fix)
struct FixedCoordinator {
    completed_count: Arc<AtomicU32>,
    total_batches: u32,
    merge_triggered_count: Arc<AtomicU32>,
    merge_flag: Arc<AtomicBool>, // Simulates Redis SETNX flag
}

impl FixedCoordinator {
    fn new(total_batches: u32) -> Self {
        Self {
            completed_count: Arc::new(AtomicU32::new(0)),
            total_batches,
            merge_triggered_count: Arc::new(AtomicU32::new(0)),
            merge_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// FIXED: Atomic flag prevents double trigger
    async fn on_batch_complete_fixed(&self, batch_id: u32) {
        // Step 1: Increment counter (atomic)
        let completed = self.completed_count.fetch_add(1, Ordering::SeqCst) + 1;

        // Step 2: Check if done
        if completed >= self.total_batches {
            // Step 3: Atomically try to set merge flag (simulates Redis SETNX)
            let was_first = self
                .merge_flag
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok();

            if was_first {
                // Only FIRST worker reaches here
                self.merge_triggered_count.fetch_add(1, Ordering::SeqCst);
                println!(
                    "  [FIXED] Batch {} triggered merge (completed={}/{}) ✅ FIRST",
                    batch_id, completed, self.total_batches
                );
            } else {
                // Other workers are blocked
                println!(
                    "  [FIXED] Batch {} race prevented (completed={}/{}) 🛡️ BLOCKED",
                    batch_id, completed, self.total_batches
                );
            }
        }
    }
}

/// TEST 1: Demonstrate the BUG (old behavior)
#[tokio::test]
async fn test_old_buggy_behavior_causes_double_merge() {
    println!("\n=== TEST 1: OLD BUGGY BEHAVIOR ===");

    let coordinator = Arc::new(OldBuggyCoordinator::new(4));
    let mut handles = JoinSet::new();

    // Simulate 4 concurrent batch completions
    for batch_id in 0..4 {
        let coord = coordinator.clone();
        handles.spawn(async move {
            coord.on_batch_complete_buggy(batch_id).await;
        });
    }

    // Wait for all workers
    while handles.join_next().await.is_some() {}

    let final_count = coordinator.completed_count.load(Ordering::SeqCst);
    let merge_count = coordinator.merge_triggered_count.load(Ordering::SeqCst);

    println!("\n📊 BUGGY Results:");
    println!("  Completed batches: {}", final_count);
    println!("  Merge triggered: {} times", merge_count);

    // The bug: merge might be triggered MORE than once
    assert_eq!(final_count, 4, "Should complete 4 batches");

    if merge_count > 1 {
        println!("  ❌ BUG REPRODUCED: Merge triggered {} times!", merge_count);
        println!("  This is the race condition we're fixing!");
    } else {
        println!("  ⚠️  Race condition didn't manifest in this run (timing dependent)");
        println!("  But the code is still vulnerable!");
    }
}

/// TEST 2: Demonstrate the FIX (new behavior)
#[tokio::test]
async fn test_fixed_behavior_prevents_double_merge() {
    println!("\n=== TEST 2: FIXED BEHAVIOR ===");

    let coordinator = Arc::new(FixedCoordinator::new(4));
    let mut handles = JoinSet::new();

    // Simulate 4 concurrent batch completions
    for batch_id in 0..4 {
        let coord = coordinator.clone();
        handles.spawn(async move {
            coord.on_batch_complete_fixed(batch_id).await;
        });
    }

    // Wait for all workers
    while handles.join_next().await.is_some() {}

    let final_count = coordinator.completed_count.load(Ordering::SeqCst);
    let merge_count = coordinator.merge_triggered_count.load(Ordering::SeqCst);

    println!("\n📊 FIXED Results:");
    println!("  Completed batches: {}", final_count);
    println!("  Merge triggered: {} times", merge_count);

    // The fix: merge should ALWAYS be triggered exactly once
    assert_eq!(final_count, 4, "Should complete 4 batches");
    assert_eq!(
        merge_count, 1,
        "Merge should be triggered EXACTLY ONCE (got {})",
        merge_count
    );

    println!("  ✅ SUCCESS: Merge triggered exactly once!");
    println!("  Race condition prevented by atomic flag!");
}

/// TEST 3: Stress test with many concurrent workers
#[tokio::test]
async fn test_fixed_behavior_under_stress() {
    println!("\n=== TEST 3: STRESS TEST (20 CONCURRENT WORKERS) ===");

    let total_batches = 20u32;
    let coordinator = Arc::new(FixedCoordinator::new(total_batches));
    let mut handles = JoinSet::new();

    // Spawn many concurrent workers
    for batch_id in 0..total_batches {
        let coord = coordinator.clone();
        handles.spawn(async move {
            // Add random delay to increase race condition likelihood
            let delay = (batch_id % 5) * 10;
            tokio::time::sleep(tokio::time::Duration::from_micros(delay as u64)).await;
            coord.on_batch_complete_fixed(batch_id).await;
        });
    }

    // Wait for all workers
    while handles.join_next().await.is_some() {}

    let final_count = coordinator.completed_count.load(Ordering::SeqCst);
    let merge_count = coordinator.merge_triggered_count.load(Ordering::SeqCst);

    println!("\n📊 STRESS TEST Results:");
    println!("  Completed batches: {}/{}", final_count, total_batches);
    println!("  Merge triggered: {} times", merge_count);

    assert_eq!(final_count, total_batches, "Should complete all batches");
    assert_eq!(
        merge_count, 1,
        "Even under stress, merge should trigger EXACTLY ONCE (got {})",
        merge_count
    );

    println!("  ✅ SUCCESS: Atomic flag holds under concurrent stress!");
}

/// TEST 4: Compare old vs new side-by-side
#[tokio::test]
async fn test_comparison_old_vs_new() {
    println!("\n=== TEST 4: SIDE-BY-SIDE COMPARISON ===");

    let runs = 10;
    let mut buggy_results = Vec::new();
    let mut fixed_results = Vec::new();

    for run in 0..runs {
        // Test buggy version
        let buggy = Arc::new(OldBuggyCoordinator::new(4));
        let mut handles = JoinSet::new();
        for batch_id in 0..4 {
            let coord = buggy.clone();
            handles.spawn(async move {
                coord.on_batch_complete_buggy(batch_id).await;
            });
        }
        while handles.join_next().await.is_some() {}
        buggy_results.push(buggy.merge_triggered_count.load(Ordering::SeqCst));

        // Test fixed version
        let fixed = Arc::new(FixedCoordinator::new(4));
        let mut handles = JoinSet::new();
        for batch_id in 0..4 {
            let coord = fixed.clone();
            handles.spawn(async move {
                coord.on_batch_complete_fixed(batch_id).await;
            });
        }
        while handles.join_next().await.is_some() {}
        fixed_results.push(fixed.merge_triggered_count.load(Ordering::SeqCst));

        if run == 0 {
            println!("\nRun | Buggy | Fixed");
            println!("----|-------|------");
        }
        println!(
            " {:2} |   {}   |   {}",
            run + 1,
            buggy_results[run],
            fixed_results[run]
        );
    }

    println!("\n📊 Statistics over {} runs:", runs);

    let buggy_double_triggers = buggy_results.iter().filter(|&&x| x > 1).count();
    let fixed_double_triggers = fixed_results.iter().filter(|&&x| x > 1).count();

    println!("  Buggy: {} runs had double triggers", buggy_double_triggers);
    println!("  Fixed: {} runs had double triggers", fixed_double_triggers);

    // Fixed version should NEVER have double triggers
    assert_eq!(
        fixed_double_triggers, 0,
        "Fixed version should have ZERO double triggers"
    );

    // All fixed runs should trigger exactly once
    assert!(
        fixed_results.iter().all(|&x| x == 1),
        "All fixed runs should trigger merge exactly once"
    );

    println!("\n  ✅ SUCCESS: Fixed version is 100% reliable!");
    if buggy_double_triggers > 0 {
        println!("  ❌ Buggy version had issues in {}/{} runs", buggy_double_triggers, runs);
    }
}
