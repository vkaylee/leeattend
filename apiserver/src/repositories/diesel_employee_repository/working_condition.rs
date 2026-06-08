use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::WorkingCondition;
use crate::repositories::employee::*;

use super::DieselEmployeeRepository;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeeWorkingConditionRepository for DieselEmployeeRepository {
    async fn get_working_conditions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employments, working_conditions};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let total: i64 = working_conditions::table
            .inner_join(employments::table)
            .filter(working_conditions::employment_id.eq(employment_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db count: {e}")))?;

        let result = working_conditions::table
            .inner_join(employments::table)
            .select((
                working_conditions::id,
                working_conditions::employment_id,
                working_conditions::type_,
                working_conditions::minute_allowance,
                working_conditions::is_paid,
                working_conditions::start_date,
                working_conditions::end_date,
            ))
            .filter(working_conditions::employment_id.eq(employment_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .order(working_conditions::start_date.desc())
            .limit(limit)
            .offset((page - 1) * limit)
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let data = result
            .into_iter()
            .map(|(id, emp_id, type_, min, paid, start, end)| {
                WorkingCondition::new(id, emp_id, type_, min, paid, start, end)
            })
            .collect();

        Ok((data, total))
    }

    async fn create_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition: &WorkingCondition,
    ) -> Result<(), AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employments, working_conditions};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employment exists and belongs to the workspace of this tenant
        let employment_exists = employments::table
            .filter(employments::id.eq(condition.employment_id()))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .select(employments::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employment: {e}")))?;

        if employment_exists.is_none() {
            return Err(AppError::NotFound("error.employment_not_found".to_string()));
        }

        diesel::insert_into(working_conditions::table)
            .values((
                working_conditions::id.eq(condition.id()),
                working_conditions::employment_id.eq(condition.employment_id()),
                working_conditions::type_.eq(condition.type_()),
                working_conditions::minute_allowance.eq(condition.minute_allowance()),
                working_conditions::is_paid.eq(condition.is_paid()),
                working_conditions::start_date.eq(condition.start_date()),
                working_conditions::end_date.eq(condition.end_date()),
            ))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn delete_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        employment_id: Uuid,
    ) -> Result<bool, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employments, working_conditions};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::delete(working_conditions::table)
            .filter(working_conditions::id.eq(id))
            .filter(working_conditions::employment_id.eq(employment_id))
            .filter(
                working_conditions::employment_id.eq_any(
                    employments::table
                        .select(employments::id)
                        .filter(employments::workspace_id.eq(workspace_id)),
                ),
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(affected > 0)
    }

    async fn get_working_conditions_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employments, working_conditions};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = working_conditions::table
            .inner_join(employments::table)
            .select((
                working_conditions::id,
                working_conditions::employment_id,
                working_conditions::type_,
                working_conditions::minute_allowance,
                working_conditions::is_paid,
                working_conditions::start_date,
                working_conditions::end_date,
            ))
            .filter(working_conditions::employment_id.eq(employment_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .filter(working_conditions::start_date.le(date))
            .filter(
                working_conditions::end_date
                    .is_null()
                    .or(working_conditions::end_date.ge(date)),
            )
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result
            .into_iter()
            .map(|(id, emp_id, type_, min, paid, start, end)| {
                WorkingCondition::new(id, emp_id, type_, min, paid, start, end)
            })
            .collect())
    }

    async fn get_working_conditions_batch_by_employments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employments, working_conditions};

        if employment_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = working_conditions::table
            .inner_join(employments::table)
            .select((
                working_conditions::id,
                working_conditions::employment_id,
                working_conditions::type_,
                working_conditions::minute_allowance,
                working_conditions::is_paid,
                working_conditions::start_date,
                working_conditions::end_date,
            ))
            .filter(working_conditions::employment_id.eq_any(employment_ids))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(employments::deleted_at.is_null())
            .filter(working_conditions::start_date.le(end_date))
            .filter(
                working_conditions::end_date
                    .is_null()
                    .or(working_conditions::end_date.ge(start_date)),
            )
            .order((
                working_conditions::employment_id,
                working_conditions::start_date,
            ))
            .load::<(
                Uuid,
                Uuid,
                String,
                i32,
                bool,
                chrono::NaiveDate,
                Option<chrono::NaiveDate>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map = std::collections::HashMap::with_capacity(employment_ids.len());
        for (id, emp_id, type_, min, paid, start, end) in result {
            map.entry(emp_id)
                .or_insert_with(Vec::new)
                .push(WorkingCondition::new(
                    id, emp_id, type_, min, paid, start, end,
                ));
        }
        Ok(map)
    }
}
