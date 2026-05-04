//! Worker Monitoring API Handlers
//!
//! Provides real-time monitoring endpoints for distributed worker system.
//! Includes both a one-shot REST endpoint and an SSE streaming endpoint.

use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use chrono::{DateTime, TimeDelta, Utc};
use futures_util::stream::Stream;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::system_auth::SystemContext;
use crate::models::enums::SystemRoleType;
use crate::AppState;

// Redis key patterns for monitoring
const WORKER_HEARTBEAT_PREFIX: &str = "monitor:worker:";
const STATS_COMPLETED_PREFIX: &str = "stats:completed:";
const STATS_FAILED_PREFIX: &str = "stats:failed:";
const QUEUE_HIGH: &str = "worker:jobs:high";
const QUEUE_LOW: &str = "worker:jobs:low";

/// Workers not seen within this duration are considered stale and excluded.
const STALE_THRESHOLD_SECS: i64 = 60;

/// SSE push interval in seconds.
const SSE_INTERVAL_SECS: u64 = 2;

/// Worker status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerStatus {
    Idle,
    Processing,
}

/// Worker heartbeat payload stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHeartbeat {
    pub id: Uuid,
    pub host: String,
    pub status: WorkerStatus,
    pub current_task_type: Option<String>,
    pub last_seen_at: DateTime<Utc>,
}

/// Combined monitor overview response — stats + worker list in one call.
#[derive(Debug, Serialize)]
pub struct MonitorOverviewResponse {
    pub pending_tasks: u64,
    pub pending_high: u64,
    pub pending_low: u64,
    pub total_workers: u64,
    pub active_workers: u64,
    pub idle_workers: u64,
    pub completed_today: u64,
    pub failed_today: u64,
    pub workers: Vec<WorkerHeartbeat>,
    pub recent_jobs: Vec<crate::queue::JobMetadata>,
}

/// Shared logic: fetch the full monitor overview from Redis.
async fn fetch_overview_data(state: &Arc<AppState>) -> AppResult<MonitorOverviewResponse> {
    let redis = state
        .infra
        .redis
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Redis not available".to_string()))?;

    let mut conn = redis.conn();

    // 1. Get pending tasks count (sum of both priority queues)
    let pending_high: u64 = conn.llen(QUEUE_HIGH).await.unwrap_or(0);
    let pending_low: u64 = conn.llen(QUEUE_LOW).await.unwrap_or(0);
    let pending_tasks = pending_high + pending_low;

    // 2. Get today's stats counters
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let completed_key = format!("{STATS_COMPLETED_PREFIX}{today}");
    let failed_key = format!("{STATS_FAILED_PREFIX}{today}");

    let completed_today: u64 = conn.get(&completed_key).await.unwrap_or(0);
    let failed_today: u64 = conn.get(&failed_key).await.unwrap_or(0);

    // 3. Scan all worker heartbeat keys
    let pattern = format!("{WORKER_HEARTBEAT_PREFIX}*");
    let worker_keys = scan_keys(&mut conn, &pattern).await;

    // 4. Parse heartbeats and filter stale workers
    let now = Utc::now();
    let stale_cutoff = now - TimeDelta::seconds(STALE_THRESHOLD_SECS);

    let (active_workers, idle_workers, workers) = if worker_keys.is_empty() {
        (0, 0, Vec::new())
    } else {
        let values: Vec<Option<String>> = conn.mget(&worker_keys).await.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Redis MGET failed for worker heartbeats");
            vec![]
        });
        filter_stale_workers(values, stale_cutoff)
    };

    let total_workers = workers.len() as u64;

    // 5. Get recent job metadata
    let queue = crate::queue::JobQueue::new(redis.clone());
    let recent_jobs = queue.list_recent_jobs(50).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list recent jobs");
        vec![]
    });

    Ok(MonitorOverviewResponse {
        pending_tasks,
        pending_high,
        pending_low,
        total_workers,
        active_workers,
        idle_workers,
        completed_today,
        failed_today,
        workers,
        recent_jobs,
    })
}

/// Get combined monitor overview: stats + worker list in a single call.
///
/// Route: GET /monitor/overview
/// Requires: PlatformAdmin or Support
pub async fn get_monitor_overview(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Json<MonitorOverviewResponse>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    Ok(Json(fetch_overview_data(&state).await?))
}

/// SSE stream: push monitor overview data every 2 seconds.
///
/// Route: GET /monitor/stream
/// Requires: PlatformAdmin or Support
pub async fn monitor_stream_sse(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;

    let interval = tokio::time::interval(Duration::from_secs(SSE_INTERVAL_SECS));
    let stream = tokio_stream::wrappers::IntervalStream::new(interval);

    let event_stream = stream.map(move |_| state.clone()).then(|state| async move {
        match fetch_overview_data(&state).await {
            Ok(data) => {
                let json = serde_json::to_string(&data).unwrap_or_default();
                Ok(Event::default().event("monitor-update").data(json))
            }
            Err(e) => {
                tracing::warn!(error = %e, "SSE: failed to fetch monitor data");
                let msg = format!("{{\"error\":\"{e}\"}}");
                Ok(Event::default().event("monitor-error").data(msg))
            }
        }
    });

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

/// Helper to parse worker heartbeats from Redis strings and filter out stale ones.
/// Returns (active_workers_count, idle_workers_count, filtered_workers_list).
fn filter_stale_workers(
    values: Vec<Option<String>>,
    stale_cutoff: DateTime<Utc>,
) -> (u64, u64, Vec<WorkerHeartbeat>) {
    let mut workers = Vec::new();
    let mut active = 0u64;
    let mut idle = 0u64;

    for value in values.into_iter().flatten() {
        if let Ok(heartbeat) = serde_json::from_str::<WorkerHeartbeat>(&value) {
            // Skip stale workers (not seen within threshold)
            if heartbeat.last_seen_at < stale_cutoff {
                continue;
            }
            match heartbeat.status {
                WorkerStatus::Processing => active += 1,
                WorkerStatus::Idle => idle += 1,
            }
            workers.push(heartbeat);
        }
    }

    (active, idle, workers)
}

/// Scan Redis keys matching a pattern using SCAN iterator.
/// Avoids KEYS command which can block Redis on large datasets.
async fn scan_keys(conn: &mut redis::aio::ConnectionManager, pattern: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut cursor = 0u64;

    loop {
        let result: Result<(u64, Vec<String>), _> = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(100)
            .query_async(conn)
            .await;

        match result {
            Ok((next_cursor, found_keys)) => {
                keys.extend(found_keys);
                cursor = next_cursor;
                if cursor == 0 {
                    break;
                }
            }
            Err(err) => {
                tracing::warn!(pattern, error = %err, "Redis SCAN failed during worker monitoring");
                break;
            }
        }
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_status_serialization() {
        let idle = WorkerStatus::Idle;
        let processing = WorkerStatus::Processing;

        let idle_json =
            serde_json::to_string(&idle).expect("WorkerStatus serialization should not fail");
        let processing_json =
            serde_json::to_string(&processing).expect("WorkerStatus serialization should not fail");

        assert_eq!(idle_json, "\"IDLE\"");
        assert_eq!(processing_json, "\"PROCESSING\"");
    }

    #[test]
    fn test_worker_heartbeat_serialization() {
        let heartbeat = WorkerHeartbeat {
            id: Uuid::nil(),
            host: "worker-pod-1".to_string(),
            status: WorkerStatus::Processing,
            current_task_type: Some("EXPORT_CHUNK".to_string()),
            last_seen_at: Utc::now(),
        };

        let json = serde_json::to_string(&heartbeat)
            .expect("WorkerHeartbeat serialization should not fail");
        assert!(json.contains("\"status\":\"PROCESSING\""));
        assert!(json.contains("\"current_task_type\":\"EXPORT_CHUNK\""));
        assert!(json.contains("\"host\":\"worker-pod-1\""));
    }

    #[test]
    fn test_monitor_overview_response_serialization() {
        let overview = MonitorOverviewResponse {
            pending_tasks: 150,
            pending_high: 100,
            pending_low: 50,
            total_workers: 50,
            active_workers: 45,
            idle_workers: 5,
            completed_today: 1200,
            failed_today: 3,
            workers: vec![],
            recent_jobs: vec![],
        };

        let json = serde_json::to_string(&overview)
            .expect("MonitorOverviewResponse serialization should not fail");
        assert!(json.contains("\"pending_tasks\":150"));
        assert!(json.contains("\"total_workers\":50"));
        assert!(json.contains("\"active_workers\":45"));
        assert!(json.contains("\"workers\":[]"));
        assert!(json.contains("\"recent_jobs\":[]"));
    }

    #[test]
    fn test_key_patterns() {
        assert_eq!(WORKER_HEARTBEAT_PREFIX, "monitor:worker:");
        assert_eq!(STATS_COMPLETED_PREFIX, "stats:completed:");
        assert_eq!(STATS_FAILED_PREFIX, "stats:failed:");

        // Test daily key format
        let today = "2026-01-30";
        let completed_key = format!("{STATS_COMPLETED_PREFIX}{today}");
        let failed_key = format!("{STATS_FAILED_PREFIX}{today}");

        assert_eq!(completed_key, "stats:completed:2026-01-30");
        assert_eq!(failed_key, "stats:failed:2026-01-30");
    }

    #[test]
    fn test_worker_heartbeat_deserialization() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "host": "worker-1",
            "status": "IDLE",
            "current_task_type": null,
            "last_seen_at": "2026-01-30T12:00:00Z"
        }"#;

        let heartbeat: WorkerHeartbeat =
            serde_json::from_str(json).expect("WorkerHeartbeat deserialization should not fail");
        assert_eq!(heartbeat.host, "worker-1");
        assert_eq!(heartbeat.status, WorkerStatus::Idle);
        assert!(heartbeat.current_task_type.is_none());
    }

    #[test]
    fn test_filter_stale_workers() {
        let now = Utc::now();
        let stale_cutoff = now - TimeDelta::seconds(STALE_THRESHOLD_SECS);

        // Worker 1: Active, very recent
        let w1 = WorkerHeartbeat {
            id: Uuid::new_v4(),
            host: "h1".to_string(),
            status: WorkerStatus::Processing,
            current_task_type: None,
            last_seen_at: now - TimeDelta::seconds(10),
        };

        // Worker 2: Idle, just before cutoff
        let w2 = WorkerHeartbeat {
            id: Uuid::new_v4(),
            host: "h2".to_string(),
            status: WorkerStatus::Idle,
            current_task_type: None,
            last_seen_at: stale_cutoff + TimeDelta::seconds(5),
        };

        // Worker 3: Processing, stale (should be dropped)
        let w3 = WorkerHeartbeat {
            id: Uuid::new_v4(),
            host: "h3".to_string(),
            status: WorkerStatus::Processing,
            current_task_type: None,
            last_seen_at: stale_cutoff - TimeDelta::seconds(5),
        };

        let values = vec![
            Some(serde_json::to_string(&w1).expect("w1 serialization should not fail")),
            Some(serde_json::to_string(&w2).expect("w2 serialization should not fail")),
            Some(serde_json::to_string(&w3).expect("w3 serialization should not fail")),
            Some("invalid json...".to_string()),
            None,
        ];

        let (active, idle, workers) = filter_stale_workers(values, stale_cutoff);

        // Should only count w1 (active) and w2 (idle). w3 is stale, others invalid
        assert_eq!(active, 1);
        assert_eq!(idle, 1);
        assert_eq!(workers.len(), 2);

        // Ensure w3 was dropped
        let hosts: Vec<String> = workers.into_iter().map(|w| w.host).collect();
        assert!(hosts.contains(&"h1".to_string()));
        assert!(hosts.contains(&"h2".to_string()));
        assert!(!hosts.contains(&"h3".to_string()));
    }
}
