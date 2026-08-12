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
const MONITOR_SOURCE_TIMEOUT_SECS: u64 = 1;
const MONITOR_WINDOW_SECS: i64 = 15 * 60;
const MONITOR_AGGREGATION_LIMIT: i64 = 20_000;
const DLQ_KEY: &str = "report:dlq";

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
/// Heartbeats within three intervals are considered live.
const LIVE_THRESHOLD_SECS: u64 = 15;
/// Heartbeats older than this are no longer usable for operational decisions.
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

/// Liveness measured from the worker heartbeat timestamp at snapshot time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WorkerLiveness {
    #[default]
    Live,
    Stale,
    Offline,
}

/// Worker heartbeat payload stored in Redis and enriched for monitor responses.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct WorkerHeartbeat {
    id: Uuid,
    host: String,
    status: WorkerStatus,
    current_task_type: Option<String>,
    last_seen_at: DateTime<Utc>,
    /// Measured liveness at the time the snapshot was assembled.
    #[serde(default)]
    liveness: WorkerLiveness,
    /// Seconds elapsed since `last_seen_at` when the snapshot was assembled.
    #[serde(default)]
    heartbeat_age_seconds: u64,
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
            liveness: WorkerLiveness::Live,
            heartbeat_age_seconds: 0,
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

    pub fn liveness(&self) -> WorkerLiveness {
        self.liveness
    }

    pub fn heartbeat_age_seconds(&self) -> u64 {
        self.heartbeat_age_seconds
    }

    fn measure_liveness(&mut self, observed_at: DateTime<Utc>) {
        let age = observed_at
            .signed_duration_since(self.last_seen_at)
            .num_seconds()
            .max(0)
            .cast_unsigned();
        self.heartbeat_age_seconds = age;
        self.liveness = worker_liveness_for_age(age);
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

#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorSourceStatus {
    Available,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorDataQuality {
    Complete,
    Partial,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, Copy, Serialize, utoipa::ToSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorFreshness {
    Fresh,
    Aging,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorSourceHealth {
    pub status: MonitorSourceStatus,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorDependencyHealth {
    pub status: MonitorSourceStatus,
    pub availability: Option<bool>,
    pub latency_ms: Option<f64>,
    pub pool_utilization_percent: Option<f64>,
    pub source: String,
    pub limitation: Option<String>,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorDependencyHealthSet {
    pub redis: MonitorDependencyHealth,
    pub postgres: MonitorDependencyHealth,
    pub database_pool: MonitorDependencyHealth,
    pub outbox_sweeper: MonitorDependencyHealth,
    pub worker_process: MonitorDependencyHealth,
    pub cpu: MonitorDependencyHealth,
    pub memory: MonitorDependencyHealth,
    pub disk: MonitorDependencyHealth,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorQuality {
    pub data_quality: MonitorDataQuality,
    pub freshness: MonitorFreshness,
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
    pub events_truncated: bool,
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

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorMetricUnknown {
    pub code: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorMetricWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub duration_seconds: i64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorMetric {
    pub value: Option<f64>,
    pub unit: String,
    pub formula: String,
    pub time_window: MonitorMetricWindow,
    pub observed_at: DateTime<Utc>,
    pub source: String,
    pub unknown: Option<MonitorMetricUnknown>,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorQueueHealth {
    pub depth_high: MonitorMetric,
    pub depth_normal: MonitorMetric,
    pub depth_low: MonitorMetric,
    pub depth_total: MonitorMetric,
    pub dead_letter_depth: MonitorMetric,
    pub oldest_queued_age_seconds: MonitorMetric,
    pub enqueue_rate_per_second: MonitorMetric,
    pub processing_rate_per_second: MonitorMetric,
    pub growth_rate_per_second: MonitorMetric,
    pub estimated_drain_seconds: MonitorMetric,
    pub queue_wait_seconds_p50: MonitorMetric,
    pub queue_wait_seconds_p95: MonitorMetric,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorReliability {
    pub success_rate: MonitorMetric,
    pub failure_rate: MonitorMetric,
    pub consecutive_failures: MonitorMetric,
    pub retryable_count: MonitorMetric,
    pub dead_letter_count: MonitorMetric,
    pub lost_count: MonitorMetric,
    pub timeout_count: MonitorMetric,
    pub execution_seconds_p50: MonitorMetric,
    pub execution_seconds_p95: MonitorMetric,
    pub execution_seconds_p99: MonitorMetric,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorOldestJob {
    pub job_id: Uuid,
    pub task_type: String,
    pub queued_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub age_seconds: u64,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorTaskTypeReliability {
    pub task_type: String,
    pub queued: u64,
    pub processing: u64,
    pub completed: u64,
    pub failed: u64,
    pub retryable: u64,
    pub dead_letter: u64,
    pub lost: u64,
    pub success_rate: MonitorMetric,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorQueueThresholds {
    pub medium_at: u64,
    pub high_at: u64,
    pub critical_at: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorUtilizationThresholds {
    pub scale_down_below_percent: u64,
    pub warning_at_percent: u64,
    pub critical_at_percent: u64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, utoipa::ToSchema)]
pub struct MonitorOperationalThresholds {
    pub queue_pressure: MonitorQueueThresholds,
    pub worker_utilization: MonitorUtilizationThresholds,
    pub live_after_seconds: u64,
    pub stale_after_seconds: u64,
    pub source: String,
}

/// Canonical operational monitoring snapshot.
///
/// Numeric values are nullable when their source could not be measured. A zero
/// is therefore a measured zero, never a placeholder for an unavailable source.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct MonitorOperationalSnapshot {
    pub pending_tasks: Option<u64>,
    pub pending_high: Option<u64>,
    pub pending_normal: Option<u64>,
    pub pending_low: Option<u64>,
    pub total_workers: Option<u64>,
    pub active_workers: Option<u64>,
    pub idle_workers: Option<u64>,
    pub completed_today: Option<u64>,
    pub failed_today: Option<u64>,
    pub observed_at: DateTime<Utc>,
    pub snapshot_age_seconds: u64,
    pub workers: Vec<WorkerHeartbeat>,
    pub recent_jobs: Vec<MonitorJob>,
    pub job_type_stats: Vec<JobTypeStats>,
    pub durable_jobs: MonitorDurableJobsSection,
    pub queue_health: MonitorQueueHealth,
    pub reliability: MonitorReliability,
    pub oldest_queued_job: Option<MonitorOldestJob>,
    pub oldest_running_job: Option<MonitorOldestJob>,
    pub task_type_reliability: Vec<MonitorTaskTypeReliability>,
    pub thresholds: MonitorOperationalThresholds,
    pub dependencies: MonitorDependencyHealthSet,
    pub quality: MonitorQuality,
}

/// Compatibility name retained for callers compiled against the former schema.
pub type MonitorOverviewResponse = MonitorOperationalSnapshot;

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

fn empty_durable_jobs_section(limit: usize) -> MonitorDurableJobsSection {
    MonitorDurableJobsSection {
        status: MonitorSourceStatus::Available,
        error_code: None,
        jobs: Some(Vec::new()),
        next_cursor_token: None,
        limit,
        events_truncated: false,
    }
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
                events_truncated: false,
            };
        }
    };

    let job_ids = page.jobs.iter().map(|job| job.id).collect::<Vec<_>>();
    const DURABLE_EVENT_LIMIT: i64 = 400;
    const DURABLE_EVENT_LIMIT_USIZE: usize = 400;
    let events = match service
        .list_events_bulk(None, &job_ids, DURABLE_EVENT_LIMIT)
        .await
    {
        Ok(events) => events,
        Err(error) => {
            tracing::warn!(error = %error, "Durable monitor event read unavailable");
            return MonitorDurableJobsSection {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some("DURABLE_MONITOR_EVENTS_UNAVAILABLE".to_string()),
                jobs: None,
                next_cursor_token: None,
                limit,
                events_truncated: false,
            };
        }
    };

    // The repository applies a global limit across all jobs. Treat a full page as
    // potentially incomplete so consumers never mistake partial history for a
    // complete event timeline.
    let events_truncated = events.len() >= DURABLE_EVENT_LIMIT_USIZE;
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
                    events_truncated: false,
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
                    events_truncated: false,
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
        events_truncated,
    }
}

fn worker_liveness_for_age(age_seconds: u64) -> WorkerLiveness {
    if age_seconds <= LIVE_THRESHOLD_SECS {
        WorkerLiveness::Live
    } else if age_seconds <= STALE_THRESHOLD_SECS as u64 {
        WorkerLiveness::Stale
    } else {
        WorkerLiveness::Offline
    }
}

fn monitor_freshness_for_age(age_seconds: u64) -> MonitorFreshness {
    if age_seconds <= LIVE_THRESHOLD_SECS {
        MonitorFreshness::Fresh
    } else if age_seconds <= STALE_THRESHOLD_SECS as u64 {
        MonitorFreshness::Aging
    } else {
        MonitorFreshness::Stale
    }
}

fn snapshot_age_seconds(observed_at: DateTime<Utc>) -> u64 {
    Utc::now()
        .signed_duration_since(observed_at)
        .num_seconds()
        .max(0)
        .cast_unsigned()
}

#[derive(Debug)]
struct RedisMonitorSnapshot {
    pending_high: Option<u64>,
    pending_normal: Option<u64>,
    pending_low: Option<u64>,
    dead_letter_depth: Option<u64>,
    completed_today: Option<u64>,
    failed_today: Option<u64>,
    workers: Vec<WorkerHeartbeat>,
    recent_jobs: Vec<crate::queue::JobMetadata>,
    redis: MonitorSourceHealth,
    worker_heartbeats: MonitorSourceHealth,
}

impl RedisMonitorSnapshot {
    fn unavailable(error_code: &str) -> Self {
        Self {
            pending_high: None,
            pending_normal: None,
            pending_low: None,
            dead_letter_depth: None,
            completed_today: None,
            failed_today: None,
            workers: Vec::new(),
            recent_jobs: Vec::new(),
            redis: MonitorSourceHealth {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some(error_code.to_string()),
            },
            worker_heartbeats: MonitorSourceHealth {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some(error_code.to_string()),
            },
        }
    }
}

fn monitor_quality(
    redis: &MonitorSourceHealth,
    durable_jobs: &MonitorDurableJobsSection,
    worker_heartbeats: &MonitorSourceHealth,
    observed_at: DateTime<Utc>,
) -> MonitorQuality {
    let durable = MonitorSourceHealth {
        status: durable_jobs.status,
        error_code: durable_jobs.error_code.clone(),
    };
    let sources = [&redis.status, &durable.status, &worker_heartbeats.status];
    let available = sources
        .iter()
        .filter(|status| matches!(status, MonitorSourceStatus::Available))
        .count();
    let degraded = sources
        .iter()
        .any(|status| matches!(status, MonitorSourceStatus::Degraded))
        || durable_jobs.events_truncated;
    let data_quality = if available == sources.len() && !degraded {
        MonitorDataQuality::Complete
    } else if available == 0 && !degraded {
        MonitorDataQuality::Unavailable
    } else if degraded {
        MonitorDataQuality::Degraded
    } else {
        MonitorDataQuality::Partial
    };
    let mut unknown_fields = Vec::new();
    if !matches!(redis.status, MonitorSourceStatus::Available) {
        unknown_fields.extend(
            [
                "pending_tasks",
                "pending_high",
                "pending_normal",
                "pending_low",
                "dead_letter_depth",
                "completed_today",
                "failed_today",
                "recent_jobs",
                "queue_health.depth",
            ]
            .into_iter()
            .map(str::to_string),
        );
    }
    if !matches!(worker_heartbeats.status, MonitorSourceStatus::Available) {
        unknown_fields.push("workers".to_string());
    }
    if !matches!(durable.status, MonitorSourceStatus::Available) {
        unknown_fields.extend(
            ["durable_jobs", "durable_events"]
                .into_iter()
                .map(str::to_string),
        );
    } else if durable_jobs.events_truncated {
        unknown_fields.push("durable_events_partial".to_string());
    }
    MonitorQuality {
        data_quality,
        freshness: monitor_freshness_for_age(snapshot_age_seconds(observed_at)),
        snapshot_sequence: format!("{}-{}", observed_at.timestamp_micros(), Uuid::new_v4()),
        unknown_fields,
        redis: redis.clone(),
        durable_db: durable,
        worker_heartbeats: worker_heartbeats.clone(),
    }
}

#[derive(Debug)]
struct RedisQueueSnapshot {
    pending_high: Option<u64>,
    pending_normal: Option<u64>,
    pending_low: Option<u64>,
    dead_letter_depth: Option<u64>,
    completed_today: Option<u64>,
    failed_today: Option<u64>,
    recent_jobs: Vec<crate::queue::JobMetadata>,
    successful_reads: usize,
}

async fn load_redis_queue_snapshot(redis: &crate::queue::RedisPool) -> RedisQueueSnapshot {
    let mut conn = redis.conn();
    let mut successful_reads = 0usize;
    let pending_high = read_queue_length(&mut conn, QUEUE_HIGH, &mut successful_reads).await;
    let pending_normal = read_queue_length(&mut conn, QUEUE_NORMAL, &mut successful_reads).await;
    let pending_low = read_queue_length(&mut conn, QUEUE_LOW, &mut successful_reads).await;
    let dead_letter_depth = read_queue_length(&mut conn, DLQ_KEY, &mut successful_reads).await;
    let today = Utc::now().format("%Y-%m-%d").to_string();
    let completed_today = read_stat_counter(
        &mut conn,
        &format!("{STATS_COMPLETED_PREFIX}{today}"),
        "completed",
        &mut successful_reads,
    )
    .await;
    let failed_today = read_stat_counter(
        &mut conn,
        &format!("{STATS_FAILED_PREFIX}{today}"),
        "failed",
        &mut successful_reads,
    )
    .await;
    let recent_jobs = match crate::queue::JobQueue::new(redis.clone())
        .list_recent_jobs_indexed(MAX_MONITOR_JOBS)
        .await
    {
        Ok(jobs) => {
            successful_reads += 1;
            jobs
        }
        Err(error) => {
            tracing::warn!(error = %error, "Monitor recent jobs read unavailable");
            Vec::new()
        }
    };
    RedisQueueSnapshot {
        pending_high,
        pending_normal,
        pending_low,
        dead_letter_depth,
        completed_today,
        failed_today,
        recent_jobs,
        successful_reads,
    }
}

async fn read_queue_length(
    conn: &mut redis::aio::ConnectionManager,
    key: &str,
    successful_reads: &mut usize,
) -> Option<u64> {
    match conn.llen(key).await {
        Ok(value) => {
            *successful_reads += 1;
            Some(value)
        }
        Err(error) => {
            tracing::warn!(error = %error, queue = key, "Monitor queue read unavailable");
            None
        }
    }
}

async fn read_stat_counter(
    conn: &mut redis::aio::ConnectionManager,
    key: &str,
    stat_name: &str,
    successful_reads: &mut usize,
) -> Option<u64> {
    match conn.get::<_, Option<u64>>(key).await {
        Ok(value) => {
            *successful_reads += 1;
            Some(value.unwrap_or(0))
        }
        Err(error) => {
            tracing::warn!(error = %error, stat = stat_name, "Monitor stats read unavailable");
            None
        }
    }
}

async fn load_worker_heartbeat_snapshot(
    conn: &mut redis::aio::ConnectionManager,
    observed_at: DateTime<Utc>,
) -> (Vec<WorkerHeartbeat>, MonitorSourceHealth) {
    let pattern = format!("{WORKER_HEARTBEAT_PREFIX}*");
    let heartbeat_read = match scan_keys(conn, &pattern).await {
        Ok(worker_keys) if worker_keys.is_empty() => Ok(Vec::new()),
        Ok(worker_keys) => conn
            .mget::<_, Vec<Option<String>>>(&worker_keys)
            .await
            .map_err(|_error| {
                redis::RedisError::from((redis::ErrorKind::IoError, "heartbeat read failed"))
            }),
        Err(error) => Err(error),
    };
    match heartbeat_read {
        Ok(values) => {
            let malformed = values
                .iter()
                .flatten()
                .any(|value| serde_json::from_str::<WorkerHeartbeat>(value).is_err());
            let stale_cutoff = observed_at - TimeDelta::seconds(STALE_THRESHOLD_SECS);
            let (_, _, workers) = filter_stale_workers_at(values, stale_cutoff, observed_at);
            (
                workers,
                MonitorSourceHealth {
                    status: if malformed {
                        MonitorSourceStatus::Degraded
                    } else {
                        MonitorSourceStatus::Available
                    },
                    error_code: malformed.then(|| "WORKER_HEARTBEAT_PAYLOAD_INVALID".to_string()),
                },
            )
        }
        Err(error) => {
            tracing::warn!(error = %error, "Monitor worker heartbeat read unavailable");
            (
                Vec::new(),
                MonitorSourceHealth {
                    status: MonitorSourceStatus::Unavailable,
                    error_code: Some("WORKER_HEARTBEATS_UNAVAILABLE".to_string()),
                },
            )
        }
    }
}

async fn load_redis_monitor_snapshot(
    redis: Option<&crate::queue::RedisPool>,
    observed_at: DateTime<Utc>,
) -> RedisMonitorSnapshot {
    let Some(redis) = redis else {
        return RedisMonitorSnapshot::unavailable("REDIS_NOT_CONFIGURED");
    };

    let queue = load_redis_queue_snapshot(redis).await;
    let mut conn = redis.conn();
    let (workers, worker_heartbeats) = load_worker_heartbeat_snapshot(&mut conn, observed_at).await;
    let total_reads = 7usize;
    let redis_status = if queue.successful_reads == 0 {
        MonitorSourceStatus::Unavailable
    } else if queue.successful_reads == total_reads {
        MonitorSourceStatus::Available
    } else {
        MonitorSourceStatus::Degraded
    };
    RedisMonitorSnapshot {
        pending_high: queue.pending_high,
        pending_normal: queue.pending_normal,
        pending_low: queue.pending_low,
        dead_letter_depth: queue.dead_letter_depth,
        completed_today: queue.completed_today,
        failed_today: queue.failed_today,
        workers,
        recent_jobs: queue.recent_jobs,
        redis: MonitorSourceHealth {
            status: redis_status,
            error_code: (queue.successful_reads < total_reads)
                .then(|| "REDIS_MONITOR_PARTIAL".to_string()),
        },
        worker_heartbeats,
    }
}

async fn load_redis_monitor_snapshot_bounded(
    redis: Option<&crate::queue::RedisPool>,
    observed_at: DateTime<Utc>,
) -> RedisMonitorSnapshot {
    match tokio::time::timeout(
        Duration::from_secs(MONITOR_SOURCE_TIMEOUT_SECS),
        load_redis_monitor_snapshot(redis, observed_at),
    )
    .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            tracing::warn!(error = %error, "Redis monitor snapshot timed out");
            RedisMonitorSnapshot::unavailable("REDIS_MONITOR_TIMEOUT")
        }
    }
}

fn dependency_health(
    status: MonitorSourceStatus,
    availability: Option<bool>,
    latency_ms: Option<f64>,
    pool_utilization_percent: Option<f64>,
    source: &str,
    limitation: Option<&str>,
    observed_at: DateTime<Utc>,
) -> MonitorDependencyHealth {
    MonitorDependencyHealth {
        status,
        availability,
        latency_ms,
        pool_utilization_percent,
        source: source.to_string(),
        limitation: limitation.map(str::to_string),
        observed_at,
    }
}

fn unavailable_dependency(
    source: &str,
    limitation: &str,
    observed_at: DateTime<Utc>,
) -> MonitorDependencyHealth {
    dependency_health(
        MonitorSourceStatus::Unavailable,
        Some(false),
        None,
        None,
        source,
        Some(limitation),
        observed_at,
    )
}

#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
async fn build_dependency_health(
    state: &Arc<AppState>,
    observed_at: DateTime<Utc>,
    redis_source: &MonitorSourceHealth,
    worker_source: &MonitorSourceHealth,
    workers: &[WorkerHeartbeat],
) -> MonitorDependencyHealthSet {
    let redis = match state.infra().redis() {
        Some(redis_pool) => {
            let started = std::time::Instant::now();
            let result = tokio::time::timeout(
                Duration::from_secs(MONITOR_SOURCE_TIMEOUT_SECS),
                redis_pool.health_check(),
            )
            .await;
            match result {
                Ok(Ok(())) => dependency_health(
                    if matches!(redis_source.status, MonitorSourceStatus::Available) {
                        MonitorSourceStatus::Available
                    } else {
                        MonitorSourceStatus::Degraded
                    },
                    Some(true),
                    Some(started.elapsed().as_secs_f64() * 1_000.0),
                    None,
                    "redis.ping",
                    (redis_source.status != MonitorSourceStatus::Available).then_some(
                        "Redis responds to PING, but one or more monitor reads were partial.",
                    ),
                    observed_at,
                ),
                Ok(Err(_)) | Err(_) => unavailable_dependency(
                    "redis.ping",
                    "Redis PING did not complete within the monitor health timeout.",
                    observed_at,
                ),
            }
        }
        None => unavailable_dependency(
            "redis.ping",
            "Redis is not configured for this process.",
            observed_at,
        ),
    };

    let postgres = {
        let started = std::time::Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs(MONITOR_SOURCE_TIMEOUT_SECS),
            state.infra().infra_job_service().health_check(),
        )
        .await;
        match result {
            Ok(Ok(())) => dependency_health(
                MonitorSourceStatus::Available,
                Some(true),
                Some(started.elapsed().as_secs_f64() * 1_000.0),
                None,
                "postgres.select_1",
                None,
                observed_at,
            ),
            Ok(Err(_)) | Err(_) => unavailable_dependency(
                "postgres.select_1",
                "PostgreSQL health query failed or timed out.",
                observed_at,
            ),
        }
    };

    let database_pool = {
        let status = state.infra().diesel().api.status();
        if status.max_size == 0 {
            unavailable_dependency(
                "deadpool.api.status",
                "The PostgreSQL pool reported no configured capacity.",
                observed_at,
            )
        } else {
            let in_use = status.size.saturating_sub(status.available);
            let utilization = (in_use as f64 / status.max_size as f64) * 100.0;
            dependency_health(
                MonitorSourceStatus::Available,
                Some(true),
                None,
                Some(utilization),
                "deadpool.api.status",
                None,
                observed_at,
            )
        }
    };

    let worker_process = match worker_source.status {
        MonitorSourceStatus::Unavailable => unavailable_dependency(
            "redis.worker_heartbeats",
            "Worker heartbeat reads are unavailable.",
            observed_at,
        ),
        MonitorSourceStatus::Degraded => dependency_health(
            MonitorSourceStatus::Degraded,
            Some(false),
            None,
            None,
            "redis.worker_heartbeats",
            Some("One or more worker heartbeat records were malformed or stale."),
            observed_at,
        ),
        MonitorSourceStatus::Available if workers.is_empty() => dependency_health(
            MonitorSourceStatus::Degraded,
            Some(false),
            None,
            None,
            "redis.worker_heartbeats",
            Some("No live worker heartbeat was observed."),
            observed_at,
        ),
        MonitorSourceStatus::Available
            if workers
                .iter()
                .any(|worker| worker.liveness() != WorkerLiveness::Live) =>
        {
            dependency_health(
                MonitorSourceStatus::Degraded,
                Some(false),
                None,
                None,
                "redis.worker_heartbeats",
                Some("At least one worker heartbeat is stale or offline."),
                observed_at,
            )
        }
        MonitorSourceStatus::Available => dependency_health(
            MonitorSourceStatus::Available,
            Some(true),
            None,
            None,
            "redis.worker_heartbeats",
            None,
            observed_at,
        ),
    };

    let unsupported =
        |source: &str, limitation: &str| unavailable_dependency(source, limitation, observed_at);

    MonitorDependencyHealthSet {
        redis,
        postgres,
        database_pool,
        outbox_sweeper: unsupported(
            "outbox.sweeper",
            "No persisted sweeper heartbeat or authoritative backlog source is available.",
        ),
        worker_process,
        cpu: unsupported(
            "system.cpu",
            "No trustworthy process CPU metric source is configured.",
        ),
        memory: unsupported(
            "system.memory",
            "No trustworthy process memory metric source is configured.",
        ),
        disk: unsupported(
            "system.disk",
            "No trustworthy disk metric source is configured.",
        ),
    }
}

#[derive(Debug, Default)]
struct DurableMonitorMetrics {
    enqueue_count: u64,
    processing_count: u64,
    success_count: u64,
    failure_count: u64,
    retryable_count: u64,
    dead_letter_count: u64,
    lost_count: u64,
    timeout_count: u64,
    consecutive_failures: u64,
    queue_wait_samples: Vec<f64>,
    execution_samples: Vec<f64>,
    oldest_queued_job: Option<MonitorOldestJob>,
    oldest_running_job: Option<MonitorOldestJob>,
    task_types: std::collections::BTreeMap<String, MonitorTaskTypeAccumulator>,
}

#[derive(Debug, Default)]
struct MonitorTaskTypeAccumulator {
    queued: u64,
    processing: u64,
    completed: u64,
    failed: u64,
    retryable: u64,
    dead_letter: u64,
    lost: u64,
}

fn monitor_metric_window(observed_at: DateTime<Utc>) -> MonitorMetricWindow {
    MonitorMetricWindow {
        start: observed_at - TimeDelta::seconds(MONITOR_WINDOW_SECS),
        end: observed_at,
        duration_seconds: MONITOR_WINDOW_SECS,
    }
}

fn known_monitor_metric(
    value: f64,
    unit: &str,
    formula: &str,
    source: &str,
    observed_at: DateTime<Utc>,
) -> MonitorMetric {
    MonitorMetric {
        value: Some(value),
        unit: unit.to_string(),
        formula: formula.to_string(),
        time_window: monitor_metric_window(observed_at),
        observed_at,
        source: source.to_string(),
        unknown: None,
    }
}

fn unknown_monitor_metric(
    unit: &str,
    formula: &str,
    source: &str,
    code: &str,
    reason: &str,
    observed_at: DateTime<Utc>,
) -> MonitorMetric {
    MonitorMetric {
        value: None,
        unit: unit.to_string(),
        formula: formula.to_string(),
        time_window: monitor_metric_window(observed_at),
        observed_at,
        source: source.to_string(),
        unknown: Some(MonitorMetricUnknown {
            code: code.to_string(),
            reason: reason.to_string(),
        }),
    }
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]
fn percentile(values: &[f64], quantile: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let index = ((sorted.len() as f64 * quantile).ceil() as usize).saturating_sub(1);
    sorted.get(index.min(sorted.len() - 1)).copied()
}

#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn build_durable_monitor_metrics(
    data: &crate::repositories::infra_job_repository::InfraJobMonitorData,
    observed_at: DateTime<Utc>,
) -> DurableMonitorMetrics {
    let mut metrics = DurableMonitorMetrics::default();
    let task_by_job = data
        .records
        .iter()
        .map(|record| (record.id, record.task_type.clone()))
        .collect::<std::collections::HashMap<_, _>>();

    let mut terminal_outcomes = Vec::new();
    for event in &data.events {
        let task_type = task_by_job
            .get(&event.job_id)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let task = metrics.task_types.entry(task_type).or_default();
        match event.to_state.as_str() {
            "queued" => {
                metrics.enqueue_count += 1;
                task.queued += 1;
            }
            "processing" => {
                metrics.processing_count += 1;
                task.processing += 1;
            }
            "completed" => {
                metrics.success_count += 1;
                task.completed += 1;
                terminal_outcomes.push(true);
            }
            "failed" => {
                metrics.failure_count += 1;
                task.failed += 1;
                terminal_outcomes.push(false);
                if event.reason_code.as_deref() == Some("worker_timeout") {
                    metrics.timeout_count += 1;
                }
            }
            "retryable" => {
                metrics.retryable_count += 1;
                task.retryable += 1;
            }
            "dead_letter" => {
                metrics.dead_letter_count += 1;
                metrics.failure_count += 1;
                task.dead_letter += 1;
                terminal_outcomes.push(false);
            }
            "lost" => {
                metrics.lost_count += 1;
                metrics.failure_count += 1;
                task.lost += 1;
                terminal_outcomes.push(false);
            }
            _ => {}
        }
    }

    metrics.consecutive_failures = terminal_outcomes
        .into_iter()
        .rev()
        .take_while(|success| !success)
        .count() as u64;

    for record in &data.records {
        if let (Some(queued_at), Some(started_at)) = (record.queued_at, record.started_at) {
            let wait = started_at
                .signed_duration_since(queued_at)
                .num_milliseconds();
            if wait >= 0
                && started_at >= monitor_metric_window(observed_at).start
                && started_at <= observed_at
            {
                metrics.queue_wait_samples.push(wait as f64 / 1_000.0);
            }
        }
        if let (Some(started_at), Some(finished_at)) = (record.started_at, record.finished_at) {
            let duration = finished_at
                .signed_duration_since(started_at)
                .num_milliseconds();
            if duration >= 0 && finished_at >= monitor_metric_window(observed_at).start {
                metrics.execution_samples.push(duration as f64 / 1_000.0);
            }
        }
        let age_seconds = |timestamp: DateTime<Utc>| {
            observed_at
                .signed_duration_since(timestamp)
                .num_seconds()
                .max(0)
                .cast_unsigned()
        };
        match (record.state.as_str(), record.queued_at, record.started_at) {
            ("queued", Some(queued_at), _) => {
                let replace = metrics
                    .oldest_queued_job
                    .as_ref()
                    .is_none_or(|oldest| queued_at < oldest.queued_at.unwrap_or(queued_at));
                if replace {
                    metrics.oldest_queued_job = Some(MonitorOldestJob {
                        job_id: record.id,
                        task_type: record.task_type.clone(),
                        queued_at: Some(queued_at),
                        started_at: None,
                        age_seconds: age_seconds(queued_at),
                    });
                }
            }
            ("processing", _, Some(started_at)) => {
                let replace = metrics
                    .oldest_running_job
                    .as_ref()
                    .is_none_or(|oldest| started_at < oldest.started_at.unwrap_or(started_at));
                if replace {
                    metrics.oldest_running_job = Some(MonitorOldestJob {
                        job_id: record.id,
                        task_type: record.task_type.clone(),
                        queued_at: record.queued_at,
                        started_at: Some(started_at),
                        age_seconds: age_seconds(started_at),
                    });
                }
            }
            _ => {}
        }
    }
    metrics
}

fn monitor_optional_metric(
    value: Option<f64>,
    unit: &str,
    formula: &str,
    source: &str,
    unknown_code: &str,
    unknown_reason: &str,
    observed_at: DateTime<Utc>,
) -> MonitorMetric {
    value.map_or_else(
        || {
            unknown_monitor_metric(
                unit,
                formula,
                source,
                unknown_code,
                unknown_reason,
                observed_at,
            )
        },
        |value| known_monitor_metric(value, unit, formula, source, observed_at),
    )
}

#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn build_queue_health(
    redis: &RedisMonitorSnapshot,
    durable: Option<(&DurableMonitorMetrics, bool)>,
    observed_at: DateTime<Utc>,
) -> MonitorQueueHealth {
    let redis_source = "redis:worker_queues";
    let durable_source = "postgres:infra_job_events";
    let unavailable_reason = "The source did not provide a trustworthy value for this metric.";
    let depth_total = match (redis.pending_high, redis.pending_normal, redis.pending_low) {
        (Some(high), Some(normal), Some(low)) => Some((high + normal + low) as f64),
        _ => None,
    };
    let (enqueue_rate, processing_rate, growth_rate, drain_time) =
        if let Some((metrics, complete)) = durable {
            let duration = MONITOR_WINDOW_SECS as f64;
            let enqueue = metrics.enqueue_count as f64 / duration;
            let processing = metrics.processing_count as f64 / duration;
            let growth = enqueue - processing;
            let drain = match (depth_total, processing - enqueue) {
                (Some(depth), rate) if rate > 0.0 => Some(depth / rate),
                _ => None,
            };
            let sample_unknown = if complete {
                None
            } else {
                Some("AGGREGATION_BOUNDED")
            };
            (
                sample_unknown.map_or(Some(enqueue), |_| None),
                sample_unknown.map_or(Some(processing), |_| None),
                sample_unknown.map_or(Some(growth), |_| None),
                sample_unknown.map_or(drain, |_| None),
            )
        } else {
            (None, None, None, None)
        };
    let wait_p50_value = durable
        .and_then(|(metrics, complete)| {
            complete.then(|| percentile(&metrics.queue_wait_samples, 0.5))
        })
        .flatten();
    let wait_p95_value = durable
        .and_then(|(metrics, complete)| {
            complete.then(|| percentile(&metrics.queue_wait_samples, 0.95))
        })
        .flatten();

    MonitorQueueHealth {
        depth_high: monitor_optional_metric(
            redis.pending_high.map(|value| value as f64),
            "jobs",
            "LLEN(worker:jobs:high)",
            redis_source,
            "REDIS_QUEUE_UNAVAILABLE",
            unavailable_reason,
            observed_at,
        ),
        depth_normal: monitor_optional_metric(
            redis.pending_normal.map(|value| value as f64),
            "jobs",
            "LLEN(worker:jobs:normal)",
            redis_source,
            "REDIS_QUEUE_UNAVAILABLE",
            unavailable_reason,
            observed_at,
        ),
        depth_low: monitor_optional_metric(
            redis.pending_low.map(|value| value as f64),
            "jobs",
            "LLEN(worker:jobs:low)",
            redis_source,
            "REDIS_QUEUE_UNAVAILABLE",
            unavailable_reason,
            observed_at,
        ),
        depth_total: monitor_optional_metric(
            depth_total,
            "jobs",
            "high + normal + low queue depth",
            redis_source,
            "REDIS_QUEUE_PARTIAL",
            unavailable_reason,
            observed_at,
        ),
        dead_letter_depth: monitor_optional_metric(
            redis.dead_letter_depth.map(|value| value as f64),
            "jobs",
            "LLEN(report:dlq)",
            redis_source,
            "REDIS_DLQ_UNAVAILABLE",
            unavailable_reason,
            observed_at,
        ),
        oldest_queued_age_seconds: monitor_optional_metric(
            durable.and_then(|(metrics, complete)| {
                complete.then(|| {
                    metrics
                        .oldest_queued_job
                        .as_ref()
                        .map(|job| job.age_seconds as f64)
                        .unwrap_or(0.0)
                })
            }),
            "seconds",
            "observed_at - oldest queued_at",
            durable_source,
            "DURABLE_MONITOR_UNAVAILABLE",
            unavailable_reason,
            observed_at,
        ),
        enqueue_rate_per_second: monitor_optional_metric(
            enqueue_rate,
            "jobs/second",
            "count(queued events in window) / window_seconds",
            durable_source,
            "DURABLE_METRICS_UNKNOWN",
            unavailable_reason,
            observed_at,
        ),
        processing_rate_per_second: monitor_optional_metric(
            processing_rate,
            "jobs/second",
            "count(processing events in window) / window_seconds",
            durable_source,
            "DURABLE_METRICS_UNKNOWN",
            unavailable_reason,
            observed_at,
        ),
        growth_rate_per_second: monitor_optional_metric(
            growth_rate,
            "jobs/second",
            "enqueue_rate - processing_rate",
            durable_source,
            "DURABLE_METRICS_UNKNOWN",
            unavailable_reason,
            observed_at,
        ),
        estimated_drain_seconds: monitor_optional_metric(
            drain_time,
            "seconds",
            "queue_depth / max(processing_rate - enqueue_rate, 0)",
            durable_source,
            "QUEUE_NOT_DRAINING",
            "Net processing rate is not positive or the source is incomplete.",
            observed_at,
        ),
        queue_wait_seconds_p50: monitor_optional_metric(
            wait_p50_value,
            "seconds",
            "p50(started_at - queued_at)",
            durable_source,
            "QUEUE_WAIT_NO_SAMPLES",
            "No complete queue wait samples were available.",
            observed_at,
        ),
        queue_wait_seconds_p95: monitor_optional_metric(
            wait_p95_value,
            "seconds",
            "p95(started_at - queued_at)",
            durable_source,
            "QUEUE_WAIT_NO_SAMPLES",
            "No complete queue wait samples were available.",
            observed_at,
        ),
    }
}

#[allow(clippy::cast_precision_loss)]
fn build_task_type_reliability(
    durable: Option<(&DurableMonitorMetrics, bool)>,
    observed_at: DateTime<Utc>,
) -> Vec<MonitorTaskTypeReliability> {
    let Some((metrics, complete)) = durable else {
        return Vec::new();
    };
    let source = "postgres:infra_job_events";
    metrics
        .task_types
        .iter()
        .map(|(task_type, counts)| {
            let terminal = counts.completed + counts.failed + counts.dead_letter + counts.lost;
            let rate = complete
                .then_some(terminal)
                .filter(|total| *total > 0)
                .map(|total| counts.completed as f64 / total as f64);
            MonitorTaskTypeReliability {
                task_type: task_type.clone(),
                queued: counts.queued,
                processing: counts.processing,
                completed: counts.completed,
                failed: counts.failed,
                retryable: counts.retryable,
                dead_letter: counts.dead_letter,
                lost: counts.lost,
                success_rate: monitor_optional_metric(
                    rate,
                    "ratio",
                    "completed / (completed + failed + dead_letter + lost)",
                    source,
                    "TASK_TYPE_NO_SAMPLES",
                    "No complete terminal outcomes were available for this task type.",
                    observed_at,
                ),
            }
        })
        .collect()
}

#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn build_reliability(
    durable: Option<(&DurableMonitorMetrics, bool)>,
    observed_at: DateTime<Utc>,
) -> MonitorReliability {
    let source = "postgres:infra_job_events";
    let complete = durable.is_some_and(|(_, complete)| complete);
    let metrics = durable.map(|(metrics, _)| metrics);
    let terminal_total = metrics
        .map(|m| m.success_count + m.failure_count)
        .unwrap_or(0);
    let success_rate = metrics.and_then(|m| {
        (terminal_total > 0).then_some(m.success_count as f64 / terminal_total as f64)
    });
    let failure_rate = metrics.and_then(|m| {
        (terminal_total > 0).then_some(m.failure_count as f64 / terminal_total as f64)
    });
    let metric_or_unknown = |value: Option<f64>, formula: &str, code: &str, reason: &str| {
        monitor_optional_metric(
            complete.then_some(value).flatten(),
            "ratio",
            formula,
            source,
            code,
            reason,
            observed_at,
        )
    };
    MonitorReliability {
        success_rate: metric_or_unknown(
            success_rate,
            "completed / (completed + failed + dead_letter + lost)",
            "RELIABILITY_NO_SAMPLES",
            "No terminal job outcomes were available.",
        ),
        failure_rate: metric_or_unknown(
            failure_rate,
            "(failed + dead_letter + lost) / (completed + failed + dead_letter + lost)",
            "RELIABILITY_NO_SAMPLES",
            "No terminal job outcomes were available.",
        ),
        consecutive_failures: monitor_optional_metric(
            complete
                .then_some(metrics.map(|m| m.consecutive_failures as f64))
                .flatten(),
            "jobs",
            "count of trailing failed, dead_letter, or lost terminal outcomes",
            source,
            "RELIABILITY_UNKNOWN",
            unavailable_reason(),
            observed_at,
        ),
        retryable_count: monitor_optional_metric(
            complete
                .then_some(metrics.map(|m| m.retryable_count as f64))
                .flatten(),
            "jobs",
            "count(to_state = retryable)",
            source,
            "RELIABILITY_UNKNOWN",
            unavailable_reason(),
            observed_at,
        ),
        dead_letter_count: monitor_optional_metric(
            complete
                .then_some(metrics.map(|m| m.dead_letter_count as f64))
                .flatten(),
            "jobs",
            "count(to_state = dead_letter)",
            source,
            "RELIABILITY_UNKNOWN",
            unavailable_reason(),
            observed_at,
        ),
        lost_count: monitor_optional_metric(
            complete
                .then_some(metrics.map(|m| m.lost_count as f64))
                .flatten(),
            "jobs",
            "count(to_state = lost)",
            source,
            "RELIABILITY_UNKNOWN",
            unavailable_reason(),
            observed_at,
        ),
        timeout_count: monitor_optional_metric(
            complete
                .then_some(metrics.map(|m| m.timeout_count as f64))
                .flatten(),
            "jobs",
            "count(reason_code = worker_timeout)",
            source,
            "RELIABILITY_UNKNOWN",
            unavailable_reason(),
            observed_at,
        ),
        execution_seconds_p50: monitor_optional_metric(
            complete
                .then(|| metrics.and_then(|m| percentile(&m.execution_samples, 0.5)))
                .flatten(),
            "seconds",
            "p50(finished_at - started_at)",
            source,
            "EXECUTION_NO_SAMPLES",
            "No complete execution duration samples were available.",
            observed_at,
        ),
        execution_seconds_p95: monitor_optional_metric(
            complete
                .then(|| metrics.and_then(|m| percentile(&m.execution_samples, 0.95)))
                .flatten(),
            "seconds",
            "p95(finished_at - started_at)",
            source,
            "EXECUTION_NO_SAMPLES",
            "No complete execution duration samples were available.",
            observed_at,
        ),
        execution_seconds_p99: monitor_optional_metric(
            complete
                .then(|| metrics.and_then(|m| percentile(&m.execution_samples, 0.99)))
                .flatten(),
            "seconds",
            "p99(finished_at - started_at)",
            source,
            "EXECUTION_NO_SAMPLES",
            "No complete execution duration samples were available.",
            observed_at,
        ),
    }
}

fn unavailable_reason() -> &'static str {
    "The durable telemetry source is unavailable or bounded before the requested metric could be trusted."
}

fn build_monitor_thresholds() -> MonitorOperationalThresholds {
    MonitorOperationalThresholds {
        queue_pressure: MonitorQueueThresholds {
            medium_at: 1,
            high_at: 6,
            critical_at: 16,
            source: "backend_default".to_string(),
        },
        worker_utilization: MonitorUtilizationThresholds {
            scale_down_below_percent: 30,
            warning_at_percent: 50,
            critical_at_percent: 80,
            source: "backend_default".to_string(),
        },
        live_after_seconds: LIVE_THRESHOLD_SECS,
        stale_after_seconds: STALE_THRESHOLD_SECS as u64,
        source: "backend_default".to_string(),
    }
}

/// Shared logic: fetch the full monitor overview without making Redis a hard dependency.
async fn fetch_overview_data(state: &Arc<AppState>) -> AppResult<MonitorOperationalSnapshot> {
    tokio::time::timeout(
        Duration::from_secs(MONITOR_QUERY_TIMEOUT_SECS),
        fetch_overview_data_inner(state),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "Monitor overview timed out");
        AppError::service_unavailable(I18nKey::ServiceUnavailable)
    })?
}

#[allow(clippy::too_many_lines)]
async fn fetch_overview_data_inner(state: &Arc<AppState>) -> AppResult<MonitorOperationalSnapshot> {
    let observed_at = Utc::now();
    let redis_snapshot =
        load_redis_monitor_snapshot_bounded(state.infra().redis(), observed_at).await;
    let durable_jobs = load_durable_jobs_section(
        state,
        ListInfraJobsParams {
            limit: 10,
            ..Default::default()
        },
        10,
    )
    .await;
    let mut quality = monitor_quality(
        &redis_snapshot.redis,
        &durable_jobs,
        &redis_snapshot.worker_heartbeats,
        observed_at,
    );
    let durable_data = if matches!(durable_jobs.status, MonitorSourceStatus::Available) {
        state
            .infra()
            .infra_job_service()
            .load_monitor_data(
                observed_at - TimeDelta::seconds(MONITOR_WINDOW_SECS),
                MONITOR_AGGREGATION_LIMIT,
            )
            .await
            .ok()
    } else {
        None
    };
    let durable_metrics = durable_data.as_ref().map(|data| {
        (
            build_durable_monitor_metrics(data, observed_at),
            !data.truncated,
        )
    });
    if durable_metrics.is_none() {
        if matches!(durable_jobs.status, MonitorSourceStatus::Available) {
            quality.durable_db = MonitorSourceHealth {
                status: MonitorSourceStatus::Degraded,
                error_code: Some("DURABLE_METRICS_UNAVAILABLE".to_string()),
            };
            quality.data_quality = MonitorDataQuality::Degraded;
        }
        quality.unknown_fields.extend(
            [
                "queue_health.historical_rates",
                "queue_health.queue_wait",
                "reliability",
                "oldest_queued_job",
                "oldest_running_job",
                "task_type_reliability",
            ]
            .into_iter()
            .map(str::to_string),
        );
    } else if durable_metrics
        .as_ref()
        .is_some_and(|(_, complete)| !complete)
    {
        quality.data_quality = MonitorDataQuality::Degraded;
        quality
            .unknown_fields
            .push("durable_metrics_bounded".to_string());
    }
    let queue_health = build_queue_health(
        &redis_snapshot,
        durable_metrics
            .as_ref()
            .map(|(metrics, complete)| (metrics, *complete)),
        observed_at,
    );
    let reliability = build_reliability(
        durable_metrics
            .as_ref()
            .map(|(metrics, complete)| (metrics, *complete)),
        observed_at,
    );
    let task_type_reliability = build_task_type_reliability(
        durable_metrics
            .as_ref()
            .map(|(metrics, complete)| (metrics, *complete)),
        observed_at,
    );
    let oldest_queued_job = durable_metrics.as_ref().and_then(|(metrics, complete)| {
        complete
            .then(|| metrics.oldest_queued_job.clone())
            .flatten()
    });
    let oldest_running_job = durable_metrics.as_ref().and_then(|(metrics, complete)| {
        complete
            .then(|| metrics.oldest_running_job.clone())
            .flatten()
    });
    let pending_tasks = match (
        redis_snapshot.pending_high,
        redis_snapshot.pending_normal,
        redis_snapshot.pending_low,
    ) {
        (Some(high), Some(normal), Some(low)) => Some(high + normal + low),
        _ => None,
    };
    let job_type_stats = compute_job_type_stats(&redis_snapshot.recent_jobs);
    let mut recent_jobs = redis_snapshot
        .recent_jobs
        .iter()
        .map(MonitorJob::from)
        .collect::<Vec<_>>();
    recent_jobs.truncate(10);
    let heartbeat_available = matches!(
        redis_snapshot.worker_heartbeats.status,
        MonitorSourceStatus::Available
    );
    let workers = redis_snapshot.workers;
    let total_workers = heartbeat_available.then_some(workers.len() as u64);
    let active_workers = heartbeat_available.then_some(
        workers
            .iter()
            .filter(|worker| worker.status() == &WorkerStatus::Processing)
            .count() as u64,
    );
    let idle_workers = heartbeat_available.then_some(
        workers
            .iter()
            .filter(|worker| worker.status() == &WorkerStatus::Idle)
            .count() as u64,
    );
    let snapshot_age_seconds = snapshot_age_seconds(observed_at);
    let dependencies = build_dependency_health(
        state,
        observed_at,
        &redis_snapshot.redis,
        &redis_snapshot.worker_heartbeats,
        &workers,
    )
    .await;
    quality.unknown_fields.extend(
        [
            "dependencies.outbox_sweeper",
            "dependencies.cpu",
            "dependencies.memory",
            "dependencies.disk",
        ]
        .into_iter()
        .map(str::to_string),
    );
    Ok(MonitorOperationalSnapshot {
        pending_tasks,
        pending_high: redis_snapshot.pending_high,
        pending_normal: redis_snapshot.pending_normal,
        pending_low: redis_snapshot.pending_low,
        total_workers,
        active_workers,
        idle_workers,
        completed_today: redis_snapshot.completed_today,
        failed_today: redis_snapshot.failed_today,
        observed_at,
        snapshot_age_seconds,
        workers,
        recent_jobs,
        job_type_stats,
        durable_jobs,
        queue_health,
        reliability,
        oldest_queued_job,
        oldest_running_job,
        task_type_reliability,
        thresholds: build_monitor_thresholds(),
        dependencies,
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
        (status = 200, description = "Operational monitor snapshot with independent source health", body = MonitorOperationalSnapshot)
    )
)]
pub async fn get_monitor_overview(
    State(state): State<Arc<AppState>>,
    ctx: SystemContext,
) -> AppResult<Json<MonitorOperationalSnapshot>> {
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
    /// Include legacy Redis jobs in this page (default: true).
    pub include_legacy: Option<bool>,
    /// Include durable ledger jobs in this page (default: true).
    pub include_durable: Option<bool>,
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
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    let search = params
        .search
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
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

    let observed_at = Utc::now();
    let redis_snapshot =
        load_redis_monitor_snapshot_bounded(state.infra().redis(), observed_at).await;
    let include_legacy = params.include_legacy.unwrap_or(true);
    let include_durable = params.include_durable.unwrap_or(true);
    let (jobs, next_cursor) = if !include_legacy {
        (Vec::new(), None)
    } else if let Some(redis) = state.infra().redis() {
        match crate::queue::JobQueue::new(redis.clone())
            .list_jobs_paginated(
                params.cursor,
                limit,
                status,
                task_type.clone(),
                search.clone(),
            )
            .await
        {
            Ok(page) => page,
            Err(error) => {
                tracing::warn!(error = %error, "Monitor job listing unavailable");
                (Vec::new(), None)
            }
        }
    } else {
        (Vec::new(), None)
    };

    let durable_jobs = if include_durable {
        load_durable_jobs_section(
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
        .await
    } else {
        empty_durable_jobs_section(durable_limit)
    };
    let quality = monitor_quality(
        &redis_snapshot.redis,
        &durable_jobs,
        &redis_snapshot.worker_heartbeats,
        observed_at,
    );

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

/// Helper retained for focused unit tests and compatibility with older callers.
#[cfg(test)]
fn filter_stale_workers(
    values: Vec<Option<String>>,
    stale_cutoff: DateTime<Utc>,
) -> (u64, u64, Vec<WorkerHeartbeat>) {
    filter_stale_workers_at(values, stale_cutoff, Utc::now())
}

/// Parse worker heartbeats and attach measured age/liveness metadata.
///
/// Stale records are retained so operators can distinguish a slow worker from
/// an absent one. Redis key expiry is what removes an offline worker entirely;
/// an old record that remains readable is explicitly marked `offline`.
fn filter_stale_workers_at(
    values: Vec<Option<String>>,
    _stale_cutoff: DateTime<Utc>,
    observed_at: DateTime<Utc>,
) -> (u64, u64, Vec<WorkerHeartbeat>) {
    let mut workers = Vec::new();
    let mut active = 0u64;
    let mut idle = 0u64;

    for value in values.into_iter().flatten() {
        if let Ok(mut heartbeat) = serde_json::from_str::<WorkerHeartbeat>(&value) {
            heartbeat.measure_liveness(observed_at);
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
        let available_dependency = |source: &str| MonitorDependencyHealth {
            status: MonitorSourceStatus::Available,
            availability: Some(true),
            latency_ms: Some(1.0),
            pool_utilization_percent: None,
            source: source.to_string(),
            limitation: None,
            observed_at: Utc::now(),
        };
        let overview = MonitorOverviewResponse {
            pending_tasks: Some(150),
            pending_high: Some(100),
            pending_normal: Some(0),
            pending_low: Some(50),
            total_workers: Some(50),
            active_workers: Some(45),
            idle_workers: Some(5),
            completed_today: Some(1200),
            failed_today: Some(3),
            observed_at: Utc::now(),
            snapshot_age_seconds: 0,
            workers: vec![],
            recent_jobs: vec![],
            job_type_stats: vec![],
            durable_jobs: MonitorDurableJobsSection {
                status: MonitorSourceStatus::Available,
                error_code: None,
                jobs: Some(vec![]),
                next_cursor_token: None,
                limit: 10,
                events_truncated: false,
            },
            dependencies: MonitorDependencyHealthSet {
                redis: available_dependency("redis.ping"),
                postgres: available_dependency("postgres.select_1"),
                database_pool: MonitorDependencyHealth {
                    pool_utilization_percent: Some(10.0),
                    ..available_dependency("deadpool.api.status")
                },
                outbox_sweeper: available_dependency("test.fixture"),
                worker_process: available_dependency("worker.heartbeat"),
                cpu: available_dependency("test.fixture"),
                memory: available_dependency("test.fixture"),
                disk: available_dependency("test.fixture"),
            },
            queue_health: build_queue_health(
                &RedisMonitorSnapshot {
                    pending_high: Some(100),
                    pending_normal: Some(0),
                    pending_low: Some(50),
                    dead_letter_depth: Some(0),
                    completed_today: Some(1200),
                    failed_today: Some(3),
                    workers: vec![],
                    recent_jobs: vec![],
                    redis: MonitorSourceHealth {
                        status: MonitorSourceStatus::Available,
                        error_code: None,
                    },
                    worker_heartbeats: MonitorSourceHealth {
                        status: MonitorSourceStatus::Available,
                        error_code: None,
                    },
                },
                None,
                Utc::now(),
            ),
            reliability: build_reliability(None, Utc::now()),
            oldest_queued_job: None,
            oldest_running_job: None,
            task_type_reliability: vec![],
            thresholds: build_monitor_thresholds(),
            quality: MonitorQuality {
                data_quality: MonitorDataQuality::Complete,
                freshness: MonitorFreshness::Fresh,
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

        // Stale workers remain visible with measured liveness; malformed values are omitted.
        assert_eq!(active, 2);
        assert_eq!(idle, 1);
        assert_eq!(workers.len(), 3);

        let stale_worker = workers
            .iter()
            .find(|worker| worker.host() == "h2")
            .expect("stale worker should remain visible");
        assert_eq!(stale_worker.liveness(), WorkerLiveness::Stale);
        assert!(stale_worker.heartbeat_age_seconds() > LIVE_THRESHOLD_SECS);
        let offline_worker = workers
            .iter()
            .find(|worker| worker.host() == "h3")
            .expect("offline worker record should remain visible");
        assert_eq!(offline_worker.liveness(), WorkerLiveness::Offline);
    }

    #[test]
    fn test_worker_liveness_boundaries_match_contract() {
        assert_eq!(worker_liveness_for_age(0), WorkerLiveness::Live);
        assert_eq!(
            worker_liveness_for_age(LIVE_THRESHOLD_SECS),
            WorkerLiveness::Live
        );
        assert_eq!(
            worker_liveness_for_age(LIVE_THRESHOLD_SECS + 1),
            WorkerLiveness::Stale
        );
        assert_eq!(
            worker_liveness_for_age(STALE_THRESHOLD_SECS as u64),
            WorkerLiveness::Stale
        );
        assert_eq!(
            worker_liveness_for_age(STALE_THRESHOLD_SECS as u64 + 1),
            WorkerLiveness::Offline
        );
    }

    #[test]
    fn test_monitor_freshness_boundaries_match_contract() {
        assert_eq!(monitor_freshness_for_age(0), MonitorFreshness::Fresh);
        assert_eq!(
            monitor_freshness_for_age(LIVE_THRESHOLD_SECS),
            MonitorFreshness::Fresh
        );
        assert_eq!(
            monitor_freshness_for_age(LIVE_THRESHOLD_SECS + 1),
            MonitorFreshness::Aging
        );
        assert_eq!(
            monitor_freshness_for_age(STALE_THRESHOLD_SECS as u64),
            MonitorFreshness::Aging
        );
        assert_eq!(
            monitor_freshness_for_age(STALE_THRESHOLD_SECS as u64 + 1),
            MonitorFreshness::Stale
        );
    }

    #[test]
    fn test_monitor_quality_is_complete_when_all_sources_are_available() {
        let available = MonitorSourceHealth {
            status: MonitorSourceStatus::Available,
            error_code: None,
        };
        let durable_jobs = MonitorDurableJobsSection {
            status: MonitorSourceStatus::Available,
            error_code: None,
            jobs: Some(Vec::new()),
            next_cursor_token: None,
            limit: 10,
            events_truncated: false,
        };

        let quality = monitor_quality(&available, &durable_jobs, &available, Utc::now());

        assert_eq!(quality.data_quality, MonitorDataQuality::Complete);
        assert_eq!(quality.freshness, MonitorFreshness::Fresh);
        assert!(quality.unknown_fields.is_empty());
    }

    #[test]
    fn test_monitor_quality_marks_truncated_durable_events_as_degraded() {
        let available = MonitorSourceHealth {
            status: MonitorSourceStatus::Available,
            error_code: None,
        };
        let durable_jobs = MonitorDurableJobsSection {
            status: MonitorSourceStatus::Available,
            error_code: None,
            jobs: Some(Vec::new()),
            next_cursor_token: None,
            limit: 10,
            events_truncated: true,
        };

        let quality = monitor_quality(&available, &durable_jobs, &available, Utc::now());

        assert_eq!(quality.data_quality, MonitorDataQuality::Degraded);
        assert!(quality
            .unknown_fields
            .iter()
            .any(|field| field == "durable_events_partial"));
    }

    #[test]
    fn test_monitor_quality_marks_unavailable_sources_and_unknown_fields() {
        let unavailable = MonitorSourceHealth {
            status: MonitorSourceStatus::Unavailable,
            error_code: Some("SOURCE_DOWN".to_string()),
        };
        let available = MonitorSourceHealth {
            status: MonitorSourceStatus::Available,
            error_code: None,
        };
        let durable_jobs = MonitorDurableJobsSection {
            status: MonitorSourceStatus::Available,
            error_code: None,
            jobs: Some(Vec::new()),
            next_cursor_token: None,
            limit: 10,
            events_truncated: false,
        };

        let quality = monitor_quality(&unavailable, &durable_jobs, &unavailable, Utc::now());

        assert_eq!(quality.data_quality, MonitorDataQuality::Partial);
        assert!(quality
            .unknown_fields
            .iter()
            .any(|field| field == "pending_tasks"));
        assert!(quality
            .unknown_fields
            .iter()
            .any(|field| field == "workers"));
        assert_eq!(quality.redis.error_code.as_deref(), Some("SOURCE_DOWN"));
        assert_eq!(
            quality.worker_heartbeats.status,
            MonitorSourceStatus::Unavailable
        );

        let degraded_durable_jobs = MonitorDurableJobsSection {
            status: MonitorSourceStatus::Degraded,
            error_code: Some("PARTIAL_SOURCE".to_string()),
            jobs: None,
            next_cursor_token: None,
            limit: 10,
            events_truncated: false,
        };
        let degraded_quality =
            monitor_quality(&available, &degraded_durable_jobs, &available, Utc::now());
        assert_eq!(degraded_quality.data_quality, MonitorDataQuality::Degraded);
    }

    #[test]
    fn test_empty_worker_heartbeat_set_reports_measured_zero_workers() {
        let observed_at = Utc::now();
        let stale_cutoff = observed_at - TimeDelta::seconds(STALE_THRESHOLD_SECS);

        let (active, idle, workers) =
            filter_stale_workers_at(Vec::new(), stale_cutoff, observed_at);

        assert_eq!(active, 0);
        assert_eq!(idle, 0);
        assert!(workers.is_empty());
    }

    #[test]
    fn test_percentile_is_deterministic_and_unknown_without_samples() {
        assert_eq!(percentile(&[], 0.5), None);
        assert_eq!(percentile(&[1.0, 3.0, 2.0, 4.0], 0.5), Some(2.0));
        assert_eq!(percentile(&[1.0, 3.0, 2.0, 4.0], 0.95), Some(4.0));
    }

    #[test]
    fn test_queue_health_preserves_unknown_when_redis_is_unavailable() {
        let observed_at = Utc::now();
        let snapshot = RedisMonitorSnapshot {
            pending_high: None,
            pending_normal: None,
            pending_low: None,
            dead_letter_depth: None,
            completed_today: None,
            failed_today: None,
            workers: Vec::new(),
            recent_jobs: Vec::new(),
            redis: MonitorSourceHealth {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some("REDIS_DOWN".to_string()),
            },
            worker_heartbeats: MonitorSourceHealth {
                status: MonitorSourceStatus::Unavailable,
                error_code: Some("REDIS_DOWN".to_string()),
            },
        };

        let health = build_queue_health(&snapshot, None, observed_at);

        assert_eq!(health.depth_total.value, None);
        assert_eq!(health.depth_total.source, "redis:worker_queues");
        assert_eq!(
            health
                .depth_total
                .unknown
                .as_ref()
                .map(|unknown| unknown.code.as_str()),
            Some("REDIS_QUEUE_PARTIAL")
        );
    }

    #[test]
    fn test_reliability_is_unknown_without_durable_samples() {
        let reliability = build_reliability(None, Utc::now());

        assert_eq!(reliability.success_rate.value, None);
        assert_eq!(reliability.failure_rate.value, None);
        assert_eq!(reliability.retryable_count.value, None);
        assert_eq!(
            reliability
                .success_rate
                .unknown
                .as_ref()
                .map(|unknown| unknown.code.as_str()),
            Some("RELIABILITY_NO_SAMPLES")
        );
    }

    #[test]
    fn test_operational_thresholds_are_backend_owned() {
        let thresholds = build_monitor_thresholds();

        assert_eq!(thresholds.source, "backend_default");
        assert_eq!(thresholds.queue_pressure.medium_at, 1);
        assert_eq!(thresholds.worker_utilization.critical_at_percent, 80);
    }
}
