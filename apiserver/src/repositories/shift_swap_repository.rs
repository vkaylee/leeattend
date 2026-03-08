use async_trait::async_trait;
use crate::models::shift_swap::{ShiftSwapRequestWithNames, ShiftSwapRequest, ShiftSwapStatus};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: ShiftSwapRepository — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait ShiftSwapRepository: super::PoolBacked + Send + Sync {
    async fn create_swap_request(&self, request: &ShiftSwapRequest) -> Result<ShiftSwapRequest, sqlx::Error>;
    async fn get_swap_request_by_id(&self, id: Uuid, workspace_id: Uuid) -> Result<Option<ShiftSwapRequest>, sqlx::Error>;
    async fn get_workspace_swaps(&self, workspace_id: Uuid, status_filter: Option<ShiftSwapStatus>) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error>;
    async fn update_swap_status(&self, id: Uuid, workspace_id: Uuid, status: ShiftSwapStatus, approver_id: Option<Uuid>) -> Result<(), sqlx::Error>;
    async fn get_employee_swaps(&self, employee_id: Uuid, workspace_id: Uuid) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error>;
    async fn has_active_swap(&self, workspace_id: Uuid, requester_id: Uuid, target_id: Uuid, swap_date: chrono::NaiveDate) -> Result<bool, sqlx::Error>;
    async fn has_approved_swap_on_date(&self, workspace_id: Uuid, employee_id: Uuid, swap_date: chrono::NaiveDate) -> Result<bool, sqlx::Error>;
    async fn get_workspace_swap_stats(&self, workspace_id: Uuid) -> Result<SwapStats, sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxShiftSwapRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxShiftSwapRepository {
    pub pool: PgPool,
}

impl SqlxShiftSwapRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl ShiftSwapRepository for SqlxShiftSwapRepository {
    async fn create_swap_request(&self, request: &ShiftSwapRequest) -> Result<ShiftSwapRequest, sqlx::Error> {
        create_swap_request(&self.pool, request).await
    }
    async fn get_swap_request_by_id(&self, id: Uuid, workspace_id: Uuid) -> Result<Option<ShiftSwapRequest>, sqlx::Error> {
        get_swap_request_by_id(&self.pool, id, workspace_id).await
    }
    async fn get_workspace_swaps(&self, workspace_id: Uuid, status_filter: Option<ShiftSwapStatus>) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error> {
        get_workspace_swaps(&self.pool, workspace_id, status_filter).await
    }
    async fn update_swap_status(&self, id: Uuid, workspace_id: Uuid, status: ShiftSwapStatus, approver_id: Option<Uuid>) -> Result<(), sqlx::Error> {
        update_swap_status(&self.pool, id, workspace_id, status, approver_id).await
    }
    async fn get_employee_swaps(&self, employee_id: Uuid, workspace_id: Uuid) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error> {
        get_employee_swaps(&self.pool, employee_id, workspace_id).await
    }
    async fn has_active_swap(&self, workspace_id: Uuid, requester_id: Uuid, target_id: Uuid, swap_date: chrono::NaiveDate) -> Result<bool, sqlx::Error> {
        has_active_swap(&self.pool, workspace_id, requester_id, target_id, swap_date).await
    }
    async fn has_approved_swap_on_date(&self, workspace_id: Uuid, employee_id: Uuid, swap_date: chrono::NaiveDate) -> Result<bool, sqlx::Error> {
        has_approved_swap_on_date(&self.pool, workspace_id, employee_id, swap_date).await
    }
    async fn get_workspace_swap_stats(&self, workspace_id: Uuid) -> Result<SwapStats, sqlx::Error> {
        get_workspace_swap_stats(&self.pool, workspace_id).await
    }
}

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
    workspace_id: Uuid,
) -> Result<Option<ShiftSwapRequest>, sqlx::Error> {
    sqlx::query_as::<_, ShiftSwapRequest>(
        "SELECT * FROM shift_swap_requests WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
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
    workspace_id: Uuid,
    status: ShiftSwapStatus,
    approver_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE shift_swap_requests
        SET status = $2, approved_by_user_id = $3, updated_at = $4
        WHERE id = $1 AND workspace_id = $5
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(approver_id)
    .bind(Utc::now())
    .bind(workspace_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_employee_swaps(
    pool: &PgPool,
    employee_id: Uuid,
    workspace_id: Uuid,
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
        WHERE s.workspace_id = $2 AND (s.requester_id = $1 OR s.target_id = $1)
        ORDER BY s.created_at DESC
        LIMIT 50
        "#,
    )
    .bind(employee_id)
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

/// Check if an active (non-cancelled/rejected) swap already exists for requester+target+date.
pub async fn has_active_swap(
    pool: &PgPool,
    workspace_id: Uuid,
    requester_id: Uuid,
    target_id: Uuid,
    swap_date: chrono::NaiveDate,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM shift_swap_requests
            WHERE workspace_id = $1 AND requester_id = $2 AND target_id = $3 AND swap_date = $4
            AND status NOT IN ('cancelled', 'rejected')
        )
        "#,
    )
    .bind(workspace_id)
    .bind(requester_id)
    .bind(target_id)
    .bind(swap_date)
    .fetch_one(pool)
    .await
}

/// Check if either employee already has an approved swap for a given date.
pub async fn has_approved_swap_on_date(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
    swap_date: chrono::NaiveDate,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM shift_swap_requests
            WHERE workspace_id = $1 AND (requester_id = $2 OR target_id = $2)
            AND swap_date = $3
            AND status = 'approved'
        )
        "#,
    )
    .bind(workspace_id)
    .bind(employee_id)
    .bind(swap_date)
    .fetch_one(pool)
    .await
}

/// Status counts for a workspace (for stats dashboard).
#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct SwapStats {
    pub pending_target: i64,
    pub pending_admin: i64,
    pub approved: i64,
    pub rejected: i64,
    pub cancelled: i64,
    pub approved_this_month: i64,
    pub rejected_this_month: i64,
}

pub async fn get_workspace_swap_stats(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<SwapStats, sqlx::Error> {
    sqlx::query_as::<_, SwapStats>(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE status = 'pending_target') AS pending_target,
            COUNT(*) FILTER (WHERE status = 'pending_admin') AS pending_admin,
            COUNT(*) FILTER (WHERE status = 'approved') AS approved,
            COUNT(*) FILTER (WHERE status = 'rejected') AS rejected,
            COUNT(*) FILTER (WHERE status = 'cancelled') AS cancelled,
            COUNT(*) FILTER (WHERE status = 'approved'
                AND created_at >= date_trunc('month', CURRENT_DATE)) AS approved_this_month,
            COUNT(*) FILTER (WHERE status = 'rejected'
                AND created_at >= date_trunc('month', CURRENT_DATE)) AS rejected_this_month
        FROM shift_swap_requests
        WHERE workspace_id = $1
        "#,
    )
    .bind(workspace_id)
    .fetch_one(pool)
    .await
}

impl super::PoolBacked for SqlxShiftSwapRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self { Self::new(pool) }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
