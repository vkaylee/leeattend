//! Worker Monitor integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::queue::RedisPool;
use chrono::{Duration, Utc};
use futures_util::StreamExt;
use redis::AsyncCommands;
use reqwest::StatusCode;
use uuid::Uuid;

/// Grant a system role to the given user so they can access system endpoints.
async fn grant_system_role(factory: &TestFactory, user_id: Uuid, role_name: &str) {
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id, assigned_by)
         SELECT $1, id, $1 FROM system_roles WHERE name = $2
         ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid,
        user_id,
        diesel::sql_types::Varchar,
        role_name.to_string()
    );
}

async fn grant_platform_admin(factory: &TestFactory, user_id: Uuid) {
    grant_system_role(factory, user_id, "platform_admin").await;
}

async fn seed_durable_job(
    factory: &TestFactory,
    workspace_id: Uuid,
    state: &str,
    created_at: chrono::DateTime<Utc>,
) -> (Uuid, Uuid, String) {
    let job_id = Uuid::new_v4();
    let request_id = Uuid::new_v4();
    let trace_id = format!("trace-{job_id}");
    let event_id = Uuid::new_v4();

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO infra_job_records
         (id, workspace_id, parent_job_id, task_type, priority, state, attempt_count,
          request_id, trace_id, created_at, updated_at)
         VALUES ($1, $2, NULL, $3, $4, $5, 2, $6, $7, $8, $8)",
        diesel::sql_types::Uuid,
        job_id,
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Varchar,
        "monitor_test".to_string(),
        diesel::sql_types::Varchar,
        "normal".to_string(),
        diesel::sql_types::Varchar,
        state.to_string(),
        diesel::sql_types::Nullable<diesel::sql_types::Uuid>,
        Some(request_id),
        diesel::sql_types::Nullable<diesel::sql_types::Varchar>,
        Some(trace_id.clone()),
        diesel::sql_types::Timestamptz,
        created_at
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO infra_job_events
         (id, job_id, workspace_id, event_type, from_state, to_state, attempt_count,
          request_id, trace_id, reason_code, reason_summary, created_at)
         VALUES ($1, $2, $3, 'monitor_test', NULL, $4, 2, $5, $6,
                 'safe_test_code', 'safe test summary', $7)",
        diesel::sql_types::Uuid,
        event_id,
        diesel::sql_types::Uuid,
        job_id,
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Varchar,
        state.to_string(),
        diesel::sql_types::Nullable<diesel::sql_types::Uuid>,
        Some(request_id),
        diesel::sql_types::Nullable<diesel::sql_types::Varchar>,
        Some(trace_id.clone()),
        diesel::sql_types::Timestamptz,
        created_at
    );

    (job_id, request_id, trace_id)
}

#[tokio::test]
async fn test_monitor_overview_rest() {
    let (address, _, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Grant platform_admin so the user can access /monitor/overview
    grant_platform_admin(&factory, user_id).await;

    // 1. Mock some data in Redis
    let redis_pool = factory.redis_pool.clone().expect("redis pool expected");
    let mut conn = redis_pool.conn();

    let worker_id = Uuid::new_v4();
    let heartbeat = serde_json::json!({
        "id": worker_id,
        "host": "test-host",
        "status": "IDLE",
        "current_task_type": null,
        "last_seen_at": chrono::Utc::now()
    });

    let _: () = conn
        .set(format!("monitor:worker:{worker_id}"), heartbeat.to_string())
        .await
        .unwrap();

    // 2. Call the overview endpoint
    let response = client
        .get(format!("{address}/api/v1/monitor/overview"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["pending_normal"].is_number());

    // total_workers might be more than 1 if other tests are running, but should be at least 1
    assert!(body["total_workers"].as_u64().unwrap() >= 1);
    let workers = body["workers"].as_array().unwrap();
    assert!(workers.iter().any(|w| w["id"] == worker_id.to_string()));
}

#[tokio::test]
async fn test_monitor_stream_sse() {
    let (address, _, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Grant platform_admin so the user can access /monitor/stream
    grant_platform_admin(&factory, user_id).await;

    // 1. Mock some data in Redis
    let redis_pool = factory.redis_pool.clone().expect("redis pool expected");
    let mut conn = redis_pool.conn();

    let worker_id = Uuid::new_v4();
    let heartbeat = serde_json::json!({
        "id": worker_id,
        "host": "sse-test-host",
        "status": "PROCESSING",
        "current_task_type": "TEST_TASK",
        "last_seen_at": chrono::Utc::now()
    });

    let _: () = conn
        .set(format!("monitor:worker:{worker_id}"), heartbeat.to_string())
        .await
        .unwrap();

    // Generate valid access token
    let token = factory.auth().create_token(user_id);

    // 2. Connect to SSE stream
    let response = client
        .get(format!(
            "{address}/api/v1/monitor/stream?access_token={token}"
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    // Note: Axum's Sse response might have other headers, but content-type should be text/event-stream
    let content_type = response
        .headers()
        .get("content-type")
        .unwrap()
        .to_str()
        .unwrap();
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

#[tokio::test]
async fn test_monitor_jobs_preserves_durable_states_and_sanitizes_fields() {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    grant_platform_admin(&factory, user_id).await;

    let states = [
        "dispatching",
        "queued",
        "processing",
        "completed",
        "failed",
        "retryable",
        "dead_letter",
        "lost",
    ];
    let mut seeded = Vec::new();
    let now = Utc::now();
    for state in states {
        seeded.push(seed_durable_job(&factory, workspace_id, state, now).await);
    }

    let response = client
        .get(format!("{address}/api/v1/monitor/jobs?durable_limit=20"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("monitor jobs request should succeed");
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response
        .json()
        .await
        .expect("monitor jobs response should be JSON");
    assert_eq!(body["durable_jobs"]["status"], "available");
    let durable_jobs = body["durable_jobs"]["jobs"]
        .as_array()
        .expect("durable jobs should be present");
    assert_eq!(durable_jobs.len(), states.len());

    for (job_id, request_id, trace_id) in seeded {
        let job = durable_jobs
            .iter()
            .find(|job| job["job_id"] == job_id.to_string())
            .expect("seeded durable job should be returned");
        assert_eq!(job["request_id"], request_id.to_string());
        assert_eq!(job["trace_id"], trace_id);
        assert!(job["events"]
            .as_array()
            .is_some_and(|events| !events.is_empty()));
    }

    let body_text = serde_json::to_string(&body).expect("monitor response should serialize");
    assert!(!body_text.contains("lease_token"));
    assert!(!body_text.contains("payload"));
    assert!(!body_text.contains("requester"));
    assert!(!body_text.contains("raw error"));

    let returned_states = durable_jobs
        .iter()
        .filter_map(|job| job["state"].as_str())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(returned_states.len(), states.len());
    for state in states {
        assert!(
            returned_states.contains(state),
            "missing {state}; returned={returned_states:?}; body={body}"
        );
    }
}

#[tokio::test]
async fn test_monitor_jobs_requires_platform_admin_or_support() {
    let (address, _workspace_id, user_id, _factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let response = reqwest::Client::new()
        .get(format!("{address}/api/v1/monitor/jobs"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("monitor authorization request should complete");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_monitor_jobs_allows_support_role() {
    let (address, _workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_system_role(&factory, user_id, "support").await;

    let response = reqwest::Client::new()
        .get(format!("{address}/api/v1/monitor/jobs?durable_limit=1"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("support monitor authorization request should complete");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_monitor_durable_pagination_order_and_state_filter() {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_platform_admin(&factory, user_id).await;

    let now = Utc::now();
    let oldest =
        seed_durable_job(&factory, workspace_id, "queued", now - Duration::seconds(2)).await;
    let middle =
        seed_durable_job(&factory, workspace_id, "queued", now - Duration::seconds(1)).await;
    let newest = seed_durable_job(&factory, workspace_id, "queued", now).await;
    let _other_state =
        seed_durable_job(&factory, workspace_id, "failed", now + Duration::seconds(1)).await;

    let client = reqwest::Client::new();
    let first = client
        .get(format!(
            "{address}/api/v1/monitor/jobs?state=queued&durable_limit=2"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("first durable page should succeed");
    assert_eq!(first.status(), StatusCode::OK);
    let first_body: serde_json::Value = first
        .json()
        .await
        .expect("first durable page should be JSON");
    let first_jobs = first_body["durable_jobs"]["jobs"]
        .as_array()
        .expect("first durable page should contain jobs");
    assert_eq!(first_jobs.len(), 2);
    assert_eq!(first_jobs[0]["job_id"], newest.0.to_string());
    assert_eq!(first_jobs[1]["job_id"], middle.0.to_string());
    let cursor = first_body["durable_jobs"]["next_cursor_token"]
        .as_str()
        .expect("first durable page should have a continuation cursor");

    let second = client
        .get(format!(
            "{address}/api/v1/monitor/jobs?state=queued&durable_limit=2&durable_cursor={cursor}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("second durable page should succeed");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body: serde_json::Value = second
        .json()
        .await
        .expect("second durable page should be JSON");
    let second_jobs = second_body["durable_jobs"]["jobs"]
        .as_array()
        .expect("second durable page should contain jobs");
    assert_eq!(second_jobs.len(), 1);
    assert_eq!(second_jobs[0]["job_id"], oldest.0.to_string());
    assert!(second_body["durable_jobs"]["next_cursor_token"].is_null());
    assert!(second_jobs
        .iter()
        .all(|job| job["state"] == serde_json::json!("queued")));
}

#[tokio::test]
async fn test_monitor_durable_detail_enforces_workspace_isolation() {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_platform_admin(&factory, user_id).await;
    let (job_id, request_id, trace_id) =
        seed_durable_job(&factory, workspace_id, "processing", Utc::now()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/monitor/jobs/{job_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("durable detail request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.expect("detail should be JSON");
    assert_eq!(body["job_id"], job_id.to_string());
    assert_eq!(body["request_id"], request_id.to_string());
    assert_eq!(body["trace_id"], trace_id);
    assert!(body["events"]
        .as_array()
        .is_some_and(|events| !events.is_empty()));

    let other_workspace = Uuid::new_v4();
    let forbidden = client
        .get(format!(
            "{address}/api/v1/workspaces/{other_workspace}/monitor/jobs/{job_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("cross-workspace detail request should complete");
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
}
