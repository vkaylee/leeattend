//! Job Types and Metadata
//!
//! Defines the core job structures for the distributed worker system.
//!
//! Each [`Job`] carries a [`JobPayload`] enum that encapsulates type-specific data:
//! - `AttendanceReport` / `PayrollReport` → [`ReportPayload`] (employee_ids, date_range, config)
//! - `MergeResults` → [`MergePayload`] (date_range, config — no employee_ids)
//!
//! Use [`Job::report_payload()`] and [`Job::merge_payload()`] for safe extraction.

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

/// Job priority level. Controls which Redis queue the job is routed to.
/// Workers dequeue high-priority jobs before low-priority ones.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobPriority {
    /// Reports — user is actively waiting for results
    High,
    /// Default priority for most jobs
    #[default]
    Normal,
    /// Background tasks like log ingestion — can wait
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
    pub rounding_policy: String,
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

/// A single worker job.
/// Generic job envelope with type-specific payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    /// Unique identifier for this job
    pub job_id: Uuid,
    /// Parent job ID (coordinator job)
    pub parent_job_id: Uuid,
    /// Batch index (0-based)
    pub batch_index: u32,
    /// Type of processing task
    pub task_type: TaskType,
    /// Workspace this job belongs to
    pub workspace_id: Uuid,
    /// Type-specific payload data
    pub payload: JobPayload,
    /// Job priority for queue routing
    #[serde(default)]
    pub priority: JobPriority,
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Current retry count
    pub retry_count: u32,
    /// Maximum retries before failure
    pub max_retries: u32,
    /// Context tracing (e.g. Request ID) for propagation
    pub trace_context: Option<String>,
}

impl Job {
    /// Create a new job with default retry settings and Normal priority.
    pub fn new(
        parent_job_id: Uuid,
        batch_index: u32,
        task_type: TaskType,
        workspace_id: Uuid,
        payload: JobPayload,
    ) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            parent_job_id,
            batch_index,
            task_type,
            workspace_id,
            payload,
            priority: JobPriority::default(),
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
            trace_context: crate::utils::logging::get_trace_context(),
        }
    }

    /// Create a new job with explicit priority.
    pub fn with_priority(
        parent_job_id: Uuid,
        batch_index: u32,
        task_type: TaskType,
        workspace_id: Uuid,
        payload: JobPayload,
        priority: JobPriority,
    ) -> Self {
        let mut job = Self::new(parent_job_id, batch_index, task_type, workspace_id, payload);
        job.priority = priority;
        job
    }

    /// Increment retry count and return whether retries are exhausted.
    pub fn increment_retry(&mut self) -> bool {
        self.retry_count += 1;
        self.retry_count > self.max_retries
    }

    /// Helper: extract report payload (attendance/payroll).
    pub fn report_payload(&self) -> Option<&ReportPayload> {
        match &self.payload {
            JobPayload::AttendanceReport(p) | JobPayload::PayrollReport(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract merge payload.
    pub fn merge_payload(&self) -> Option<&MergePayload> {
        match &self.payload {
            JobPayload::MergeResults(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract accrual payload.
    pub fn accrual_payload(&self) -> Option<&AccrualPayload> {
        match &self.payload {
            JobPayload::LeaveAccrual(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract log ingestion payload.
    pub fn log_ingestion_payload(&self) -> Option<&LogIngestionPayload> {
        match &self.payload {
            JobPayload::LogIngestion(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract reprocess payload.
    pub fn reprocess_payload(&self) -> Option<&ReprocessPayload> {
        match &self.payload {
            JobPayload::ReprocessAttendance(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract claim staged logs payload.
    pub fn claim_staged_logs_payload(&self) -> Option<&ClaimStagedLogsPayload> {
        match &self.payload {
            JobPayload::ClaimStagedLogs(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract leave import payload.
    pub fn leave_import_payload(&self) -> Option<&LeaveImportPayload> {
        match &self.payload {
            JobPayload::LeaveImport(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract housekeeping payload.
    pub fn housekeeping_payload(&self) -> Option<&HousekeepingPayload> {
        match &self.payload {
            JobPayload::NightlyHousekeeping(p) => Some(p),
            _ => None,
        }
    }

    /// Helper: extract employee import payload.
    pub fn employee_import_payload(&self) -> Option<&EmployeeImportPayload> {
        match &self.payload {
            JobPayload::EmployeeImport(p) => Some(p),
            _ => None,
        }
    }

    pub fn employee_import_chunk_payload(&self) -> Option<&EmployeeImportChunkPayload> {
        match &self.payload {
            JobPayload::EmployeeImportChunk(p) => Some(p),
            _ => None,
        }
    }

    pub fn leave_import_chunk_payload(&self) -> Option<&LeaveImportChunkPayload> {
        match &self.payload {
            JobPayload::LeaveImportChunk(p) => Some(p),
            _ => None,
        }
    }

    pub fn finalize_import_payload(&self) -> Option<&FinalizeImportPayload> {
        match &self.payload {
            JobPayload::FinalizeImport(p) => Some(p),
            _ => None,
        }
    }

    pub fn expiring_contracts_payload(&self) -> Option<&ExpiringContractsPayload> {
        match &self.payload {
            JobPayload::ExpiringContractsCheck(p) => Some(p),
            _ => None,
        }
    }

    pub fn sync_hire_date_payload(&self) -> Option<&SyncHireDatePayload> {
        match &self.payload {
            JobPayload::SyncEmployeeHireDate(p) => Some(p),
            _ => None,
        }
    }

    pub fn bulk_termination_payload(&self) -> Option<&BulkTerminationPayload> {
        match &self.payload {
            JobPayload::BulkTermination(p) => Some(p),
            _ => None,
        }
    }
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
    #[allow(clippy::too_many_arguments)]
    pub fn with_date_range(
        job_id: Uuid,
        total_batches: u32,
        workspace_id: Uuid,
        requester_id: Uuid,
        task_type: Option<TaskType>,
        report_type: Option<ReportType>,
        date_range: DateRange,
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
            date_range: Some(date_range),
            created_at: Utc::now(),
            completed_at: None,
            error_message: None,
            task_type,
            payload,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_serialization() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "Asia/Ho_Chi_Minh".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: Job = serde_json::from_str(&json).unwrap();

        assert_eq!(job.job_id, deserialized.job_id);
        assert_eq!(job.batch_index, deserialized.batch_index);
        assert_eq!(job.task_type, deserialized.task_type);
    }

    #[test]
    fn test_job_metadata_progress() {
        let mut meta = JobMetadata::new(
            Uuid::new_v4(),
            10,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Some(ReportType::Summary),
            None,
        );

        assert!(!meta.is_complete());
        assert!((meta.progress_percent() - 0.0).abs() < f32::EPSILON);

        meta.completed_batches = 5;
        assert!((meta.progress_percent() - 50.0).abs() < f32::EPSILON);

        meta.completed_batches = 8;
        meta.failed_batches = 2;
        assert!(meta.is_complete());
        assert!((meta.progress_percent() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_retry_logic() {
        let mut job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(!job.increment_retry()); // 1 <= 3
        assert!(!job.increment_retry()); // 2 <= 3
        assert!(!job.increment_retry()); // 3 <= 3
        assert!(job.increment_retry()); // 4 > 3, exhausted
    }

    #[test]
    fn test_payload_helpers() {
        let report_job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![Uuid::new_v4()],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(report_job.report_payload().is_some());
        assert!(report_job.merge_payload().is_none());

        let merge_job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::MergeResults,
            Uuid::new_v4(),
            JobPayload::MergeResults(MergePayload {
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(merge_job.merge_payload().is_some());
        assert!(merge_job.report_payload().is_none());
    }

    // ====== NEW TESTS: Serde Compatibility & Payload Validation ======

    /// Verify the tagged-enum serde format produces expected JSON structure.
    /// This is critical for Redis interoperability — changing the format
    /// would break in-flight jobs during a rolling deploy.
    #[test]
    fn test_job_payload_serde_tagged_format() {
        let payload = JobPayload::AttendanceReport(ReportPayload {
            employee_ids: vec![],
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: "floor".to_string(),
                work_unit_minutes: 30,
            },
        });

        let json = serde_json::to_value(&payload).unwrap();
        // Tagged enum: {"type": "AttendanceReport", "data": {...}}
        assert_eq!(json["type"], "AttendanceReport");
        assert!(json["data"].is_object());
        assert!(json["data"]["employee_ids"].is_array());
        assert!(json["data"]["config"]["report_type"].is_string());
    }

    /// MergeResults payload uses separate MergePayload (no employee_ids).
    #[test]
    fn test_merge_payload_serde_format() {
        let payload = JobPayload::MergeResults(MergePayload {
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            config: ReportConfig {
                report_type: ReportType::Detail,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                rounding_policy: "ceil".to_string(),
                work_unit_minutes: 15,
            },
        });

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "MergeResults");
        // MergePayload has no employee_ids field
        assert!(json["data"]["employee_ids"].is_null());
        assert_eq!(json["data"]["config"]["report_type"], "detail");
    }

    /// PayrollReport shares ReportPayload with AttendanceReport.
    /// Both should be extractable via report_payload().
    #[test]
    fn test_payroll_report_payload_extraction() {
        let emp_id = Uuid::new_v4();
        let payroll_job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::PayrollReport,
            Uuid::new_v4(),
            JobPayload::PayrollReport(ReportPayload {
                employee_ids: vec![emp_id],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        // PayrollReport and AttendanceReport both return via report_payload()
        let payload = payroll_job.report_payload().expect("should extract");
        assert_eq!(payload.employee_ids.len(), 1);
        assert_eq!(payload.employee_ids[0], emp_id);

        // Should NOT be a merge payload
        assert!(payroll_job.merge_payload().is_none());
    }

    /// Serde roundtrip must preserve all Job fields including nested payload.
    #[test]
    fn test_full_job_serde_roundtrip() {
        let parent_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let emp_ids: Vec<Uuid> = (0..5).map(|_| Uuid::new_v4()).collect();

        let job = Job::new(
            parent_id,
            3,
            TaskType::AttendanceReport,
            ws_id,
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: emp_ids.clone(),
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 3, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Detail,
                    timezone: "Asia/Tokyo".to_string(),
                    rounding_policy: "round".to_string(),
                    work_unit_minutes: 15,
                },
            }),
        );

        let json = serde_json::to_string(&job).unwrap();
        let roundtripped: Job = serde_json::from_str(&json).unwrap();

        // Top-level fields
        assert_eq!(roundtripped.job_id, job.job_id);
        assert_eq!(roundtripped.parent_job_id, parent_id);
        assert_eq!(roundtripped.batch_index, 3);
        assert_eq!(roundtripped.task_type, TaskType::AttendanceReport);
        assert_eq!(roundtripped.workspace_id, ws_id);
        assert_eq!(roundtripped.retry_count, 0);
        assert_eq!(roundtripped.max_retries, 3);

        // Nested payload
        let payload = roundtripped.report_payload().expect("should be report");
        assert_eq!(payload.employee_ids, emp_ids);
        assert_eq!(
            payload.date_range.start,
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
        );
        assert_eq!(payload.config.timezone, "Asia/Tokyo");
        assert_eq!(payload.config.report_type, ReportType::Detail);
        assert_eq!(payload.config.work_unit_minutes, 15);
    }

    /// Job::new() should set sensible defaults.
    #[test]
    fn test_job_new_defaults() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert_eq!(job.retry_count, 0);
        assert_eq!(job.max_retries, 3);
        assert_eq!(job.batch_index, 0);
        // job_id should be unique
        assert_ne!(job.job_id, Uuid::nil());
    }

    /// JobMetadata::with_date_range should store the date range.
    #[test]
    fn test_metadata_with_date_range() {
        let dr = DateRange {
            start: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        };
        let meta = JobMetadata::with_date_range(
            Uuid::new_v4(),
            5,
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Some(ReportType::Detail),
            dr,
            None,
        );

        assert!(meta.date_range.is_some());
        let stored_dr = meta.date_range.unwrap();
        assert_eq!(
            stored_dr.start,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
        );
        assert_eq!(stored_dr.end, NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    }

    /// Zero total_batches should report 100% progress (division by zero guard).
    #[test]
    fn test_metadata_zero_batches_progress() {
        let meta = JobMetadata::new(
            Uuid::new_v4(),
            0, // No batches
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            Some(ReportType::Summary),
            None,
        );

        assert!((meta.progress_percent() - 100.0).abs() < f32::EPSILON);
        assert!(meta.is_complete()); // 0 + 0 >= 0
    }

    /// TaskType serde uses snake_case.
    #[test]
    fn test_task_type_serde_snake_case() {
        let json = serde_json::to_string(&TaskType::AttendanceReport).unwrap();
        assert_eq!(json, "\"attendance_report\"");

        let json = serde_json::to_string(&TaskType::MergeResults).unwrap();
        assert_eq!(json, "\"merge_results\"");

        let json = serde_json::to_string(&TaskType::PayrollReport).unwrap();
        assert_eq!(json, "\"payroll_report\"");

        // Roundtrip
        let decoded: TaskType = serde_json::from_str("\"attendance_report\"").unwrap();
        assert_eq!(decoded, TaskType::AttendanceReport);
    }

    /// JobStatus serde uses snake_case.
    #[test]
    fn test_job_status_serde() {
        assert_eq!(
            serde_json::to_string(&JobStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&JobStatus::Processing).unwrap(),
            "\"processing\""
        );
        assert_eq!(
            serde_json::to_string(&JobStatus::Completed).unwrap(),
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&JobStatus::Failed).unwrap(),
            "\"failed\""
        );
    }

    // ====== PRIORITY + LOG INGESTION TESTS ======

    /// JobPriority serde uses snake_case.
    #[test]
    fn test_job_priority_serde() {
        assert_eq!(
            serde_json::to_string(&JobPriority::High).unwrap(),
            "\"high\""
        );
        assert_eq!(
            serde_json::to_string(&JobPriority::Normal).unwrap(),
            "\"normal\""
        );
        assert_eq!(serde_json::to_string(&JobPriority::Low).unwrap(), "\"low\"");

        // Roundtrip
        let decoded: JobPriority = serde_json::from_str("\"low\"").unwrap();
        assert_eq!(decoded, JobPriority::Low);
    }

    /// Default priority is Normal.
    #[test]
    fn test_job_priority_default() {
        let priority = JobPriority::default();
        assert_eq!(priority, JobPriority::Normal);
    }

    /// Job::new defaults to Normal priority.
    #[test]
    fn test_job_new_defaults_to_normal_priority() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert_eq!(job.priority, JobPriority::Normal);
    }

    /// Job::with_priority sets the correct priority.
    #[test]
    fn test_job_with_priority() {
        let job = Job::with_priority(
            Uuid::new_v4(),
            0,
            TaskType::LogIngestion,
            Uuid::new_v4(),
            JobPayload::LogIngestion(LogIngestionPayload {
                workspace_id: Uuid::new_v4(),
                employee_id_number: "EMP001".to_string(),
                punch_time_raw: "2026-01-26 09:00:00".to_string(),
                device_id: Some("D1".to_string()),
                raw_data: None,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                date_fmt: "DD/MM/YYYY".to_string(),
                time_fmt: "24h".to_string(),
            }),
            JobPriority::Low,
        );

        assert_eq!(job.priority, JobPriority::Low);
        assert_eq!(job.task_type, TaskType::LogIngestion);
    }

    /// LogIngestionPayload serde roundtrip.
    #[test]
    fn test_log_ingestion_payload_serde_roundtrip() {
        let ws_id = Uuid::new_v4();
        let payload = LogIngestionPayload {
            workspace_id: ws_id,
            employee_id_number: "EMP042".to_string(),
            punch_time_raw: "26/01/2026 09:15:00".to_string(),
            device_id: Some("ZK-MAIN".to_string()),
            raw_data: Some(serde_json::json!({"punch": 0, "status": 1})),
            timezone: "Asia/Ho_Chi_Minh".to_string(),
            date_fmt: "DD/MM/YYYY".to_string(),
            time_fmt: "24h".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let decoded: LogIngestionPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.workspace_id, ws_id);
        assert_eq!(decoded.employee_id_number, "EMP042");
        assert_eq!(decoded.device_id, Some("ZK-MAIN".to_string()));
        assert_eq!(decoded.raw_data.unwrap()["punch"], 0);
    }

    /// LogIngestion tagged enum serde format.
    #[test]
    fn test_log_ingestion_payload_tagged_format() {
        let payload = JobPayload::LogIngestion(LogIngestionPayload {
            workspace_id: Uuid::new_v4(),
            employee_id_number: "EMP001".to_string(),
            punch_time_raw: "2026-01-26 09:00:00".to_string(),
            device_id: None,
            raw_data: None,
            timezone: "UTC".to_string(),
            date_fmt: "YYYY-MM-DD".to_string(),
            time_fmt: "24h".to_string(),
        });

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "LogIngestion");
        assert!(json["data"].is_object());
        assert_eq!(json["data"]["employee_id_number"], "EMP001");
    }

    /// LogIngestion task type serde.
    #[test]
    fn test_log_ingestion_task_type_serde() {
        let json = serde_json::to_string(&TaskType::LogIngestion).unwrap();
        assert_eq!(json, "\"log_ingestion\"");

        let decoded: TaskType = serde_json::from_str("\"log_ingestion\"").unwrap();
        assert_eq!(decoded, TaskType::LogIngestion);
    }

    /// Full Job with LogIngestion + Low priority roundtrip.
    #[test]
    fn test_log_ingestion_job_full_roundtrip() {
        let parent_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let job = Job::with_priority(
            parent_id,
            0,
            TaskType::LogIngestion,
            ws_id,
            JobPayload::LogIngestion(LogIngestionPayload {
                workspace_id: ws_id,
                employee_id_number: "EMP999".to_string(),
                punch_time_raw: "2026-03-04 13:00:00".to_string(),
                device_id: Some("DEV-1".to_string()),
                raw_data: Some(serde_json::json!({"punch": 0, "status": 1, "verify": 1})),
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                date_fmt: "YYYY-MM-DD".to_string(),
                time_fmt: "24h".to_string(),
            }),
            JobPriority::Low,
        );

        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"log_ingestion\""));
        assert!(json.contains("\"low\""));

        let decoded: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.job_id, job.job_id);
        assert_eq!(decoded.task_type, TaskType::LogIngestion);
        assert_eq!(decoded.priority, JobPriority::Low);

        let payload = decoded.log_ingestion_payload().expect("should extract");
        assert_eq!(payload.employee_id_number, "EMP999");
        assert_eq!(payload.workspace_id, ws_id);
    }

    /// Backward compatibility: Job JSON without "priority" field should
    /// deserialize with default Normal priority.
    #[test]
    fn test_job_backward_compatible_no_priority_field() {
        // Simulate a Job serialized BEFORE the priority field was added
        let json = serde_json::json!({
            "job_id": Uuid::new_v4(),
            "parent_job_id": Uuid::new_v4(),
            "batch_index": 0,
            "task_type": "attendance_report",
            "workspace_id": Uuid::new_v4(),
            "payload": {
                "type": "AttendanceReport",
                "data": {
                    "employee_ids": [],
                    "date_range": {"start": "2026-01-01", "end": "2026-01-31"},
                    "config": {
                        "report_type": "summary",
                        "timezone": "UTC",
                        "rounding_policy": "floor",
                        "work_unit_minutes": 30
                    }
                }
            },
            "created_at": "2026-01-01T00:00:00Z",
            "retry_count": 0,
            "max_retries": 3
        });

        let job: Job = serde_json::from_value(json).unwrap();
        // Missing "priority" field should default to Normal
        assert_eq!(job.priority, JobPriority::Normal);
    }

    /// log_ingestion_payload() returns None for non-LogIngestion jobs.
    #[test]
    fn test_log_ingestion_payload_helper_returns_none_for_other_types() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(job.log_ingestion_payload().is_none());
        assert!(job.report_payload().is_some());
    }

    // ====== LEAVE IMPORT TESTS ======

    /// LeaveImportPayload serde roundtrip.
    #[test]
    fn test_leave_import_payload_serde_roundtrip() {
        let user_id = Uuid::new_v4();
        let import_job_id = Uuid::new_v4();
        let payload = LeaveImportPayload {
            file_path: "/storage/imports/leave_2026-03.xlsx".to_string(),
            user_id,
            auto_approve: true,
            import_job_id,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let decoded: LeaveImportPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.file_path, "/storage/imports/leave_2026-03.xlsx");
        assert_eq!(decoded.user_id, user_id);
        assert!(decoded.auto_approve);
        assert_eq!(decoded.import_job_id, import_job_id);
    }

    /// LeaveImport tagged enum serde format.
    #[test]
    fn test_leave_import_payload_tagged_format() {
        let payload = JobPayload::LeaveImport(LeaveImportPayload {
            file_path: "/tmp/test.xlsx".to_string(),
            user_id: Uuid::new_v4(),
            auto_approve: false,
            import_job_id: Uuid::new_v4(),
        });

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "LeaveImport");
        assert!(json["data"].is_object());
        assert_eq!(json["data"]["file_path"], "/tmp/test.xlsx");
        assert_eq!(json["data"]["auto_approve"], false);
    }

    /// LeaveImport task type serde.
    #[test]
    fn test_leave_import_task_type_serde() {
        let json = serde_json::to_string(&TaskType::LeaveImport).unwrap();
        assert_eq!(json, "\"leave_import\"");

        let decoded: TaskType = serde_json::from_str("\"leave_import\"").unwrap();
        assert_eq!(decoded, TaskType::LeaveImport);
    }

    /// Full Job with LeaveImport payload roundtrip.
    #[test]
    fn test_leave_import_job_full_roundtrip() {
        let parent_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let import_job_id = Uuid::new_v4();

        let job = Job::new(
            parent_id,
            0,
            TaskType::LeaveImport,
            ws_id,
            JobPayload::LeaveImport(LeaveImportPayload {
                file_path: "/storage/imports/leave.xlsx".to_string(),
                user_id,
                auto_approve: true,
                import_job_id,
            }),
        );

        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"leave_import\""));

        let decoded: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.job_id, job.job_id);
        assert_eq!(decoded.task_type, TaskType::LeaveImport);

        let payload = decoded.leave_import_payload().expect("should extract");
        assert_eq!(payload.file_path, "/storage/imports/leave.xlsx");
        assert_eq!(payload.user_id, user_id);
        assert!(payload.auto_approve);
        assert_eq!(payload.import_job_id, import_job_id);
    }

    /// leave_import_payload() returns None for non-LeaveImport jobs.
    #[test]
    fn test_leave_import_payload_helper_returns_none_for_other_types() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );

        assert!(job.leave_import_payload().is_none());
    }

    // ====== EMPLOYEE IMPORT TESTS ======

    /// EmployeeImportPayload serde roundtrip.
    #[test]
    fn test_employee_import_payload_serde_roundtrip() {
        let user_id = Uuid::new_v4();
        let import_job_id = Uuid::new_v4();
        let payload = EmployeeImportPayload {
            import_job_id,
            user_id,
        };

        let json = serde_json::to_string(&payload).unwrap();
        let decoded: EmployeeImportPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.user_id, user_id);
        assert_eq!(decoded.import_job_id, import_job_id);
    }

    /// EmployeeImport tagged enum serde format.
    #[test]
    fn test_employee_import_payload_tagged_format() {
        let payload = JobPayload::EmployeeImport(EmployeeImportPayload {
            user_id: Uuid::new_v4(),
            import_job_id: Uuid::new_v4(),
        });

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["type"], "EmployeeImport");
        assert!(json["data"].is_object());
    }

    /// EmployeeImport task type serde.
    #[test]
    fn test_employee_import_task_type_serde() {
        let json = serde_json::to_string(&TaskType::EmployeeImport).unwrap();
        assert_eq!(json, "\"employee_import\"");

        let decoded: TaskType = serde_json::from_str("\"employee_import\"").unwrap();
        assert_eq!(decoded, TaskType::EmployeeImport);
    }

    /// Full Job with EmployeeImport payload roundtrip.
    #[test]
    fn test_employee_import_job_full_roundtrip() {
        let parent_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let import_job_id = Uuid::new_v4();

        let job = Job::new(
            parent_id,
            0,
            TaskType::EmployeeImport,
            ws_id,
            JobPayload::EmployeeImport(EmployeeImportPayload {
                import_job_id,
                user_id,
            }),
        );

        let json = serde_json::to_string(&job).unwrap();
        assert!(json.contains("\"employee_import\""));

        let decoded: Job = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.job_id, job.job_id);
        assert_eq!(decoded.task_type, TaskType::EmployeeImport);

        let payload = decoded.employee_import_payload().expect("should extract");
        assert_eq!(payload.user_id, user_id);
        assert_eq!(payload.import_job_id, import_job_id);
    }

    /// employee_import_payload() returns None for non-EmployeeImport jobs.
    #[test]
    fn test_employee_import_payload_helper_returns_none_for_other_types() {
        let job = Job::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            JobPayload::AttendanceReport(ReportPayload {
                employee_ids: vec![],
                date_range: DateRange {
                    start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
                },
                config: ReportConfig {
                    report_type: ReportType::Summary,
                    timezone: "UTC".to_string(),
                    rounding_policy: "floor".to_string(),
                    work_unit_minutes: 30,
                },
            }),
        );
        assert!(job.employee_import_payload().is_none());
    }
}
