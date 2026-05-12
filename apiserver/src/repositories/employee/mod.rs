//! Employee repository — split into sub-modules for maintainability.
//!
//! | File                    | Contents                                         |
//! |-------------------------|--------------------------------------------------|
//! | `core_queries.rs`       | CRUD, search, employment periods, payroll config |
//! | `recruitment_queries.rs`| Recruitment source CRUD + breakdown              |
//! | `analytics_queries.rs`  | Turnover analytics dashboard queries             |

mod analytics_queries;
mod core_queries;
mod recruitment_queries;

// Re-export free functions so existing callers (e.g. services) keep working.
pub use analytics_queries::*;
pub use core_queries::*;
pub use recruitment_queries::*;

use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Employee, EmployeePayrollConfig, RecruitmentSource, RecruitmentSourceBreakdown,
    TurnoverAnalytics, WorkingCondition,
};

/// Minimal employee info for paginated reports
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmployeeBasicInfo {
    pub id: Uuid,
    pub full_name: String,
    pub employee_id_number: String,
}

// ---------------------------------------------------------------------------
// Trait: EmployeeRepository — DIP abstraction over employee data access
// ---------------------------------------------------------------------------
#[async_trait]
pub trait EmployeeRepository: super::PoolBacked + Send + Sync {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError>;

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError>;

    async fn get_active_employees(&self, workspace_id: Uuid) -> Result<Vec<Employee>, AppError>;

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_paginated_employee_ids(
        &self,
        workspace_id: Uuid,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError>;

    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError>;

    async fn delete_employee(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, AppError>;

    async fn get_working_conditions(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError>;

    async fn create_working_condition(&self, condition: &WorkingCondition) -> Result<(), AppError>;

    async fn delete_working_condition(&self, id: Uuid, employee_id: Uuid)
        -> Result<bool, AppError>;

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError>;

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError>;

    async fn get_employee_payroll_config_at(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError>;

    async fn set_employee_recruitment_source(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError>;

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError>;

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, AppError>;

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError>;

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, AppError>;

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError>;

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError>;

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError>;

    async fn get_employee_id_by_user_id(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError>;
}

// ---------------------------------------------------------------------------
// SqlxEmployeeRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxEmployeeRepository {
    pub pool: PgPool,
}

impl SqlxEmployeeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmployeeRepository for SqlxEmployeeRepository {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        core_queries::find_by_number(&self.pool, workspace_id, employee_id_number)
            .await
            .map_err(Into::into)
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError> {
        core_queries::get_employee_by_id(&self.pool, id, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, AppError> {
        core_queries::get_employee_by_id_number(&self.pool, id_number, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        core_queries::get_all_employees(&self.pool, workspace_id, options)
            .await
            .map_err(Into::into)
    }

    async fn get_active_employees(&self, workspace_id: Uuid) -> Result<Vec<Employee>, AppError> {
        core_queries::get_active_employees(&self.pool, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        core_queries::get_active_employee_ids(&self.pool, workspace_id, filter_ids)
            .await
            .map_err(Into::into)
    }

    async fn get_paginated_employee_ids(
        &self,
        workspace_id: Uuid,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
        core_queries::get_paginated_employee_ids(&self.pool, workspace_id, args)
            .await
            .map_err(Into::into)
    }

    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        core_queries::update_employee(&self.pool, id, workspace_id, req)
            .await
            .map_err(Into::into)
    }

    async fn delete_employee(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, AppError> {
        core_queries::delete_employee(&self.pool, id, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_working_conditions(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        core_queries::get_working_conditions(&self.pool, workspace_id, employee_id)
            .await
            .map_err(Into::into)
    }

    async fn create_working_condition(&self, condition: &WorkingCondition) -> Result<(), AppError> {
        core_queries::create_working_condition(&self.pool, condition)
            .await
            .map_err(Into::into)
    }

    async fn delete_working_condition(
        &self,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, AppError> {
        core_queries::delete_working_condition(&self.pool, id, employee_id)
            .await
            .map_err(Into::into)
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError> {
        core_queries::get_existing_employee_ids_by_numbers(&self.pool, workspace_id, id_numbers)
            .await
            .map_err(Into::into)
    }

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError> {
        core_queries::upsert_employee_payroll_config(
            &self.pool,
            employee_id,
            std_working_days,
            hours_per_day,
            effective_date,
        )
        .await
        .map_err(Into::into)
    }

    async fn get_employee_payroll_config_at(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError> {
        core_queries::get_employee_payroll_config_at(&self.pool, employee_id, date)
            .await
            .map_err(Into::into)
    }

    async fn set_employee_recruitment_source(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        recruitment_queries::set_employee_recruitment_source(
            &self.pool,
            workspace_id,
            employee_id,
            recruitment_source_id,
        )
        .await
        .map_err(Into::into)
    }

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError> {
        recruitment_queries::create_recruitment_source(&self.pool, source)
            .await
            .map_err(Into::into)
    }

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, AppError> {
        recruitment_queries::get_recruitment_sources(&self.pool, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        recruitment_queries::get_recruitment_source_by_id(&self.pool, id, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        recruitment_queries::update_recruitment_source(&self.pool, id, workspace_id, req)
            .await
            .map_err(Into::into)
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, AppError> {
        recruitment_queries::delete_recruitment_source(&self.pool, id, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        recruitment_queries::get_recruitment_source_breakdown(&self.pool, workspace_id)
            .await
            .map_err(Into::into)
    }

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError> {
        analytics_queries::get_turnover_analytics(&self.pool, workspace_id, start_date, end_date)
            .await
            .map_err(Into::into)
    }

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let depts: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT edh.department_id 
            FROM employees e
            JOIN employee_department_history edh ON e.id = edh.employee_id
            WHERE e.workspace_id = $1 AND e.user_id = $2
            "#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(depts.into_iter().map(|(id,)| id).collect())
    }

    async fn get_employee_id_by_user_id(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        let employee: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM employees
            WHERE user_id = $1 AND workspace_id = $2 AND is_active = true
            "#,
        )
        .bind(user_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(employee.map(|(id,)| id))
    }
}

impl super::PoolBacked for SqlxEmployeeRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
