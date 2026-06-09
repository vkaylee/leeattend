use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::roster::{EmployeeShiftEntity, ShiftEntity};
use crate::models::shift::Shift;
use crate::repositories::shift_repository::ShiftRepository;
use crate::schema::{employee_shifts, shifts};

pub struct DieselShiftRepository {
    diesel_pools: DieselPools,
}

impl DieselShiftRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl ShiftRepository for DieselShiftRepository {
    async fn create_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::CreateShiftRequest,
        is_active: bool,
        parent_shift_id: Option<Uuid>,
    ) -> Result<Shift, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_shift = ShiftEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name: req.name.clone(),
            start_time: req.start_time,
            end_time: req.end_time,
            break_duration_minutes: req.break_duration_minutes,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            break_is_overtime: req.break_is_overtime,
            early_punch_in_grace_period_minutes: req.early_punch_in_grace_period_minutes,
            break_start: req.break_start,
            break_end: req.break_end,
            is_break_auto_deduct: req.is_break_auto_deduct,
            is_active,
            parent_shift_id,
            allow_early_ot: req.allow_early_ot,
            max_early_ot_minutes: req.max_early_ot_minutes,
            report_symbol: req.report_symbol.clone(),
            location_id: req.location_id,
            employer_id: req.employer_id,
        };

        let entity: ShiftEntity = diesel::insert_into(shifts::table)
            .values(&new_shift)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn get_shift_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Shift>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: Option<ShiftEntity> = shifts::table
            .filter(shifts::id.eq(id))
            .filter(shifts::workspace_id.eq(workspace_id))
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_workspace_shifts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        location_id: Option<Uuid>,
        employer_id: Option<Uuid>,
    ) -> Result<Vec<Shift>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = shifts::table
            .filter(shifts::workspace_id.eq(workspace_id))
            .filter(shifts::is_active.eq(true))
            .into_boxed();

        if let Some(loc_id) = location_id {
            query = query.filter(
                shifts::location_id
                    .is_null()
                    .or(shifts::location_id.eq(loc_id)),
            );
        }

        if let Some(emp_id) = employer_id {
            query = query.filter(
                shifts::employer_id
                    .is_null()
                    .or(shifts::employer_id.eq(emp_id)),
            );
        }

        let entities: Vec<ShiftEntity> = query
            .order(shifts::name.asc())
            .load(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn update_shift_direct(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: &crate::models::UpdateShiftRequest,
    ) -> Result<Shift, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: ShiftEntity = diesel::update(shifts::table)
            .filter(shifts::id.eq(id))
            .filter(shifts::workspace_id.eq(workspace_id))
            .set((
                shifts::name.eq(&req.name),
                shifts::report_symbol.eq(&req.report_symbol),
                shifts::start_time.eq(req.start_time),
                shifts::end_time.eq(req.end_time),
                shifts::break_duration_minutes.eq(req.break_duration_minutes),
                shifts::break_start.eq(req.break_start),
                shifts::break_end.eq(req.break_end),
                shifts::is_break_auto_deduct.eq(req.is_break_auto_deduct),
                shifts::break_is_overtime.eq(req.break_is_overtime),
                shifts::early_punch_in_grace_period_minutes
                    .eq(req.early_punch_in_grace_period_minutes),
                shifts::allow_early_ot.eq(req.allow_early_ot),
                shifts::max_early_ot_minutes.eq(req.max_early_ot_minutes),
                shifts::location_id.eq(req.location_id),
                shifts::employer_id.eq(req.employer_id),
                shifts::updated_at.eq(Utc::now()),
            ))
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn archive_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::update(shifts::table)
            .filter(shifts::id.eq(id))
            .filter(shifts::workspace_id.eq(workspace_id))
            .set((
                shifts::is_active.eq(false),
                shifts::updated_at.eq(Utc::now()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn assign_employment_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        shift_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let new_assignment = EmployeeShiftEntity {
            id: Uuid::new_v4(),
            employment_id,
            shift_id,
            start_date,
            end_date,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        diesel::insert_into(employee_shifts::table)
            .values(&new_assignment)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_employment_active_shift(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Shift>, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: Option<ShiftEntity> = shifts::table
            .inner_join(employee_shifts::table.on(shifts::id.eq(employee_shifts::shift_id)))
            .filter(employee_shifts::employment_id.eq(employment_id))
            .filter(employee_shifts::start_date.le(date))
            .filter(
                employee_shifts::end_date
                    .is_null()
                    .or(employee_shifts::end_date.ge(date)),
            )
            .order(employee_shifts::created_at.desc())
            .select(ShiftEntity::as_select())
            .first(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn update_assignments_to_new_version(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        old_shift_id: Uuid,
        new_shift_id: Uuid,
        effective_date: NaiveDate,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&auth_token.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let yesterday = effective_date.pred_opt().unwrap_or(effective_date);

        conn.transaction::<_, AppError, _>(|conn| {
            Box::pin(async move {
                diesel::sql_query(
                    "UPDATE employee_shifts
                     SET end_date = $2, updated_at = CURRENT_TIMESTAMP
                     WHERE shift_id = $1 AND (end_date IS NULL OR end_date >= $3)",
                )
                .bind::<diesel::sql_types::Uuid, _>(old_shift_id)
                .bind::<diesel::sql_types::Date, _>(yesterday)
                .bind::<diesel::sql_types::Date, _>(effective_date)
                .execute(conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                diesel::sql_query(
                    "INSERT INTO employee_shifts (employment_id, shift_id, start_date, end_date)
                     SELECT employment_id, $2, $3, NULL
                     FROM employee_shifts
                     WHERE shift_id = $1 AND end_date = $4",
                )
                .bind::<diesel::sql_types::Uuid, _>(old_shift_id)
                .bind::<diesel::sql_types::Uuid, _>(new_shift_id)
                .bind::<diesel::sql_types::Date, _>(effective_date)
                .bind::<diesel::sql_types::Date, _>(yesterday)
                .execute(conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                Ok(())
            })
        })
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselShiftRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
