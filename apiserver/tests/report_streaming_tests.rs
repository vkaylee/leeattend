//! Integration tests for the report streaming system (Redis Pub/Sub).
//!
//! Tests cover:
//! - StreamMessage serialization/deserialization
//! - ReportPublisher publishing methods
//! - ReportSubscriber subscription and message receiving
//! - End-to-end pub/sub integration
//!
//! Note: Tests marked with #[ignore] require a running Redis instance.
//! Run with: cargo test -- --ignored

use apiserver::queue::{RedisPool, ReportPublisher, ReportSubscriber, StreamMessage};
use serde_json::json;
use uuid::Uuid;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a test Redis pool.
/// Requires REDIS_URL environment variable.
#[allow(dead_code)]
async fn create_test_redis_pool() -> RedisPool {
    dotenvy::dotenv().ok();
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    RedisPool::new(&redis_url)
        .await
        .expect("Failed to connect to Redis for testing")
}

// ============================================================================
// Unit Tests (No Redis Required)
// ============================================================================

#[test]
fn test_stream_message_chunk_construction() {
    let data = json!({
        "employees": [
            {"id": "123", "name": "Alice", "hours": 8.0}
        ]
    });

    let msg = StreamMessage::chunk(0, 5, data.clone());

    match msg {
        StreamMessage::Chunk { batch_index, total_batches, data: msg_data } => {
            assert_eq!(batch_index, 0);
            assert_eq!(total_batches, 5);
            assert_eq!(msg_data, data);
        }
        _ => panic!("Expected Chunk variant"),
    }
}

#[test]
fn test_stream_message_done_construction() {
    let msg = StreamMessage::done(10, 500);

    match msg {
        StreamMessage::Done { total_batches, total_records } => {
            assert_eq!(total_batches, 10);
            assert_eq!(total_records, 500);
        }
        _ => panic!("Expected Done variant"),
    }
}

#[test]
fn test_stream_message_error_construction() {
    let msg = StreamMessage::error("Database timeout", Some(3));

    match msg {
        StreamMessage::Error { message, batch_index } => {
            assert_eq!(message, "Database timeout");
            assert_eq!(batch_index, Some(3));
        }
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_stream_message_error_without_batch_index() {
    let msg = StreamMessage::error("General failure", None);

    match msg {
        StreamMessage::Error { message, batch_index } => {
            assert_eq!(message, "General failure");
            assert_eq!(batch_index, None);
        }
        _ => panic!("Expected Error variant"),
    }
}

#[test]
fn test_stream_message_chunk_json_format() {
    let data = json!({"test": "value"});
    let msg = StreamMessage::chunk(1, 3, data);
    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"CHUNK\""));
    assert!(json.contains("\"batch_index\":1"));
    assert!(json.contains("\"total_batches\":3"));
    assert!(json.contains("\"test\":\"value\""));
}

#[test]
fn test_stream_message_done_json_format() {
    let msg = StreamMessage::done(7, 350);
    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"DONE\""));
    assert!(json.contains("\"total_batches\":7"));
    assert!(json.contains("\"total_records\":350"));
}

#[test]
fn test_stream_message_error_json_format() {
    let msg = StreamMessage::error("Connection lost", Some(4));
    let json = serde_json::to_string(&msg).unwrap();

    assert!(json.contains("\"type\":\"ERROR\""));
    assert!(json.contains("\"message\":\"Connection lost\""));
    assert!(json.contains("\"batch_index\":4"));
}

#[test]
fn test_stream_message_roundtrip_chunk() {
    let original_data = json!({
        "records": [
            {"id": 1, "value": "test1"},
            {"id": 2, "value": "test2"}
        ],
        "metadata": {"count": 2}
    });

    let original = StreamMessage::chunk(2, 10, original_data.clone());
    let json = serde_json::to_string(&original).unwrap();
    let decoded: StreamMessage = serde_json::from_str(&json).unwrap();

    match decoded {
        StreamMessage::Chunk { batch_index, total_batches, data } => {
            assert_eq!(batch_index, 2);
            assert_eq!(total_batches, 10);
            assert_eq!(data, original_data);
        }
        _ => panic!("Expected Chunk after deserialization"),
    }
}

#[test]
fn test_stream_message_roundtrip_done() {
    let original = StreamMessage::done(15, 1500);
    let json = serde_json::to_string(&original).unwrap();
    let decoded: StreamMessage = serde_json::from_str(&json).unwrap();

    match decoded {
        StreamMessage::Done { total_batches, total_records } => {
            assert_eq!(total_batches, 15);
            assert_eq!(total_records, 1500);
        }
        _ => panic!("Expected Done after deserialization"),
    }
}

#[test]
fn test_stream_message_roundtrip_error() {
    let original = StreamMessage::error("Query timeout", Some(7));
    let json = serde_json::to_string(&original).unwrap();
    let decoded: StreamMessage = serde_json::from_str(&json).unwrap();

    match decoded {
        StreamMessage::Error { message, batch_index } => {
            assert_eq!(message, "Query timeout");
            assert_eq!(batch_index, Some(7));
        }
        _ => panic!("Expected Error after deserialization"),
    }
}

// ============================================================================
// Integration Tests (Require Redis)
// ============================================================================

#[tokio::test]
#[ignore] // Requires Redis
async fn test_publisher_publish_chunk() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let publisher = ReportPublisher::new(pool);
    let job_id = Uuid::new_v4();

    let data = json!({"batch": "data", "count": 100});
    let result = publisher.publish_chunk(job_id, 0, 5, data).await;

    assert!(result.is_ok(), "Failed to publish chunk: {:?}", result.err());
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_publisher_publish_done() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let publisher = ReportPublisher::new(pool);
    let job_id = Uuid::new_v4();

    let result = publisher.publish_done(job_id, 10, 1000).await;

    assert!(result.is_ok(), "Failed to publish done: {:?}", result.err());
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_publisher_publish_error() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let publisher = ReportPublisher::new(pool);
    let job_id = Uuid::new_v4();

    let result = publisher.publish_error(job_id, "Test error", Some(2)).await;

    assert!(result.is_ok(), "Failed to publish error: {:?}", result.err());
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_publisher_subscriber_count_no_subscribers() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let publisher = ReportPublisher::new(pool);
    let job_id = Uuid::new_v4();

    let count = publisher.subscriber_count(job_id)
        .await
        .expect("Failed to get subscriber count");

    assert_eq!(count, 0, "Expected 0 subscribers for new job");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_subscriber_creation() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    let subscriber = ReportSubscriber::new(&pool, job_id).await;

    assert!(subscriber.is_ok(), "Failed to create subscriber: {:?}", subscriber.err());
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_subscriber_close() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    let subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    let result = subscriber.close().await;

    assert!(result.is_ok(), "Failed to close subscriber: {:?}", result.err());
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_single_chunk() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    // Create subscriber first (must subscribe before publishing)
    let mut subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    // Give subscription time to register
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Publish a chunk
    let publisher = ReportPublisher::new(pool.clone());
    let test_data = json!({"employee_id": "E001", "hours": 8.5});
    publisher.publish_chunk(job_id, 0, 1, test_data.clone())
        .await
        .expect("Failed to publish chunk");

    // Receive the message
    let received = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next_message()
    )
    .await
    .expect("Timeout waiting for message")
    .expect("No message received");

    // Verify the message
    match received {
        StreamMessage::Chunk { batch_index, total_batches, data } => {
            assert_eq!(batch_index, 0);
            assert_eq!(total_batches, 1);
            assert_eq!(data, test_data);
        }
        _ => panic!("Expected Chunk message, got {:?}", received),
    }

    // Cleanup
    subscriber.close().await.expect("Failed to close subscriber");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_multiple_chunks() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    // Create subscriber
    let mut subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Publish multiple chunks
    let publisher = ReportPublisher::new(pool.clone());
    let total_chunks = 3;

    for i in 0..total_chunks {
        let data = json!({"batch": i, "data": format!("chunk_{}", i)});
        publisher.publish_chunk(job_id, i, total_chunks, data)
            .await
            .expect("Failed to publish chunk");
    }

    // Receive all chunks
    for expected_idx in 0..total_chunks {
        let received = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            subscriber.next_message()
        )
        .await
        .expect("Timeout waiting for message")
        .expect("No message received");

        match received {
            StreamMessage::Chunk { batch_index, total_batches, .. } => {
                assert_eq!(batch_index, expected_idx);
                assert_eq!(total_batches, total_chunks);
            }
            _ => panic!("Expected Chunk message"),
        }
    }

    subscriber.close().await.expect("Failed to close subscriber");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_done_message() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    let mut subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Publish done message
    let publisher = ReportPublisher::new(pool.clone());
    publisher.publish_done(job_id, 5, 250)
        .await
        .expect("Failed to publish done");

    // Receive done message
    let received = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next_message()
    )
    .await
    .expect("Timeout waiting for message")
    .expect("No message received");

    match received {
        StreamMessage::Done { total_batches, total_records } => {
            assert_eq!(total_batches, 5);
            assert_eq!(total_records, 250);
        }
        _ => panic!("Expected Done message"),
    }

    subscriber.close().await.expect("Failed to close subscriber");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_error_message() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    let mut subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Publish error message
    let publisher = ReportPublisher::new(pool.clone());
    publisher.publish_error(job_id, "Database connection failed", Some(3))
        .await
        .expect("Failed to publish error");

    // Receive error message
    let received = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next_message()
    )
    .await
    .expect("Timeout waiting for message")
    .expect("No message received");

    match received {
        StreamMessage::Error { message, batch_index } => {
            assert_eq!(message, "Database connection failed");
            assert_eq!(batch_index, Some(3));
        }
        _ => panic!("Expected Error message"),
    }

    subscriber.close().await.expect("Failed to close subscriber");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_full_workflow() {
    // Requires Redis
    // Simulates a complete report generation workflow:
    // CHUNK -> CHUNK -> CHUNK -> DONE
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    let mut subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let publisher = ReportPublisher::new(pool.clone());
    let total_batches = 3;
    let records_per_batch = 100;

    // Publish chunks
    for i in 0..total_batches {
        let data = json!({
            "employees": vec![
                json!({"id": format!("E{:03}", i * records_per_batch), "hours": 8.0})
            ]
        });
        publisher.publish_chunk(job_id, i, total_batches, data)
            .await
            .expect("Failed to publish chunk");
    }

    // Publish completion
    publisher.publish_done(job_id, total_batches, (total_batches * records_per_batch) as usize)
        .await
        .expect("Failed to publish done");

    // Receive all chunks
    for expected_idx in 0..total_batches {
        let msg = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            subscriber.next_message()
        )
        .await
        .expect("Timeout")
        .expect("No message");

        match msg {
            StreamMessage::Chunk { batch_index, .. } => {
                assert_eq!(batch_index, expected_idx);
            }
            _ => panic!("Expected Chunk"),
        }
    }

    // Receive done message
    let done_msg = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        subscriber.next_message()
    )
    .await
    .expect("Timeout")
    .expect("No message");

    match done_msg {
        StreamMessage::Done { total_batches: tb, total_records: tr } => {
            assert_eq!(tb, total_batches);
            assert_eq!(tr, (total_batches * records_per_batch) as usize);
        }
        _ => panic!("Expected Done"),
    }

    subscriber.close().await.expect("Failed to close subscriber");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_subscriber_count() {
    // Requires Redis
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();
    let publisher = ReportPublisher::new(pool.clone());

    // No subscribers initially
    let count = publisher.subscriber_count(job_id).await.expect("Failed to get count");
    assert_eq!(count, 0);

    // Create a subscriber
    let subscriber = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Should have 1 subscriber
    let count = publisher.subscriber_count(job_id).await.expect("Failed to get count");
    assert_eq!(count, 1);

    // Close subscriber
    subscriber.close().await.expect("Failed to close");

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    // Back to 0 subscribers
    let count = publisher.subscriber_count(job_id).await.expect("Failed to get count");
    assert_eq!(count, 0);
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_pubsub_integration_multiple_subscribers() {
    // Requires Redis
    // Multiple clients can subscribe to the same job
    let pool = create_test_redis_pool().await;
    let job_id = Uuid::new_v4();

    // Create two subscribers
    let mut sub1 = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber 1");

    let mut sub2 = ReportSubscriber::new(&pool, job_id)
        .await
        .expect("Failed to create subscriber 2");

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Verify subscriber count
    let publisher = ReportPublisher::new(pool.clone());
    let count = publisher.subscriber_count(job_id).await.expect("Failed to get count");
    assert_eq!(count, 2);

    // Publish a message
    let test_data = json!({"test": "multi-subscriber"});
    publisher.publish_chunk(job_id, 0, 1, test_data.clone())
        .await
        .expect("Failed to publish");

    // Both subscribers should receive it
    let msg1 = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        sub1.next_message()
    )
    .await
    .expect("Timeout sub1")
    .expect("No message sub1");

    let msg2 = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        sub2.next_message()
    )
    .await
    .expect("Timeout sub2")
    .expect("No message sub2");

    // Both should be identical chunks
    match (msg1, msg2) {
        (
            StreamMessage::Chunk { data: data1, .. },
            StreamMessage::Chunk { data: data2, .. }
        ) => {
            assert_eq!(data1, test_data);
            assert_eq!(data2, test_data);
        }
        _ => panic!("Expected both to be Chunk messages"),
    }

    sub1.close().await.expect("Failed to close sub1");
    sub2.close().await.expect("Failed to close sub2");
}

#[tokio::test]
#[ignore] // Requires Redis
async fn test_redis_pool_health_check() {
    // Requires Redis
    let pool = create_test_redis_pool().await;

    let result = pool.health_check().await;

    assert!(result.is_ok(), "Redis health check failed: {:?}", result.err());
}
