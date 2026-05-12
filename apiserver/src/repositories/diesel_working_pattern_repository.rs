use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::working_pattern::{
    EmployeeWorkingPatternEntity, WorkingPatternDetailEntity, WorkingPatternEntity,
};
use crate::models::{
    BulkAssignResponse, EmployeeWorkingPattern, WorkingPattern, WorkingPatternDetail,
};
use crate::schema::{employee_working_patterns, working_pattern_details, working_patterns};

pub struct DieselWorkingPatternRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselWorkingPatternRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

macro_rules! map_db_err {
    ($e:expr) => {
        $e.map_err(|e| AppError::Internal(format!("db: {e}")))
    };
}

impl DieselWorkingPatternRepository {
    pub async fn create_pattern(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<WorkingPattern, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        map_db_err!(
            diesel::insert_into(working_patterns::table)
                .values((
                    working_patterns::workspace_id.eq(workspace_id),
                    working_patterns::name.eq(name),
                    working_patterns::description.eq(description),
                ))
                .returning(WorkingPatternEntity::as_returning())
                .get_result(&mut conn)
                .await
        )
        .map(Into::into)
    }

    pub async fn add_pattern_detail(
        &self,
        pattern_id: Uuid,
        day_of_week: i32,
        shift_id: Option<Uuid>,
    ) -> Result<WorkingPatternDetail, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        map_db_err!(
            diesel::insert_into(working_pattern_details::table)
                .values((
                    working_pattern_details::pattern_id.eq(pattern_id),
                    working_pattern_details::day_of_week.eq(day_of_week),
                    working_pattern_details::shift_id.eq(shift_id),
                ))
                .returning(WorkingPatternDetailEntity::as_returning())
                .get_result(&mut conn)
                .await
        )
        .map(Into::into)
    }

    pub async fn get_patterns(&self, workspace_id: Uuid) -> Result<Vec<WorkingPattern>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        map_db_err!(
            working_patterns::table
                .filter(working_patterns::workspace_id.eq(workspace_id))
                .filter(working_patterns::is_active.eq(true))
                .order(working_patterns::name.asc())
                .select(WorkingPatternEntity::as_select())
                .load(&mut conn)
                .await
        )
        .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    pub async fn get_pattern_details(
        &self,
        pattern_id: Uuid,
    ) -> Result<Vec<WorkingPatternDetail>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        map_db_err!(
            working_pattern_details::table
                .filter(working_pattern_details::pattern_id.eq(pattern_id))
                .order(working_pattern_details::day_of_week.asc())
                .select(WorkingPatternDetailEntity::as_select())
                .load(&mut conn)
                .await
        )
        .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    pub async fn assign_pattern(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        pattern_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<EmployeeWorkingPattern, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        map_db_err!(
            diesel::insert_into(employee_working_patterns::table)
                .values((
                    employee_working_patterns::workspace_id.eq(workspace_id),
                    employee_working_patterns::employee_id.eq(employee_id),
                    employee_working_patterns::pattern_id.eq(pattern_id),
                    employee_working_patterns::start_date.eq(start_date),
                    employee_working_patterns::end_date.eq(end_date),
                ))
                .returning(EmployeeWorkingPatternEntity::as_returning())
                .get_result(&mut conn)
                .await
        )
        .map(Into::into)
    }

    pub async fn get_employee_pattern_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeWorkingPattern>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        map_db_err!(
            employee_working_patterns::table
                .filter(employee_working_patterns::workspace_id.eq(workspace_id))
                .filter(employee_working_patterns::employee_id.eq(employee_id))
                .order(employee_working_patterns::start_date.desc())
                .select(EmployeeWorkingPatternEntity::as_select())
                .load(&mut conn)
                .await
        )
        .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    pub async fn bulk_assign_pattern(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        pattern_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<BulkAssignResponse, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let conflicts: Vec<Uuid> = employee_working_patterns::table
            .filter(employee_working_patterns::workspace_id.eq(workspace_id))
            .filter(employee_working_patterns::start_date.eq(start_date))
            .filter(employee_working_patterns::employee_id.eq_any(employee_ids))
            .select(employee_working_patterns::employee_id)
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if !conflicts.is_empty() {
            return Ok(BulkAssignResponse {
                success_count: 0,
                failed_employees: conflicts,
            });
        }

        for &employee_id in employee_ids {
            diesel::insert_into(employee_working_patterns::table)
                .values((
                    employee_working_patterns::workspace_id.eq(workspace_id),
                    employee_working_patterns::employee_id.eq(employee_id),
                    employee_working_patterns::pattern_id.eq(pattern_id),
                    employee_working_patterns::start_date.eq(start_date),
                    employee_working_patterns::end_date.eq(end_date),
                ))
                .execute(&mut conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
        }

        Ok(BulkAssignResponse {
            success_count: employee_ids.len(),
            failed_employees: vec![],
        })
    }

    pub async fn delete_pattern(
        &self,
        workspace_id: Uuid,
        pattern_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(
            working_pattern_details::table
                .filter(working_pattern_details::pattern_id.eq(pattern_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(
            working_patterns::table
                .filter(working_patterns::id.eq(pattern_id))
                .filter(working_patterns::workspace_id.eq(workspace_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}

impl crate::repositories::PoolBacked for DieselWorkingPatternRepository {
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselWorkingPatternRepository requires DieselPools. Use new() instead.");
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
