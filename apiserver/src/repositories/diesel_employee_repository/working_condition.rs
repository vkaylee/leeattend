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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::{DieselPools, TenantAuthToken};
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_working_condition_flow() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://test_user:test_password@db_system:5432/leeattend_test_template".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselEmployeeRepository::new(pools.clone());
        let ws_id = Uuid::new_v4();
        let token = TenantAuthToken::system_override(ws_id);

        let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
            c
        } else {
            println!("Database not available, skipping integration assertions");
            return;
        };

        // 1. Insert Workspace to satisfy FK constraints
        let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
            id: ws_id,
            name: "Test Workspace".to_string(),
            created_at: Some(chrono::Utc::now()),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: None,
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            custom_db_url_encrypted: None,
        };
        diesel::insert_into(crate::schema::workspaces::table)
            .values(&workspace_entity)
            .execute(&mut *conn)
            .await
            .expect("insert workspace");

        let employer_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();

        // Insert Employer
        diesel::sql_query("INSERT INTO employers (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
            .bind::<diesel::sql_types::Uuid, _>(employer_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("Test Employer")
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert employer");

        // Insert Employee
        diesel::sql_query("INSERT INTO employees (id, workspace_id, employee_id_number, full_name, hire_date, original_hire_date, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>("EMP-WC-TEST")
            .bind::<diesel::sql_types::Text, _>("Test WC Employee")
            .bind::<diesel::sql_types::Date, _>(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
            .bind::<diesel::sql_types::Date, _>(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
            .bind::<diesel::sql_types::Bool, _>(true)
            .execute(&mut *conn)
            .await
            .expect("insert employee");

        // Insert Employment
        diesel::sql_query("INSERT INTO employments (id, workspace_id, employee_id, employer_id, status, start_date, created_at, updated_at, employment_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind::<diesel::sql_types::Uuid, _>(employment_id)
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Uuid, _>(employer_id)
            .bind::<diesel::sql_types::Text, _>("active")
            .bind::<diesel::sql_types::Date, _>(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Text, _>("full_time")
            .execute(&mut *conn)
            .await
            .expect("insert employment");

        drop(conn);

        // 2. Create Working Condition
        let wc_id = Uuid::new_v4();
        let cond = WorkingCondition::new(
            wc_id,
            employment_id,
            "break".to_string(),
            30,
            true,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            None,
        );

        let create_res = repo.create_working_condition(&token, &cond).await;
        assert!(create_res.is_ok());

        // 3. Get Working Conditions
        let (list, total) = repo
            .get_working_conditions(&token, employment_id, 1, 10)
            .await
            .unwrap();
        assert_eq!(total, 1);
        assert_eq!(list[0].id(), wc_id);
        assert_eq!(list[0].type_(), "break");

        // 4. Get Working Conditions At
        let list_at = repo
            .get_working_conditions_at(
                &token,
                employment_id,
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(list_at.len(), 1);

        // 5. Batch Get
        let batch_res = repo
            .get_working_conditions_batch_by_employments(
                &token,
                &[employment_id],
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            )
            .await
            .unwrap();
        assert!(batch_res.contains_key(&employment_id));

        // 6. Delete
        let deleted = repo
            .delete_working_condition(&token, wc_id, employment_id)
            .await
            .unwrap();
        assert!(deleted);

        // Verify deleted
        let (_, total_after) = repo
            .get_working_conditions(&token, employment_id, 1, 10)
            .await
            .unwrap();
        assert_eq!(total_after, 0);
    }
}
