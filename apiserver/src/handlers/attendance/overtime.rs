use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{AttendanceDailySummary, MemberRole};
use crate::repositories::workspace_repository::WorkspaceRepository;
use crate::AppState;

#[derive(Deserialize)]
pub struct ClaimOtRequest {
    pub date: NaiveDate,
    pub claimed_minutes: i32,
    pub reason: Option<String>,
}

pub async fn claim_ot(
    State(state): State<Arc<AppState>>,
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    MemberContext { user_id, .. }: MemberContext,
    Json(req): Json<ClaimOtRequest>,
) -> AppResult<StatusCode> {
    // 1. Verify ownership (Employee can only claim for themselves) or Admin
    let employee =
        crate::repositories::employee_repository::get_employee_by_id(state.db.api(), employee_id, ws_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Employee not found".to_string()))?;

    if let Some(uid) = employee.user_id {
        if uid != user_id {
            // If not the user, check if Admin/Owner/Manager?
            let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
                pool: state.db.api().clone(),
            }
            .find_primary_role(ws_id, user_id)
            .await
            .unwrap_or(None);

            if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
                return Err(AppError::Forbidden("Forbidden".to_string()));
            }
        }
    } else {
        let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
            pool: state.db.api().clone(),
        }
        .find_primary_role(ws_id, user_id)
        .await
        .unwrap_or(None);

        if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
            return Err(AppError::Forbidden("Forbidden".to_string()));
        }
    }

    crate::services::attendance_summary_service::AttendanceSummaryService::claim_ot(
        state.db.api(),
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
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    MemberContext { user_id, .. }: MemberContext,
    Json(req): Json<ApproveOtRequest>,
) -> AppResult<StatusCode> {
    // RBAC: Only Admin/Owner/Manager
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.api().clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await
    .unwrap_or(None);

    if !matches!(role, Some(MemberRole::Admin) | Some(MemberRole::Owner)) {
        return Err(AppError::Forbidden("Forbidden".to_string()));
    }

    let status_enum = match req.status.as_str() {
        "approved" => crate::models::OtStatus::Approved,
        "rejected" => crate::models::OtStatus::Rejected,
        _ => return Err(AppError::Validation("Invalid status".to_string())),
    };

    crate::services::attendance_summary_service::AttendanceSummaryService::approve_ot(
        state.db.api(),
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
    Path(ws_id): Path<Uuid>,
    MemberContext { user_id, .. }: MemberContext,
    Query(query): Query<SummaryQuery>,
) -> AppResult<Json<Vec<AttendanceDailySummary>>> {
    // RBAC and ID Resolution
    let role = crate::repositories::workspace_repository::SqlxWorkspaceRepository {
        pool: state.db.api().clone(),
    }
    .find_primary_role(ws_id, user_id)
    .await?
    .ok_or_else(|| AppError::Forbidden("Forbidden".to_string()))?;

    let is_admin = matches!(role, MemberRole::Admin | MemberRole::Owner);

    let target_employee_id = if is_admin {
        query.employee_id
    } else {
        // Find my employee ID
        let emp_record = sqlx::query!(
            "SELECT id FROM employees WHERE workspace_id = $1 AND user_id = $2",
            ws_id,
            user_id
        )
        .fetch_optional(state.db.api())
        .await?;

        let my_id = emp_record
            .ok_or_else(|| AppError::Forbidden("Forbidden".to_string()))?
            .id;

        if let Some(req_id) = query.employee_id {
            if req_id != my_id {
                return Err(AppError::Forbidden("Forbidden".to_string()));
            }
        }
        Some(my_id)
    };

    let summaries = if let Some(emp_id) = target_employee_id {
        sqlx::query_as!(
            crate::models::AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: crate::models::OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at FROM attendance_daily_summaries WHERE workspace_id = $1 AND employee_id = $2 AND date >= $3 AND date <= $4 ORDER BY date ASC",
            ws_id, emp_id, query.start_date, query.end_date
        )
        .fetch_all(state.db.api())
        .await?
    } else {
        sqlx::query_as!(
            crate::models::AttendanceDailySummary,
            "SELECT id, workspace_id, employee_id, date, actual_work_minutes, system_detected_ot_minutes, claimed_ot_minutes, approved_ot_minutes, ot_status as \"ot_status: crate::models::OtStatus\", ot_reason, manager_note, claim_entered_by_user_id, claim_entered_at, paper_form_photo_url, entered_via, is_holiday, holiday_name, created_at, updated_at FROM attendance_daily_summaries WHERE workspace_id = $1 AND date >= $2 AND date <= $3 ORDER BY date ASC, employee_id ASC",
            ws_id, query.start_date, query.end_date
        )
        .fetch_all(state.db.api())
        .await?
    };

    Ok(Json(summaries))
}
