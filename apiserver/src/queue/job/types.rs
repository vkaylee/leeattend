//! Type definitions for the job system: enums, payload structs, and metadata.
//!
//! This module contains all the data types used by [`super::Job`] and the
//! worker infrastructure. It is re-exported via `pub(super) use types::*`
//! so sibling modules and tests see a flat namespace.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of task to be processed by workers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AttendanceReport,
    PayrollReport,
    MergeResults,
    LeaveAccrual,
    LogIngestion,
    ReprocessAttendance,
    ClaimStagedLogs,
    LeaveImport,
    NightlyHousekeeping,
    EmployeeImport,
    EmployeeImportChunk,
    LeaveImportChunk,
    FinalizeImport,
    ExpiringContractsCheck,
    SyncEmployeeHireDate,
    BulkTermination,
}

/// Job priority controls which Redis queue the job is routed to.
/// Dequeue order: High → Normal (via BLPOP), with weighted fairness
/// ensuring Low gets at least every Nth slot (see `LOW_QUEUE_WEIGHT`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    /// Reports — user is actively waiting. Routes to `queue:high`.
    High,
    /// Default priority for most jobs (reprocess, accrual, merge, payroll).
    /// Routes to `queue:normal`.
    #[default]
    Normal,
    /// Background tasks like log ingestion, housekeeping. Routes to `queue:low`.
    Low,
}

/// Report format type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Summary,
    Detail,
}

/// Date range for report generation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

/// Report generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub report_type: ReportType,
    pub timezone: String,
    pub rounding_policy: crate::models::RoundingPolicy,
    pub work_unit_minutes: i32,
}

/// Payload for report generation jobs (attendance/payroll).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPayload {
    pub employee_ids: Vec<Uuid>,
    pub date_range: DateRange,
    pub config: ReportConfig,
}

/// Payload for merge jobs (aggregates batch results into final output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergePayload {
    pub date_range: DateRange,
    pub config: ReportConfig,
}

/// Payload for leave accrual jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccrualPayload {
    pub date: NaiveDate,
    pub force: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub employee_ids: Option<Vec<Uuid>>,
}

/// Payload for attendance log ingestion jobs.
/// Contains all fields needed to process a single attendance log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIngestionPayload {
    pub workspace_id: Uuid,
    pub employee_id_number: String,
    pub punch_time_raw: String,
    pub device_id: Option<String>,
    pub raw_data: Option<serde_json::Value>,
    pub timezone: String,
    pub date_fmt: String,
    pub time_fmt: String,
}

/// Payload for bulk attendance reprocessing jobs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReprocessPayload {
    pub employee_id: Uuid,
    /// When set, reprocess this exact employment instead of the employee's
    /// active one — required when a specific employment's shift was reassigned.
    #[serde(default)]
    pub employment_id: Option<Uuid>,
    pub date_range: DateRange,
}

/// Payload for claiming orphan staged attendance logs after employee creation/import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimStagedLogsPayload {
    pub workspace_id: Uuid,
    pub employee_id: Uuid,
    pub employee_id_number: String,
}

/// Payload for bulk leave request import from Excel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveImportPayload {
    /// Path to the uploaded Excel file on disk
    pub file_path: String,
    /// User who initiated the import
    pub user_id: Uuid,
    /// Whether to auto-approve imported leave requests
    pub auto_approve: bool,
    /// DB tracking job ID (attendance_import_jobs table)
    pub import_job_id: Uuid,
}

/// Payload for nightly system maintenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HousekeepingPayload {
    /// Target date for calculating upcoming partitions (usually Utc::now().date_naive())
    pub reference_date: NaiveDate,
}

/// Payload for expiring contracts check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpiringContractsPayload {
    pub reference_date: NaiveDate,
}

/// Payload for bulk employee import from Excel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeImportPayload {
    /// DB tracking job ID (attendance_import_jobs table)
    pub import_job_id: Uuid,
    /// User who initiated the import
    pub user_id: Uuid,
}

/// Payload for processing a chunk of bulk employee import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeImportChunkPayload {
    pub import_job_id: Uuid,
    pub user_id: Uuid,
    pub file_path: String,
    pub start_index: usize,
    pub end_index: usize,
}

/// Payload for processing a chunk of bulk leave import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveImportChunkPayload {
    pub import_job_id: Uuid,
    pub user_id: Uuid,
    pub file_path: String,
    pub auto_approve: bool,
    pub start_index: usize,
    pub end_index: usize,
}

/// Payload for finalizing bulk import after all chunks are done.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalizeImportPayload {
    pub import_job_id: Uuid,
}

/// Payload for syncing an employee's hire date based on their earliest contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHireDatePayload {
    pub employee_id: Uuid,
    pub triggered_by_contract_id: Option<Uuid>,
}

/// Payload for bulk employee termination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTerminationRow {
    pub employment_id: Uuid,
    pub termination_date: NaiveDate,
    pub termination_reason: String,
    pub termination_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTerminationPayload {
    pub rows: Vec<BulkTerminationRow>,
}

/// Job payload variants — each job type carries its own data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JobPayload {
    AttendanceReport(ReportPayload),
    MergeResults(MergePayload),
    PayrollReport(ReportPayload),
    LeaveAccrual(AccrualPayload),
    LogIngestion(LogIngestionPayload),
    ReprocessAttendance(ReprocessPayload),
    ClaimStagedLogs(ClaimStagedLogsPayload),
    LeaveImport(LeaveImportPayload),
    NightlyHousekeeping(HousekeepingPayload),
    EmployeeImport(EmployeeImportPayload),
    EmployeeImportChunk(EmployeeImportChunkPayload),
    LeaveImportChunk(LeaveImportChunkPayload),
    FinalizeImport(FinalizeImportPayload),
    ExpiringContractsCheck(ExpiringContractsPayload),
    SyncEmployeeHireDate(SyncHireDatePayload),
    BulkTermination(BulkTerminationPayload),
}

/// Status of a job in the processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Metadata for tracking overall job progress.
/// Stored in Redis with TTL for auto-cleanup.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct JobMetadata {
    /// Unique job identifier (same as parent_job_id in Job)
    pub job_id: Uuid,
    /// Current status
    pub status: JobStatus,
    /// Total number of batches
    pub total_batches: u32,
    /// Completed batch count
    pub completed_batches: u32,
    /// Failed batch count
    pub failed_batches: u32,
    /// Workspace ID
    pub workspace_id: Uuid,
    /// User who requested the report
    pub requester_id: Uuid,
    /// Type of report (None if not a report job)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_type: Option<ReportType>,
    /// Date range for the report
    #[serde(default)]
    pub date_range: Option<DateRange>,
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Type of task
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    /// Payload configurations (PII masked)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

pub struct WithDateRangeParams {
    pub job_id: Uuid,
    pub total_batches: u32,
    pub workspace_id: Uuid,
    pub requester_id: Uuid,
    pub task_type: Option<TaskType>,
    pub report_type: Option<ReportType>,
    pub date_range: DateRange,
    pub payload: Option<serde_json::Value>,
}

impl JobMetadata {
    /// Create new job metadata in pending state.
    pub fn new(
        job_id: Uuid,
        total_batches: u32,
        workspace_id: Uuid,
        requester_id: Uuid,
        task_type: Option<TaskType>,
        report_type: Option<ReportType>,
        payload: Option<serde_json::Value>,
    ) -> Self {
        Self {
            job_id,
            status: JobStatus::Pending,
            total_batches,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id,
            requester_id,
            report_type,
            date_range: None,
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
            task_type,
            payload,
        }
    }

    /// Create new job metadata with date range.
    pub fn with_date_range(
        params: WithDateRangeParams,
    ) -> Self {
        Self {
            job_id: params.job_id,
            status: JobStatus::Pending,
            total_batches: params.total_batches,
            completed_batches: 0,
            failed_batches: 0,
            workspace_id: params.workspace_id,
            requester_id: params.requester_id,
            report_type: params.report_type,
            date_range: Some(params.date_range),
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
            task_type: params.task_type,
            payload: params.payload,
        }
    }

    /// Check if all batches are processed.
    pub fn is_complete(&self) -> bool {
        self.completed_batches + self.failed_batches >= self.total_batches
    }

    /// Calculate progress percentage.
    #[allow(clippy::cast_precision_loss)]
    pub fn progress_percent(&self) -> f32 {
        if self.total_batches == 0 {
            return 100.0;
        }
        ((self.completed_batches + self.failed_batches) as f32 / self.total_batches as f32) * 100.0
    }
}
