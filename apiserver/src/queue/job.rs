use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of task to be processed by workers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AttendanceReport,
    PayrollReport,
    MergeResults,
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

/// A single report processing job (micro-job).
/// Each job processes a batch of employees for a date range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportJob {
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
    /// Employee IDs to process (max 20 per job)
    pub employee_ids: Vec<Uuid>,
    /// Date range for the report
    pub date_range: DateRange,
    /// Report configuration
    pub config: ReportConfig,
    /// Job creation timestamp
    pub created_at: DateTime<Utc>,
    /// Current retry count
    pub retry_count: u32,
    /// Maximum retries before failure
    pub max_retries: u32,
}

impl ReportJob {
    /// Create a new report job with default retry settings.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent_job_id: Uuid,
        batch_index: u32,
        task_type: TaskType,
        workspace_id: Uuid,
        employee_ids: Vec<Uuid>,
        date_range: DateRange,
        config: ReportConfig,
    ) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            parent_job_id,
            batch_index,
            task_type,
            workspace_id,
            employee_ids,
            date_range,
            config,
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
    /// Unique job identifier (same as parent_job_id in ReportJob)
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
    fn test_report_job_serialization() {
        let job = ReportJob::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            vec![Uuid::new_v4(), Uuid::new_v4()],
            DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            ReportConfig {
                report_type: ReportType::Summary,
                timezone: "Asia/Ho_Chi_Minh".to_string(),
                rounding_policy: "floor".to_string(),
                work_unit_minutes: 30,
            },
        );

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: ReportJob = serde_json::from_str(&json).unwrap();

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
        let mut job = ReportJob::new(
            Uuid::new_v4(),
            0,
            TaskType::AttendanceReport,
            Uuid::new_v4(),
            vec![],
            DateRange {
                start: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                end: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            },
            ReportConfig {
                report_type: ReportType::Summary,
                timezone: "UTC".to_string(),
                rounding_policy: "floor".to_string(),
                work_unit_minutes: 30,
            },
        );

        assert!(!job.increment_retry()); // 1 <= 3
        assert!(!job.increment_retry()); // 2 <= 3
        assert!(!job.increment_retry()); // 3 <= 3
        assert!(job.increment_retry()); // 4 > 3, exhausted
    }
}
