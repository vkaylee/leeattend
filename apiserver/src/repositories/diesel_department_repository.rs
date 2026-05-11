use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::diesel::department::{
    DieselDepartment, NewDieselDepartment, UpdateDieselDepartment,
};
use crate::models::{Department, DepartmentPayrollConfig, EmployeeDepartmentHistory};
use crate::repositories::department_repository::DepartmentRepository;
use crate::schema::departments;

pub struct DieselDepartmentRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselDepartmentRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl DepartmentRepository for DieselDepartmentRepository {
    async fn create_department(
        &self,
        workspace_id: Uuid,
        name: String,
        description: Option<String>,
        manager_id: Option<Uuid>,
        default_working_pattern_id: Option<Uuid>,
    ) -> Result<Department, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

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
            .get_result(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(result.into())
    }

    async fn get_department_by_id(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Department>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let dept: Option<DieselDepartment> = departments::table
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(dept.map(Into::into))
    }

    async fn update_department(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: crate::models::UpdateDepartmentRequest,
    ) -> Result<Department, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let update_model = UpdateDieselDepartment {
            name: req.name,
            description: req.description,
            manager_id: req.manager_id,
            default_working_pattern_id: req.default_working_pattern_id,
        };

        let result: DieselDepartment = diesel::update(departments::table)
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .set(&update_model)
            .get_result(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(result.into())
    }

    async fn delete_department(&self, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        diesel::delete(departments::table)
            .filter(departments::id.eq(id))
            .filter(departments::workspace_id.eq(workspace_id))
            .execute(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(())
    }

    async fn get_departments(&self, workspace_id: Uuid) -> Result<Vec<Department>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let results: Vec<DieselDepartment> = departments::table
            .filter(departments::workspace_id.eq(workspace_id))
            .order(departments::name.asc())
            .load(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn transfer_employee(
        &self,
        employee_id: Uuid,
        department_id: Uuid,
        effective_date: NaiveDate,
        notes: Option<String>,
    ) -> Result<EmployeeDepartmentHistory, sqlx::Error> {
        // We will keep this with SQLx for now since it's just a PoC and requires complex inserts
        crate::repositories::department_repository::transfer_employee(
            &self.sqlx_pool,
            employee_id,
            department_id,
            effective_date,
            notes,
        )
        .await
    }

    async fn get_employee_department_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeeDepartmentHistory>, sqlx::Error> {
        crate::repositories::department_repository::get_employee_department_history(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
        )
        .await
    }

    async fn get_current_department(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Option<EmployeeDepartmentHistory>, sqlx::Error> {
        crate::repositories::department_repository::get_current_department(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
        )
        .await
    }

    async fn upsert_department_payroll_config(
        &self,
        department_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<DepartmentPayrollConfig, sqlx::Error> {
        crate::repositories::department_repository::upsert_department_payroll_config(
            &self.sqlx_pool,
            department_id,
            std_working_days,
            hours_per_day,
            effective_date,
        )
        .await
    }

    async fn get_department_payroll_config_at(
        &self,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<DepartmentPayrollConfig>, sqlx::Error> {
        crate::repositories::department_repository::get_department_payroll_config_at(
            &self.sqlx_pool,
            department_id,
            date,
        )
        .await
    }
}

impl crate::repositories::PoolBacked for DieselDepartmentRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!(
            "DieselDepartmentRepository requires diesel_pools to be manually injected via new()"
        );
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
