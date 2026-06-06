use crate::errors::AppError;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::repositories::employee::*;

use super::DieselEmployeeRepository;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl EmployeePayrollConfigRepository for DieselEmployeeRepository {
    async fn upsert_employee_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        std_working_days: rust_decimal::Decimal,
        hours_per_day: rust_decimal::Decimal,
        effective_date: chrono::NaiveDate,
    ) -> Result<crate::models::EmployeePayrollConfig, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employee_payroll_configs, employees};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        // Check if employee exists and belongs to the workspace of this tenant
        let employee_exists = employees::table
            .filter(employees::id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .select(employees::id)
            .first::<Uuid>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db check employee: {e}")))?;

        if employee_exists.is_none() {
            return Err(AppError::NotFound("error.employee_not_found".to_string()));
        }

        let result = diesel::insert_into(employee_payroll_configs::table)
            .values((
                employee_payroll_configs::employee_id.eq(employee_id),
                employee_payroll_configs::effective_date.eq(effective_date),
                employee_payroll_configs::std_working_days.eq(std_working_days),
                employee_payroll_configs::hours_per_day.eq(hours_per_day),
                employee_payroll_configs::created_at.eq(chrono::Utc::now()),
            ))
            .on_conflict((
                employee_payroll_configs::employee_id,
                employee_payroll_configs::effective_date,
            ))
            .do_update()
            .set((
                employee_payroll_configs::std_working_days.eq(std_working_days),
                employee_payroll_configs::hours_per_day.eq(hours_per_day),
            ))
            .returning((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .get_result::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(crate::models::EmployeePayrollConfig {
            id: result.0,
            employee_id: result.1,
            std_working_days: result.2,
            hours_per_day: result.3,
            effective_date: result.4,
            created_at: result.5.unwrap_or_else(chrono::Utc::now),
        })
    }

    async fn get_employee_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: chrono::NaiveDate,
    ) -> Result<Option<crate::models::EmployeePayrollConfig>, AppError> {
        let workspace_id = auth_token.workspace_id();
        use crate::schema::{employee_payroll_configs, employees};

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let result = employee_payroll_configs::table
            .inner_join(employees::table)
            .filter(employee_payroll_configs::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .filter(employee_payroll_configs::effective_date.le(date))
            .order(employee_payroll_configs::effective_date.desc())
            .select((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .first::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(|t| crate::models::EmployeePayrollConfig {
            id: t.0,
            employee_id: t.1,
            std_working_days: t.2,
            hours_per_day: t.3,
            effective_date: t.4,
            created_at: t.5.unwrap_or_else(chrono::Utc::now),
        }))
    }

    async fn get_employee_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeePayrollConfig>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::{employee_payroll_configs, employees};

        let results = employee_payroll_configs::table
            .inner_join(employees::table)
            .filter(employee_payroll_configs::employee_id.eq(employee_id))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .order(employee_payroll_configs::effective_date.desc())
            .select((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .load::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results
            .into_iter()
            .map(|t| crate::models::EmployeePayrollConfig {
                id: t.0,
                employee_id: t.1,
                std_working_days: t.2,
                hours_per_day: t.3,
                effective_date: t.4,
                created_at: t.5.unwrap_or_else(chrono::Utc::now),
            })
            .collect())
    }

    async fn get_employee_payroll_configs_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>>, AppError>
    {
        let workspace_id = auth_token.workspace_id();
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        use crate::schema::{employee_payroll_configs, employees};

        let results = employee_payroll_configs::table
            .inner_join(employees::table)
            .filter(employee_payroll_configs::employee_id.eq_any(employee_ids))
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::deleted_at.is_null())
            .order((
                employee_payroll_configs::employee_id.asc(),
                employee_payroll_configs::effective_date.desc(),
            ))
            .select((
                employee_payroll_configs::id,
                employee_payroll_configs::employee_id,
                employee_payroll_configs::std_working_days,
                employee_payroll_configs::hours_per_day,
                employee_payroll_configs::effective_date,
                employee_payroll_configs::created_at,
            ))
            .load::<(
                Uuid,
                Uuid,
                rust_decimal::Decimal,
                rust_decimal::Decimal,
                chrono::NaiveDate,
                Option<chrono::DateTime<chrono::Utc>>,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map: std::collections::HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>> =
            std::collections::HashMap::new();
        for t in results {
            let config = crate::models::EmployeePayrollConfig {
                id: t.0,
                employee_id: t.1,
                std_working_days: t.2,
                hours_per_day: t.3,
                effective_date: t.4,
                created_at: t.5.unwrap_or_else(chrono::Utc::now),
            };
            map.entry(config.employee_id).or_default().push(config);
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;
    use crate::diesel_pool::TenantAuthToken;

    #[tokio::test]
    async fn test_upsert_employee_payroll_config_non_existent_employee() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselEmployeeRepository::new(pools);
        let ws_id = Uuid::new_v4();
        let auth_token = TenantAuthToken::system_override(ws_id);

        // Pass a random employee_id which does not exist in the tenant's workspace
        let emp_id = Uuid::new_v4();
        let res = repo
            .upsert_employee_payroll_config(
                &auth_token,
                emp_id,
                rust_decimal::Decimal::new(22, 0),
                rust_decimal::Decimal::new(8, 0),
                chrono::NaiveDate::from_ymd_opt(2026, 6, 6).unwrap(),
            )
            .await;

        assert!(res.is_err());
        match res.unwrap_err() {
            AppError::NotFound(msg) => {
                assert_eq!(msg, "error.employee_not_found");
            }
            AppError::Internal(msg) => {
                assert!(
                    msg.contains("connecting") || msg.contains("connection") || msg.contains("db")
                );
            }
            other => panic!(
                "Expected AppError::NotFound or db connection error, got {:?}",
                other
            ),
        }
    }
}
