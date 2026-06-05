#![allow(unused_imports)]
use crate::errors::AppError;
use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::models::diesel::employee::{
    CountRow, DeptRow, EmployeeDocumentEntity, EmployeeEmergencyContactEntity, EmployeeEntity,
    EmployeeProfileEntity, EmployeeRow, RecruitmentSourceBreakdownRow, RecruitmentSourceEntity,
    TenureRow, UpdateDieselEmployee,
};
use crate::models::{
    CreateEmergencyContactRequest, Employee, EmployeeDocument, EmployeeEmergencyContact,
    EmployeeFilterOptions, EmployeePaginationArgs, EmployeePayrollConfig, EmployeeProfile,
    RecruitmentSource, RecruitmentSourceBreakdown, TurnoverAnalytics, TurnoverByDepartment,
    TurnoverBySource, TurnoverTenureBucket, UpdateEmployeeProfileRequest, UpdateEmployeeRequest,
    WorkingCondition,
};
use crate::repositories::employee::*;
use crate::schema::{
    employee_documents, employee_emergency_contacts, employee_profiles, employees,
    recruitment_sources,
};
use diesel::sql_types::{Date, Uuid as UuidType};

pub struct DieselEmployeeRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselEmployeeRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

pub const EMPLOYEE_SELECT_QUERY: &str = r"
    SELECT
        e.id, e.workspace_id, e.user_id, e.full_name, e.employee_id_number,
        e.original_hire_date, e.hire_date, e.termination_date,
        e.tags, e.is_active, e.created_at, e.updated_at,
        d_cur.name as department_name, e.primary_location_id, e.employer_id
    FROM employees e
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, department_id
        FROM employee_department_history
        ORDER BY employee_id, effective_date DESC
    ) edh_cur ON e.id = edh_cur.employee_id
    LEFT JOIN departments d_cur ON edh_cur.department_id = d_cur.id
";

pub const EMPLOYEE_DEPT_JOIN: &str = r"
    LEFT JOIN (
        SELECT DISTINCT ON (employee_id) employee_id, department_id
        FROM employee_department_history
        ORDER BY employee_id, effective_date DESC
    ) edh_sort ON e.id = edh_sort.employee_id
    LEFT JOIN departments d_sort ON edh_sort.department_id = d_sort.id AND d_sort.workspace_id = e.workspace_id
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
            Some("status") => format!("ORDER BY e.is_active {dir}, e.full_name ASC"),
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

mod core;
mod ess;
mod payroll;
mod recruitment;
pub mod working_condition;

pub use core::*;
pub use ess::*;
pub use payroll::*;
pub use recruitment::*;
pub use working_condition::*;

pub mod assignments;
pub use assignments::*;

// EmployeeRepository is now a supertrait with a blanket impl.
// DieselEmployeeRepository automatically implements EmployeeRepository because
// it implements all sub-traits: EmployeeCoreRepository, EmployeeWorkingConditionRepository,
// EmployeePayrollConfigRepository, EmployeeRecruitmentRepository,
// EmployeeSelfServiceRepository, and EmployeeAssignmentRepository.
// No manual delegation code is needed.
