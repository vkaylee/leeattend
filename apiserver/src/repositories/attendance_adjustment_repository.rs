use crate::errors::AppError;
use crate::models::{AdjustmentStatus, AttendanceAdjustmentRequest};
use async_trait::async_trait;
use sqlx::{query, query_as, PgPool};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: AttendanceAdjustmentRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait AttendanceAdjustmentRepository: Send + Sync {
    async fn create_request(&self, request: &AttendanceAdjustmentRequest) -> Result<(), AppError>;
    async fn get_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<AttendanceAdjustmentRequest>, AppError>;
    async fn update_adjustment_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: AdjustmentStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError>;
    async fn get_workspace_adjustment_requests(
        &self,
        workspace_id: Uuid,
        status: Option<AdjustmentStatus>,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError>;
    async fn get_employee_adjustment_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<AttendanceAdjustmentRequest>, AppError>;
}

pub async fn create_request(
    pool: &PgPool,
    request: &AttendanceAdjustmentRequest,
) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO attendance_adjustment_requests (id, employee_id, time_entry_id, requested_punch_in_time, requested_punch_out_time, is_break_overtime, reason, status, requested_by_user_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        request.id,
        request.employee_id,
        request.time_entry_id,
        request.requested_punch_in_time,
        request.requested_punch_out_time,
        request.is_break_overtime,
        request.reason,
        request.status as AdjustmentStatus,
        request.requested_by_user_id,
        request.created_at,
        request.updated_at,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_request_by_id(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
) -> Result<Option<AttendanceAdjustmentRequest>, sqlx::Error> {
    query_as!(
        AttendanceAdjustmentRequest,
        "SELECT r.id, r.employee_id, r.time_entry_id, r.requested_punch_in_time as \"requested_punch_in_time!\", r.requested_punch_out_time, r.is_break_overtime, r.reason, r.status as \"status: AdjustmentStatus\", r.requested_by_user_id, r.approved_by_user_id, r.created_at as \"created_at!\", r.updated_at as \"updated_at!\" FROM attendance_adjustment_requests r JOIN employees e ON r.employee_id = e.id WHERE r.id = $1 AND e.workspace_id = $2",
        id,
        workspace_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_adjustment_status(
    pool: &PgPool,
    workspace_id: Uuid,
    id: Uuid,
    status: AdjustmentStatus,
    approved_by: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    query!(
        "UPDATE attendance_adjustment_requests SET status = $1, approved_by_user_id = $2, updated_at = NOW() WHERE id = $3 AND employee_id IN (SELECT id FROM employees WHERE workspace_id = $4)",
        status as AdjustmentStatus,
        approved_by,
        id,
        workspace_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_workspace_adjustment_requests(
    pool: &PgPool,
    workspace_id: Uuid,
    status: Option<AdjustmentStatus>,
) -> Result<Vec<AttendanceAdjustmentRequest>, sqlx::Error> {
    let mut query_str = String::from("SELECT r.id, r.employee_id, r.time_entry_id, r.requested_punch_in_time, r.requested_punch_out_time, r.is_break_overtime, r.reason, r.status, r.requested_by_user_id, r.approved_by_user_id, r.created_at, r.updated_at FROM attendance_adjustment_requests r JOIN employees e ON r.employee_id = e.id WHERE e.workspace_id = $1");

    if status.is_some() {
        query_str.push_str(" AND r.status = $2");
    }
    query_str.push_str(" ORDER BY r.created_at DESC");

    if let Some(s) = status {
        query_as::<_, AttendanceAdjustmentRequest>(&query_str)
            .bind(workspace_id)
            .bind(s)
            .fetch_all(pool)
            .await
    } else {
        query_as::<_, AttendanceAdjustmentRequest>(&query_str)
            .bind(workspace_id)
            .fetch_all(pool)
            .await
    }
}

pub async fn get_employee_adjustment_requests(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Vec<AttendanceAdjustmentRequest>, sqlx::Error> {
    query_as!(
        AttendanceAdjustmentRequest,
        "SELECT r.id, r.employee_id, r.time_entry_id, r.requested_punch_in_time as \"requested_punch_in_time!\", r.requested_punch_out_time, r.is_break_overtime, r.reason, r.status as \"status: AdjustmentStatus\", r.requested_by_user_id, r.approved_by_user_id, r.created_at as \"created_at!\", r.updated_at as \"updated_at!\" FROM attendance_adjustment_requests r JOIN employees e ON r.employee_id = e.id WHERE r.employee_id = $1 AND e.workspace_id = $2 ORDER BY r.created_at DESC",
        employee_id,
        workspace_id
    )
    .fetch_all(pool)
    .await
}
