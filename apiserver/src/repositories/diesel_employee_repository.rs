use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::diesel::employee::{EmployeeEntity, UpdateDieselEmployee};
use crate::models::{
    Employee, EmployeeFilterOptions, EmployeePaginationArgs, EmployeePayrollConfig,
    RecruitmentSource, RecruitmentSourceBreakdown, TurnoverAnalytics, UpdateEmployeeRequest,
    WorkingCondition,
};
use crate::repositories::employee::{self, EmployeeBasicInfo, EmployeeRepository};
use crate::schema::employees;

pub struct DieselEmployeeRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselEmployeeRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl EmployeeRepository for DieselEmployeeRepository {
    async fn find_by_number(
        &self,
        workspace_id: Uuid,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let normalized_id = crate::utils::normalize_employee_id(employee_id_number);
        let emp: Option<EmployeeEntity> = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::employee_id_number.eq(normalized_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(emp.map(Into::into))
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let emp: Option<EmployeeEntity> = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::id.eq(id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(emp.map(Into::into))
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        workspace_id: Uuid,
    ) -> Result<Option<Employee>, sqlx::Error> {
        self.find_by_number(workspace_id, id_number).await
    }

    async fn get_all_employees(
        &self,
        workspace_id: Uuid,
        options: EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), sqlx::Error> {
        employee::get_all_employees(&self.sqlx_pool, workspace_id, options).await
    }

    async fn get_active_employees(&self, workspace_id: Uuid) -> Result<Vec<Employee>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let emps: Vec<EmployeeEntity> = employees::table
            .filter(employees::workspace_id.eq(workspace_id))
            .filter(employees::is_active.eq(true))
            .order(employees::full_name.asc())
            .load(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(emps.into_iter().map(Into::into).collect())
    }

    async fn get_active_employee_ids(
        &self,
        workspace_id: Uuid,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        employee::get_active_employee_ids(&self.sqlx_pool, workspace_id, filter_ids).await
    }

    async fn get_paginated_employee_ids(
        &self,
        workspace_id: Uuid,
        args: EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), sqlx::Error> {
        employee::get_paginated_employee_ids(&self.sqlx_pool, workspace_id, args).await
    }

    async fn update_employee(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let normalized_id = req
            .employee_id_number
            .as_ref()
            .map(|id| id.as_str().to_string());

        let tags_mapped = req.tags.as_ref().map(|tags| {
            tags.iter()
                .map(|t| Some(t.clone()))
                .collect::<Vec<Option<String>>>()
        });

        let update_model = UpdateDieselEmployee {
            user_id: req.user_id,
            full_name: req.full_name.clone(),
            employee_id_number: normalized_id,
            tags: tags_mapped,
            is_active: req.is_active,
            hire_date: req.hire_date,
            termination_date: req.termination_date.map(Some),
            updated_at: Some(chrono::Utc::now()),
        };

        let result: Option<EmployeeEntity> = diesel::update(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .set(&update_model)
            .get_result(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(result.map(Into::into))
    }

    async fn delete_employee(&self, id: Uuid, workspace_id: Uuid) -> Result<u64, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let affected = diesel::delete(employees::table)
            .filter(employees::id.eq(id))
            .filter(employees::workspace_id.eq(workspace_id))
            .execute(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(affected as u64)
    }

    async fn get_working_conditions(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, sqlx::Error> {
        employee::get_working_conditions(&self.sqlx_pool, workspace_id, employee_id).await
    }

    async fn create_working_condition(
        &self,
        condition: &WorkingCondition,
    ) -> Result<(), sqlx::Error> {
        employee::create_working_condition(&self.sqlx_pool, condition).await
    }

    async fn delete_working_condition(
        &self,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        employee::delete_working_condition(&self.sqlx_pool, id, employee_id).await
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        workspace_id: Uuid,
        id_numbers: &[String],
    ) -> Result<Vec<String>, sqlx::Error> {
        employee::get_existing_employee_ids_by_numbers(&self.sqlx_pool, workspace_id, id_numbers)
            .await
    }

    async fn upsert_employee_payroll_config(
        &self,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, sqlx::Error> {
        employee::upsert_employee_payroll_config(
            &self.sqlx_pool,
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
        employee::get_employee_payroll_config_at(&self.sqlx_pool, employee_id, date).await
    }

    async fn set_employee_recruitment_source(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        employee::set_employee_recruitment_source(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
            recruitment_source_id,
        )
        .await
    }

    async fn create_recruitment_source(
        &self,
        source: &RecruitmentSource,
    ) -> Result<(), sqlx::Error> {
        employee::create_recruitment_source(&self.sqlx_pool, source).await
    }

    async fn get_recruitment_sources(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSource>, sqlx::Error> {
        employee::get_recruitment_sources(&self.sqlx_pool, workspace_id).await
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<RecruitmentSource>, sqlx::Error> {
        employee::get_recruitment_source_by_id(&self.sqlx_pool, id, workspace_id).await
    }

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, sqlx::Error> {
        employee::update_recruitment_source(&self.sqlx_pool, id, workspace_id, req).await
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<u64, sqlx::Error> {
        employee::delete_recruitment_source(&self.sqlx_pool, id, workspace_id).await
    }

    async fn get_recruitment_source_breakdown(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, sqlx::Error> {
        employee::get_recruitment_source_breakdown(&self.sqlx_pool, workspace_id).await
    }

    async fn get_turnover_analytics(
        &self,
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, sqlx::Error> {
        employee::get_turnover_analytics(&self.sqlx_pool, workspace_id, start_date, end_date).await
    }

    async fn get_managed_departments(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, sqlx::Error> {
        // Fallback to SqlxEmployeeRepository implementation
        let repo =
            crate::repositories::employee::SqlxEmployeeRepository::new(self.sqlx_pool.clone());
        repo.get_managed_departments(workspace_id, user_id).await
    }

    async fn get_employee_id_by_user_id(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let repo =
            crate::repositories::employee::SqlxEmployeeRepository::new(self.sqlx_pool.clone());
        repo.get_employee_id_by_user_id(workspace_id, user_id).await
    }
}

impl crate::repositories::PoolBacked for DieselEmployeeRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!("DieselEmployeeRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
