//! Worker Monitor integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use redis::AsyncCommands;
use reqwest::StatusCode;
use uuid::Uuid;
use futures_util::StreamExt;
use apiserver::queue::RedisPool;

async fn get_test_redis() -> RedisPool {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set for monitor tests");
    RedisPool::new(&redis_url).await.expect("Failed to create Redis pool")
}

#[tokio::test]
async fn test_monitor_overview_rest() {
    let (address, _, _, _, _): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Mock some data in Redis
    let redis_pool = get_test_redis().await;
    let mut conn = redis_pool.conn();
    
    let worker_id = Uuid::new_v4();
    let heartbeat = serde_json::json!({
        "id": worker_id,
        "host": "test-host",
        "status": "IDLE",
        "current_task_type": null,
        "last_seen_at": chrono::Utc::now()
    });
    
    let _: () = conn.set(format!("monitor:worker:{}", worker_id), heartbeat.to_string()).await.unwrap();

    // 2. Call the overview endpoint
    let response = client
        .get(format!("{address}/api/monitor/overview"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    
    // total_workers might be more than 1 if other tests are running, but should be at least 1
    assert!(body["total_workers"].as_u64().unwrap() >= 1);
    let workers = body["workers"].as_array().unwrap();
    assert!(workers.iter().any(|w| w["id"] == worker_id.to_string()));
}

#[tokio::test]
async fn test_monitor_stream_sse() {
    let (address, _, _, _, _): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Mock some data in Redis
    let redis_pool = get_test_redis().await;
    let mut conn = redis_pool.conn();
    
    let worker_id = Uuid::new_v4();
    let heartbeat = serde_json::json!({
        "id": worker_id,
        "host": "sse-test-host",
        "status": "PROCESSING",
        "current_task_type": "TEST_TASK",
        "last_seen_at": chrono::Utc::now()
    });
    
    let _: () = conn.set(format!("monitor:worker:{}", worker_id), heartbeat.to_string()).await.unwrap();

    // 2. Connect to SSE stream
    let response = client
        .get(format!("{address}/api/monitor/stream"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Note: Axum's Sse response might have other headers, but content-type should be text/event-stream
    let content_type = response.headers().get("content-type").unwrap().to_str().unwrap();
    assert!(content_type.contains("text/event-stream"));

    // 3. Read first chunk
    let mut stream = response.bytes_stream();
    let chunk = stream.next().await.unwrap().unwrap();
    let body = String::from_utf8_lossy(&chunk);
    
    // SSE format: event: monitor-update\ndata: {...}\n\n
    assert!(body.contains("event: monitor-update"));
    assert!(body.contains("sse-test-host"));
    assert!(body.contains("\"status\":\"PROCESSING\""));
}
