use tokio::sync::Semaphore;
use std::sync::Arc;

#[tokio::test]
async fn test_semaphore_safety_pattern() {
    let sem = Arc::new(Semaphore::new(1));
    let sem_clone = sem.clone();
    
    // Simulate the logic we just implemented
    let handle = tokio::spawn(async move {
        // This simulates our worker loop
        loop {
            let _permit = match sem_clone.acquire().await {
                Ok(p) => p,
                Err(_) => {
                    // This is our safe exit path
                    return "safe_exit";
                }
            };
            // Do some work
            tokio::task::yield_now().await;
        }
    });

    // Close the semaphore to trigger the exit
    sem.close();
    
    let result = handle.await.unwrap();
    assert_eq!(result, "safe_exit", "Worker should exit safely when semaphore is closed, not panic");
}
