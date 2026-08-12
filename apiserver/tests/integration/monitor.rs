//! Worker Monitor integration tests

use crate::common::app::{
    setup_app_tower, setup_app_tower_with_durable_db_url, setup_app_tower_without_redis,
    spawn_app_with_factory,
};
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::handlers::system_auth::SystemContext;
use apiserver::handlers::worker_monitor::{get_monitor_overview, MonitorSourceStatus};
use apiserver::models::enums::SystemRoleType;
use apiserver::queue::RedisPool;
use axum::extract::State;
use chrono::{Duration, Utc};
use futures_util::StreamExt;
use redis::AsyncCommands;
use reqwest::StatusCode;
use std::collections::HashSet;
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

    let queue_depth = &body["queue_health"]["depth_total"];
    assert!(queue_depth["value"].is_number());
    assert_eq!(queue_depth["unit"], "jobs");
    assert!(queue_depth["formula"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(queue_depth["time_window"]["start"].is_string());
    assert!(queue_depth["time_window"]["end"].is_string());
    assert!(queue_depth["observed_at"].is_string());
    assert_eq!(body["thresholds"]["source"], "backend_default");
    assert!(body["reliability"]["success_rate"]["unknown"]["code"].is_string());

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
    assert!(body.contains("\"queue_health\""));
    assert!(body.contains("\"observed_at\""));
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
async fn test_monitor_correlations_validates_and_sanitizes_lookup() {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_platform_admin(&factory, user_id).await;
    let (job_id, request_id, trace_id) =
        seed_durable_job(&factory, workspace_id, "failed", Utc::now()).await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "{address}/api/v1/monitor/correlations?request_id={request_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("correlation request should succeed");
    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await.expect("correlation response JSON");
    assert_eq!(body["jobs"][0]["job_id"], job_id.to_string());
    assert_eq!(body["jobs"][0]["request_id"], request_id.to_string());
    assert_eq!(body["jobs"][0]["trace_id"], trace_id);
    let body_text = serde_json::to_string(&body).expect("response should serialize");
    assert!(!body_text.contains("lease_token"));
    assert!(!body_text.contains("payload"));
    assert!(!body_text.contains("raw error"));

    let invalid = client
        .get(format!(
            "{address}/api/v1/monitor/correlations?trace_id=bad%20trace"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("invalid correlation request should complete");
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);

    let both = client
        .get(format!(
            "{address}/api/v1/monitor/correlations?request_id={request_id}&trace_id={trace_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("ambiguous correlation request should complete");
    assert_eq!(both.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_monitor_correlations_requires_platform_admin_or_support() {
    let (address, _workspace_id, user_id, _factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let response = reqwest::Client::new()
        .get(format!(
            "{address}/api/v1/monitor/correlations?request_id={}",
            Uuid::new_v4()
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("correlation authorization request should complete");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
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

fn platform_admin_context() -> SystemContext {
    SystemContext {
        user_id: Uuid::nil(),
        roles: HashSet::from([SystemRoleType::PlatformAdmin]),
    }
}

#[tokio::test]
async fn test_monitor_overview_reports_redis_unavailable_without_fabricating_zeroes() {
    let (_test_db, _factory, state, _app) = setup_app_tower_without_redis().await;

    let response = get_monitor_overview(State(state), platform_admin_context())
        .await
        .expect("monitor overview should remain available without Redis");
    let snapshot = response.0;

    assert_eq!(
        snapshot.quality.redis.status,
        MonitorSourceStatus::Unavailable
    );
    assert_eq!(
        snapshot.quality.worker_heartbeats.status,
        MonitorSourceStatus::Unavailable
    );
    assert!(snapshot.pending_tasks.is_none());
    assert!(snapshot.total_workers.is_none());
    assert!(snapshot.active_workers.is_none());
    assert!(snapshot.idle_workers.is_none());
    assert_eq!(
        snapshot.dependencies.redis.status,
        MonitorSourceStatus::Unavailable
    );
    assert!(snapshot.dependencies.redis.availability.is_none());
    assert_eq!(
        snapshot.dependencies.worker_process.status,
        MonitorSourceStatus::Unavailable
    );
    assert_eq!(
        snapshot.dependencies.cpu.status,
        MonitorSourceStatus::Unavailable
    );
    assert!(snapshot.dependencies.cpu.limitation.is_some());
}

#[tokio::test]
async fn test_monitor_overview_reports_durable_db_unavailable_independently() {
    let (_test_db, _factory, state, _app) = setup_app_tower_with_durable_db_url(
        "postgres://postgres:postgres@127.0.0.1:1/monitor_unavailable",
    )
    .await;

    let response = get_monitor_overview(State(state), platform_admin_context())
        .await
        .expect("monitor overview should degrade when durable DB is unavailable");
    let snapshot = response.0;

    assert_eq!(
        snapshot.quality.durable_db.status,
        MonitorSourceStatus::Unavailable
    );
    assert_eq!(
        snapshot.quality.durable_db.error_code.as_deref(),
        Some("DURABLE_MONITOR_UNAVAILABLE")
    );
    assert!(snapshot.durable_jobs.jobs.is_none());
    assert!(snapshot
        .quality
        .unknown_fields
        .iter()
        .any(|field| field == "durable_jobs"));
    assert_eq!(
        snapshot.dependencies.postgres.status,
        MonitorSourceStatus::Unavailable
    );
    assert!(snapshot.dependencies.postgres.latency_ms.is_none());
}

async fn seed_monitor_alert(factory: &TestFactory, status: &str) -> Uuid {
    let alert_id = Uuid::new_v4();
    let now = Utc::now();
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO monitor_alerts
         (id, fingerprint, alert_type, severity, status, title, summary, source,
          first_seen_at, last_seen_at, occurrence_count, resolved_at, created_at, updated_at)
         VALUES ($1, $2, 'monitor_test', 'warning', $3, 'Monitor test alert',
                 'Safe monitor test alert', 'integration_test', $4, $4, 1,
                 CASE WHEN $3 = 'resolved' THEN $4 ELSE NULL END, $4, $4)",
        diesel::sql_types::Uuid,
        alert_id,
        diesel::sql_types::Varchar,
        format!("fingerprint-{alert_id}"),
        diesel::sql_types::Varchar,
        status.to_string(),
        diesel::sql_types::Timestamptz,
        now
    );
    alert_id
}

#[tokio::test]
async fn test_monitor_alert_list_is_cursor_paginated_and_bounded() {
    let (address, _workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_platform_admin(&factory, user_id).await;
    for status in ["active", "resolved", "active"] {
        seed_monitor_alert(&factory, status).await;
    }
    let client = reqwest::Client::new();
    let first = client
        .get(format!("{address}/api/v1/monitor/alerts?limit=2"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("first alert page should complete");
    assert_eq!(first.status(), StatusCode::OK);
    let body: serde_json::Value = first.json().await.expect("alert page should be JSON");
    assert_eq!(body["data"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["limit"], 2);
    assert!(body["total"].as_u64().is_some_and(|total| total >= 3));
    let cursor = body["next_cursor"]
        .as_str()
        .expect("first alert page should have a cursor");

    let second = client
        .get(format!(
            "{address}/api/v1/monitor/alerts?limit=2&cursor={cursor}"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("second alert page should complete");
    assert_eq!(second.status(), StatusCode::OK);
    let second_body: serde_json::Value = second.json().await.expect("second page should be JSON");
    assert_eq!(second_body["data"].as_array().map(Vec::len), Some(1));
    assert!(second_body["next_cursor"].is_null());

    let unbounded = client
        .get(format!("{address}/api/v1/monitor/alerts?limit=10000"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("bounded alert request should complete");
    assert_eq!(unbounded.status(), StatusCode::OK);
    let unbounded_body: serde_json::Value =
        unbounded.json().await.expect("bounded page should be JSON");
    assert_eq!(unbounded_body["limit"], 100);
}

#[tokio::test]
async fn test_monitor_alert_operator_mutations_require_support_or_platform_admin() {
    let (address, _workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let alert_id = seed_monitor_alert(&factory, "active").await;
    let response = reqwest::Client::new()
        .post(format!(
            "{address}/api/v1/monitor/alerts/{alert_id}/acknowledge"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"reason": "not authorized"}))
        .send()
        .await
        .expect("unauthorized mutation request should complete");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_monitor_alert_operator_mutations_allow_support_and_are_idempotent() {
    let (address, _workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_system_role(&factory, user_id, "support").await;
    let alert_id = seed_monitor_alert(&factory, "active").await;
    let client = reqwest::Client::new();
    let acknowledge = client
        .post(format!(
            "{address}/api/v1/monitor/alerts/{alert_id}/acknowledge"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"reason": "operator reviewed"}))
        .send()
        .await
        .expect("acknowledge request should complete");
    assert_eq!(acknowledge.status(), StatusCode::OK);
    let repeat = client
        .post(format!(
            "{address}/api/v1/monitor/alerts/{alert_id}/acknowledge"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"reason": "same action"}))
        .send()
        .await
        .expect("repeat acknowledge request should complete");
    assert_eq!(repeat.status(), StatusCode::OK);
    let resolve = client
        .post(format!(
            "{address}/api/v1/monitor/alerts/{alert_id}/resolve"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"reason": "issue mitigated"}))
        .send()
        .await
        .expect("resolve request should complete");
    assert_eq!(resolve.status(), StatusCode::OK);
    let body: serde_json::Value = resolve
        .json()
        .await
        .expect("resolve response should be JSON");
    assert_eq!(body["status"], "resolved");
}

#[tokio::test]
async fn test_monitor_alert_operator_mutations_reject_unsanitized_or_unbounded_reason() {
    let (address, _workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    grant_platform_admin(&factory, user_id).await;
    let alert_id = seed_monitor_alert(&factory, "active").await;
    let response = reqwest::Client::new()
        .post(format!(
            "{address}/api/v1/monitor/alerts/{alert_id}/resolve"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"reason": "   "}))
        .send()
        .await
        .expect("invalid reason request should complete");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_monitor_overview_reports_zero_workers_when_heartbeat_source_is_empty() {
    let (_test_db, _factory, state, _app) = setup_app_tower().await;

    let response = get_monitor_overview(State(state), platform_admin_context())
        .await
        .expect("monitor overview should load with no worker heartbeats");
    let snapshot = response.0;

    assert_eq!(
        snapshot.quality.worker_heartbeats.status,
        MonitorSourceStatus::Available
    );
    assert!(snapshot.workers.is_empty());
    assert_eq!(snapshot.total_workers, Some(0));
    assert_eq!(snapshot.active_workers, Some(0));
    assert_eq!(snapshot.idle_workers, Some(0));
    assert_eq!(
        snapshot.dependencies.worker_process.status,
        MonitorSourceStatus::Degraded
    );
    assert_eq!(
        snapshot.dependencies.worker_process.availability,
        Some(true)
    );
}
