//! Report Export API Handlers
//!
//! Async API endpoints for distributed report job management:
//! - Create async export jobs
//! - Poll job status
//! - Download completed reports
//! - Cancel pending jobs

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::queue::ReportType;
use crate::services::report_coordinator::{
    CreateJobRequest, JobProgress as CoordinatorJobProgress, ReportCoordinator, ReportError,
};
use crate::AppState;

// ============================================================================
// Request/Response Types
// ============================================================================

/// Request body for creating an async export job.
#[derive(Debug, Deserialize)]
pub struct ExportReportRequest {
    /// Report type: "summary" or "detail"
    pub report_type: String,
    /// Start date for the report period
    pub start_date: NaiveDate,
    /// End date for the report period
    pub end_date: NaiveDate,
    /// Optional filter for specific employee IDs
    #[serde(default)]
    pub employee_ids: Option<Vec<Uuid>>,
    /// Output format (default: "xlsx")
    #[serde(default = "default_format")]
    pub format: String,
}

fn default_format() -> String {
    "xlsx".to_string()
}

/// Response for job creation (202 Accepted).
#[derive(Debug, Serialize)]
pub struct ExportJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub total_batches: u32,
    pub estimated_seconds: u64,
    pub status_url: String,
}

/// Response for job status polling.
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub progress: JobProgress,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
}

/// Progress details for a job.
#[derive(Debug, Serialize)]
pub struct JobProgress {
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub percentage: f32,
}

impl From<CoordinatorJobProgress> for JobProgress {
    fn from(p: CoordinatorJobProgress) -> Self {
        Self {
            total_batches: p.total_batches,
            completed_batches: p.completed_batches,
            failed_batches: p.failed_batches,
            percentage: p.percentage,
        }
    }
}

// ============================================================================
// Error Response Helper
// ============================================================================

impl From<ReportError> for AppError {
    fn from(err: ReportError) -> Self {
        match err {
            ReportError::NoEmployees => {
                AppError::Validation("No active employees found for report".to_string())
            }
            ReportError::JobNotFound => AppError::NotFound("Job not found".to_string()),
            ReportError::CannotCancel => {
                AppError::Conflict("Cannot cancel completed or failed job".to_string())
            }
            ReportError::QueueError(e) => AppError::from(format!("Queue error: {e}")),
            ReportError::DatabaseError(e) => AppError::from(format!("Database error: {e}")),
        }
    }
}

/// Get coordinator from state or return 503 Service Unavailable.
fn get_coordinator(state: &AppState) -> AppResult<&ReportCoordinator> {
    state
        .reports.coordinator
        .as_ref()
        .map(|c| c.as_ref())
        .ok_or_else(|| {
            AppError::ServiceUnavailable(
                "Async export not available. Redis is required for distributed processing."
                    .to_string(),
            )
        })
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/workspaces/:ws_id/reports/export
///
/// Create an async export job for attendance reports.
/// Returns 202 Accepted with job_id and status_url for polling.
pub async fn create_export_job(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    ctx: MemberContext,
    Json(request): Json<ExportReportRequest>,
) -> AppResult<(StatusCode, Json<ExportJobResponse>)> {
    // RBAC: Require Admin or Owner for report generation
    ctx.require_permission(Permission::ReportsExport)?;

    // Validate report type
    let report_type = match request.report_type.as_str() {
        "summary" => ReportType::Summary,
        "detail" => ReportType::Detail,
        _ => {
            return Err(AppError::Validation(
                "Invalid report_type. Must be 'summary' or 'detail'.".to_string(),
            ))
        }
    };

    // Validate date range
    if request.end_date < request.start_date {
        return Err(AppError::Validation(
            "end_date must be >= start_date".to_string(),
        ));
    }

    let date_diff = request
        .end_date
        .signed_duration_since(request.start_date)
        .num_days();
    if date_diff > 365 {
        return Err(AppError::Validation(
            "Date range cannot exceed 365 days".to_string(),
        ));
    }

    // Validate format
    if request.format != "xlsx" && request.format != "csv" {
        return Err(AppError::Validation(
            "Invalid format. Must be 'xlsx' or 'csv'.".to_string(),
        ));
    }

    // Get coordinator
    let coordinator = get_coordinator(&state)?;

    // Create job request
    let create_request = CreateJobRequest {
        report_type,
        start_date: request.start_date,
        end_date: request.end_date,
        employee_ids: request.employee_ids,
        sort: None,
    };

    // Create async job
    let result = coordinator
        .create_report_job(ws_id, ctx.user_id, create_request)
        .await?;

    tracing::info!(
        job_id = %result.job_id,
        workspace_id = %ws_id,
        user_id = %ctx.user_id,
        total_batches = result.total_batches,
        "Export job created"
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(ExportJobResponse {
            job_id: result.job_id,
            status: result.status,
            total_batches: result.total_batches,
            estimated_seconds: result.estimated_seconds,
            status_url: result.status_url,
        }),
    ))
}

/// GET /api/workspaces/:ws_id/reports/jobs/:job_id
///
/// Poll job status. Returns current progress and status.
pub async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<JobStatusResponse>> {
    // RBAC: Require Admin or Owner
    ctx.require_permission(Permission::ReportsExport)?;

    let coordinator = get_coordinator(&state)?;

    let status = coordinator.get_job_status(job_id).await?;

    // Verify job belongs to this workspace
    let redis = state
        .infra.redis
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Redis not available".to_string()))?;
    let queue = crate::queue::JobQueue::new(redis.clone());
    let metadata = queue
        .get_job_metadata(job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    if metadata.workspace_id != ws_id {
        return Err(AppError::Forbidden(
            "Job does not belong to this workspace".to_string(),
        ));
    }

    Ok(Json(JobStatusResponse {
        job_id: status.job_id,
        status: status.status,
        progress: status.progress.into(),
        error: status.error,
        created_at: status.created_at.to_rfc3339(),
        completed_at: status.completed_at.map(|t| t.to_rfc3339()),
    }))
}

/// GET /api/workspaces/:ws_id/reports/jobs/:job_id/download
///
/// Download completed report. Serves Excel file directly from Redis.
/// Returns 202 if job not yet complete, 410 if failed.
pub async fn download_report(
    State(state): State<Arc<AppState>>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    // RBAC: Require Admin or Owner
    ctx.require_permission(Permission::ReportsExport)?;

    let coordinator = get_coordinator(&state)?;

    // Get Redis pool for workspace validation
    let redis = state
        .infra.redis
        .as_ref()
        .ok_or_else(|| AppError::ServiceUnavailable("Redis not available".to_string()))?;

    // Verify job belongs to this workspace
    let queue = crate::queue::JobQueue::new(redis.clone());
    let metadata = queue
        .get_job_metadata(job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    if metadata.workspace_id != ws_id {
        return Err(AppError::Forbidden(
            "Job does not belong to this workspace".to_string(),
        ));
    }

    let status = coordinator.get_job_status(job_id).await?;

    match status.status.as_str() {
        "completed" => {
            // Fetch Excel data from Redis (queue already initialized above)
            let result = queue.get_result_data(job_id).await?;

            match result {
                Some((data, filename)) => {
                    let body = Body::from(data);
                    Ok((
                        [
                            (
                                header::CONTENT_TYPE,
                                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                            ),
                            (
                                header::CONTENT_DISPOSITION,
                                &format!("attachment; filename=\"{filename}\""),
                            ),
                        ],
                        body,
                    )
                        .into_response())
                }
                None => Err(AppError::NotFound(
                    "Report data expired or not available".to_string(),
                )),
            }
        }
        "failed" => Err(AppError::from(
            status.error.unwrap_or_else(|| "Job failed".to_string()),
        )),
        "pending" | "processing" => Ok((
            StatusCode::ACCEPTED,
            "Job not yet complete. Continue polling status endpoint.",
        )
            .into_response()),
        _ => Err(AppError::from(format!(
            "Unknown job status: {}",
            status.status
        ))),
    }
}

/// DELETE /api/workspaces/:ws_id/reports/jobs/:job_id
///
/// Cancel a pending or processing job.
/// Returns 204 No Content on success.
/// Returns 404 if job not found, 409 if already completed.
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    // RBAC: Require Admin or Owner
    ctx.require_permission(Permission::ReportsExport)?;

    let coordinator = get_coordinator(&state)?;

    // Verify job belongs to this workspace
    let queue = crate::queue::JobQueue::new(
        state
            .infra.redis
            .as_ref()
            .ok_or_else(|| AppError::ServiceUnavailable("Redis not available".to_string()))?
            .clone(),
    );
    let metadata = queue
        .get_job_metadata(job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    if metadata.workspace_id != ws_id {
        return Err(AppError::Forbidden(
            "Job does not belong to this workspace".to_string(),
        ));
    }

    coordinator.cancel_job(job_id).await?;

    tracing::info!(
        job_id = %job_id,
        user_id = %ctx.user_id,
        "Export job cancelled"
    );

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_format() {
        assert_eq!(default_format(), "xlsx");
    }

    #[test]
    fn test_job_progress_conversion() {
        let coordinator_progress = CoordinatorJobProgress {
            total_batches: 100,
            completed_batches: 50,
            failed_batches: 2,
            percentage: 50.0,
        };

        let api_progress: JobProgress = coordinator_progress.into();

        assert_eq!(api_progress.total_batches, 100);
        assert_eq!(api_progress.completed_batches, 50);
        assert_eq!(api_progress.failed_batches, 2);
        assert_eq!(api_progress.percentage, 50.0);
    }
}
