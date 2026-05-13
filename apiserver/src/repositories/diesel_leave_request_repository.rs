use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::leave::*;
use crate::models::{LeaveRequest, LeaveStatus};
use crate::repositories::leave_request_repository::{
    create_leave_request, get_approved_leave_breakdown, update_leave_request_status,
    LeaveBreakdown, LeaveRequestRepository,
};

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
        sqlx::query_scalar(
            r#"
            SELECT edh.department_id
            FROM leave_requests lr
            JOIN employee_department_history edh ON lr.employee_id = edh.employee_id
            JOIN employees e ON lr.employee_id = e.id
            WHERE lr.id = $1 AND e.workspace_id = $2
            ORDER BY edh.effective_date DESC
            LIMIT 1
            "#,
        )
        .bind(request_id)
        .bind(workspace_id)
        .fetch_optional(&self.sqlx_pool)
        .await
        .map_err(Into::into)
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
        sqlx::query_as(
            r#"SELECT lr.id, lr.leave_type_id, lt.code
               FROM leave_requests lr
               JOIN leave_types lt ON lt.id = lr.leave_type_id
               JOIN employees e ON e.id = lr.employee_id
               WHERE lr.employee_id = $1
                 AND e.workspace_id = $3
                 AND lr.status IN ('approved'::leave_status_type, 'pre_approved'::leave_status_type)
                 AND lr.start_date <= $2
                 AND lr.end_date >= $2
                 AND lr.requested_by_user_id IS NULL"#,
        )
        .bind(employee_id)
        .bind(date)
        .bind(workspace_id)
        .fetch_optional(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))
    }

    async fn cancel_leave_request(&self, _workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE leave_requests SET status = 'cancelled'::leave_status_type, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.sqlx_pool)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        Ok(())
    }

    async fn get_employee_leave_history_requests(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        year: i32,
        leave_type_id: Option<Uuid>,
    ) -> Result<Vec<crate::repositories::leave_request_repository::LeaveHistoryRequestItem>, AppError>
    {
        let rows = sqlx::query!(
            r#"
            SELECT lr.id, lr.start_date, lr.end_date, lt.name as leave_type_name,
                   lr.reason, lr.created_at
            FROM leave_requests lr
            JOIN leave_types lt ON lr.leave_type_id = lt.id
            JOIN employees e ON lr.employee_id = e.id
            WHERE lr.employee_id = $1
              AND e.workspace_id = $4
              AND lr.status IN ('approved', 'pre_approved')
              AND EXTRACT(YEAR FROM lr.start_date) = $2
              AND (lt.id = $3 OR $3 IS NULL)
            "#,
            employee_id,
            year as f64,
            leave_type_id,
            workspace_id
        )
        .fetch_all(&self.sqlx_pool)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(
                |row| crate::repositories::leave_request_repository::LeaveHistoryRequestItem {
                    id: row.id,
                    start_date: row.start_date,
                    end_date: row.end_date,
                    leave_type_name: row.leave_type_name,
                    reason: row.reason,
                    created_at: row.created_at,
                },
            )
            .collect())
    }

    async fn check_leave_type_exists(
        &self,
        workspace_id: Uuid,
        leave_type_id: Uuid,
    ) -> Result<bool, AppError> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM leave_types WHERE id = $1 AND workspace_id = $2)",
        )
        .bind(leave_type_id)
        .bind(workspace_id)
        .fetch_one(&self.sqlx_pool)
        .await
        .unwrap_or(false);
        Ok(exists)
    }

    async fn get_leave_type_name(
        &self,
        workspace_id: Uuid,
        leave_type_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let name: Option<String> =
            sqlx::query_scalar("SELECT name FROM leave_types WHERE id = $1 AND workspace_id = $2")
                .bind(leave_type_id)
                .bind(workspace_id)
                .fetch_optional(&self.sqlx_pool)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        Ok(name)
    }
}
