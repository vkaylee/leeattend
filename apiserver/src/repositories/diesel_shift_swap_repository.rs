use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::models::diesel::roster::ShiftSwapRequestEntity;
use crate::models::shift_swap::{ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus};
use crate::repositories::shift_swap_repository::{ShiftSwapRepository, SwapStats};
use crate::schema::shift_swap_requests;

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
    ) -> Result<ShiftSwapRequest, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

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
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(result.into())
    }

    async fn get_swap_request_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<ShiftSwapRequest>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let entity: Option<ShiftSwapRequestEntity> = shift_swap_requests::table
            .filter(shift_swap_requests::id.eq(id))
            .filter(shift_swap_requests::workspace_id.eq(workspace_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(entity.map(Into::into))
    }

    async fn get_workspace_swaps(
        &self,
        workspace_id: Uuid,
        status_filter: Option<ShiftSwapStatus>,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error> {
        crate::repositories::shift_swap_repository::get_workspace_swaps(
            &self.sqlx_pool,
            workspace_id,
            status_filter,
        )
        .await
    }

    async fn update_swap_status(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        status: ShiftSwapStatus,
        approver_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

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
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(())
    }

    async fn get_employee_swaps(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Vec<ShiftSwapRequestWithNames>, sqlx::Error> {
        crate::repositories::shift_swap_repository::get_employee_swaps(
            &self.sqlx_pool,
            employee_id,
            workspace_id,
        )
        .await
    }

    async fn has_active_swap(
        &self,
        workspace_id: Uuid,
        requester_id: Uuid,
        target_id: Uuid,
        swap_date: NaiveDate,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

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
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(exists)
    }

    async fn has_approved_swap_on_date(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        swap_date: NaiveDate,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

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
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(exists)
    }

    async fn get_workspace_swap_stats(&self, workspace_id: Uuid) -> Result<SwapStats, sqlx::Error> {
        crate::repositories::shift_swap_repository::get_workspace_swap_stats(
            &self.sqlx_pool,
            workspace_id,
        )
        .await
    }
}

impl crate::repositories::PoolBacked for DieselShiftSwapRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselShiftSwapRepository requires DieselPools. Use new() instead.");
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
