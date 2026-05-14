use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::AttendanceDailySummary;
use crate::AppState;

#[derive(Deserialize)]
pub struct ClaimOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: Option<String>,
}

pub async fn claim_ot(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<ClaimOtRequest>,
) -> AppResult<StatusCode> {
    // 1. Verify ownership (Employee can only claim for themselves) or has OvertimeClaim permission
    let employee = state
        .employees
        .repo
        .get_employee_by_id(employee_id, ctx.workspace_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Employee not found".to_string()))?;

    if let Some(uid) = employee.user_id {
        if uid != ctx.user_id {
            // Not self → need OvertimeApprove permission
            ctx.require_permission(Permission::OvertimeApprove)?;
        }
    } else {
        ctx.require_permission(Permission::OvertimeApprove)?;
    }

    state
        .attendance
        .summary_service
        .claim_ot(
            ctx.workspace_id,
            employee_id,
            req.date,
            req.claimed_minutes,
            req.reason,
        )
        .await
        .map_err(|e| AppError::Validation(format!("Claim OT Error: {e}")))?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct ApproveOtRequest {
    pub date: NaiveDate,
    pub approved_minutes: i32,
    pub manager_note: Option<String>,
    pub status: String, // "approved" or "rejected"
}

pub async fn approve_ot(
    State(state): State<Arc<AppState>>,
    Path((_, employee_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<ApproveOtRequest>,
) -> AppResult<StatusCode> {
    // RBAC: Only users with OvertimeApprove permission, scoped to department if TeamLead
    let target_dept_id =
        crate::handlers::common::current_department_id(&state, ctx.workspace_id, employee_id).await;

    ctx.require_scoped_permission(Permission::OvertimeApprove, target_dept_id)?;

    let status_enum = match req.status.as_str() {
        "approved" => crate::models::OtStatus::Approved,
        "rejected" => crate::models::OtStatus::Rejected,
        _ => return Err(AppError::Validation("Invalid status".to_string())),
    };

    state
        .attendance
        .summary_service
        .approve_ot(
            ctx.workspace_id,
            employee_id,
            req.date,
            req.approved_minutes,
            req.manager_note,
            status_enum,
        )
        .await
        .map_err(|e| AppError::Validation(format!("Approve OT Error: {e}")))?;

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct SummaryQuery {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub employee_id: Option<Uuid>,
}

pub async fn get_attendance_summaries(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<SummaryQuery>,
) -> AppResult<Json<Vec<AttendanceDailySummary>>> {
    // RBAC: if user has AttendanceSummariesRead, they can query any employee
    // Otherwise, they can only query their own data
    let has_read_perm = ctx.has_permission(Permission::AttendanceSummariesRead);

    let target_employee_id = if has_read_perm {
        query.employee_id
    } else {
        let my_id = state
            .employees
            .repo
            .get_employee_id_by_user_id(ctx.workspace_id, ctx.user_id)
            .await?
            .ok_or_else(|| AppError::Forbidden("Forbidden".to_string()))?;

        if let Some(req_id) = query.employee_id {
            if req_id != my_id {
                return Err(AppError::Forbidden("Forbidden".to_string()));
            }
        }
        Some(my_id)
    };

    let summaries = state
        .attendance
        .attendance_daily_summary_repo
        .get_summaries(
            ctx.workspace_id,
            target_employee_id,
            query.start_date,
            query.end_date,
        )
        .await?;

    Ok(Json(summaries))
}
