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
use crate::repositories::infra_job_repository::ListInfraJobsParams;
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
const MONITOR_DURABLE_STATES: [&str; 8] = [
    "dispatching",
    "queued",
    "processing",
    "completed",
    "failed",
    "retryable",
    "dead_letter",
    "lost",
];
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

/// Durable lifecycle states are intentionally distinct from the legacy Redis status set.
#[derive(Debug, Clone, Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorInfraJobState {
    Dispatching,
    Queued,
    Processing,
    Completed,
    Failed,
    Retryable,
    #[serde(rename = "dead_letter")]
    DeadLetter,
    Lost,
}

impl TryFrom<&str> for MonitorInfraJobState {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "dispatching" => Ok(Self::Dispatching),
            "queued" => Ok(Self::Queued),
            "processing" => Ok(Self::Processing),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "retryable" => Ok(Self::Retryable),
            "dead_letter" => Ok(Self::DeadLetter),
            "lost" => Ok(Self::Lost),
            _ => Err(AppError::Internal(
                "Unknown durable monitor job state".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorSourceStatus {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum MonitorDataQuality {
    Complete,
    Partial,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorSourceHealth {
    pub status: MonitorSourceStatus,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorQuality {
    pub data_quality: MonitorDataQuality,
    pub freshness: String,
    pub snapshot_sequence: String,
    pub unknown_fields: Vec<String>,
    pub redis: MonitorSourceHealth,
    pub durable_db: MonitorSourceHealth,
    pub worker_heartbeats: MonitorSourceHealth,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorInfraJobEvent {
    pub event_id: Uuid,
    pub job_id: Uuid,
    pub event_type: String,
    pub from_state: Option<MonitorInfraJobState>,
    pub to_state: MonitorInfraJobState,
    pub attempt_count: i32,
    pub worker_id: Option<String>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorInfraJobRecord {
    pub job_id: Uuid,
    pub parent_job_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub task_type: String,
    pub priority: String,
    pub state: MonitorInfraJobState,
    pub attempt_count: i32,
    pub worker: Option<String>,
    pub request_id: Option<Uuid>,
    pub trace_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub queued_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub error_code: Option<String>,
    pub error_summary: Option<String>,
    pub events: Vec<MonitorInfraJobEvent>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorDurableJobsSection {
    pub status: MonitorSourceStatus,
    pub error_code: Option<String>,
    pub jobs: Option<Vec<MonitorInfraJobRecord>>,
    pub next_cursor_token: Option<String>,
    pub limit: usize,
}

fn sanitize_monitor_summary(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let redacted = normalized
        .split(' ')
        .map(|part| {
            let digits = part.chars().filter(char::is_ascii_digit).count();
            if part.contains('@') || digits >= 8 {
                "[REDACTED]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let mut summary = redacted
        .chars()
        .take(MAX_MONITOR_ERROR_SUMMARY_CHARS)
        .collect::<String>();
    if redacted.chars().count() > MAX_MONITOR_ERROR_SUMMARY_CHARS {
        summary.push('…');
    }
    summary
}

fn sanitize_monitor_code(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.chars().take(100).collect())
}

pub(crate) fn durable_event_dto(
    event: &crate::models::diesel::infra_job::InfraJobEvent,
) -> AppResult<MonitorInfraJobEvent> {
    Ok(MonitorInfraJobEvent {
        event_id: event.id,
        job_id: event.job_id,
        event_type: event.event_type.clone(),
        from_state: event
            .from_state
            .as_deref()
            .map(MonitorInfraJobState::try_from)
            .transpose()?,
        to_state: MonitorInfraJobState::try_from(event.to_state.as_str())?,
        attempt_count: event.attempt_count,
        worker_id: event.worker_id.clone(),
        request_id: event.request_id,
        trace_id: event.trace_id.clone(),
        error_code: sanitize_monitor_code(event.reason_code.as_deref()),
        error_summary: event
            .reason_summary
            .as_deref()
            .map(sanitize_monitor_summary),
        created_at: event.created_at,
    })
}

pub(crate) fn durable_job_dto(
    job: &crate::models::diesel::infra_job::InfraJobRecord,
    events: Vec<MonitorInfraJobEvent>,
) -> AppResult<MonitorInfraJobRecord> {
    Ok(MonitorInfraJobRecord {
        job_id: job.id,
        parent_job_id: job.parent_job_id,
        workspace_id: job.workspace_id,
        task_type: job.task_type.clone(),
        priority: job.priority.clone(),
        state: MonitorInfraJobState::try_from(job.state.as_str())?,
        attempt_count: job.attempt_count,
        worker: job.lease_owner.clone(),
        request_id: job.request_id,
        trace_id: job.trace_id.clone(),
        created_at: job.created_at,
        queued_at: job.queued_at,
        started_at: job.started_at,
        finished_at: job.finished_at,
        updated_at: job.updated_at,
        error_code: sanitize_monitor_code(job.error_code.as_deref()),
        error_summary: job.error_summary.as_deref().map(sanitize_monitor_summary),
        events,
    })
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
    pub durable_jobs: MonitorDurableJobsSection,
    pub quality: MonitorQuality,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorJobsResponse {
    pub jobs: Vec<MonitorJob>,
    pub next_cursor: Option<i64>,
    pub durable_jobs: MonitorDurableJobsSection,
    pub quality: MonitorQuality,
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

async fn load_durable_jobs_section(
    state: &Arc<AppState>,
    params: ListInfraJobsParams,
    limit: usize,
) -> MonitorDurableJobsSection {
    let service = state.infra().infra_job_service();
    let page = match service.list_global(params).await {
        Ok(page) => page,
        Err(error) => {
            tracing::warn!(error = %error, "Durable monitor read unavailable");
            return MonitorDurableJobsSection {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some("DURABLE_MONITOR_UNAVAILABLE".to_string()),
                jobs: None,
                next_cursor_token: None,
                limit,
            };
        }
    };

    let job_ids = page.jobs.iter().map(|job| job.id).collect::<Vec<_>>();
    let events = match service.list_events_bulk(None, &job_ids, 400).await {
        Ok(events) => events,
        Err(error) => {
            tracing::warn!(error = %error, "Durable monitor event read unavailable");
            return MonitorDurableJobsSection {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some("DURABLE_MONITOR_EVENTS_UNAVAILABLE".to_string()),
                jobs: None,
                next_cursor_token: None,
                limit,
            };
        }
    };

    let mut events_by_job = std::collections::HashMap::<Uuid, Vec<MonitorInfraJobEvent>>::new();
    for event in events {
        match durable_event_dto(&event) {
            Ok(event) => events_by_job.entry(event.job_id).or_default().push(event),
            Err(error) => {
                tracing::error!(error = %error, "Durable monitor event state is invalid");
                return MonitorDurableJobsSection {
                    status: MonitorSourceStatus::Unavailable,
                    error_code: Some("DURABLE_MONITOR_INVALID_STATE".to_string()),
                    jobs: None,
                    next_cursor_token: None,
                    limit,
                };
            }
        }
    }

    let mut jobs = Vec::with_capacity(page.jobs.len());
    for job in page.jobs {
        let events = events_by_job.remove(&job.id).unwrap_or_default();
        match durable_job_dto(&job, events) {
            Ok(job) => jobs.push(job),
            Err(error) => {
                tracing::error!(error = %error, "Durable monitor job state is invalid");
                return MonitorDurableJobsSection {
                    status: MonitorSourceStatus::Unavailable,
                    error_code: Some("DURABLE_MONITOR_INVALID_STATE".to_string()),
                    jobs: None,
                    next_cursor_token: None,
                    limit,
                };
            }
        }
    }

    let next_cursor_token = page
        .next_cursor
        .as_ref()
        .map(crate::services::InfraJobService::cursor_token);
    MonitorDurableJobsSection {
        status: MonitorSourceStatus::Available,
        error_code: None,
        jobs: Some(jobs),
        next_cursor_token,
        limit,
    }
}

fn monitor_quality(
    durable_jobs: &MonitorDurableJobsSection,
    observed_at: DateTime<Utc>,
) -> MonitorQuality {
    let durable_available = matches!(&durable_jobs.status, MonitorSourceStatus::Available);
    MonitorQuality {
        data_quality: if durable_available {
            MonitorDataQuality::Complete
        } else {
            MonitorDataQuality::Partial
        },
        freshness: "fresh".to_string(),
        snapshot_sequence: format!("{}-{}", observed_at.timestamp_micros(), Uuid::new_v4()),
        unknown_fields: if durable_available {
            Vec::new()
        } else {
            vec!["durable_jobs".to_string(), "durable_events".to_string()]
        },
        redis: MonitorSourceHealth {
            status: MonitorSourceStatus::Available,
            error_code: None,
        },
        durable_db: MonitorSourceHealth {
            status: durable_jobs.status.clone(),
            error_code: durable_jobs.error_code.clone(),
        },
        worker_heartbeats: MonitorSourceHealth {
            status: MonitorSourceStatus::Available,
            error_code: None,
        },
    }
}

/// Shared logic: fetch the full monitor overview from Redis and the durable ledger.
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

    let durable_jobs = load_durable_jobs_section(
        state,
        ListInfraJobsParams {
            limit: 10,
            ..Default::default()
        },
        10,
    )
    .await;
    let quality = monitor_quality(&durable_jobs, now);

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
        durable_jobs,
        quality,
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
    /// Cursor timestamp for Redis pagination (exclusive).
    pub cursor: Option<i64>,
    /// Number of legacy Redis items to return (default: 20).
    pub limit: Option<usize>,
    /// Legacy Redis status filter: pending, processing, completed, failed.
    pub status: Option<String>,
    /// Task type filter applied to both compatible sources.
    pub task_type: Option<String>,
    /// Bounded exact task type/error-code search for durable records.
    pub search: Option<String>,
    /// Exact durable lifecycle state.
    pub state: Option<String>,
    /// Exact durable job identifier.
    pub job_id: Option<Uuid>,
    /// Opaque durable composite cursor.
    pub durable_cursor: Option<String>,
    /// Number of durable items to return (default: 20).
    pub durable_limit: Option<usize>,
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

    let state_filter = params.state.map(|value| value.trim().to_ascii_lowercase());
    if state_filter
        .as_deref()
        .is_some_and(|value| !MONITOR_DURABLE_STATES.contains(&value))
    {
        return Err(AppError::validation(I18nKey::Validation));
    }

    let durable_cursor = params
        .durable_cursor
        .as_deref()
        .map(crate::services::InfraJobService::parse_cursor_token)
        .transpose()?;
    let durable_limit = params
        .durable_limit
        .unwrap_or(20)
        .clamp(1, MAX_MONITOR_QUERY_LIMIT);
    let durable_limit_i64 = i64::try_from(durable_limit)
        .map_err(|_conversion_error| AppError::validation(I18nKey::Validation))?;

    let queue = state.infra().job_queue()?;

    let (jobs, next_cursor) = queue
        .list_jobs_paginated(
            params.cursor,
            limit,
            status,
            task_type.clone(),
            search.clone(),
        )
        .await
        .map_err(|error| {
            tracing::error!(error = %error, "Monitor job listing failed");
            AppError::service_unavailable(I18nKey::ServiceUnavailable)
        })?;

    let durable_jobs = load_durable_jobs_section(
        &state,
        ListInfraJobsParams {
            cursor: durable_cursor,
            limit: durable_limit_i64,
            state: state_filter,
            task_type,
            job_id: params.job_id,
            search,
        },
        durable_limit,
    )
    .await;
    let quality = monitor_quality(&durable_jobs, Utc::now());

    Ok(Json(MonitorJobsResponse {
        jobs: jobs.iter().map(MonitorJob::from).collect(),
        next_cursor,
        durable_jobs,
        quality,
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
            durable_jobs: MonitorDurableJobsSection {
                status: MonitorSourceStatus::Available,
                error_code: None,
                jobs: Some(vec![]),
                next_cursor_token: None,
                limit: 10,
            },
            quality: MonitorQuality {
                data_quality: MonitorDataQuality::Complete,
                freshness: "fresh".to_string(),
                snapshot_sequence: "test-sequence".to_string(),
                unknown_fields: vec![],
                redis: MonitorSourceHealth {
                    status: MonitorSourceStatus::Available,
                    error_code: None,
                },
                durable_db: MonitorSourceHealth {
                    status: MonitorSourceStatus::Available,
                    error_code: None,
                },
                worker_heartbeats: MonitorSourceHealth {
                    status: MonitorSourceStatus::Available,
                    error_code: None,
                },
            },
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
