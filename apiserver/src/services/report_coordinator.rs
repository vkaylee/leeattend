//! Report Coordinator Service
//!
//! Splits large report requests into micro-jobs for distributed processing.
//! Tracks job progress and triggers result aggregation when all batches complete.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::queue::{
    DateRange, JobMetadata, JobStatus, RedisPool, ReportConfig, ReportJob, ReportQueue,
    ReportType, TaskType,
};
use crate::repositories::employee_repository;

// Configuration constants for batch processing
pub const BATCH_SIZE: usize = 200;
pub const JOB_TIMEOUT_SECS: u64 = 3600; // 1 hour max job duration
pub const ESTIMATED_SECONDS_PER_BATCH: u64 = 2;

/// Request to create a new distributed report job.
#[derive(Debug, Deserialize)]
pub struct CreateReportJobRequest {
    pub report_type: ReportType,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default)]
    pub employee_ids: Option<Vec<Uuid>>,
    #[serde(default)]
    pub sort: Option<String>,
}

/// Response after creating a report job.
#[derive(Debug, Serialize)]
pub struct CreateReportJobResponse {
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
            ReportError::QueueError(e) => write!(f, "Queue error: {}", e),
            ReportError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for ReportError {}

/// Workspace config subset for report generation.
struct WorkspaceConfig {
    timezone: String,
    rounding_policy: String,
    work_unit_minutes: i32,
}

/// Coordinates distributed report generation.
/// Splits requests into micro-jobs and tracks completion.
pub struct ReportCoordinator {
    queue: ReportQueue,
    db: PgPool,
}

impl ReportCoordinator {
    /// Create a new ReportCoordinator.
    pub fn new(redis: RedisPool, db: PgPool) -> Self {
        Self {
            queue: ReportQueue::new(redis),
            db,
        }
    }

    /// Create a new distributed report job.
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
    pub async fn create_report_job(
        &self,
        workspace_id: Uuid,
        requester_id: Uuid,
        request: CreateReportJobRequest,
    ) -> Result<CreateReportJobResponse, ReportError> {
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
            .map(|chunk| chunk.to_vec())
            .collect();

        let total_batches = batches.len() as u32;

        // 5. Create micro-jobs
        let jobs: Vec<ReportJob> = batches
            .into_iter()
            .enumerate()
            .map(|(idx, emp_ids)| {
                ReportJob::new(
                    parent_job_id,
                    idx as u32,
                    TaskType::AttendanceReport,
                    workspace_id,
                    emp_ids,
                    DateRange {
                        start: request.start_date,
                        end: request.end_date,
                    },
                    ReportConfig {
                        report_type: request.report_type.clone(),
                        timezone: workspace.timezone.clone(),
                        rounding_policy: workspace.rounding_policy.clone(),
                        work_unit_minutes: workspace.work_unit_minutes,
                    },
                )
            })
            .collect();

        // 6. Create parent metadata with date range
        let meta = JobMetadata::with_date_range(
            parent_job_id,
            total_batches,
            workspace_id,
            requester_id,
            request.report_type.clone(),
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

        Ok(CreateReportJobResponse {
            job_id: parent_job_id,
            status: "pending".to_string(),
            total_batches,
            estimated_seconds,
            status_url: format!(
                "/api/workspaces/{}/reports/jobs/{}",
                workspace_id, parent_job_id
            ),
        })
    }

    /// Get job status for polling.
    ///
    /// # Arguments
    /// * `job_id` - Parent job ID to query
    ///
    /// # Returns
    /// Current job status with progress and result URL if complete.
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
                    "All batches complete, triggering merge (first worker)"
                );

                // Enqueue merge job with date range from metadata
                let date_range = meta.date_range.clone().unwrap_or(DateRange {
                    start: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                    end: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                });

                let merge_job = ReportJob::new(
                    parent_job_id,
                    0, // batch_index 0 for merge
                    TaskType::MergeResults,
                    meta.workspace_id,
                    vec![], // Not needed for merge
                    date_range,
                    ReportConfig {
                        report_type: meta.report_type.clone(),
                        timezone: "UTC".to_string(),
                        rounding_policy: "floor".to_string(),
                        work_unit_minutes: 30,
                    },
                );

                self.queue
                    .enqueue_jobs(vec![merge_job])
                    .await
                    .map_err(|e| ReportError::QueueError(e.to_string()))?;
            } else {
                tracing::debug!(
                    parent_job_id = %parent_job_id,
                    completed = completed_count,
                    total = meta.total_batches,
                    "All batches complete, but merge already triggered by another worker (race prevented)"
                );
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
        employee_repository::get_active_employee_ids(&self.db, workspace_id, filter_ids)
            .await
            .map_err(|e| ReportError::DatabaseError(e.to_string()))
    }

    /// Fetch workspace configuration for report jobs.
    async fn fetch_workspace_config(
        &self,
        workspace_id: Uuid,
    ) -> Result<WorkspaceConfig, ReportError> {
        let row = sqlx::query!(
            r#"SELECT timezone, rounding_policy, work_unit_minutes
               FROM workspaces WHERE id = $1"#,
            workspace_id
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| ReportError::DatabaseError(e.to_string()))?;

        Ok(WorkspaceConfig {
            timezone: row.timezone,
            rounding_policy: row.rounding_policy,
            work_unit_minutes: row.work_unit_minutes,
        })
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
            .map(|chunk| chunk.to_vec())
            .collect();

        // 550 employees / 200 = 2 full batches + 1 partial
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 200);
        assert_eq!(batches[1].len(), 200);
        assert_eq!(batches[2].len(), 150);
    }

    #[test]
    fn test_edge_cases() {
        // Exactly 200 employees - 1 batch
        let employees: Vec<Uuid> = (0..200).map(|_| Uuid::new_v4()).collect();
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 200);

        // 201 employees - 2 batches
        let employees: Vec<Uuid> = (0..201).map(|_| Uuid::new_v4()).collect();
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].len(), 200);
        assert_eq!(batches[1].len(), 1);

        // 1 employee - 1 batch
        let employees: Vec<Uuid> = vec![Uuid::new_v4()];
        let batches: Vec<Vec<Uuid>> = employees
            .chunks(BATCH_SIZE)
            .map(|chunk| chunk.to_vec())
            .collect();
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
    }

    #[test]
    fn test_estimated_time_calculation() {
        let total_batches = 35u32; // 7000 employees / 200
        let estimated = (total_batches as u64) * ESTIMATED_SECONDS_PER_BATCH;
        assert_eq!(estimated, 70); // 70 seconds
    }
}
