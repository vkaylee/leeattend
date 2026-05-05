use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct AttendanceLogQuery {
    pub employee_id: Option<Uuid>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_attendance_logs(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let logs = state
        .attendance
        .time_entry_repo
        .get_paginated_logs(
            workspace_id,
            crate::models::TimeEntryLogFilter {
                employee_id: query.employee_id,
                from: query.from,
                to: query.to,
                limit,
                offset,
            },
        )
        .await?;

    let total = state
        .attendance
        .time_entry_repo
        .count_logs(workspace_id, query.employee_id, query.from, query.to)
        .await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

pub async fn get_raw_attendance_logs(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<AttendanceLogQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let logs = state
        .attendance
        .raw_log_repo
        .get_paginated_raw_logs(
            workspace_id,
            query.employee_id,
            query.from,
            query.to,
            limit,
            offset,
        )
        .await?;

    let total = state
        .attendance
        .raw_log_repo
        .count_raw_logs(workspace_id, query.employee_id, query.from, query.to)
        .await?;

    Ok(Json(serde_json::json!({
        "data": logs,
        "total": total,
        "page": page,
        "limit": limit,
    })))
}

#[derive(Deserialize)]
pub struct ReprocessQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

pub async fn force_reprocess_attendance(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::AttendanceReprocess)?;

    const MAX_REPROCESS_DAYS: i64 = 90;

    let date_range = (query.end_date - query.start_date).num_days();
    if date_range < 0 {
        return Err(AppError::Validation(
            "end_date must be after start_date".to_string(),
        ));
    }
    if date_range > MAX_REPROCESS_DAYS {
        return Err(AppError::Validation(format!(
            "Max reprocess range is {MAX_REPROCESS_DAYS} days"
        )));
    }

    let workspace = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.infra.db.api().clone(),
    }
    .get_workspace_by_id(ctx.workspace_id)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound("Workspace not found".to_string()))?;

    let tz: chrono_tz::Tz = workspace
        .timezone
        .parse()
        .map_err(|_e| AppError::from("Invalid timezone configuration"))?;

    let mut curr_date = query.start_date;
    while curr_date <= query.end_date {
        state
            .attendance
            .pairing_service
            .reprocess_daily_attendance(ctx.workspace_id, employee_id, curr_date, tz)
            .await
            .map_err(|e| AppError::from(format!("Processing failed for {curr_date}: {e}")))?;

        if let Some(next) = curr_date.succ_opt() {
            curr_date = next;
        } else {
            break;
        }
    }

    Ok(StatusCode::OK)
}

pub async fn force_reprocess_all_attendance(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ReprocessQuery>,
) -> AppResult<Json<serde_json::Value>> {
    ctx.require_permission(Permission::AttendanceReprocess)?;

    const MAX_REPROCESS_DAYS: i64 = 90;

    let date_range = (query.end_date - query.start_date).num_days();
    if date_range < 0 {
        return Err(AppError::Validation(
            "end_date must be after start_date".to_string(),
        ));
    }
    if date_range > MAX_REPROCESS_DAYS {
        return Err(AppError::Validation(format!(
            "Max reprocess range is {MAX_REPROCESS_DAYS} days"
        )));
    }

    let redis = state.infra.redis.as_ref().ok_or_else(|| {
        AppError::Internal("Redis is required for background processing".to_string())
    })?;

    tracing::info!("Using Redis for enqueuing jobs");

    let queue = crate::queue::JobQueue::new(redis.clone());

    // Fetch all active employees
    let employees = sqlx::query!(
        "SELECT id FROM employees WHERE workspace_id = $1 AND is_active = true",
        ctx.workspace_id
    )
    .fetch_all(state.infra.db.api())
    .await
    .map_err(AppError::from)?;

    tracing::info!(
        employee_count = employees.len(),
        workspace_id = %ctx.workspace_id,
        employee_ids = ?employees.iter().map(|e| e.id).collect::<Vec<_>>(),
        "Found employees for bulk reprocess"
    );

    if employees.is_empty() {
        return Err(AppError::Validation(
            "No active employees found".to_string(),
        ));
    }

    let parent_job_id = crate::utils::id::new_id();
    let total_batches = employees.len() as u32;

    tracing::info!(
        workspace_id = %ctx.workspace_id,
        employee_count = total_batches,
        job_id = %parent_job_id,
        "Enqueuing bulk reprocess jobs"
    );

    let meta = crate::queue::JobMetadata::with_date_range(
        parent_job_id,
        total_batches,
        ctx.workspace_id,
        ctx.user_id,
        None, // No report type
        crate::queue::DateRange {
            start: query.start_date,
            end: query.end_date,
        },
    );

    queue
        .create_job_metadata(&meta)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create job metadata: {e}")))?;

    let jobs: Vec<crate::queue::Job> = employees
        .into_iter()
        .enumerate()
        .map(|(idx, emp)| {
            crate::queue::Job::with_priority(
                parent_job_id,
                idx as u32,
                crate::queue::TaskType::ReprocessAttendance,
                ctx.workspace_id,
                crate::queue::JobPayload::ReprocessAttendance(crate::queue::ReprocessPayload {
                    employee_id: emp.id,
                    date_range: crate::queue::DateRange {
                        start: query.start_date,
                        end: query.end_date,
                    },
                }),
                crate::queue::JobPriority::Normal,
            )
        })
        .collect();

    tracing::debug!(job_count = jobs.len(), "Enqueuing jobs to Redis");

    queue
        .enqueue_jobs(jobs)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to enqueue jobs: {e}")))?;

    tracing::info!(job_id = %parent_job_id, "Bulk reprocess jobs enqueued successfully");

    Ok(Json(serde_json::json!({
        "message": "Reprocess jobs queued successfully",
        "job_id": parent_job_id
    })))
}
