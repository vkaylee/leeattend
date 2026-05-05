//! Report Coordinator Service
//!
//! Splits large report requests into micro-jobs ([`Job`] with [`JobPayload`])
//! and enqueues them via [`JobQueue`] for distributed worker processing.
//! Tracks job progress and triggers result aggregation when all batches complete.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::queue::{
    DateRange, Job, JobMetadata, JobPayload, JobPriority, JobQueue, JobStatus, MergePayload,
    RedisPool, ReportConfig, ReportPayload, ReportType, TaskType,
};
use crate::repositories::employee;

// Configuration constants for batch processing
pub const BATCH_SIZE: usize = 50;
pub const JOB_TIMEOUT_SECS: u64 = 3600; // 1 hour max job duration
pub const ESTIMATED_SECONDS_PER_BATCH: u64 = 1;

/// Request to create a new distributed job.
#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub report_type: ReportType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default)]
    pub employee_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub sort: Option<String>,
}

/// Response after creating a job.
#[derive(Debug, Serialize)]
pub struct CreateJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub total_batches: u32,
    pub estimated_seconds: u64,
    pub status_url: String,
}

/// Progress tracking for a job.
#[derive(Debug, Serialize)]
pub struct JobProgress {
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub percentage: f32,
}

/// Response for job status polling.
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub progress: JobProgress,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Errors that can occur during report coordination.
#[derive(Debug)]
pub enum ReportError {
    NoEmployees,
    JobNotFound,
    CannotCancel,
    QueueError(String),
    DatabaseError(String),
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::NoEmployees => write!(f, "No active employees found for report"),
            ReportError::JobNotFound => write!(f, "Job not found"),
            ReportError::CannotCancel => write!(f, "Cannot cancel completed or failed job"),
            ReportError::QueueError(e) => write!(f, "Queue error: {e}"),
            ReportError::DatabaseError(e) => write!(f, "Database error: {e}"),
        }
    }
}

impl std::error::Error for ReportError {}

/// Workspace config subset for report generation.
#[derive(sqlx::FromRow)]
struct WorkspaceConfig {
    timezone: String,
}

/// Coordinates distributed report generation.
/// Splits requests into micro-jobs and tracks completion.
pub struct ReportCoordinator {
    queue: JobQueue,
    db: PgPool,
}

impl ReportCoordinator {
    /// Create a new ReportCoordinator.
    pub fn new(redis: RedisPool, db: PgPool) -> Self {
        Self {
            queue: JobQueue::new(redis),
            db,
        }
    }

    /// Create a new distributed job.
    ///
    /// Splits employees into batches of BATCH_SIZE, creates micro-jobs,
    /// and enqueues them for worker processing.
    ///
    /// # Arguments
    /// * `workspace_id` - Workspace to generate report for
    /// * `requester_id` - User requesting the report
    /// * `request` - Report parameters
    ///
    /// # Returns
    /// Job creation response with job_id and status URL.
    #[tracing::instrument(skip_all, err)]
    pub async fn create_report_job(
        &self,
        workspace_id: Uuid,
        requester_id: Uuid,
        request: CreateJobRequest,
    ) -> Result<CreateJobResponse, ReportError> {
        // 1. Fetch employee list
        let employees = self
            .fetch_employees(workspace_id, request.employee_ids.as_ref())
            .await?;

        if employees.is_empty() {
            return Err(ReportError::NoEmployees);
        }

        // 2. Generate parent job ID
        let parent_job_id = Uuid::new_v4();

        // 3. Fetch workspace config for jobs
        let workspace = self.fetch_workspace_config(workspace_id).await?;

        // 4. Split into batches
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();

        let total_batches = batches.len() as u32;

        // 5. Create micro-jobs
        let jobs: Vec<Job> = batches
            .into_iter()
            .enumerate()
            .map(|(idx, emp_ids)| {
                Job::with_priority(
                    parent_job_id,
                    idx as u32,
                    TaskType::AttendanceReport,
                    workspace_id,
                    JobPayload::AttendanceReport(ReportPayload {
                        employee_ids: emp_ids,
                        date_range: DateRange {
                            start: request.start_date,
                            end: request.end_date,
                        },
                        config: ReportConfig {
                            report_type: request.report_type.clone(),
                            timezone: workspace.timezone.clone(),
                            rounding_policy: "floor".to_string(),
                            work_unit_minutes: 30,
                        },
                    }),
                    JobPriority::High,
                )
            })
            .collect();

        // 6. Create parent metadata with date range
        let meta = JobMetadata::with_date_range(
            parent_job_id,
            total_batches,
            workspace_id,
            requester_id,
            Some(request.report_type.clone()),
            DateRange {
                start: request.start_date,
                end: request.end_date,
            },
        );

        self.queue
            .create_job_metadata(&meta)
            .await
            .map_err(|e| ReportError::QueueError(e.to_string()))?;

        // 7. Enqueue all jobs atomically
        self.queue
            .enqueue_jobs(jobs)
            .await
            .map_err(|e| ReportError::QueueError(e.to_string()))?;

        tracing::info!(
            job_id = %parent_job_id,
            workspace_id = %workspace_id,
            total_batches = total_batches,
            employee_count = employees.len(),
            "Report job created and queued"
        );

        // 8. Calculate estimated time
        let estimated_seconds = (total_batches as u64) * ESTIMATED_SECONDS_PER_BATCH;

        Ok(CreateJobResponse {
            job_id: parent_job_id,
            status: "pending".to_string(),
            total_batches,
            estimated_seconds,
            status_url: format!("/api/workspaces/{workspace_id}/reports/jobs/{parent_job_id}"),
        })
    }

    /// Get job status for polling.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID to query
    ///
    /// # Returns
    /// Current job status with progress and result URL if complete.
    #[tracing::instrument(skip_all, err)]
    pub async fn get_job_status(&self, job_id: Uuid) -> Result<JobStatusResponse, ReportError> {
        let meta = self
            .queue
            .get_job_metadata(job_id)
            .await
            .map_err(|e| ReportError::QueueError(e.to_string()))?
            .ok_or(ReportError::JobNotFound)?;

        let percentage = if meta.total_batches > 0 {
            (meta.completed_batches as f32 / meta.total_batches as f32) * 100.0
        } else {
            0.0
        };

        Ok(JobStatusResponse {
            job_id: meta.job_id,
            status: format!("{:?}", meta.status).to_lowercase(),
            progress: JobProgress {
                total_batches: meta.total_batches,
                completed_batches: meta.completed_batches,
                failed_batches: meta.failed_batches,
                percentage,
            },
            error: meta.error_message,
            created_at: meta.created_at,
            completed_at: meta.completed_at,
        })
    }

    /// Cancel a pending or processing job.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID to cancel
    ///
    /// # Returns
    /// Ok(()) if cancelled, error if already complete or not found.
    #[tracing::instrument(skip_all, err)]
    pub async fn cancel_job(&self, job_id: Uuid) -> Result<(), ReportError> {
        let mut meta = self
            .queue
            .get_job_metadata(job_id)
            .await
            .map_err(|e| ReportError::QueueError(e.to_string()))?
            .ok_or(ReportError::JobNotFound)?;

        if meta.status == JobStatus::Completed || meta.status == JobStatus::Failed {
            return Err(ReportError::CannotCancel);
        }

        meta.status = JobStatus::Failed;
        meta.error_message = Some("Cancelled by user".to_string());
        meta.completed_at = Some(Utc::now());

        self.queue
            .update_job_metadata(&meta)
            .await
            .map_err(|e| ReportError::QueueError(e.to_string()))?;

        tracing::info!(job_id = %job_id, "Report job cancelled");
        Ok(())
    }

    /// Called by workers when a batch completes.
    ///
    /// Updates completion counts and triggers merge job when all batches done.
    ///
    /// # Arguments
    /// * `parent_job_id` - Parent job ID
    /// * `batch_index` - Completed batch index
    /// * `success` - Whether the batch succeeded
    ///
    /// # Returns
    /// true if this was the last batch (merge should be triggered).
    #[tracing::instrument(skip_all, err)]
    pub async fn on_batch_complete(
        &self,
        parent_job_id: Uuid,
        _batch_index: u32,
        success: bool,
    ) -> Result<bool, ReportError> {
        let completed_count = if success {
            self.queue
                .increment_completed(parent_job_id)
                .await
                .map_err(|e| ReportError::QueueError(e.to_string()))?
        } else {
            self.queue
                .increment_failed(parent_job_id)
                .await
                .map_err(|e| ReportError::QueueError(e.to_string()))?;
            0 // Don't check for completion on failure
        };

        // Get total to check if complete
        let meta = self
            .queue
            .get_job_metadata(parent_job_id)
            .await
            .map_err(|e| ReportError::QueueError(e.to_string()))?
            .ok_or(ReportError::JobNotFound)?;

        let all_done = completed_count >= meta.total_batches;

        if all_done {
            // Atomically try to set merge trigger flag
            // This prevents race condition where multiple workers trigger merge
            let should_trigger = self
                .queue
                .try_trigger_merge(parent_job_id)
                .await
                .map_err(|e| ReportError::QueueError(e.to_string()))?;

            if should_trigger {
                tracing::info!(
                    parent_job_id = %parent_job_id,
                    completed = completed_count,
                    total = meta.total_batches,
                    "All batches complete"
                );

                if let Some(ref report_type) = meta.report_type {
                    // It's a report job, trigger merge
                    tracing::info!(parent_job_id = %parent_job_id, "Triggering merge job");

                    #[allow(clippy::expect_used)]
                    let date_range = meta.date_range.clone().unwrap_or(DateRange {
                        start: NaiveDate::from_ymd_opt(2000, 1, 1).expect("2000-01-01 is valid"),
                        end: NaiveDate::from_ymd_opt(2000, 1, 1).expect("2000-01-01 is valid"),
                    });

                    let merge_job = Job::with_priority(
                        parent_job_id,
                        0, // batch_index 0 for merge
                        TaskType::MergeResults,
                        meta.workspace_id,
                        JobPayload::MergeResults(MergePayload {
                            date_range,
                            config: ReportConfig {
                                report_type: report_type.clone(),
                                timezone: "UTC".to_string(),
                                rounding_policy: "floor".to_string(),
                                work_unit_minutes: 30,
                            },
                        }),
                        JobPriority::High,
                    );

                    self.queue
                        .enqueue_jobs(vec![merge_job])
                        .await
                        .map_err(|e| ReportError::QueueError(e.to_string()))?;
                } else {
                    // Not a report job (e.g. ReprocessAttendance), update status directly
                    tracing::info!(parent_job_id = %parent_job_id, "Updating status to Completed (no merge needed)");
                    self.queue
                        .update_job_status(parent_job_id, JobStatus::Completed)
                        .await
                        .map_err(|e| ReportError::QueueError(e.to_string()))?;
                }
            }
        }

        Ok(all_done)
    }

    /// Fetch employee IDs for report generation.
    async fn fetch_employees(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, ReportError> {
        employee::get_active_employee_ids(&self.db, workspace_id, filter_ids)
            .await
            .map_err(|e| ReportError::DatabaseError(e.to_string()))
    }

    /// Fetch workspace configuration for jobs.
    async fn fetch_workspace_config(
        &self,
        workspace_id: Uuid,
    ) -> Result<WorkspaceConfig, ReportError> {
        let row = sqlx::query_as!(
            WorkspaceConfig,
            r#"SELECT timezone
               FROM workspaces WHERE id = $1"#,
            workspace_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e: sqlx::Error| ReportError::DatabaseError(e.to_string()))?;

        Ok(row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_splitting() {
        // Simulate batch splitting logic
        let employees: Vec<Uuid> = (0..550).map(|_| Uuid::new_v4()).collect();

        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();

        // 550 employees / 50 = 11 full batches
        assert_eq!(batches.len(), 11);
        assert_eq!(batches[0].len(), 50);
        assert_eq!(batches[10].len(), 50);
    }

    #[test]
    fn test_edge_cases() {
        // Exactly 50 employees - 1 batch
        let employees: Vec<Uuid> = (0..50).map(|_| Uuid::new_v4()).collect();
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 50);

        // 51 employees - 2 batches
        let employees: Vec<Uuid> = (0..51).map(|_| Uuid::new_v4()).collect();
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 50);
        assert_eq!(batches[1].len(), 1);

        // 1 employee - 1 batch
        let employees: Vec<Uuid> = vec![Uuid::new_v4()];
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
    }

    #[test]
    fn test_estimated_time_calculation() {
        let total_batches = 140u32; // 7000 employees / 50
        let estimated = (total_batches as u64) * ESTIMATED_SECONDS_PER_BATCH;
        assert_eq!(estimated, 140); // 140 seconds
    }

    #[test]
    fn test_report_error_display() {
        assert_eq!(
            format!("{}", ReportError::NoEmployees),
            "No active employees found for report"
        );
        assert_eq!(format!("{}", ReportError::JobNotFound), "Job not found");
        assert_eq!(
            format!("{}", ReportError::CannotCancel),
            "Cannot cancel completed or failed job"
        );
        assert_eq!(
            format!("{}", ReportError::QueueError("timeout".into())),
            "Queue error: timeout"
        );
        assert_eq!(
            format!("{}", ReportError::DatabaseError("conn refused".into())),
            "Database error: conn refused"
        );
    }

    #[test]
    fn test_batch_splitting_large() {
        let employees: Vec<Uuid> = (0..1000).map(|_| Uuid::new_v4()).collect();
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();
        assert_eq!(batches.len(), 20);
        assert!(batches.iter().all(|b| b.len() == 50));
    }

    #[test]
    fn test_batch_splitting_remainder() {
        let employees: Vec<Uuid> = (0..73).map(|_| Uuid::new_v4()).collect();
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(<[uuid::Uuid]>::to_vec)
            .collect();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 50);
        assert_eq!(batches[1].len(), 23);
    }

    #[test]
    fn test_batch_size_constant() {
        assert_eq!(BATCH_SIZE, 50);
        assert_eq!(JOB_TIMEOUT_SECS, 3600);
        assert_eq!(ESTIMATED_SECONDS_PER_BATCH, 1);
    }

    #[test]
    fn test_create_job_response_serialization() {
        let response = CreateJobResponse {
            job_id: Uuid::nil(),
            status: "pending".into(),
            total_batches: 5,
            estimated_seconds: 5,
            status_url: "/api/reports/jobs/x".into(),
        };
        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["status"], "pending");
        assert_eq!(json["total_batches"], 5);
    }

    #[test]
    fn test_job_progress_serialization() {
        let progress = JobProgress {
            total_batches: 10,
            completed_batches: 5,
            failed_batches: 1,
            percentage: 50.0,
        };
        let json = serde_json::to_value(&progress).unwrap();
        assert_eq!(json["total_batches"], 10);
        assert_eq!(json["completed_batches"], 5);
        assert_eq!(json["failed_batches"], 1);
    }

    #[test]
    fn test_report_error_is_debug() {
        // Ensure all variants implement Debug
        let _ = format!("{:?}", ReportError::NoEmployees);
        let _ = format!("{:?}", ReportError::JobNotFound);
        let _ = format!("{:?}", ReportError::CannotCancel);
        let _ = format!("{:?}", ReportError::QueueError("x".into()));
        let _ = format!("{:?}", ReportError::DatabaseError("y".into()));
    }
}
