// Submodules
pub mod assignments;
mod crud;
mod ess;
mod lookup;
mod payroll;
mod recruitment;
mod search;
mod segments;
pub mod working_condition;

pub use assignments::*;

use crate::errors::AppError;
use crate::models::{Employee, UpdateEmployeeRequest};
use crate::repositories::employee::*;
use async_trait::async_trait;
use chrono::NaiveDate;
use uuid::Uuid;

pub struct DieselEmployeeRepository {
    diesel_pools: crate::diesel_pool::DieselPools,
    crypto_service: Option<std::sync::Arc<crate::crypto::CryptoService>>,
}

impl DieselEmployeeRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self {
            diesel_pools,
            crypto_service: None,
        }
    }

    pub fn new_with_crypto(
        diesel_pools: crate::diesel_pool::DieselPools,
        crypto_service: Option<std::sync::Arc<crate::crypto::CryptoService>>,
    ) -> Self {
        Self {
            diesel_pools,
            crypto_service,
        }
    }
}

// ── EmployeeCoreRepository delegation ──────────────────────────────────
//
// Methods are implemented in sub-modules (search, crud, segments, lookup)
// with an `_impl` suffix. This trait impl delegates to them.

#[async_trait]
impl EmployeeCoreRepository for DieselEmployeeRepository {
    async fn find_by_number(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        self.find_by_number_impl(auth_token, employee_id_number)
            .await
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        self.get_employee_by_id_impl(id, auth_token).await
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        self.get_employee_by_id_number_impl(id_number, auth_token)
            .await
    }

    async fn get_employees_by_ids(
        &self,
        ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        self.get_employees_by_ids_impl(ids, auth_token).await
    }

    async fn get_employees_by_id_numbers(
        &self,
        id_numbers: &[String],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        self.get_employees_by_id_numbers_impl(id_numbers, auth_token)
            .await
    }

    async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        self.get_all_employees_impl(auth_token, options).await
    }

    async fn get_active_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        self.get_active_employees_impl(auth_token).await
    }

    async fn get_active_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        self.get_active_employee_ids_impl(auth_token, filter_ids)
            .await
    }

    async fn get_paginated_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
        self.get_paginated_employee_ids_impl(auth_token, args).await
    }

    async fn get_employee_departments_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
        self.get_employee_departments_at_impl(auth_token, date)
            .await
    }

    async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        self.update_employee_impl(id, auth_token, req).await
    }

    async fn update_employee_transactional(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        self.update_employee_transactional_impl(id, auth_token, req)
            .await
    }

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError> {
        self.create_employee_impl(employee).await
    }

    async fn create_employee_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        hire_date: chrono::NaiveDate,
        recruitment_source_id: Option<Uuid>,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        self.create_employee_transactional_impl(
            auth_token,
            employee,
            hire_date,
            recruitment_source_id,
            employer_id,
            primary_location_id,
            department_id,
        )
        .await
    }

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError> {
        self.create_employee_skip_duplicates_impl(employee).await
    }

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: NaiveDate,
        employer_id: Uuid,
        primary_location_id: Option<Uuid>,
        department_id: Option<Uuid>,
    ) -> Result<Option<Employee>, AppError> {
        self.rehire_employee_impl(
            auth_token,
            employee_id,
            hire_date,
            employer_id,
            primary_location_id,
            department_id,
        )
        .await
    }

    async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        self.delete_employee_impl(id, auth_token).await
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError> {
        self.get_existing_employee_ids_by_numbers_impl(auth_token, id_numbers)
            .await
    }

    async fn get_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        self.get_employee_ids_by_numbers_impl(auth_token, id_numbers)
            .await
    }

    async fn bulk_create_employees_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employees: &[Employee],
    ) -> Result<(), AppError> {
        self.bulk_create_employees_skip_duplicates_impl(auth_token, employees)
            .await
    }

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        self.get_managed_departments_impl(auth_token, user_id).await
    }

    async fn get_employee_id_by_user_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        self.get_employee_id_by_user_id_impl(auth_token, user_id)
            .await
    }

    async fn get_employee_timezone_fallback(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<chrono_tz::Tz, AppError> {
        self.get_employee_timezone_fallback_impl(auth_token, employee_id)
            .await
    }

    async fn get_employee_timezone_at_date(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        tentative_date: chrono::NaiveDate,
    ) -> Result<chrono_tz::Tz, AppError> {
        self.get_employee_timezone_at_date_impl(auth_token, employee_id, tentative_date)
            .await
    }

    async fn get_headquarters_location_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        self.get_headquarters_location_id_impl(auth_token).await
    }
}

pub const EMPLOYEE_SELECT_QUERY: &str = r"
    SELECT
        e.id, e.workspace_id, e.user_id, e.full_name, e.employee_id_number,
        e.tags, e.created_at, e.updated_at,
        d_cur.name as department_name, emp_cur.primary_location_id, emp_cur.employer_id,
        CASE WHEN emp_cur.status = 'active' THEN emp_cur.id ELSE NULL END as active_employment_id
    FROM employees e
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, employer_id, primary_location_id, department_id, id, status
        FROM employments
        WHERE deleted_at IS NULL
        ORDER BY employee_id, (status = 'active') DESC, start_date DESC, id DESC
    ) emp_cur ON e.id = emp_cur.employee_id
    LEFT JOIN departments d_cur ON emp_cur.department_id = d_cur.id
";

pub const EMPLOYEE_DEPT_JOIN: &str = r"
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, department_id, status
        FROM employments
        WHERE deleted_at IS NULL
        ORDER BY employee_id, (status = 'active') DESC, start_date DESC, id DESC
    ) emp_sort ON e.id = emp_sort.employee_id
    LEFT JOIN departments d_sort ON emp_sort.department_id = d_sort.id AND d_sort.workspace_id = e.workspace_id
";

struct EmployeeSearchSql {
    pub where_clause: String,
    pub order_clause: String,
    pub has_search: bool,
    pub needs_dept_join: bool,
}

fn build_employee_search_sql(
    search_query: Option<&str>,
    sort_by: Option<&str>,
    sort_dir: Option<&str>,
) -> EmployeeSearchSql {
    let has_search = search_query.is_some_and(|q| !q.trim().is_empty());

    let mut where_clause = "WHERE e.workspace_id = $1 AND e.deleted_at IS NULL".to_string();
    if has_search {
        where_clause.push_str(" AND (e.full_name ILIKE $2 OR e.employee_id_number ILIKE $2)");
    }

    let order_clause = if has_search {
        "ORDER BY \
            (e.employee_id_number = $5) DESC, \
            (e.employee_id_number ILIKE $5 || '%') DESC, \
            (e.full_name ILIKE $5) DESC, \
            (e.full_name ILIKE $5 || '%') DESC, \
            e.full_name ASC"
            .to_string()
    } else {
        let dir = match sort_dir {
            Some(d) if d.eq_ignore_ascii_case("desc") => "DESC",
            _ => "ASC",
        };
        match sort_by {
            Some("employee_id_number") => format!(
                "ORDER BY (CASE WHEN e.employee_id_number ~ '^[0-9]+$' \
                THEN CAST(e.employee_id_number AS NUMERIC) END) {dir} NULLS LAST, \
                e.employee_id_number {dir}"
            ),
            Some("department") => {
                format!("ORDER BY COALESCE(d_sort.name, '') {dir}, e.full_name ASC")
            }
            Some("status") => {
                format!("ORDER BY COALESCE(emp_cur.status, '') {dir}, e.full_name ASC")
            }
            _ => format!("ORDER BY e.full_name {dir}"),
        }
    };

    let needs_dept_join = !has_search && sort_by == Some("department");

    EmployeeSearchSql {
        where_clause,
        order_clause,
        has_search,
        needs_dept_join,
    }
}

// EmployeeRepository is now a supertrait with a blanket impl.
// DieselEmployeeRepository automatically implements EmployeeRepository because
// it implements all sub-traits: EmployeeCoreRepository, EmployeeWorkingConditionRepository,
// EmployeePayrollConfigRepository, EmployeeRecruitmentRepository,
// EmployeeSelfServiceRepository, and EmployeeAssignmentRepository.
// No manual delegation code is needed.
