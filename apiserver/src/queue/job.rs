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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AttendanceReport,
    PayrollReport,
    MergeResults,
    LeaveAccrual,
}

/// Report format type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    Summary,
    Detail,
}

/// Date range for report generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

/// Job payload variants — each job type carries its own data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum JobPayload {
    AttendanceReport(ReportPayload),
    MergeResults(MergePayload),
    PayrollReport(ReportPayload),
    LeaveAccrual(AccrualPayload),
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
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Current retry count
    pub retry_count: u32,
    /// Maximum retries before failure
    pub max_retries: u32,
}

impl Job {
    /// Create a new job with default retry settings.
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
            created_at: Utc::now(),
            retry_count: 0,
            max_retries: 3,
        }
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
}

/// Status of a job in the processing pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Metadata for tracking overall job progress.
/// Stored in Redis with TTL for auto-cleanup.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Type of report
    pub report_type: ReportType,
    /// Date range for the report
    #[serde(default)]
    pub date_range: Option<DateRange>,
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Completion timestamp (if completed)
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

impl JobMetadata {
    /// Create new job metadata in pending state.
    pub fn new(
        job_id: Uuid,
        total_batches: u32,
        workspace_id: Uuid,
        requester_id: Uuid,
        report_type: ReportType,
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
        }
    }

    /// Create new job metadata with date range.
    pub fn with_date_range(
        job_id: Uuid,
        total_batches: u32,
        workspace_id: Uuid,
        requester_id: Uuid,
        report_type: ReportType,
        date_range: DateRange,
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
        }
    }

    /// Check if all batches are processed.
    pub fn is_complete(&self) -> bool {
        self.completed_batches + self.failed_batches >= self.total_batches
    }

    /// Calculate progress percentage.
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
            ReportType::Summary,
        );

        assert!(!meta.is_complete());
        assert_eq!(meta.progress_percent(), 0.0);

        meta.completed_batches = 5;
        assert_eq!(meta.progress_percent(), 50.0);

        meta.completed_batches = 8;
        meta.failed_batches = 2;
        assert!(meta.is_complete());
        assert_eq!(meta.progress_percent(), 100.0);
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
        assert_eq!(payload.date_range.start, NaiveDate::from_ymd_opt(2026, 3, 1).unwrap());
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
            ReportType::Detail,
            dr,
        );

        assert!(meta.date_range.is_some());
        let stored_dr = meta.date_range.unwrap();
        assert_eq!(stored_dr.start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
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
            ReportType::Summary,
        );

        assert_eq!(meta.progress_percent(), 100.0);
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
        assert_eq!(serde_json::to_string(&JobStatus::Pending).unwrap(), "\"pending\"");
        assert_eq!(serde_json::to_string(&JobStatus::Processing).unwrap(), "\"processing\"");
        assert_eq!(serde_json::to_string(&JobStatus::Completed).unwrap(), "\"completed\"");
        assert_eq!(serde_json::to_string(&JobStatus::Failed).unwrap(), "\"failed\"");
    }
}
