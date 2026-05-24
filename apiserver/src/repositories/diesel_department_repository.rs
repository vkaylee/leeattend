use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::diesel::department::{
    DieselDepartment, NewDieselDepartment, UpdateDieselDepartment,
};
use crate::models::{Department, DepartmentPayrollConfig, EmployeeDepartmentHistory};
use crate::repositories::department_repository::DepartmentRepository;
use crate::schema::departments;

#[derive(Debug, Queryable, QueryableByName)]
pub struct DepartmentWithCountEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub workspace_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub description: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub manager_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    pub default_working_pattern_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub created_at: Option<chrono::DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub updated_at: Option<chrono::DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub active_employee_count: i64,
}

impl From<DepartmentWithCountEntity> for Department {
    fn from(entity: DepartmentWithCountEntity) -> Self {
        Self {
            id: entity.id,
            workspace_id: entity.workspace_id,
            name: entity.name,
            description: entity.description,
            manager_id: entity.manager_id,
            default_working_pattern_id: entity.default_working_pattern_id,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
            updated_at: entity.updated_at.unwrap_or_else(Utc::now),
            active_employee_count: entity.active_employee_count,
        }
    }
}

#[derive(Debug, Queryable, QueryableByName)]
pub struct EmployeeDepartmentHistoryEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub employee_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub effective_date: NaiveDate,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    pub notes: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub created_at: Option<chrono::DateTime<Utc>>,
}

impl From<EmployeeDepartmentHistoryEntity> for EmployeeDepartmentHistory {
    fn from(entity: EmployeeDepartmentHistoryEntity) -> Self {
        Self {
            id: entity.id,
            employee_id: entity.employee_id,
            department_id: entity.department_id,
            effective_date: entity.effective_date,
            notes: entity.notes,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
        }
    }
}

#[derive(Debug, Queryable, QueryableByName, Selectable)]
#[diesel(table_name = crate::schema::department_payroll_configs)]
pub struct DepartmentPayrollConfigEntity {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub department_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub std_working_days: bigdecimal::BigDecimal,
    #[diesel(sql_type = diesel::sql_types::Date)]
    pub effective_date: NaiveDate,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    pub created_at: Option<chrono::DateTime<Utc>>,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    pub hours_per_day: bigdecimal::BigDecimal,
}

impl From<DepartmentPayrollConfigEntity> for DepartmentPayrollConfig {
    fn from(entity: DepartmentPayrollConfigEntity) -> Self {
        use std::str::FromStr;
        Self {
            id: entity.id,
            department_id: entity.department_id,
            std_working_days: rust_decimal::Decimal::from_str(&entity.std_working_days.to_string())
                .unwrap_or_default(),
            hours_per_day: rust_decimal::Decimal::from_str(&entity.hours_per_day.to_string())
                .unwrap_or_default(),
            effective_date: entity.effective_date,
            created_at: entity.created_at.unwrap_or_else(Utc::now),
        }
    }
}

pub struct DieselDepartmentRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselDepartmentRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl DepartmentRepository for DieselDepartmentRepository {
    async fn create_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
        default_working_pattern_id: Option<Uuid>,
    ) -> Result<Department, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let new_dept = NewDieselDepartment {
            id: Uuid::new_v4(),
            workspace_id,
            name,
            description,
            manager_id,
            default_working_pattern_id,
        };

        let result: DieselDepartment = diesel::insert_into(departments::table)
            .values(&new_dept)
            .get_result(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Re-fetch using sql_query to include active_employee_count
        let entity: DepartmentWithCountEntity = crate::crud_sql_query!(&mut conn,
            r#"SELECT d.id, d.workspace_id, d.name, d.description, d.manager_id, d.default_working_pattern_id, d.created_at, d.updated_at,
               COALESCE((SELECT COUNT(DISTINCT e.id) FROM employees e
                 JOIN (SELECT DISTINCT ON (employee_id) employee_id, department_id FROM employee_department_history ORDER BY employee_id, effective_date DESC) edh ON e.id = edh.employee_id
                 WHERE edh.department_id = d.id AND e.is_active = true), 0) as active_employee_count
               FROM departments d WHERE d.id = $1 AND d.workspace_id = $2"#
        )
        .bind::<diesel::sql_types::Uuid, _>(result.id)
        .bind::<diesel::sql_types::Uuid, _>(result.workspace_id)
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn get_department_by_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<Option<Department>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let entity: Option<DepartmentWithCountEntity> = crate::crud_sql_query!(&mut conn,
            r#"SELECT d.id, d.workspace_id, d.name, d.description, d.manager_id, d.default_working_pattern_id, d.created_at, d.updated_at,
               COALESCE((SELECT COUNT(DISTINCT e.id) FROM employees e
                 JOIN (SELECT DISTINCT ON (employee_id) employee_id, department_id FROM employee_department_history ORDER BY employee_id, effective_date DESC) edh ON e.id = edh.employee_id
                 WHERE edh.department_id = d.id AND e.is_active = true), 0) as active_employee_count
               FROM departments d WHERE d.id = $1 AND d.workspace_id = $2"#
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_department_by_name(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        name: &str,
    ) -> Result<Option<Uuid>, AppError> {
        #[derive(Debug, diesel::QueryableByName)]
        struct IdOnly {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }

        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result: Option<IdOnly> = crate::crud_sql_query!(
            &mut conn,
            "SELECT id FROM departments WHERE workspace_id = $1 AND LOWER(name) = $2 LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .bind::<diesel::sql_types::Text, _>(name.to_lowercase())
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(|r| r.id))
    }

    async fn update_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let update_model = UpdateDieselDepartment {
            name: req.name,
            description: req.description,
            manager_id: req.manager_id,
            default_working_pattern_id: req.default_working_pattern_id,
        };

        diesel::update(departments::table)
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .set(&update_model)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity: DepartmentWithCountEntity = crate::crud_sql_query!(&mut conn,
            r#"SELECT d.id, d.workspace_id, d.name, d.description, d.manager_id, d.default_working_pattern_id, d.created_at, d.updated_at,
               COALESCE((SELECT COUNT(DISTINCT e.id) FROM employees e
                 JOIN (SELECT DISTINCT ON (employee_id) employee_id, department_id FROM employee_department_history ORDER BY employee_id, effective_date DESC) edh ON e.id = edh.employee_id
                 WHERE edh.department_id = d.id AND e.is_active = true), 0) as active_employee_count
               FROM departments d WHERE d.id = $1 AND d.workspace_id = $2"#
        )
        .bind::<diesel::sql_types::Uuid, _>(id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn delete_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        diesel::delete(departments::table)
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Department>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results: Vec<DepartmentWithCountEntity> = crate::crud_sql_query!(&mut conn,
            r#"SELECT d.id, d.workspace_id, d.name, d.description, d.manager_id, d.default_working_pattern_id, d.created_at, d.updated_at,
               COALESCE((SELECT COUNT(DISTINCT e.id) FROM employees e
                 JOIN (SELECT DISTINCT ON (employee_id) employee_id, department_id FROM employee_department_history ORDER BY employee_id, effective_date DESC) edh ON e.id = edh.employee_id
                 WHERE edh.department_id = d.id AND e.is_active = true), 0) as active_employee_count
               FROM departments d WHERE d.workspace_id = $1 ORDER BY d.name"#
        )
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn transfer_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let _workspace_id = auth_token.workspace_id();

        let entity: EmployeeDepartmentHistoryEntity = crate::crud_sql_query!(&mut conn,
            r#"INSERT INTO employee_department_history (employee_id, department_id, effective_date, notes)
               VALUES ($1, $2, $3, $4)
               RETURNING id, employee_id, department_id, effective_date, notes, created_at"#
        )
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(department_id)
        .bind::<diesel::sql_types::Date, _>(effective_date)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(notes)
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn get_employee_department_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results: Vec<EmployeeDepartmentHistoryEntity> = crate::crud_sql_query!(&mut conn,
            r#"SELECT edh.id, edh.employee_id, edh.department_id, edh.effective_date, edh.notes, edh.created_at
               FROM employee_department_history edh
               JOIN employees e ON e.id = edh.employee_id
               WHERE edh.employee_id = $1 AND e.workspace_id = $2
               ORDER BY edh.effective_date DESC"#
        )
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn get_current_department(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let result: Option<EmployeeDepartmentHistoryEntity> = crate::crud_sql_query!(&mut conn,
            r#"SELECT edh.id, edh.employee_id, edh.department_id, edh.effective_date, edh.notes, edh.created_at
               FROM employee_department_history edh
               JOIN employees e ON e.id = edh.employee_id
               WHERE edh.employee_id = $1 AND e.workspace_id = $2 AND edh.effective_date <= CURRENT_DATE
               ORDER BY edh.effective_date DESC LIMIT 1"#
        )
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn upsert_department_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let _workspace_id = auth_token.workspace_id();

        use std::str::FromStr;
        let std_wd =
            bigdecimal::BigDecimal::from_str(&std_working_days.to_string()).unwrap_or_default();
        let h_per_d =
            bigdecimal::BigDecimal::from_str(&hours_per_day.to_string()).unwrap_or_default();

        let entity: DepartmentPayrollConfigEntity = crate::crud_sql_query!(&mut conn,
            r#"INSERT INTO department_payroll_configs (department_id, std_working_days, hours_per_day, effective_date)
               VALUES ($1, $2, $3, $4)
               ON CONFLICT (department_id, effective_date) DO UPDATE
               SET std_working_days = EXCLUDED.std_working_days,
                   hours_per_day = EXCLUDED.hours_per_day
               RETURNING id, department_id, std_working_days, hours_per_day, effective_date, created_at"#
        )
        .bind::<diesel::sql_types::Uuid, _>(department_id)
        .bind::<diesel::sql_types::Numeric, _>(std_wd)
        .bind::<diesel::sql_types::Numeric, _>(h_per_d)
        .bind::<diesel::sql_types::Date, _>(effective_date)
        .get_result(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn get_department_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let _workspace_id = auth_token.workspace_id();

        let result: Option<DepartmentPayrollConfigEntity> = crate::crud_sql_query!(
            &mut conn,
            r#"SELECT id, department_id, std_working_days, hours_per_day, effective_date, created_at
               FROM department_payroll_configs
               WHERE department_id = $1 AND effective_date <= $2
               ORDER BY effective_date DESC LIMIT 1"#,
        )
        .bind::<diesel::sql_types::Uuid, _>(department_id)
        .bind::<diesel::sql_types::Date, _>(date)
        .get_result(&mut *conn)
        .await
        .optional()
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }

    async fn get_department_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        department_id: Uuid,
    ) -> Result<Vec<DepartmentPayrollConfig>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;
        let workspace_id = auth_token.workspace_id();

        let results: Vec<DepartmentPayrollConfigEntity> = crate::crud_sql_query!(&mut conn,
            r#"SELECT dpc.id, dpc.department_id, dpc.std_working_days, dpc.hours_per_day, dpc.effective_date, dpc.created_at
               FROM department_payroll_configs dpc
               JOIN departments d ON d.id = dpc.department_id
               WHERE dpc.department_id = $1 AND d.workspace_id = $2
               ORDER BY dpc.effective_date DESC"#,
        )
        .bind::<diesel::sql_types::Uuid, _>(department_id)
        .bind::<diesel::sql_types::Uuid, _>(workspace_id)
        .load(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(results.into_iter().map(Into::into).collect())
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
        let _repo = DieselDepartmentRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
