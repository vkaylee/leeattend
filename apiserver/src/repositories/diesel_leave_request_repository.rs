use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::leave::*;
use crate::models::{LeaveRequest, LeaveStatus};
use crate::repositories::leave_request_repository::{
    create_leave_request, get_approved_leave_breakdown, update_leave_request_status,
    LeaveBreakdown, LeaveRequestRepository, SqlxLeaveRequestRepository,
};
use crate::repositories::PoolBacked;
use crate::schema::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DieselLeaveRequestRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselLeaveRequestRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

impl PoolBacked for DieselLeaveRequestRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!(
            "DieselLeaveRequestRepository requires diesel_pools to be manually injected via new()"
        );
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}

#[async_trait]
impl LeaveRequestRepository for DieselLeaveRequestRepository {
    async fn create_leave_request(&self, request: &LeaveRequest) -> Result<(), AppError> {
        create_leave_request(&self.sqlx_pool, request)
            .await
            .map_err(Into::into)
    }

    async fn get_leave_request_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<LeaveRequest>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = leave_requests::table
            .inner_join(employees::table)
            .filter(leave_requests::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .select(LeaveRequestEntity::as_select())
            .first::<LeaveRequestEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn update_leave_request_status(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        status: LeaveStatus,
        approved_by: Option<Uuid>,
    ) -> Result<(), AppError> {
        update_leave_request_status(&self.sqlx_pool, workspace_id, id, status, approved_by)
            .await
            .map_err(Into::into)
    }

    async fn get_employee_leave_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = leave_requests::table
            .inner_join(employees::table)
            .filter(leave_requests::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .order(leave_requests::start_date.desc())
            .select(LeaveRequestEntity::as_select())
            .load::<LeaveRequestEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_workspace_leave_requests(
        &self,
        workspace_id: Uuid,
        status: Option<LeaveStatus>,
    ) -> Result<Vec<LeaveRequest>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut query = leave_requests::table
            .inner_join(employees::table)
            .filter(employees::workspace_id.eq(workspace_id))
            .into_boxed();

        if let Some(s) = status {
            query = query.filter(leave_requests::status.eq(s));
        }

        let entities = query
            .order(leave_requests::created_at.desc())
            .select(LeaveRequestEntity::as_select())
            .load::<LeaveRequestEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_approved_leave_breakdown(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        workspace_id: Uuid,
    ) -> Result<LeaveBreakdown, AppError> {
        get_approved_leave_breakdown(
            &self.sqlx_pool,
            employee_id,
            start_date,
            end_date,
            workspace_id,
        )
        .await
        .map_err(Into::into)
    }

    async fn get_employee_department_for_request(
        &self,
        request_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        // Complex history join, fallback to SQLx
        let repo = SqlxLeaveRequestRepository::new(self.sqlx_pool.clone());
        repo.get_employee_department_for_request(request_id, workspace_id)
            .await
    }

    async fn get_leave_types(&self, workspace_id: Uuid) -> Result<Vec<(String, String)>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let rows = leave_types::table
            .filter(leave_types::workspace_id.eq(workspace_id))
            .order(leave_types::code.asc())
            .select((leave_types::code, leave_types::name))
            .load::<(String, String)>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows)
    }

    async fn get_system_generated_holiday_leave(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<(Uuid, Uuid, String)>, AppError> {
        let repo = SqlxLeaveRequestRepository::new(self.sqlx_pool.clone());
        repo.get_system_generated_holiday_leave(workspace_id, employee_id, date)
            .await
    }

    async fn cancel_leave_request(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let repo = SqlxLeaveRequestRepository::new(self.sqlx_pool.clone());
        repo.cancel_leave_request(workspace_id, id).await
    }

    async fn get_employee_leave_history_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<crate::repositories::leave_request_repository::LeaveHistoryRequestItem>, AppError> {
        let repo = SqlxLeaveRequestRepository::new(self.sqlx_pool.clone());
        repo.get_employee_leave_history_requests(workspace_id, employee_id, year, leave_type_id).await
    }

    async fn check_leave_type_exists(
        &self,
        workspace_id: Uuid,
        leave_type_id: Uuid,
    ) -> Result<bool, AppError> {
        let repo = SqlxLeaveRequestRepository::new(self.sqlx_pool.clone());
        repo.check_leave_type_exists(workspace_id, leave_type_id).await
    }

    async fn get_leave_type_name(
        &self,
        workspace_id: Uuid,
        leave_type_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let repo = SqlxLeaveRequestRepository::new(self.sqlx_pool.clone());
        repo.get_leave_type_name(workspace_id, leave_type_id).await
    }
}
