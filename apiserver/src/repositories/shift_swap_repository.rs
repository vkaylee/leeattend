use crate::models::shift_swap::{ShiftSwapRequestWithNames, ShiftSwapRequest, ShiftSwapStatus};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_swap_request(
    pool: &PgPool,
    request: &ShiftSwapRequest,
) -> Result<ShiftSwapRequest, sqlx::Error> {
    sqlx::query_as::<_, ShiftSwapRequest>(
        r#"
        INSERT INTO shift_swap_requests (
            id, workspace_id, requester_id, target_id, swap_date,
            requester_shift_id, target_shift_id,
            status, reason, requested_by_user_id,
            created_at, updated_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        RETURNING *
        "#,
    )
    .bind(request.id)
    .bind(request.workspace_id)
    .bind(request.requester_id)
    .bind(request.target_id)
    .bind(request.swap_date)
    .bind(request.requester_shift_id)
    .bind(request.target_shift_id)
    .bind(request.status)
    .bind(request.reason.as_deref())
    .bind(request.requested_by_user_id)
    .bind(request.created_at)
    .bind(request.updated_at)
    .fetch_one(pool)
    .await
}

pub async fn get_swap_request_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<ShiftSwapRequest>, sqlx::Error> {
    sqlx::query_as::<_, ShiftSwapRequest>(
        "SELECT * FROM shift_swap_requests WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_workspace_swaps(
    pool: &PgPool,
    workspace_id: Uuid,
    status_filter: Option<ShiftSwapStatus>,
) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ShiftSwapRequestWithNames>(
        r#"
        SELECT
            s.*,
            req.full_name AS requester_name,
            tgt.full_name AS target_name,
            req_shift.name AS requester_shift_name,
            tgt_shift.name AS target_shift_name
        FROM shift_swap_requests s
        LEFT JOIN employees req ON req.id = s.requester_id
        LEFT JOIN employees tgt ON tgt.id = s.target_id
        LEFT JOIN shifts req_shift ON req_shift.id = s.requester_shift_id
        LEFT JOIN shifts tgt_shift ON tgt_shift.id = s.target_shift_id
        WHERE s.workspace_id = $1
          AND ($2::VARCHAR IS NULL OR s.status = $2)
        ORDER BY s.created_at DESC
        LIMIT 100
        "#,
    )
    .bind(workspace_id)
    .bind(status_filter)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn update_swap_status(
    pool: &PgPool,
    id: Uuid,
    status: ShiftSwapStatus,
    approver_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE shift_swap_requests
        SET status = $2, approved_by_user_id = $3, updated_at = $4
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(approver_id)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_employee_swaps(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error> {
    sqlx::query_as::<_, ShiftSwapRequestWithNames>(
        r#"
        SELECT
            s.*,
            req.full_name AS requester_name,
            tgt.full_name AS target_name,
            req_shift.name AS requester_shift_name,
            tgt_shift.name AS target_shift_name
        FROM shift_swap_requests s
        LEFT JOIN employees req ON req.id = s.requester_id
        LEFT JOIN employees tgt ON tgt.id = s.target_id
        LEFT JOIN shifts req_shift ON req_shift.id = s.requester_shift_id
        LEFT JOIN shifts tgt_shift ON tgt_shift.id = s.target_shift_id
        WHERE s.requester_id = $1 OR s.target_id = $1
        ORDER BY s.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(employee_id)
    .fetch_all(pool)
    .await
}
