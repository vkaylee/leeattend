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
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::queue::ReportType;
use crate::services::report_coordinator::{
    CreateReportJobRequest, JobProgress as CoordinatorJobProgress, ReportCoordinator, ReportError,
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
    pub result_url: Option<String>,
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

/// Convert ReportError to HTTP status code and message.
fn map_report_error(err: ReportError) -> (StatusCode, String) {
    match err {
        ReportError::NoEmployees => (
            StatusCode::BAD_REQUEST,
            "No active employees found for report".to_string(),
        ),
        ReportError::JobNotFound => (StatusCode::NOT_FOUND, "Job not found".to_string()),
        ReportError::CannotCancel => (
            StatusCode::CONFLICT,
            "Cannot cancel completed or failed job".to_string(),
        ),
        ReportError::QueueError(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Queue error: {}", e),
        ),
        ReportError::DatabaseError(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        ),
    }
}

/// Get coordinator from state or return 503 Service Unavailable.
fn get_coordinator(state: &AppState) -> Result<&ReportCoordinator, (StatusCode, String)> {
    state.coordinator.as_ref().map(|c| c.as_ref()).ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Async export not available. Redis is required for distributed processing.".to_string(),
    ))
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
) -> Result<(StatusCode, Json<ExportJobResponse>), (StatusCode, String)> {
    // RBAC: Require Admin or Owner for report generation
    if !ctx.is_admin_or_owner() {
        return Err((
            StatusCode::FORBIDDEN,
            "Admin or Owner role required for report export".to_string(),
        ));
    }

    // Validate report type
    let report_type = match request.report_type.as_str() {
        "summary" => ReportType::Summary,
        "detail" => ReportType::Detail,
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid report_type. Must be 'summary' or 'detail'.".to_string(),
            ))
        }
    };

    // Validate date range
    if request.end_date < request.start_date {
        return Err((
            StatusCode::BAD_REQUEST,
            "end_date must be >= start_date".to_string(),
        ));
    }

    let date_diff = request
        .end_date
        .signed_duration_since(request.start_date)
        .num_days();
    if date_diff > 365 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Date range cannot exceed 365 days".to_string(),
        ));
    }

    // Validate format
    if request.format != "xlsx" && request.format != "csv" {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid format. Must be 'xlsx' or 'csv'.".to_string(),
        ));
    }

    // Get coordinator
    let coordinator = get_coordinator(&state)?;

    // Create job request
    let create_request = CreateReportJobRequest {
        report_type,
        start_date: request.start_date,
        end_date: request.end_date,
        employee_ids: request.employee_ids,
        sort: None,
    };

    // Create async job
    let result = coordinator
        .create_report_job(ws_id, ctx.user_id, create_request)
        .await
        .map_err(map_report_error)?;

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
/// Poll job status. Returns current progress, status, and result_url when complete.
pub async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<Json<JobStatusResponse>, (StatusCode, String)> {
    // RBAC: Require Admin or Owner
    if !ctx.is_admin_or_owner() {
        return Err((
            StatusCode::FORBIDDEN,
            "Admin or Owner role required".to_string(),
        ));
    }

    let coordinator = get_coordinator(&state)?;

    let status = coordinator
        .get_job_status(job_id)
        .await
        .map_err(map_report_error)?;

    Ok(Json(JobStatusResponse {
        job_id: status.job_id,
        status: status.status,
        progress: status.progress.into(),
        result_url: status.result_url,
        error: status.error,
        created_at: status.created_at.to_rfc3339(),
        completed_at: status.completed_at.map(|t| t.to_rfc3339()),
    }))
}

/// GET /api/workspaces/:ws_id/reports/jobs/:job_id/download
///
/// Download completed report. Redirects to S3 presigned URL.
/// Returns 202 if job not yet complete, 410 if failed.
pub async fn download_report(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // RBAC: Require Admin or Owner
    if !ctx.is_admin_or_owner() {
        return Err((
            StatusCode::FORBIDDEN,
            "Admin or Owner role required".to_string(),
        ));
    }

    let coordinator = get_coordinator(&state)?;

    let status = coordinator
        .get_job_status(job_id)
        .await
        .map_err(map_report_error)?;

    match status.status.as_str() {
        "completed" => {
            if let Some(url) = status.result_url {
                Ok(Redirect::temporary(&url).into_response())
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Result URL not available".to_string(),
                ))
            }
        }
        "failed" => Err((
            StatusCode::GONE,
            status.error.unwrap_or_else(|| "Job failed".to_string()),
        )),
        "pending" | "processing" => Err((
            StatusCode::ACCEPTED,
            "Job not yet complete. Continue polling status endpoint.".to_string(),
        )),
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unknown job status: {}", status.status),
        )),
    }
}

/// DELETE /api/workspaces/:ws_id/reports/jobs/:job_id
///
/// Cancel a pending or processing job.
/// Returns 204 No Content on success.
/// Returns 404 if job not found, 409 if already completed.
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, job_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> Result<StatusCode, (StatusCode, String)> {
    // RBAC: Require Admin or Owner
    if !ctx.is_admin_or_owner() {
        return Err((
            StatusCode::FORBIDDEN,
            "Admin or Owner role required".to_string(),
        ));
    }

    let coordinator = get_coordinator(&state)?;

    coordinator
        .cancel_job(job_id)
        .await
        .map_err(map_report_error)?;

    tracing::info!(
        job_id = %job_id,
        user_id = %ctx.user_id,
        "Export job cancelled"
    );

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/reports/local/*key
///
/// Serve locally stored report files.
/// Only used when storage is configured for local filesystem.
///
/// Security:
/// - Requires authentication via MemberContext
/// - Workspace access validation from key path
/// - Path traversal protection via canonicalization
pub async fn serve_local_report(
    State(state): State<Arc<AppState>>,
    Path(key): Path<String>,
    ctx: MemberContext,
) -> Result<Response, (StatusCode, String)> {
    // 1. Extract workspace_id from key (format: workspace_id/year/month/filename)
    let workspace_id = key
        .split('/')
        .next()
        .and_then(|id_str| Uuid::parse_str(id_str).ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid key format: expected workspace_id/year/month/filename".to_string(),
            )
        })?;

    // 2. RBAC: Verify user has access to this workspace
    if ctx.workspace_id != workspace_id {
        tracing::warn!(
            user_id = %ctx.user_id,
            requested_workspace = %workspace_id,
            user_workspace = %ctx.workspace_id,
            "Unauthorized workspace access attempt"
        );
        return Err((
            StatusCode::FORBIDDEN,
            "Access denied: workspace mismatch".to_string(),
        ));
    }

    // 3. Get local storage base path
    let base_path = match &state.storage {
        crate::StorageBackend::Local { base_path } => base_path.join("reports"),
        _ => {
            return Err((
                StatusCode::NOT_FOUND,
                "Local storage not configured".to_string(),
            ))
        }
    };

    // 4. Build and validate file path
    let file_path = base_path.join(&key);

    // 5. Security: Prevent path traversal
    let canonical_base = match tokio::fs::canonicalize(&base_path).await {
        Ok(p) => p,
        Err(_) => {
            tracing::error!(base = %base_path.display(), "Base storage path does not exist");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Storage not properly configured".to_string(),
            ));
        }
    };

    let canonical_file = match tokio::fs::canonicalize(&file_path).await {
        Ok(p) => p,
        Err(_) => {
            return Err((StatusCode::NOT_FOUND, "Report not found".to_string()));
        }
    };

    if !canonical_file.starts_with(&canonical_base) {
        tracing::warn!(
            requested = %key,
            user_id = %ctx.user_id,
            "Path traversal attempt blocked"
        );
        return Err((StatusCode::FORBIDDEN, "Access denied".to_string()));
    }

    // 6. Open file
    let file = File::open(&canonical_file).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            (StatusCode::NOT_FOUND, "Report not found".to_string())
        } else {
            tracing::error!(error = %e, path = %canonical_file.display(), "Failed to open report file");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to read report".to_string(),
            )
        }
    })?;

    // 7. Get file metadata for Content-Length
    let metadata = file.metadata().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to get file metadata");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read report".to_string(),
        )
    })?;

    // 8. Extract filename for Content-Disposition
    let filename = canonical_file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("report.xlsx");

    // 9. Stream the file
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // 10. Build response with headers
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
            .unwrap_or_else(|_| HeaderValue::from_static("attachment; filename=\"report.xlsx\"")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&metadata.len().to_string())
            .unwrap_or_else(|_| HeaderValue::from_static("0")),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store, must-revalidate"),
    );

    tracing::info!(
        key = %key,
        size = metadata.len(),
        "Serving local report file"
    );

    let mut response = Response::new(body);
    *response.headers_mut() = headers;
    Ok(response)
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
