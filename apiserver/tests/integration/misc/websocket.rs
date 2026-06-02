//! Websocket integration tests

use crate::common::app::spawn_app_with_factory;
use apiserver::queue::{JobMetadata, JobQueue, JobStatus, ReportPublisher, ReportType};
use apiserver::services::auth_service::AuthService;
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

#[tokio::test]
async fn test_websocket_connection_race_completed() {
    let (address, workspace_id, user_id, factory, _test_db) = spawn_app_with_factory().await;
    let redis_pool = factory
        .redis_pool
        .clone()
        .expect("Redis pool required for WS test");

    // 1. Generate a valid token
    let token = AuthService::generate_token(user_id, "test_secret_tower", 60).unwrap();

    // 2. Set up completed job metadata in Redis
    let job_id = Uuid::new_v4();
    let queue = JobQueue::new(redis_pool.clone());
    let meta = JobMetadata {
        job_id,
        status: JobStatus::Completed,
        total_batches: 2,
        completed_batches: 2,
        failed_batches: 0,
        workspace_id,
        requester_id: user_id,
        report_type: Some(ReportType::Summary),
        date_range: None,
        created_at: chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
        error_message: None,
        task_type: None,
        payload: None,
    };
    queue.create_job_metadata(&meta).await.unwrap();

    // 3. Connect to report streaming WS route
    let ws_url = format!(
        "{}/api/v1/ws/reports/{}?token={}",
        address.replace("http", "ws"),
        job_id,
        token
    );

    let (mut ws_stream, _response) = connect_async(ws_url).await.expect("Failed to connect");

    // 4. Expect immediate DONE message and connection closed
    let msg = ws_stream
        .next()
        .await
        .expect("Expected message")
        .expect("Failed to read message");
    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "DONE");
    } else {
        panic!("Expected text message");
    }

    // Verify socket closed
    let next_msg = ws_stream.next().await;
    assert!(next_msg.is_none() || matches!(next_msg, Some(Ok(Message::Close(_)))));
}

#[tokio::test]
async fn test_websocket_connection_streaming() {
    let (address, workspace_id, user_id, factory, _test_db) = spawn_app_with_factory().await;
    let redis_pool = factory
        .redis_pool
        .clone()
        .expect("Redis pool required for WS test");

    // 1. Generate token
    let token = AuthService::generate_token(user_id, "test_secret_tower", 60).unwrap();

    // 2. Set up pending job metadata
    let job_id = Uuid::new_v4();
    let queue = JobQueue::new(redis_pool.clone());
    let meta = JobMetadata::new(
        job_id,
        2,
        workspace_id,
        user_id,
        None,
        Some(ReportType::Summary),
        None,
    );
    queue.create_job_metadata(&meta).await.unwrap();

    // 3. Connect to WS
    let ws_url = format!(
        "{}/api/v1/ws/reports/{}?token={}",
        address.replace("http", "ws"),
        job_id,
        token
    );

    let (mut ws_stream, _response) = connect_async(ws_url).await.expect("Failed to connect");

    // 4. Publish stream messages via ReportPublisher
    let publisher = ReportPublisher::new(redis_pool.clone());

    // Wait for subscription to be registered in Redis
    let mut attempts = 0;
    while publisher.subscriber_count(job_id).await.unwrap_or(0) == 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        attempts += 1;
        if attempts > 200 {
            panic!("WebSocket client failed to subscribe in time");
        }
    }

    // Publish chunk
    publisher.publish_chunk(job_id, 0, 2).await.unwrap();

    // Read and verify CHUNK message
    let msg = ws_stream
        .next()
        .await
        .expect("Expected CHUNK message")
        .expect("Failed to read CHUNK");
    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "CHUNK");
        assert_eq!(parsed["batch_index"], 0);
        assert_eq!(parsed["total_batches"], 2);
    } else {
        panic!("Expected text message");
    }

    // Publish done
    publisher.publish_done(job_id, 2, 10).await.unwrap();

    // Read and verify DONE message
    let msg = ws_stream
        .next()
        .await
        .expect("Expected DONE message")
        .expect("Failed to read DONE");
    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "DONE");
        assert_eq!(parsed["total_batches"], 2);
        assert_eq!(parsed["total_records"], 10);
    } else {
        panic!("Expected text message");
    }

    // Verify stream ends
    let next_msg = ws_stream.next().await;
    assert!(next_msg.is_none() || matches!(next_msg, Some(Ok(Message::Close(_)))));
}

#[tokio::test]
async fn test_websocket_auth_failures() {
    let (address, _workspace_id, _user_id, _factory, _test_db) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let job_id = Uuid::new_v4();

    // 1. Missing token
    let url = format!("{}/api/v1/ws/reports/{}", address, job_id);
    let res = client.get(&url).send().await.unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);
    let body = res.text().await.unwrap();
    assert!(body.contains("Missing authentication token"));

    // 2. Invalid token
    let url_invalid = format!(
        "{}/api/v1/ws/reports/{}?token=invalid_token",
        address, job_id
    );
    let res = client.get(&url_invalid).send().await.unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::UNAUTHORIZED);
    let body = res.text().await.unwrap();
    assert!(body.contains("Invalid or expired token"));
}

#[tokio::test]
async fn test_websocket_job_not_found() {
    let (address, _workspace_id, user_id, factory, _test_db) = spawn_app_with_factory().await;
    let token = AuthService::generate_token(user_id, "test_secret_tower", 60).unwrap();
    let non_existent_job_id = Uuid::new_v4();

    let ws_url = format!(
        "{}/api/v1/ws/reports/{}?token={}",
        address.replace("http", "ws"),
        non_existent_job_id,
        token
    );
    let res = connect_async(ws_url).await;
    match res {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
        }
        other => panic!("Expected HTTP 404 error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_websocket_forbidden_workspace() {
    let (address, workspace_id, user_id, factory, _test_db) = spawn_app_with_factory().await;
    let redis_pool = factory.redis_pool.clone().expect("Redis pool required");

    // Create another user that is NOT a member of the workspace
    let other_user_id = factory.user().create().await;
    let other_token = AuthService::generate_token(other_user_id, "test_secret_tower", 60).unwrap();

    // Set up job metadata owned by user_id in workspace_id
    let job_id = Uuid::new_v4();
    let queue = JobQueue::new(redis_pool.clone());
    let meta = JobMetadata::new(
        job_id,
        2,
        workspace_id,
        user_id,
        None,
        Some(ReportType::Summary),
        None,
    );
    queue.create_job_metadata(&meta).await.unwrap();

    let ws_url = format!(
        "{}/api/v1/ws/reports/{}?token={}",
        address.replace("http", "ws"),
        job_id,
        other_token
    );
    let res = connect_async(ws_url).await;
    match res {
        Err(tokio_tungstenite::tungstenite::Error::Http(response)) => {
            assert_eq!(response.status(), reqwest::StatusCode::FORBIDDEN);
        }
        other => panic!("Expected HTTP 403 error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_websocket_connection_race_failed() {
    let (address, workspace_id, user_id, factory, _test_db) = spawn_app_with_factory().await;
    let redis_pool = factory.redis_pool.clone().expect("Redis pool required");
    let token = AuthService::generate_token(user_id, "test_secret_tower", 60).unwrap();

    let job_id = Uuid::new_v4();
    let queue = JobQueue::new(redis_pool.clone());
    let meta = JobMetadata {
        job_id,
        status: JobStatus::Failed,
        total_batches: 2,
        completed_batches: 0,
        failed_batches: 2,
        workspace_id,
        requester_id: user_id,
        report_type: Some(ReportType::Summary),
        date_range: None,
        created_at: chrono::Utc::now(),
        completed_at: Some(chrono::Utc::now()),
        error_message: Some("Database timeout".to_string()),
        task_type: None,
        payload: None,
    };
    queue.create_job_metadata(&meta).await.unwrap();

    let ws_url = format!(
        "{}/api/v1/ws/reports/{}?token={}",
        address.replace("http", "ws"),
        job_id,
        token
    );

    let (mut ws_stream, _response) = connect_async(ws_url).await.expect("Failed to connect");

    // Expect ERROR message and connection closed
    let msg = ws_stream
        .next()
        .await
        .expect("Expected message")
        .expect("Failed to read message");
    if let Message::Text(text) = msg {
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["type"], "ERROR");
        assert_eq!(parsed["message"], "Database timeout");
    } else {
        panic!("Expected text message");
    }

    let next_msg = ws_stream.next().await;
    assert!(next_msg.is_none() || matches!(next_msg, Some(Ok(Message::Close(_)))));
}
