use super::mock_repo::MockEmployeeRepo;
use crate::errors::AppError;
use crate::models::{Employee, EmployeeFilterOptions, EmployeePaginationArgs};
use crate::repositories::employee::{EmployeeBasicInfo, EmployeeCoreRepository};
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

#[async_trait]
impl EmployeeCoreRepository for MockEmployeeRepo {
    async fn find_by_number(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _n: &str,
    ) -> Result<Option<Employee>, AppError> {
        Ok(self.employee.lock().expect("lock").clone())
    }
    async fn get_employee_by_id(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        Ok(self.employee.lock().expect("lock").clone())
    }
    async fn get_employee_by_id_number(
        &self,
        _n: &str,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        Ok(self.employee.lock().expect("lock").clone())
    }
    async fn get_employees_by_ids(
        &self,
        _i: &[Uuid],
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        Ok(vec![])
    }
    async fn get_all_employees(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _o: EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        let emps = self.employees.lock().expect("lock").clone();
        let total = *self.total.lock().expect("lock");
        Ok((emps, total))
    }
    async fn get_active_employees(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        Ok(self.employees.lock().expect("lock").clone())
    }
    async fn get_active_employee_ids(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _f: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        Ok(vec![])
    }
    async fn get_paginated_employee_ids(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _ar: EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
        Ok((vec![], 0))
    }
    async fn get_employee_departments_at(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _d: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn update_employee(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
        _r: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        Ok(self.updated_employee.lock().expect("lock").clone())
    }
    async fn update_employee_transactional(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
        _r: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        Ok(self.updated_employee.lock().expect("lock").clone())
    }
    async fn create_employee(&self, _e: &Employee) -> Result<(), AppError> {
        Ok(())
    }
    async fn create_employee_transactional(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &Employee,
        _h: NaiveDate,
        _rc: Option<Uuid>,
        _em: Uuid,
        _p: Option<Uuid>,
        _d: Option<Uuid>,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn create_employee_skip_duplicates(&self, _e: &Employee) -> Result<bool, AppError> {
        Ok(true)
    }
    async fn rehire_employee(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _h: NaiveDate,
        _em: Uuid,
        _p: Option<Uuid>,
        _d: Option<Uuid>,
    ) -> Result<Option<Employee>, AppError> {
        Ok(self.updated_employee.lock().expect("lock").clone())
    }
    async fn delete_employee(
        &self,
        _i: Uuid,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        Ok(*self.delete_emp_rows.lock().expect("lock"))
    }
    async fn get_existing_employee_ids_by_numbers(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _n: &[String],
    ) -> Result<Vec<String>, AppError> {
        Ok(vec![])
    }
    async fn get_employee_ids_by_numbers(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _id: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        Ok(std::collections::HashMap::new())
    }
    async fn bulk_create_employees_skip_duplicates(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: &[Employee],
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn get_managed_departments(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _u: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        Ok(vec![])
    }
    async fn get_employee_id_by_user_id(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _u: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        Ok(*self.employee_id_by_user_id.lock().expect("lock"))
    }
    async fn get_employee_timezone_fallback(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
    ) -> Result<chrono_tz::Tz, AppError> {
        Ok(chrono_tz::UTC)
    }
    async fn get_employee_timezone_at_date(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
        _e: Uuid,
        _t: NaiveDate,
    ) -> Result<chrono_tz::Tz, AppError> {
        Ok(chrono_tz::UTC)
    }
    async fn get_headquarters_location_id(
        &self,
        _a: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        Ok(None)
    }
}
