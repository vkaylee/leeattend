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
        d_cur.name as department_name, e.primary_location_id
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

    let mut where_clause = "WHERE e.workspace_id = $1".to_string();
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
mod working_condition;

pub use core::*;
pub use ess::*;
pub use payroll::*;
pub use recruitment::*;
pub use working_condition::*;

#[async_trait]
#[async_trait]
impl EmployeeRepository for DieselEmployeeRepository {
    async fn find_by_number(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id_number: &str,
    ) -> Result<Option<Employee>, AppError> {
        EmployeeCoreRepository::find_by_number(self, auth_token, employee_id_number).await
    }

    async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        EmployeeCoreRepository::get_employee_by_id(self, id, auth_token).await
    }

    async fn get_employee_by_id_number(
        &self,
        id_number: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Employee>, AppError> {
        EmployeeCoreRepository::get_employee_by_id_number(self, id_number, auth_token).await
    }

    async fn get_employees_by_ids(
        &self,
        ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        EmployeeCoreRepository::get_employees_by_ids(self, ids, auth_token).await
    }

    async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        EmployeeCoreRepository::get_all_employees(self, auth_token, options).await
    }

    async fn get_active_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Employee>, AppError> {
        EmployeeCoreRepository::get_active_employees(self, auth_token).await
    }

    async fn get_active_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        filter_ids: Option<&Vec<Uuid>>,
    ) -> Result<Vec<Uuid>, AppError> {
        EmployeeCoreRepository::get_active_employee_ids(self, auth_token, filter_ids).await
    }

    async fn get_paginated_employee_ids(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        args: crate::models::EmployeePaginationArgs<'_>,
    ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
        EmployeeCoreRepository::get_paginated_employee_ids(self, auth_token, args).await
    }

    async fn get_employee_departments_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
        EmployeeCoreRepository::get_employee_departments_at(self, auth_token, date).await
    }

    async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateEmployeeRequest,
    ) -> Result<Option<Employee>, AppError> {
        EmployeeCoreRepository::update_employee(self, id, auth_token, req).await
    }

    async fn create_employee(&self, employee: &Employee) -> Result<(), AppError> {
        EmployeeCoreRepository::create_employee(self, employee).await
    }

    async fn create_employee_transactional(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee: &Employee,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        EmployeeCoreRepository::create_employee_transactional(
            self,
            auth_token,
            employee,
            recruitment_source_id,
        )
        .await
    }

    async fn create_employee_skip_duplicates(&self, employee: &Employee) -> Result<bool, AppError> {
        EmployeeCoreRepository::create_employee_skip_duplicates(self, employee).await
    }

    async fn rehire_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        hire_date: NaiveDate,
    ) -> Result<Option<Employee>, AppError> {
        EmployeeCoreRepository::rehire_employee(self, auth_token, employee_id, hire_date).await
    }

    async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        EmployeeCoreRepository::delete_employee(self, id, auth_token).await
    }

    async fn get_existing_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<Vec<String>, AppError> {
        EmployeeCoreRepository::get_existing_employee_ids_by_numbers(self, auth_token, id_numbers)
            .await
    }

    async fn get_employee_ids_by_numbers(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id_numbers: &[String],
    ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
        EmployeeCoreRepository::get_employee_ids_by_numbers(self, auth_token, id_numbers).await
    }

    async fn bulk_create_employees_skip_duplicates(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employees: &[Employee],
    ) -> Result<(), AppError> {
        EmployeeCoreRepository::bulk_create_employees_skip_duplicates(self, auth_token, employees)
            .await
    }

    async fn get_managed_departments(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        EmployeeCoreRepository::get_managed_departments(self, auth_token, user_id).await
    }

    async fn get_employee_id_by_user_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, AppError> {
        EmployeeCoreRepository::get_employee_id_by_user_id(self, auth_token, user_id).await
    }

    async fn get_employee_timezone_fallback(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<chrono_tz::Tz, AppError> {
        EmployeeCoreRepository::get_employee_timezone_fallback(self, auth_token, employee_id).await
    }

    async fn get_headquarters_location_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Uuid>, AppError> {
        EmployeeCoreRepository::get_headquarters_location_id(self, auth_token).await
    }

    async fn get_working_conditions(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        EmployeeWorkingConditionRepository::get_working_conditions(self, auth_token, employee_id)
            .await
    }

    async fn create_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition: &WorkingCondition,
    ) -> Result<(), AppError> {
        EmployeeWorkingConditionRepository::create_working_condition(self, auth_token, condition)
            .await
    }

    async fn delete_working_condition(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        id: Uuid,
        employee_id: Uuid,
    ) -> Result<bool, AppError> {
        EmployeeWorkingConditionRepository::delete_working_condition(
            self,
            auth_token,
            id,
            employee_id,
        )
        .await
    }

    async fn get_working_conditions_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Vec<WorkingCondition>, AppError> {
        EmployeeWorkingConditionRepository::get_working_conditions_at(
            self,
            auth_token,
            employee_id,
            date,
        )
        .await
    }

    async fn get_working_conditions_batch(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        EmployeeWorkingConditionRepository::get_working_conditions_batch(
            self,
            auth_token,
            employee_ids,
            start_date,
            end_date,
        )
        .await
    }

    async fn upsert_employee_payroll_config(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        std_working_days: Decimal,
        hours_per_day: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeePayrollConfig, AppError> {
        EmployeePayrollConfigRepository::upsert_employee_payroll_config(
            self,
            auth_token,
            employee_id,
            std_working_days,
            hours_per_day,
            effective_date,
        )
        .await
    }

    async fn get_employee_payroll_config_at(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<EmployeePayrollConfig>, AppError> {
        EmployeePayrollConfigRepository::get_employee_payroll_config_at(
            self,
            auth_token,
            employee_id,
            date,
        )
        .await
    }

    async fn get_employee_payroll_configs(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<EmployeePayrollConfig>, AppError> {
        EmployeePayrollConfigRepository::get_employee_payroll_configs(self, auth_token, employee_id)
            .await
    }

    async fn set_employee_recruitment_source(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        recruitment_source_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        EmployeeRecruitmentRepository::set_employee_recruitment_source(
            self,
            auth_token,
            employee_id,
            recruitment_source_id,
        )
        .await
    }

    async fn create_recruitment_source(&self, source: &RecruitmentSource) -> Result<(), AppError> {
        EmployeeRecruitmentRepository::create_recruitment_source(self, source).await
    }

    async fn get_recruitment_sources(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSource>, AppError> {
        EmployeeRecruitmentRepository::get_recruitment_sources(self, auth_token).await
    }

    async fn get_recruitment_source_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        EmployeeRecruitmentRepository::get_recruitment_source_by_id(self, id, auth_token).await
    }

    async fn update_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::UpdateRecruitmentSourceRequest,
    ) -> Result<Option<RecruitmentSource>, AppError> {
        EmployeeRecruitmentRepository::update_recruitment_source(self, id, auth_token, req).await
    }

    async fn delete_recruitment_source(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        EmployeeRecruitmentRepository::delete_recruitment_source(self, id, auth_token).await
    }

    async fn get_recruitment_source_breakdown(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
        EmployeeRecruitmentRepository::get_recruitment_source_breakdown(self, auth_token).await
    }

    async fn get_turnover_analytics(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<TurnoverAnalytics, AppError> {
        EmployeeRecruitmentRepository::get_turnover_analytics(
            self, auth_token, start_date, end_date,
        )
        .await
    }

    async fn get_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Option<crate::models::EmployeeProfile>, AppError> {
        EmployeeSelfServiceRepository::get_employee_profile(self, auth_token, employee_id).await
    }

    async fn upsert_employee_profile(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &crate::models::UpdateEmployeeProfileRequest,
    ) -> Result<crate::models::EmployeeProfile, AppError> {
        EmployeeSelfServiceRepository::upsert_employee_profile(self, auth_token, employee_id, req)
            .await
    }

    async fn get_emergency_contacts(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeEmergencyContact>, AppError> {
        EmployeeSelfServiceRepository::get_emergency_contacts(self, auth_token, employee_id).await
    }

    async fn upsert_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        req: &crate::models::CreateEmergencyContactRequest,
    ) -> Result<crate::models::EmployeeEmergencyContact, AppError> {
        EmployeeSelfServiceRepository::upsert_emergency_contact(self, auth_token, employee_id, req)
            .await
    }

    async fn delete_emergency_contact(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        contact_id: Uuid,
    ) -> Result<u64, AppError> {
        EmployeeSelfServiceRepository::delete_emergency_contact(
            self,
            auth_token,
            employee_id,
            contact_id,
        )
        .await
    }

    async fn get_employee_documents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
    ) -> Result<Vec<crate::models::EmployeeDocument>, AppError> {
        EmployeeSelfServiceRepository::get_employee_documents(self, auth_token, employee_id).await
    }

    async fn create_employee_document(
        &self,
        doc: &crate::models::EmployeeDocument,
    ) -> Result<(), AppError> {
        EmployeeSelfServiceRepository::create_employee_document(self, doc).await
    }
}

// TESTS ARE MOVED/COMMENTED OUT FOR NOW
