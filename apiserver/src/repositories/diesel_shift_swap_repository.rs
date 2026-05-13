use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::roster::ShiftSwapRequestEntity;
use crate::models::shift_swap::{ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus};
use crate::repositories::shift_swap_repository::{ShiftSwapRepository, SwapStats};
use crate::schema::shift_swap_requests;

#[derive(Debug, diesel::QueryableByName)]
pub struct ShiftSwapRequestWithNamesEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub requester_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub target_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub swap_date: NaiveDate,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub requester_shift_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub target_shift_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub reason: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub requested_by_user_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub approved_by_user_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub created_at: chrono::DateTime<Utc>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub updated_at: chrono::DateTime<Utc>,

    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub requester_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub target_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub requester_shift_name: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub target_shift_name: Option<String>,
}

impl From<ShiftSwapRequestWithNamesEntity> for ShiftSwapRequestWithNames {
    fn from(entity: ShiftSwapRequestWithNamesEntity) -> Self {
        let status = match entity.status.as_str() {
            "pending_admin" => ShiftSwapStatus::PendingAdmin,
            "approved" => ShiftSwapStatus::Approved,
            "rejected" => ShiftSwapStatus::Rejected,
            "cancelled" => ShiftSwapStatus::Cancelled,
            _ => ShiftSwapStatus::PendingTarget,
        };
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            requester_id: entity.requester_id,
            target_id: entity.target_id,
            swap_date: entity.swap_date,
            requester_shift_id: entity.requester_shift_id,
            target_shift_id: entity.target_shift_id,
            status,
            reason: entity.reason,
            requested_by_user_id: entity.requested_by_user_id,
            approved_by_user_id: entity.approved_by_user_id,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            requester_name: entity.requester_name,
            target_name: entity.target_name,
            requester_shift_name: entity.requester_shift_name,
            target_shift_name: entity.target_shift_name,
        }
    }
}

#[derive(Debug, diesel::QueryableByName)]
pub struct SwapStatsEntity {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub pending_target: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub pending_admin: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub approved: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub rejected: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub cancelled: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub approved_this_month: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub rejected_this_month: i64,
}

impl From<SwapStatsEntity> for SwapStats {
    fn from(entity: SwapStatsEntity) -> Self {
        Self {
            pending_target: entity.pending_target,
            pending_admin: entity.pending_admin,
            approved: entity.approved,
            rejected: entity.rejected,
            cancelled: entity.cancelled,
            approved_this_month: entity.approved_this_month,
            rejected_this_month: entity.rejected_this_month,
        }
    }
}
pub struct DieselShiftSwapRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselShiftSwapRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl ShiftSwapRepository for DieselShiftSwapRepository {
    async fn create_swap_request(
        &self,
        request: &ShiftSwapRequest,
    ) -> Result<ShiftSwapRequest, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = ShiftSwapRequestEntity {
            id: request.id,
            workspace_id: request.workspace_id,
            requester_id: request.requester_id,
            target_id: request.target_id,
            swap_date: request.swap_date,
            requester_shift_id: request.requester_shift_id,
            target_shift_id: request.target_shift_id,
            status: match request.status {
                ShiftSwapStatus::PendingTarget => "pending_target".to_string(),
                ShiftSwapStatus::PendingAdmin => "pending_admin".to_string(),
                ShiftSwapStatus::Approved => "approved".to_string(),
                ShiftSwapStatus::Rejected => "rejected".to_string(),
                ShiftSwapStatus::Cancelled => "cancelled".to_string(),
            },
            reason: request.reason.clone(),
            requested_by_user_id: request.requested_by_user_id,
            approved_by_user_id: request.approved_by_user_id,
            created_at: request.created_at,
            updated_at: request.updated_at,
        };

        let result: ShiftSwapRequestEntity = diesel::insert_into(shift_swap_requests::table)
            .values(&entity)
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_swap_request_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<ShiftSwapRequest>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: Option<ShiftSwapRequestEntity> = shift_swap_requests::table
            .filter(shift_swap_requests::id.eq(id))
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_workspace_swaps(
        &self,
        workspace_id: Uuid,
        status_filter: Option<ShiftSwapStatus>,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let status_str = status_filter.map(|s| match s {
            ShiftSwapStatus::PendingTarget => "pending_target",
            ShiftSwapStatus::PendingAdmin => "pending_admin",
            ShiftSwapStatus::Approved => "approved",
            ShiftSwapStatus::Rejected => "rejected",
            ShiftSwapStatus::Cancelled => "cancelled",
        });

        let entities: Vec<ShiftSwapRequestWithNamesEntity> = diesel::sql_query(
            r#"
            SELECT
                s.id, s.workspace_id, s.requester_id, s.target_id, s.swap_date,
                s.requester_shift_id, s.target_shift_id, s.status::TEXT as status, s.reason,
                s.requested_by_user_id, s.approved_by_user_id, s.created_at, s.updated_at,
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
              AND ($2::VARCHAR IS NULL OR s.status::VARCHAR = $2)
            ORDER BY s.created_at DESC
            LIMIT 100
            "#,
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(status_str)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn update_swap_status(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        status: ShiftSwapStatus,
        approver_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let status_str = match status {
            ShiftSwapStatus::PendingTarget => "pending_target",
            ShiftSwapStatus::PendingAdmin => "pending_admin",
            ShiftSwapStatus::Approved => "approved",
            ShiftSwapStatus::Rejected => "rejected",
            ShiftSwapStatus::Cancelled => "cancelled",
        };

        diesel::update(shift_swap_requests::table)
            .filter(shift_swap_requests::id.eq(id))
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .set((
                shift_swap_requests::status.eq(status_str),
                shift_swap_requests::approved_by_user_id.eq(approver_id),
                shift_swap_requests::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_employee_swaps(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities: Vec<ShiftSwapRequestWithNamesEntity> = diesel::sql_query(
            r#"
            SELECT
                s.id, s.workspace_id, s.requester_id, s.target_id, s.swap_date,
                s.requester_shift_id, s.target_shift_id, s.status::TEXT as status, s.reason,
                s.requested_by_user_id, s.approved_by_user_id, s.created_at, s.updated_at,
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
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn has_active_swap(
        &self,
        workspace_id: Uuid,
        requester_id: Uuid,
        target_id: Uuid,
        swap_date: NaiveDate,
    ) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            shift_swap_requests::table
                .filter(shift_swap_requests::workspace_id.eq(workspace_id))
                .filter(shift_swap_requests::requester_id.eq(requester_id))
                .filter(shift_swap_requests::target_id.eq(target_id))
                .filter(shift_swap_requests::swap_date.eq(swap_date))
                .filter(shift_swap_requests::status.ne_all(["cancelled", "rejected"])),
        ))
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(exists)
    }

    async fn has_approved_swap_on_date(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        swap_date: NaiveDate,
    ) -> Result<bool, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let exists: bool = diesel::select(diesel::dsl::exists(
            shift_swap_requests::table
                .filter(shift_swap_requests::workspace_id.eq(workspace_id))
                .filter(
                    shift_swap_requests::requester_id
                        .eq(employee_id)
                        .or(shift_swap_requests::target_id.eq(employee_id)),
                )
                .filter(shift_swap_requests::swap_date.eq(swap_date))
                .filter(shift_swap_requests::status.eq("approved")),
        ))
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(exists)
    }

    async fn get_workspace_swap_stats(&self, workspace_id: Uuid) -> Result<SwapStats, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: SwapStatsEntity = diesel::sql_query(
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
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }
}

impl crate::repositories::PoolBacked for DieselShiftSwapRepository {
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselShiftSwapRepository requires DieselPools. Use new() instead.");
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
