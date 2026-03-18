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
pub use core_queries::*;
pub use recruitment_queries::*;
pub use analytics_queries::*;

use async_trait::async_trait;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Employee, EmployeePayrollConfig, EmploymentPeriod, RecruitmentSource,
    RecruitmentSourceBreakdown, TurnoverAnalytics, WorkingCondition,
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
#[allow(clippy::too_many_arguments)]
pub trait EmployeeRepository: super::PoolBacked + Send + Sync {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, sqlx::Error>;

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, sqlx::Error>;

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, sqlx::Error>;

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        search_query: Option<String>,
        page: i64,
        limit: i64,
        sort_by: Option<String>,
        sort_dir: Option<String>,
    ) -> Result<(Vec<Employee>, i64), sqlx::Error>;

    async fn get_active_employees(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<Employee>, sqlx::Error>;

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, sqlx::Error>;

    async fn get_paginated_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
        department_id: Option<Uuid>,
        page: i64,
        limit: i64,
        sort_desc: bool,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error>;

    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        user_id: Option<Uuid>,
        full_name: Option<String>,
        employee_id_number: Option<String>,
        hire_date: Option<NaiveDate>,
        termination_date: Option<NaiveDate>,
        tags: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<Option<Employee>, sqlx::Error>;

    async fn delete_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error>;

    async fn get_working_conditions(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, sqlx::Error>;

    async fn create_working_condition(
        &self,
        condition: &WorkingCondition,
    ) -> Result<(), sqlx::Error>;

    async fn delete_working_condition(
        &self,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, sqlx::Error>;

    async fn get_employment_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmploymentPeriod>, sqlx::Error>;

    async fn create_employment_period(
        &self,
        period: &EmploymentPeriod,
    ) -> Result<(), sqlx::Error>;

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, sqlx::Error>;

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, sqlx::Error>;

    async fn get_employee_payroll_config_at(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, sqlx::Error>;

    async fn set_employee_recruitment_source(
        &self,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error>;

    async fn create_recruitment_source(
        &self,
        source: &RecruitmentSource,
    ) -> Result<(), sqlx::Error>;

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, sqlx::Error>;

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, sqlx::Error>;

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: Option<String>,
        referrer_employee_id: Option<Uuid>,
        referral_bonus_amount: Option<Decimal>,
        referral_bonus_paid: Option<bool>,
        agency_contact_name: Option<String>,
        agency_contact_email: Option<String>,
        agency_fee_amount: Option<Decimal>,
        agency_fee_paid: Option<bool>,
        notes: Option<String>,
    ) -> Result<Option<RecruitmentSource>, sqlx::Error>;

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error>;

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, sqlx::Error>;

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, sqlx::Error>;

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, sqlx::Error>;

    async fn get_employee_id_by_user_id(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxEmployeeRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxEmployeeRepository {
    pub pool: PgPool,
}

impl SqlxEmployeeRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl EmployeeRepository for SqlxEmployeeRepository {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, sqlx::Error> {
        core_queries::find_by_number(&self.pool, workspace_id, employee_id_number).await
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, sqlx::Error> {
        core_queries::get_employee_by_id(&self.pool, id, workspace_id).await
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, sqlx::Error> {
        core_queries::get_employee_by_id_number(&self.pool, id_number, workspace_id).await
    }

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        search_query: Option<String>,
        page: i64,
        limit: i64,
        sort_by: Option<String>,
        sort_dir: Option<String>,
    ) -> Result<(Vec<Employee>, i64), sqlx::Error> {
        core_queries::get_all_employees(&self.pool, workspace_id, search_query, page, limit, sort_by, sort_dir).await
    }

    async fn get_active_employees(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<Employee>, sqlx::Error> {
        core_queries::get_active_employees(&self.pool, workspace_id).await
    }

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        core_queries::get_active_employee_ids(&self.pool, workspace_id, filter_ids).await
    }

    async fn get_paginated_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
        department_id: Option<Uuid>,
        page: i64,
        limit: i64,
        sort_desc: bool,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
        core_queries::get_paginated_employee_ids(
            &self.pool,
            workspace_id,
            filter_ids,
            department_id,
            page,
            limit,
            sort_desc,
        )
        .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        user_id: Option<Uuid>,
        full_name: Option<String>,
        employee_id_number: Option<String>,
        hire_date: Option<NaiveDate>,
        termination_date: Option<NaiveDate>,
        tags: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<Option<Employee>, sqlx::Error> {
        core_queries::update_employee(
            &self.pool,
            id,
            workspace_id,
            user_id,
            full_name,
            employee_id_number,
            hire_date,
            termination_date,
            tags,
            is_active,
        )
        .await
    }

    async fn delete_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        core_queries::delete_employee(&self.pool, id, workspace_id).await
    }

    async fn get_working_conditions(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, sqlx::Error> {
        core_queries::get_working_conditions(&self.pool, workspace_id, employee_id).await
    }

    async fn create_working_condition(
        &self,
        condition: &WorkingCondition,
    ) -> Result<(), sqlx::Error> {
        core_queries::create_working_condition(&self.pool, condition).await
    }

    async fn delete_working_condition(
        &self,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        core_queries::delete_working_condition(&self.pool, id, employee_id).await
    }

    async fn get_employment_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmploymentPeriod>, sqlx::Error> {
        core_queries::get_employment_history(&self.pool, workspace_id, employee_id).await
    }

    async fn create_employment_period(
        &self,
        period: &EmploymentPeriod,
    ) -> Result<(), sqlx::Error> {
        core_queries::create_employment_period(&self.pool, period).await
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, sqlx::Error> {
        core_queries::get_existing_employee_ids_by_numbers(&self.pool, workspace_id, id_numbers).await
    }

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, sqlx::Error> {
        core_queries::upsert_employee_payroll_config(
            &self.pool,
            employee_id,
            std_working_days,
            hours_per_day,
            effective_date,
        )
        .await
    }

    async fn get_employee_payroll_config_at(
        &self,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, sqlx::Error> {
        core_queries::get_employee_payroll_config_at(&self.pool, employee_id, date).await
    }

    async fn set_employee_recruitment_source(
        &self,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        recruitment_queries::set_employee_recruitment_source(&self.pool, employee_id, recruitment_source_id).await
    }

    async fn create_recruitment_source(
        &self,
        source: &RecruitmentSource,
    ) -> Result<(), sqlx::Error> {
        recruitment_queries::create_recruitment_source(&self.pool, source).await
    }

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, sqlx::Error> {
        recruitment_queries::get_recruitment_sources(&self.pool, workspace_id).await
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, sqlx::Error> {
        recruitment_queries::get_recruitment_source_by_id(&self.pool, id, workspace_id).await
    }

    #[allow(clippy::too_many_arguments)]
    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: Option<String>,
        referrer_employee_id: Option<Uuid>,
        referral_bonus_amount: Option<Decimal>,
        referral_bonus_paid: Option<bool>,
        agency_contact_name: Option<String>,
        agency_contact_email: Option<String>,
        agency_fee_amount: Option<Decimal>,
        agency_fee_paid: Option<bool>,
        notes: Option<String>,
    ) -> Result<Option<RecruitmentSource>, sqlx::Error> {
        recruitment_queries::update_recruitment_source(
            &self.pool,
            id,
            workspace_id,
            name,
            referrer_employee_id,
            referral_bonus_amount,
            referral_bonus_paid,
            agency_contact_name,
            agency_contact_email,
            agency_fee_amount,
            agency_fee_paid,
            notes,
        )
        .await
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        recruitment_queries::delete_recruitment_source(&self.pool, id, workspace_id).await
    }

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, sqlx::Error> {
        recruitment_queries::get_recruitment_source_breakdown(&self.pool, workspace_id).await
    }

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, sqlx::Error> {
        analytics_queries::get_turnover_analytics(&self.pool, workspace_id, start_date, end_date).await
    }

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        let depts: Vec<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT edh.department_id 
            FROM employees e
            JOIN employee_department_history edh ON e.id = edh.employee_id
            WHERE e.workspace_id = $1 AND e.user_id = $2
            "#
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
    ) -> Result<Option<Uuid>, sqlx::Error> {
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
    fn from_pool(pool: sqlx::PgPool) -> Self { Self::new(pool) }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
