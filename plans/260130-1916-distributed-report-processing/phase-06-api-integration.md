# Phase 6: API Integration

## Context Links
- [Plan Overview](./plan.md)
- [Phase 2: Coordinator](./phase-02-coordinator-service.md)
- [Phase 5: Result Aggregation](./phase-05-result-aggregation.md)
- [Current Handlers](../../apiserver/src/handlers/attendance.rs)

## Overview
- **Priority**: P1 (Core)
- **Status**: pending
- **Effort**: 3h
- **Description**: Create async API endpoints for job submission, status polling, and result download.

## Key Insights

- Job submission returns immediately with job_id (async pattern)
- Client polls status endpoint until completion
- Download redirects to S3 presigned URL
- Keep existing sync endpoints for small reports

## Requirements

### Functional
- F1: POST endpoint to create async export job
- F2: GET endpoint to poll job status
- F3: GET endpoint to download completed report
- F4: DELETE endpoint to cancel pending job
- F5: Fallback to sync for small datasets (< 100 employees)

### Non-Functional
- NF1: Job submission < 500ms response time
- NF2: Status polling < 50ms response time
- NF3: Rate limit: 5 concurrent jobs per workspace
- NF4: WebSocket option for real-time progress (optional)

## Architecture

### API Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              API ENDPOINTS                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  POST /api/workspaces/:ws_id/reports/export                          │    │
│  │                                                                       │    │
│  │  Request:                                                             │    │
│  │  {                                                                    │    │
│  │    "report_type": "summary" | "detail",                              │    │
│  │    "start_date": "2026-01-01",                                       │    │
│  │    "end_date": "2026-01-31",                                         │    │
│  │    "employee_ids": [...] | null,                                      │    │
│  │    "format": "xlsx" | "csv"                                          │    │
│  │  }                                                                    │    │
│  │                                                                       │    │
│  │  Response (202 Accepted):                                             │    │
│  │  {                                                                    │    │
│  │    "job_id": "uuid",                                                  │    │
│  │    "status": "pending",                                               │    │
│  │    "total_batches": 350,                                              │    │
│  │    "estimated_seconds": 120,                                          │    │
│  │    "status_url": "/api/workspaces/{ws}/reports/jobs/{job_id}"        │    │
│  │  }                                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  GET /api/workspaces/:ws_id/reports/jobs/:job_id                     │    │
│  │                                                                       │    │
│  │  Response (200 OK):                                                   │    │
│  │  {                                                                    │    │
│  │    "job_id": "uuid",                                                  │    │
│  │    "status": "processing" | "completed" | "failed",                  │    │
│  │    "progress": {                                                      │    │
│  │      "total_batches": 350,                                           │    │
│  │      "completed_batches": 175,                                        │    │
│  │      "percentage": 50.0                                              │    │
│  │    },                                                                 │    │
│  │    "result_url": "https://s3.../report.xlsx" | null,                 │    │
│  │    "error": null | "Error message"                                   │    │
│  │  }                                                                    │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  GET /api/workspaces/:ws_id/reports/jobs/:job_id/download            │    │
│  │                                                                       │    │
│  │  Response (302 Redirect) to S3 presigned URL                         │    │
│  │  OR 200 with direct file stream if small                             │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │  DELETE /api/workspaces/:ws_id/reports/jobs/:job_id                  │    │
│  │                                                                       │    │
│  │  Response (204 No Content) - Job cancelled                           │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Polling Pattern (Client Side)

```javascript
// Client-side polling example
async function exportReport(params) {
  // 1. Start job
  const { job_id, status_url, estimated_seconds } = await fetch('/api/.../export', {
    method: 'POST',
    body: JSON.stringify(params)
  }).then(r => r.json());

  // 2. Poll with exponential backoff
  let delay = 1000; // Start at 1 second
  const maxDelay = 10000; // Max 10 seconds

  while (true) {
    await sleep(delay);

    const status = await fetch(status_url).then(r => r.json());

    if (status.status === 'completed') {
      return status.result_url;
    }

    if (status.status === 'failed') {
      throw new Error(status.error);
    }

    // Increase delay with cap
    delay = Math.min(delay * 1.5, maxDelay);
  }
}
```

## Related Code Files

### Files to Create
- `apiserver/src/handlers/report_export.rs` - Export API handlers

### Files to Modify
- `apiserver/src/handlers/mod.rs` - Export new handlers
- `apiserver/src/lib.rs` - Add routes
- `apiserver/src/AppState` - Add coordinator reference

## Implementation Steps

### Step 1: Define Request/Response Types

```rust
// handlers/report_export.rs
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect},
    Json,
};
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use uuid::Uuid;
use std::sync::Arc;

use crate::AppState;
use crate::services::{ReportCoordinator, CreateReportJobRequest, ReportType};

#[derive(Debug, Deserialize)]
pub struct ExportReportRequest {
    pub report_type: String,  // "summary" | "detail"
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    #[serde(default)]
    pub employee_ids: Option<Vec<Uuid>>,
    #[serde(default = "default_format")]
    pub format: String,  // "xlsx" | "csv"
}

fn default_format() -> String {
    "xlsx".to_string()
}

#[derive(Debug, Serialize)]
pub struct ExportJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub total_batches: u32,
    pub estimated_seconds: u64,
    pub status_url: String,
}

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

#[derive(Debug, Serialize)]
pub struct JobProgress {
    pub total_batches: u32,
    pub completed_batches: u32,
    pub failed_batches: u32,
    pub percentage: f32,
}
```

### Step 2: Implement Export Handler

```rust
/// POST /api/workspaces/:ws_id/reports/export
pub async fn create_export_job(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    user_id: UserId,
    Json(request): Json<ExportReportRequest>,
) -> Result<(StatusCode, Json<ExportJobResponse>), (StatusCode, String)> {
    // Validate report type
    let report_type = match request.report_type.as_str() {
        "summary" => ReportType::Summary,
        "detail" => ReportType::Detail,
        _ => return Err((StatusCode::BAD_REQUEST, "Invalid report_type".to_string())),
    };

    // Check if Redis is available
    let coordinator = state.coordinator.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Async export not available".to_string()))?;

    // Validate date range
    if request.end_date < request.start_date {
        return Err((StatusCode::BAD_REQUEST, "end_date must be >= start_date".to_string()));
    }

    let date_diff = request.end_date.signed_duration_since(request.start_date).num_days();
    if date_diff > 365 {
        return Err((StatusCode::BAD_REQUEST, "Date range cannot exceed 365 days".to_string()));
    }

    // Create job
    let create_request = CreateReportJobRequest {
        report_type,
        start_date: request.start_date,
        end_date: request.end_date,
        employee_ids: request.employee_ids,
        sort: None,
    };

    let result = coordinator.create_report_job(ws_id, user_id.0, create_request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ExportJobResponse {
            job_id: result.job_id,
            status: result.status,
            total_batches: result.total_batches,
            estimated_seconds: result.estimated_seconds,
            status_url: result.status_url,
        })
    ))
}
```

### Step 3: Implement Status Handler

```rust
/// GET /api/workspaces/:ws_id/reports/jobs/:job_id
pub async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
    _user_id: UserId,
) -> Result<Json<JobStatusResponse>, (StatusCode, String)> {
    let coordinator = state.coordinator.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Service not available".to_string()))?;

    let status = coordinator.get_job_status(job_id).await
        .map_err(|e| match e {
            ReportError::JobNotFound => (StatusCode::NOT_FOUND, "Job not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)),
        })?;

    Ok(Json(JobStatusResponse {
        job_id: status.job_id,
        status: status.status,
        progress: JobProgress {
            total_batches: status.progress.total_batches,
            completed_batches: status.progress.completed_batches,
            failed_batches: status.progress.failed_batches,
            percentage: status.progress.percentage,
        },
        result_url: status.result_url,
        error: status.error,
        created_at: status.created_at.to_rfc3339(),
        completed_at: status.completed_at.map(|t| t.to_rfc3339()),
    }))
}
```

### Step 4: Implement Download Handler

```rust
/// GET /api/workspaces/:ws_id/reports/jobs/:job_id/download
pub async fn download_report(
    State(state): State<Arc<AppState>>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
    _user_id: UserId,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let coordinator = state.coordinator.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Service not available".to_string()))?;

    let status = coordinator.get_job_status(job_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;

    match status.status.as_str() {
        "completed" => {
            if let Some(url) = status.result_url {
                Ok(Redirect::temporary(&url).into_response())
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Result URL not found".to_string()))
            }
        }
        "failed" => {
            Err((StatusCode::GONE, status.error.unwrap_or_else(|| "Job failed".to_string())))
        }
        "pending" | "processing" => {
            Err((StatusCode::ACCEPTED, "Job not yet complete".to_string()))
        }
        _ => {
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Unknown job status".to_string()))
        }
    }
}
```

### Step 5: Implement Cancel Handler

```rust
/// DELETE /api/workspaces/:ws_id/reports/jobs/:job_id
pub async fn cancel_job(
    State(state): State<Arc<AppState>>,
    Path((ws_id, job_id)): Path<(Uuid, Uuid)>,
    _user_id: UserId,
) -> Result<StatusCode, (StatusCode, String)> {
    let coordinator = state.coordinator.as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Service not available".to_string()))?;

    coordinator.cancel_job(job_id).await
        .map_err(|e| match e {
            ReportError::JobNotFound => (StatusCode::NOT_FOUND, "Job not found".to_string()),
            ReportError::CannotCancel => (StatusCode::CONFLICT, "Cannot cancel completed job".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)),
        })?;

    Ok(StatusCode::NO_CONTENT)
}
```

### Step 6: Add Routes

```rust
// lib.rs additions
.route(
    "/workspaces/:ws_id/reports/export",
    post(handlers::report_export::create_export_job),
)
.route(
    "/workspaces/:ws_id/reports/jobs/:job_id",
    get(handlers::report_export::get_job_status)
        .delete(handlers::report_export::cancel_job),
)
.route(
    "/workspaces/:ws_id/reports/jobs/:job_id/download",
    get(handlers::report_export::download_report),
)
```

### Step 7: Update AppState

```rust
// lib.rs
pub struct AppState {
    pub agent_manager: AgentManager,
    pub db: PgPool,
    pub agent_repo: Arc<dyn AgentRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub agent_service: Arc<AgentService>,
    pub jwt_secret: String,
    pub attendance_service: Arc<AttendanceLogService>,
    // NEW
    pub coordinator: Option<Arc<ReportCoordinator>>,
}
```

### Step 8: Small Report Fallback (Optional)

```rust
/// Smart routing: sync for small, async for large
pub async fn create_export_job(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<Uuid>,
    user_id: UserId,
    Json(request): Json<ExportReportRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Count employees for smart routing
    let employee_count = if let Some(ref ids) = request.employee_ids {
        ids.len()
    } else {
        employee_repository::count_active_employees(&state.db, ws_id)
            .await
            .unwrap_or(0) as usize
    };

    const SYNC_THRESHOLD: usize = 100;

    if employee_count <= SYNC_THRESHOLD {
        // Use synchronous generation for small reports
        let result = ReportService::generate_sync_report(
            &state.db,
            ws_id,
            &request,
        ).await?;

        // Return file directly
        let headers = [
            (header::CONTENT_TYPE, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
            (header::CONTENT_DISPOSITION, "attachment; filename=\"report.xlsx\""),
        ];

        return Ok((StatusCode::OK, headers, result).into_response());
    }

    // Large report - use async processing
    // ... (existing async code)
}
```

## Todo List

- [ ] Create `handlers/report_export.rs`
- [ ] Implement `create_export_job` handler
- [ ] Implement `get_job_status` handler
- [ ] Implement `download_report` handler with redirect
- [ ] Implement `cancel_job` handler
- [ ] Add routes to Router
- [ ] Update AppState with coordinator
- [ ] Add request validation (date range, etc.)
- [ ] Add rate limiting per workspace
- [ ] Write API integration tests

## Success Criteria

- [ ] Job submission returns 202 Accepted with job_id
- [ ] Status polling returns current progress
- [ ] Download redirects to S3 URL when complete
- [ ] Cancel stops pending/processing jobs
- [ ] Small reports use sync path (< 100 employees)

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Client abandons poll | Jobs have TTL, auto-cleanup |
| Rate limit bypass | Redis-based rate counter per workspace |
| Invalid presigned URL | Validate URL before redirect |
| Large response on status | Keep status payload minimal |

## Security Considerations

- Validate user has access to workspace before any operation
- Validate user owns the job (or is admin) for status/cancel
- Don't expose internal error details in responses
- Rate limit to prevent DoS via job creation

## Next Steps

After this phase, the system is complete. Consider:
1. Add WebSocket for real-time progress updates
2. Add job history endpoint
3. Add admin dashboard for monitoring all jobs
4. Add metrics (Prometheus) for observability
