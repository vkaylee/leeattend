//! Worker Monitoring API Handlers
//!
//! Provides real-time monitoring endpoints for distributed worker system.
//! Includes both a one-shot REST endpoint and an SSE streaming endpoint.

use axum::{
    extract::{Query, State},
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
use crate::utils::i18n::I18nKey;
use crate::AppState;

// Redis key patterns for monitoring
const WORKER_HEARTBEAT_PREFIX: &str = "monitor:worker:";
const STATS_COMPLETED_PREFIX: &str = "stats:completed:";
const STATS_FAILED_PREFIX: &str = "stats:failed:";
const QUEUE_HIGH: &str = "worker:jobs:high";
const QUEUE_NORMAL: &str = "worker:jobs:normal";
const QUEUE_LOW: &str = "worker:jobs:low";
const MAX_MONITOR_ERROR_SUMMARY_CHARS: usize = 160;
const MAX_MONITOR_JOBS: usize = 100;
const MAX_MONITOR_QUERY_LIMIT: usize = 100;
const MAX_MONITOR_SEARCH_CHARS: usize = 64;
const MONITOR_QUERY_TIMEOUT_SECS: u64 = 5;

const MONITOR_STATUSES: [&str; 4] = ["pending", "processing", "completed", "failed"];
const MONITOR_TASK_TYPES: [&str; 18] = [
    "attendance_report",
    "payroll_report",
    "merge_results",
    "leave_accrual",
    "log_ingestion",
    "reprocess_attendance",
    "claim_staged_logs",
    "leave_import",
    "nightly_housekeeping",
    "employee_import",
    "employee_import_chunk",
    "leave_import_chunk",
    "finalize_import",
    "expiring_contracts_check",
    "sync_employee_hire_date",
    "bulk_termination",
    "summary",
    "detail",
];

/// Workers not seen within this duration are considered stale and excluded.
const STALE_THRESHOLD_SECS: i64 = 60;

/// SSE push interval in seconds.
const SSE_INTERVAL_SECS: u64 = 2;

/// Worker status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkerStatus {
    Idle,
    Processing,
}

/// Worker heartbeat payload stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkerHeartbeat {
    id: Uuid,
    host: String,
    status: WorkerStatus,
    current_task_type: Option<String>,
    last_seen_at: DateTime<Utc>,
}

impl WorkerHeartbeat {
    pub fn new(
        id: Uuid,
        host: String,
        status: WorkerStatus,
        current_task_type: Option<String>,
        last_seen_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            host,
            status,
            current_task_type,
            last_seen_at,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn status(&self) -> &WorkerStatus {
        &self.status
    }

    pub fn current_task_type(&self) -> Option<&str> {
        self.current_task_type.as_deref()
    }

    pub fn last_seen_at(&self) -> DateTime<Utc> {
        self.last_seen_at
    }
}

/// Statistics of jobs grouped by task type.
#[derive(Debug, Serialize, utoipa::ToSchema, Clone)]
pub struct JobTypeStats {
    pub task_type: String,
    pub total: u64,
    pub completed: u64,
    pub failed: u64,
    pub processing: u64,
    pub pending: u64,
}

/// Privacy-safe job representation for infrastructure monitoring.
///
/// Queue metadata may contain payloads and requester/workspace identifiers;
/// those fields stay internal and are intentionally not exposed here.
#[derive(Debug, Serialize, utoipa::ToSchema, Clone)]
pub struct MonitorJob {
    pub job_id: Uuid,
    pub status: String,
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub report_type: Option<String>,
    pub task_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_summary: Option<String>,
}

fn enum_label<T: Serialize>(value: &T) -> Option<String> {
    serde_json::to_value(value)
        .ok()
        .and_then(|json| json.as_str().map(str::to_owned))
}

fn summarize_error(error: &str) -> String {
    let normalized = error.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut summary = normalized;
    if summary.chars().count() > MAX_MONITOR_ERROR_SUMMARY_CHARS {
        summary = summary
            .chars()
            .take(MAX_MONITOR_ERROR_SUMMARY_CHARS)
            .collect::<String>();
        summary.push('…');
    }
    summary
}

impl From<&crate::queue::JobMetadata> for MonitorJob {
    fn from(job: &crate::queue::JobMetadata) -> Self {
        Self {
            job_id: job.job_id,
            status: enum_label(&job.status).unwrap_or_else(|| "unknown".to_string()),
            total_batches: job.total_batches,
            completed_batches: job.completed_batches,
            failed_batches: job.failed_batches,
            report_type: job.report_type.as_ref().and_then(enum_label),
            task_type: job.task_type.as_ref().and_then(enum_label),
            created_at: job.created_at,
            completed_at: job.completed_at,
            error_summary: job.error_message.as_deref().map(summarize_error),
        }
    }
}

/// Combined monitor overview response — stats + worker list in one call.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorOverviewResponse {
    pub pending_tasks: u64,
    pub pending_high: u64,
    pub pending_normal: u64,
    pub pending_low: u64,
    pub total_workers: u64,
    pub active_workers: u64,
    pub idle_workers: u64,
    pub completed_today: u64,
    pub failed_today: u64,
    pub observed_at: DateTime<Utc>,
    pub workers: Vec<WorkerHeartbeat>,
    pub recent_jobs: Vec<MonitorJob>,
    pub job_type_stats: Vec<JobTypeStats>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorJobsResponse {
    pub jobs: Vec<MonitorJob>,
    pub next_cursor: Option<i64>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorStreamError {
    pub code: &'static str,
    pub message: &'static str,
    pub retryable: bool,
}

#[allow(dead_code)]
fn monitor_stream_error() -> MonitorStreamError {
    MonitorStreamError {
        code: "MONITOR_UNAVAILABLE",
        message: "Monitoring data is temporarily unavailable",
        retryable: true,
    }
}

fn compute_job_type_stats(all_recent_jobs: &[crate::queue::JobMetadata]) -> Vec<JobTypeStats> {
    let mut type_map: std::collections::HashMap<String, JobTypeStats> =
        std::collections::HashMap::new();
    for job in all_recent_jobs {
        let task_type_str = job
            .task_type
            .as_ref()
            .map(|t| match t {
                crate::queue::TaskType::AttendanceReport => "attendance_report",
                crate::queue::TaskType::PayrollReport => "payroll_report",
                crate::queue::TaskType::MergeResults => "merge_results",
                crate::queue::TaskType::LeaveAccrual => "leave_accrual",
                crate::queue::TaskType::LogIngestion => "log_ingestion",
                crate::queue::TaskType::ReprocessAttendance => "reprocess_attendance",
                crate::queue::TaskType::ClaimStagedLogs => "claim_staged_logs",
                crate::queue::TaskType::LeaveImport => "leave_import",
                crate::queue::TaskType::NightlyHousekeeping => "nightly_housekeeping",
                crate::queue::TaskType::EmployeeImport => "employee_import",
                crate::queue::TaskType::EmployeeImportChunk => "employee_import_chunk",
                crate::queue::TaskType::LeaveImportChunk => "leave_import_chunk",
                crate::queue::TaskType::FinalizeImport => "finalize_import",
                crate::queue::TaskType::ExpiringContractsCheck => "expiring_contracts_check",
                crate::queue::TaskType::SyncEmployeeHireDate => "sync_employee_hire_date",
                crate::queue::TaskType::BulkTermination => "bulk_termination",
            })
            .or_else(|| {
                job.report_type.as_ref().map(|r| match r {
                    crate::queue::ReportType::Summary => "summary",
                    crate::queue::ReportType::Detail => "detail",
                })
            })
            .unwrap_or("unknown")
            .to_string();

        let t_stats = type_map
            .entry(task_type_str.clone())
            .or_insert_with(|| JobTypeStats {
                task_type: task_type_str,
                total: 0,
                completed: 0,
                failed: 0,
                processing: 0,
                pending: 0,
            });

        t_stats.total += 1;
        match job.status {
            crate::queue::JobStatus::Pending => t_stats.pending += 1,
            crate::queue::JobStatus::Processing => t_stats.processing += 1,
            crate::queue::JobStatus::Completed => t_stats.completed += 1,
            crate::queue::JobStatus::Failed => t_stats.failed += 1,
        }
    }
    let mut job_type_stats: Vec<JobTypeStats> = type_map.into_values().collect();
    job_type_stats.sort_by(|a, b| b.total.cmp(&a.total));
    job_type_stats
}

/// Shared logic: fetch the full monitor overview from Redis.
async fn fetch_overview_data(state: &Arc<AppState>) -> AppResult<MonitorOverviewResponse> {
    let redis = state.infra().require_redis()?;

    tokio::time::timeout(
        Duration::from_secs(MONITOR_QUERY_TIMEOUT_SECS),
        fetch_overview_data_inner(state, redis),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Monitor overview timed out");
        AppError::service_unavailable(I18nKey::ServiceUnavailable)
    })?
}

async fn fetch_overview_data_inner(
    state: &Arc<AppState>,
    redis: &crate::queue::RedisPool,
) -> AppResult<MonitorOverviewResponse> {
    let mut conn = redis.conn();

    // Queue lengths are O(1); include every priority queue in the total.
    let pending_high: u64 = conn.llen(QUEUE_HIGH).await.map_err(|error| {
        tracing::error!(error = %error, "Monitor high queue read failed");
        AppError::service_unavailable(I18nKey::ServiceUnavailable)
    })?;
    let pending_normal: u64 = conn.llen(QUEUE_NORMAL).await.map_err(|error| {
        tracing::error!(error = %error, "Monitor normal queue read failed");
        AppError::service_unavailable(I18nKey::ServiceUnavailable)
    })?;
    let pending_low: u64 = conn.llen(QUEUE_LOW).await.map_err(|error| {
        tracing::error!(error = %error, "Monitor low queue read failed");
        AppError::service_unavailable(I18nKey::ServiceUnavailable)
    })?;
    let pending_tasks = pending_high + pending_normal + pending_low;

    // Get today's stats counters.
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let completed_key = format!("{STATS_COMPLETED_PREFIX}{today}");
    let failed_key = format!("{STATS_FAILED_PREFIX}{today}");

    let completed_today: u64 = conn
        .get::<_, Option<u64>>(&completed_key)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "Monitor completed stats read failed");
            AppError::service_unavailable(I18nKey::ServiceUnavailable)
        })?
        .unwrap_or(0);
    let failed_today: u64 = conn
        .get::<_, Option<u64>>(&failed_key)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "Monitor failed stats read failed");
            AppError::service_unavailable(I18nKey::ServiceUnavailable)
        })?
        .unwrap_or(0);

    // Scan only the small worker-heartbeat namespace.
    let pattern = format!("{WORKER_HEARTBEAT_PREFIX}*");
    let worker_keys = scan_keys(&mut conn, &pattern).await.map_err(|error| {
        tracing::error!(error = %error, "Monitor worker heartbeat scan failed");
        AppError::service_unavailable(I18nKey::ServiceUnavailable)
    })?;

    let now = Utc::now();
    let stale_cutoff = now - TimeDelta::seconds(STALE_THRESHOLD_SECS);

    let (active_workers, idle_workers, workers) = if worker_keys.is_empty() {
        (0, 0, Vec::new())
    } else {
        let values: Vec<Option<String>> = conn.mget(&worker_keys).await.map_err(|error| {
            AppError::Internal(format!("monitor worker heartbeat read failed: {error}"))
        })?;
        filter_stale_workers(values, stale_cutoff)
    };

    let total_workers = workers.len() as u64;
    let queue = state.infra().job_queue()?;
    let all_recent_jobs = queue
        .list_recent_jobs_indexed(MAX_MONITOR_JOBS)
        .await
        .map_err(|error| AppError::Internal(format!("monitor recent jobs read failed: {error}")))?;
    let job_type_stats = compute_job_type_stats(&all_recent_jobs);
    let mut recent_jobs = all_recent_jobs
        .iter()
        .map(MonitorJob::from)
        .collect::<Vec<_>>();
    recent_jobs.truncate(10);

    Ok(MonitorOverviewResponse {
        pending_tasks,
        pending_high,
        pending_normal,
        pending_low,
        total_workers,
        active_workers,
        idle_workers,
        completed_today,
        failed_today,
        observed_at: now,
        workers,
        recent_jobs,
        job_type_stats,
    })
}

/// Get combined monitor overview: stats + worker list in a single call.
///
/// Route: GET /monitor/overview
/// Requires: PlatformAdmin or Support
#[utoipa::path(
    get,
    path = "/monitor/overview",
    responses(
        (status = 200, description = "Monitor overview statistics and active worker status", body = MonitorOverviewResponse)
    )
)]
pub async fn get_monitor_overview(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Json<MonitorOverviewResponse>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;
    Ok(Json(fetch_overview_data(&state).await?))
}

/// Query parameters for listing monitor jobs.
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct MonitorJobsParams {
    /// Cursor timestamp for pagination (exclusive)
    pub cursor: Option<i64>,
    /// Number of items to return (default: 20)
    pub limit: Option<usize>,
    /// Status filter: pending, processing, completed, failed
    pub status: Option<String>,
    /// Task type filter
    pub task_type: Option<String>,
    /// Search query matching the exact job ID only.
    pub search: Option<String>,
}

/// Get paginated and filtered background jobs.
///
/// Route: GET /monitor/jobs
/// Requires: PlatformAdmin or Support
#[utoipa::path(
    get,
    path = "/monitor/jobs",
    params(
        MonitorJobsParams
    ),
    responses(
        (status = 200, description = "Paginated background jobs list", body = MonitorJobsResponse)
    )
)]
pub async fn get_monitor_jobs(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
    Query(params): Query<MonitorJobsParams>,
) -> AppResult<Json<MonitorJobsResponse>> {
    ctx.require_any_role(&[SystemRoleType::PlatformAdmin, SystemRoleType::Support])?;

    let limit = params.limit.unwrap_or(20).clamp(1, MAX_MONITOR_QUERY_LIMIT);
    let status = params.status.map(|value| value.trim().to_ascii_lowercase());
    if status
        .as_deref()
        .is_some_and(|value| !MONITOR_STATUSES.contains(&value))
    {
        return Err(AppError::validation(I18nKey::Validation));
    }

    let task_type = params
        .task_type
        .map(|value| value.trim().to_ascii_lowercase());
    if task_type
        .as_deref()
        .is_some_and(|value| !MONITOR_TASK_TYPES.contains(&value))
    {
        return Err(AppError::validation(I18nKey::Validation));
    }

    let search = params.search.map(|value| value.trim().to_owned());
    if search
        .as_deref()
        .is_some_and(|value| value.chars().count() > MAX_MONITOR_SEARCH_CHARS)
    {
        return Err(AppError::validation(I18nKey::Validation));
    }

    let queue = state.infra().job_queue()?;

    let (jobs, next_cursor) = queue
        .list_jobs_paginated(params.cursor, limit, status, task_type, search)
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "Monitor job listing failed");
            AppError::service_unavailable(I18nKey::ServiceUnavailable)
        })?;

    Ok(Json(MonitorJobsResponse {
        jobs: jobs.iter().map(MonitorJob::from).collect(),
        next_cursor,
    }))
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

    let event_stream = stream
        .map(move |_| Arc::clone(&state))
        .then(|state| async move {
            match fetch_overview_data(&state).await {
                Ok(data) => {
                    match serde_json::to_string(&data) {
                        Ok(json) => Ok(Event::default().event("monitor-update").data(json)),
                        Err(error) => {
                            tracing::error!(error = %error, "SSE monitor payload serialization failed");
                            let json = serde_json::to_string(&monitor_stream_error())
                                .unwrap_or_else(|_| "{\"code\":\"MONITOR_UNAVAILABLE\",\"message\":\"Monitoring data is temporarily unavailable\",\"retryable\":true}".to_string());
                            Ok(Event::default().event("monitor-error").data(json))
                        }
                    }
                }
                Err(error) => {
                    tracing::warn!(error = %error, "SSE: failed to fetch monitor data");
                    let json = serde_json::to_string(&monitor_stream_error())
                        .unwrap_or_else(|_| "{\"code\":\"MONITOR_UNAVAILABLE\",\"message\":\"Monitoring data is temporarily unavailable\",\"retryable\":true}".to_string());
                    Ok(Event::default().event("monitor-error").data(json))
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
            if heartbeat.last_seen_at() < stale_cutoff {
                continue;
            }
            match heartbeat.status() {
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
async fn scan_keys(
    conn: &mut redis::aio::ConnectionManager,
    pattern: &str,
) -> Result<Vec<String>, redis::RedisError> {
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
                return Err(err);
            }
        }
    }

    Ok(keys)
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
        let heartbeat = WorkerHeartbeat::new(
            Uuid::nil(),
            "worker-pod-1".to_string(),
            WorkerStatus::Processing,
            Some("EXPORT_CHUNK".to_string()),
            Utc::now(),
        );

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
            pending_normal: 0,
            pending_low: 50,
            total_workers: 50,
            active_workers: 45,
            idle_workers: 5,
            completed_today: 1200,
            failed_today: 3,
            observed_at: Utc::now(),
            workers: vec![],
            recent_jobs: vec![],
            job_type_stats: vec![],
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
        assert_eq!(heartbeat.host(), "worker-1");
        assert_eq!(heartbeat.status(), &WorkerStatus::Idle);
        assert!(heartbeat.current_task_type().is_none());
    }

    #[test]
    fn test_filter_stale_workers() {
        let now = Utc::now();
        let stale_cutoff = now - TimeDelta::seconds(STALE_THRESHOLD_SECS);

        // Worker 1: Active, very recent
        let w1 = WorkerHeartbeat::new(
            Uuid::new_v4(),
            "h1".to_string(),
            WorkerStatus::Processing,
            None,
            now - TimeDelta::seconds(10),
        );

        // Worker 2: Idle, just before cutoff
        let w2 = WorkerHeartbeat::new(
            Uuid::new_v4(),
            "h2".to_string(),
            WorkerStatus::Idle,
            None,
            stale_cutoff + TimeDelta::seconds(5),
        );

        // Worker 3: Processing, stale (should be dropped)
        let w3 = WorkerHeartbeat::new(
            Uuid::new_v4(),
            "h3".to_string(),
            WorkerStatus::Processing,
            None,
            stale_cutoff - TimeDelta::seconds(5),
        );

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
        let hosts: Vec<String> = workers.into_iter().map(|w| w.host().to_string()).collect();
        assert!(hosts.contains(&"h1".to_string()));
        assert!(hosts.contains(&"h2".to_string()));
        assert!(!hosts.contains(&"h3".to_string()));
    }
}
