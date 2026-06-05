use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::utils::id::new_id;

use crate::errors::AppError;
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, RehireEmployeeRequest,
    UpdateEmployeeRequest, WorkingCondition,
};
use crate::repositories::{department_repository::DepartmentRepository, EmployeeRepository};

pub struct EmployeeService {
    repo: Arc<dyn EmployeeRepository>,
    dept_repo: Arc<dyn DepartmentRepository>,
    user_repo: Arc<dyn crate::repositories::user_repository::UserRepository>,
    staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
}

impl EmployeeService {
    pub fn new(
        repo: Arc<dyn EmployeeRepository>,
        dept_repo: Arc<dyn DepartmentRepository>,
        user_repo: Arc<dyn crate::repositories::user_repository::UserRepository>,
        staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    ) -> Self {
        Self {
            repo,
            dept_repo,
            user_repo,
            staging_repo,
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_working_conditions(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        page: i64,
        limit: i64,
    ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let emp = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        self.repo
            .get_working_conditions(auth_token, emp.id(), page, limit)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_working_condition(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: CreateWorkingConditionRequest,
    ) -> Result<WorkingCondition, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let emp = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        let condition = WorkingCondition::new(
            new_id(),
            emp.id(),
            req.type_,
            req.minute_allowance,
            req.is_paid,
            req.start_date,
            req.end_date,
        );

        self.repo
            .create_working_condition(auth_token, &condition)
            .await?;

        Ok(condition)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_working_condition(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        condition_id: Uuid,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let emp = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        let deleted = self
            .repo
            .delete_working_condition(auth_token, condition_id, emp.id())
            .await?;

        if !deleted {
            return Err(AppError::NotFound("Working condition not found".into()));
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_employee(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: CreateEmployeeRequest,
    ) -> Result<Employee, AppError> {
        let workspace_id = auth_token.workspace_id();
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            self.user_repo
                .get_user_by_id(user_id)
                .await?
                .ok_or(AppError::Validation("User not found".into()))?;

            // Enforce 1-to-1 User/Employee link in this workspace
            if self
                .repo
                .get_employee_id_by_user_id(auth_token, user_id)
                .await?
                .is_some()
            {
                return Err(AppError::Validation(
                    "User is already linked to another employee in this workspace".into(),
                ));
            }
        }

        let employee = Employee::new(
            new_id(),
            workspace_id,
            req.user_id,
            req.full_name,
            req.employee_id_number.into_inner(),
            req.hire_date,
            req.hire_date,
            None,
            req.tags.unwrap_or_default(),
            true,
            Utc::now(),
            Utc::now(),
            None,
            req.primary_location_id,
            req.employer_id,
        );

        // Use repository to create employee and set recruitment source transactionally
        self.repo
            .create_employee_transactional(auth_token, &employee, req.recruitment_source_id)
            .await?;

        // Claim staged logs asynchronously (best-effort, non-fatal if fails)
        if let Err(e) = self
            .staging_repo
            .claim_staged_logs(workspace_id, employee.employee_id_number(), employee.id())
            .await
        {
            tracing::warn!(
                "Failed to claim staged logs for newly created employee {}: {}",
                employee.id(),
                e
            );
        }

        Ok(employee)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn rehire_employee(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: RehireEmployeeRequest,
    ) -> Result<Employee, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let employee = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        if employee.is_active() {
            return Err(AppError::Validation("Employee is already active".into()));
        }

        if let Some(term_date) = employee.termination_date() {
            if req.hire_date <= term_date {
                return Err(AppError::Validation(
                    "Rehire date must be after termination date".into(),
                ));
            }
        }

        self.repo
            .rehire_employee(auth_token, employee_id, req.hire_date)
            .await?
            .ok_or(AppError::NotFound("Employee not found after rehire".into()))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_employee_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Employee, AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo
            .get_employee_by_id(id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_all_employees(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        options: crate::models::EmployeeFilterOptions,
    ) -> Result<(Vec<Employee>, i64), AppError> {
        let _workspace_id = auth_token.workspace_id();
        self.repo.get_all_employees(auth_token, options).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: UpdateEmployeeRequest,
    ) -> Result<Employee, AppError> {
        let workspace_id = auth_token.workspace_id();
        // Optional: Check if user_id exists if provided
        if let Some(user_id) = req.user_id {
            self.user_repo
                .get_user_by_id(user_id)
                .await?
                .ok_or(AppError::Validation("User not found".into()))?;

            // Enforce 1-to-1 User/Employee link in this workspace
            if let Some(existing_emp_id) = self
                .repo
                .get_employee_id_by_user_id(auth_token, user_id)
                .await?
            {
                if existing_emp_id != id {
                    return Err(AppError::Validation(
                        "User is already linked to another employee in this workspace".into(),
                    ));
                }
            }
        }

        let employee = self
            .repo
            .update_employee(id, auth_token, &req)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        // Update recruitment source if provided
        if req.recruitment_source_id.is_some() {
            self.repo
                .set_employee_recruitment_source(auth_token, id, req.recruitment_source_id)
                .await?;
        }

        // If employee_id_number was updated, claim staged logs directly
        if let Some(new_id_number) = req.employee_id_number {
            let normalized = new_id_number.into_inner();
            if let Err(e) = self
                .staging_repo
                .claim_staged_logs(workspace_id, &normalized, id)
                .await
            {
                tracing::warn!(
                    "Failed to claim staged logs for updated employee {}: {}",
                    id,
                    e
                );
            }
        }

        Ok(employee)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn delete_employee(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<(), AppError> {
        let _workspace_id = auth_token.workspace_id();
        let rows_affected = self.repo.delete_employee(id, auth_token).await?;

        if rows_affected == 0 {
            Err(AppError::NotFound("Employee not found".into()))
        } else {
            Ok(())
        }
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn get_department_history(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let _ = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        self.dept_repo
            .get_employee_department_history(auth_token, employee_id)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn transfer_employee(
        &self,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: crate::models::TransferEmployeeRequest,
    ) -> Result<crate::models::EmployeeDepartmentHistory, AppError> {
        let _workspace_id = auth_token.workspace_id();
        let _ = self
            .repo
            .get_employee_by_id(employee_id, auth_token)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

        self.dept_repo
            .transfer_employee(
                auth_token,
                employee_id,
                req.department_id,
                req.effective_date,
                req.notes,
            )
            .await
    }
}

#[cfg(test)]
#[allow(clippy::unimplemented)]
mod tests {
    use super::*;
    use crate::models::{
        EmployeeFilterOptions, RecruitmentSource, RecruitmentSourceBreakdown, TurnoverAnalytics,
    };
    use crate::repositories::employee::{
        EmployeeBasicInfo, EmployeeCoreRepository, EmployeePayrollConfigRepository,
        EmployeeRecruitmentRepository, EmployeeSelfServiceRepository,
        EmployeeWorkingConditionRepository,
    };
    use async_trait::async_trait;
    use chrono::{NaiveDate, Utc};
    use rust_decimal::Decimal;
    use std::sync::Mutex;
    use uuid::Uuid;

    // ── Test Helpers ─────────────────────────────────────────────────

    fn make_employee(ws: Uuid) -> Employee {
        Employee::new(
            Uuid::new_v4(),
            ws,
            None,
            "Test Employee".into(),
            "EMP001".into(),
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            None,
            vec![],
            true,
            Utc::now(),
            Utc::now(),
            None,
            Uuid::new_v4(),
            None,
        )
    }

    fn make_working_condition(emp_id: Uuid) -> WorkingCondition {
        WorkingCondition::new(
            Uuid::new_v4(),
            emp_id,
            "late_allowance".into(),
            15,
            true,
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date"),
            None,
        )
    }

    // ── Mock Repository ──────────────────────────────────────────────

    /// Configurable mock — set the inner Option fields to control what each
    /// method returns. Defaults to "not found" / empty for all.
    struct MockEmployeeRepo {
        employee: Mutex<Option<Employee>>,
        employees: Mutex<Vec<Employee>>,
        total: Mutex<i64>,
        working_conditions: Mutex<Vec<WorkingCondition>>,
        delete_wc_result: Mutex<bool>,
        delete_emp_rows: Mutex<u64>,
        updated_employee: Mutex<Option<Employee>>,
        recruitment_sources: Mutex<Vec<RecruitmentSource>>,
        delete_rs_rows: Mutex<u64>,
        turnover: Mutex<Option<TurnoverAnalytics>>,
        updated_recruitment_source: Mutex<Option<RecruitmentSource>>,
        recruitment_source_breakdown: Mutex<Vec<RecruitmentSourceBreakdown>>,
    }

    impl MockEmployeeRepo {
        fn empty() -> Self {
            Self {
                employee: Mutex::new(None),
                employees: Mutex::new(vec![]),
                total: Mutex::new(0),
                working_conditions: Mutex::new(vec![]),
                delete_wc_result: Mutex::new(false),
                delete_emp_rows: Mutex::new(0),

                updated_employee: Mutex::new(None),
                recruitment_sources: Mutex::new(vec![]),
                delete_rs_rows: Mutex::new(0),
                turnover: Mutex::new(None),
                updated_recruitment_source: Mutex::new(None),
                recruitment_source_breakdown: Mutex::new(vec![]),
            }
        }

        fn with_employee(emp: Employee) -> Self {
            let mut mock = Self::empty();
            mock.employee = Mutex::new(Some(emp));
            mock
        }
    }

    #[async_trait]
    impl EmployeeCoreRepository for MockEmployeeRepo {
        async fn find_by_number(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _num: &str,
        ) -> Result<Option<Employee>, AppError> {
            Ok(self.employee.lock().expect("lock").clone())
        }

        async fn get_employee_by_id(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Employee>, AppError> {
            Ok(self.employee.lock().expect("lock").clone())
        }

        async fn get_employee_by_id_number(
            &self,
            _num: &str,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Employee>, AppError> {
            Ok(self.employee.lock().expect("lock").clone())
        }

        async fn get_employees_by_ids(
            &self,
            _ids: &[uuid::Uuid],
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<crate::models::Employee>, crate::errors::AppError> {
            Ok(vec![])
        }

        async fn get_all_employees(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _opts: EmployeeFilterOptions,
        ) -> Result<(Vec<Employee>, i64), AppError> {
            let emps = self.employees.lock().expect("lock").clone();
            let total = *self.total.lock().expect("lock");
            Ok((emps, total))
        }

        async fn get_active_employees(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<Employee>, AppError> {
            Ok(self.employees.lock().expect("lock").clone())
        }

        async fn get_active_employee_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _filter: Option<&Vec<Uuid>>,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }

        async fn get_paginated_employee_ids(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _args: crate::models::EmployeePaginationArgs<'_>,
        ) -> Result<(Vec<EmployeeBasicInfo>, i64), AppError> {
            Ok((vec![], 0))
        }

        async fn get_employee_departments_at(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _date: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, (Option<Uuid>, String)>, AppError> {
            Ok(std::collections::HashMap::new())
        }

        async fn update_employee(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _req: &crate::models::UpdateEmployeeRequest,
        ) -> Result<Option<Employee>, AppError> {
            Ok(self.updated_employee.lock().expect("lock").clone())
        }

        async fn create_employee(&self, _employee: &Employee) -> Result<(), AppError> {
            Ok(())
        }

        async fn create_employee_transactional(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee: &Employee,
            _recruitment_source_id: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn create_employee_skip_duplicates(
            &self,
            _employee: &Employee,
        ) -> Result<bool, AppError> {
            Ok(true)
        }

        async fn rehire_employee(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _emp: Uuid,
            _date: NaiveDate,
        ) -> Result<Option<Employee>, AppError> {
            Ok(self.updated_employee.lock().expect("lock").clone())
        }

        async fn delete_employee(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            Ok(*self.delete_emp_rows.lock().expect("lock"))
        }

        async fn get_existing_employee_ids_by_numbers(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _nums: &[String],
        ) -> Result<Vec<String>, AppError> {
            Ok(vec![])
        }

        async fn get_employee_ids_by_numbers(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id_numbers: &[String],
        ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
            Ok(std::collections::HashMap::new())
        }

        async fn bulk_create_employees_skip_duplicates(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employees: &[Employee],
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn get_managed_departments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _user: Uuid,
        ) -> Result<Vec<Uuid>, AppError> {
            Ok(vec![])
        }

        async fn get_employee_id_by_user_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _user: Uuid,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }

        async fn get_employee_timezone_fallback(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }

        async fn get_employee_timezone_at_date(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _tentative_date: chrono::NaiveDate,
        ) -> Result<chrono_tz::Tz, AppError> {
            Ok(chrono_tz::UTC)
        }

        async fn get_headquarters_location_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<Uuid>, AppError> {
            Ok(None)
        }
    }

    #[async_trait]
    impl EmployeeWorkingConditionRepository for MockEmployeeRepo {
        async fn get_working_conditions(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _emp: Uuid,
            page: i64,
            limit: i64,
        ) -> Result<(Vec<WorkingCondition>, i64), AppError> {
            let list = self.working_conditions.lock().expect("lock").clone();
            let total = list.len() as i64;
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(list.len());
            let paginated = if start < list.len() {
                list[start..end].to_vec()
            } else {
                vec![]
            };
            Ok((paginated, total))
        }

        async fn create_working_condition(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _cond: &WorkingCondition,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn delete_working_condition(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _emp: Uuid,
        ) -> Result<bool, AppError> {
            Ok(*self.delete_wc_result.lock().expect("lock"))
        }

        async fn get_working_conditions_at(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _date: NaiveDate,
        ) -> Result<Vec<WorkingCondition>, AppError> {
            Ok(vec![])
        }

        async fn get_working_conditions_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
            _start_date: NaiveDate,
            _end_date: NaiveDate,
        ) -> Result<std::collections::HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl EmployeePayrollConfigRepository for MockEmployeeRepo {
        async fn upsert_employee_payroll_config(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            employee_id: Uuid,
            std_working_days: Decimal,
            hours_per_day: Decimal,
            effective_date: NaiveDate,
        ) -> Result<crate::models::EmployeePayrollConfig, AppError> {
            Ok(crate::models::EmployeePayrollConfig {
                id: Uuid::new_v4(),
                employee_id,
                std_working_days,
                hours_per_day,
                effective_date,
                created_at: Utc::now(),
            })
        }

        async fn get_employee_payroll_config_at(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _emp: Uuid,
            _date: NaiveDate,
        ) -> Result<Option<crate::models::EmployeePayrollConfig>, AppError> {
            Ok(None)
        }

        async fn get_employee_payroll_configs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _emp: Uuid,
        ) -> Result<Vec<crate::models::EmployeePayrollConfig>, AppError> {
            Ok(vec![])
        }
        async fn get_employee_payroll_configs_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<Uuid, Vec<crate::models::EmployeePayrollConfig>>,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
    }

    #[async_trait]
    impl EmployeeRecruitmentRepository for MockEmployeeRepo {
        async fn set_employee_recruitment_source(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _recruitment_source_id: Option<Uuid>,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn create_recruitment_source(
            &self,
            _source: &RecruitmentSource,
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn get_recruitment_sources(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            page: i64,
            limit: i64,
        ) -> Result<(Vec<RecruitmentSource>, i64), AppError> {
            let list = self.recruitment_sources.lock().expect("lock").clone();
            let total = list.len() as i64;
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(list.len());
            let paginated = if start < list.len() {
                list[start..end].to_vec()
            } else {
                vec![]
            };
            Ok((paginated, total))
        }

        async fn get_recruitment_source_by_id(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            Ok(None)
        }

        async fn update_recruitment_source(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _req: &crate::models::UpdateRecruitmentSourceRequest,
        ) -> Result<Option<RecruitmentSource>, AppError> {
            Ok(self
                .updated_recruitment_source
                .lock()
                .expect("lock")
                .clone())
        }

        async fn delete_recruitment_source(
            &self,
            _id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<u64, AppError> {
            Ok(*self.delete_rs_rows.lock().expect("lock"))
        }

        async fn get_recruitment_source_breakdown(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Vec<RecruitmentSourceBreakdown>, AppError> {
            Ok(self
                .recruitment_source_breakdown
                .lock()
                .expect("lock")
                .clone())
        }

        async fn get_turnover_analytics(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            start_date: NaiveDate,
            end_date: NaiveDate,
        ) -> Result<TurnoverAnalytics, AppError> {
            Ok(self
                .turnover
                .lock()
                .expect("lock")
                .clone()
                .unwrap_or(TurnoverAnalytics {
                    tenure_distribution: vec![],
                    by_source: vec![],
                    by_department: vec![],
                    total_terminated: 0,
                    period_start: start_date.to_string(),
                    period_end: end_date.to_string(),
                }))
        }
    }

    #[async_trait]
    impl EmployeeSelfServiceRepository for MockEmployeeRepo {
        async fn get_employee_profile(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<Option<crate::models::EmployeeProfile>, AppError> {
            Ok(None)
        }

        async fn upsert_employee_profile(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _req: &crate::models::UpdateEmployeeProfileRequest,
        ) -> Result<crate::models::EmployeeProfile, AppError> {
            unimplemented!()
        }

        async fn get_emergency_contacts(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<Vec<crate::models::EmployeeEmergencyContact>, AppError> {
            Ok(vec![])
        }

        async fn upsert_emergency_contact(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _req: &crate::models::CreateEmergencyContactRequest,
        ) -> Result<crate::models::EmployeeEmergencyContact, AppError> {
            unimplemented!()
        }

        async fn delete_emergency_contact(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _contact_id: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }

        async fn get_employee_documents(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<Vec<crate::models::EmployeeDocument>, AppError> {
            Ok(vec![])
        }

        async fn create_employee_document(
            &self,
            _doc: &crate::models::EmployeeDocument,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[async_trait]
    impl crate::repositories::diesel_employee_repository::assignments::EmployeeAssignmentRepository
        for MockEmployeeRepo
    {
        async fn list_employee_assignments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<Vec<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
            Ok(vec![])
        }

        async fn get_employee_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _assignment_id: Uuid,
        ) -> Result<Option<crate::models::employee_assignment::EmployeeAssignment>, AppError>
        {
            Ok(None)
        }

        async fn create_employee_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _assignment: &crate::models::employee_assignment::NewEmployeeAssignment,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            Ok(crate::models::employee_assignment::EmployeeAssignment {
                id: Uuid::new_v4(),
                workspace_id: _assignment.workspace_id,
                employee_id: _assignment.employee_id,
                home_location_id: _assignment.home_location_id,
                host_location_id: _assignment.host_location_id,
                start_date: _assignment.start_date,
                end_date: _assignment.end_date,
                reason: _assignment.reason.clone(),
                status: _assignment.status,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }

        async fn update_employee_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _assignment_id: Uuid,
            _req: &crate::models::employee_assignment::UpdateEmployeeAssignmentRequest,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            Ok(crate::models::employee_assignment::EmployeeAssignment {
                id: _assignment_id,
                workspace_id: Uuid::new_v4(),
                employee_id: _employee_id,
                home_location_id: Uuid::new_v4(),
                host_location_id: Uuid::new_v4(),
                start_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                end_date: None,
                reason: None,
                status: crate::models::employee_assignment::AssignmentStatusEnum::Active,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }

        async fn delete_employee_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
            _assignment_id: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }
    }

    /// Minimal no-op DepartmentRepository for unit tests.
    struct NopDepartmentRepo;

    #[async_trait]
    impl crate::repositories::department_repository::DepartmentRepository for NopDepartmentRepo {
        async fn get_employee_department_history_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[uuid::Uuid],
        ) -> Result<
            std::collections::HashMap<uuid::Uuid, Vec<crate::models::EmployeeDepartmentHistory>>,
            crate::errors::AppError,
        > {
            Ok(std::collections::HashMap::new())
        }

        async fn get_department_ids_by_names(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _names: &[String],
        ) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
            Ok(std::collections::HashMap::new())
        }

        async fn bulk_create_departments_skip_duplicates(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _departments: &[crate::models::Department],
        ) -> Result<(), AppError> {
            Ok(())
        }

        async fn create_department(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _name: String,
            _desc: Option<String>,
            _mgr: Option<Uuid>,
            _pat: Option<Uuid>,
        ) -> Result<crate::models::Department, AppError> {
            unimplemented!()
        }
        async fn get_department_by_id(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<Option<crate::models::Department>, AppError> {
            unimplemented!()
        }
        async fn get_department_by_name(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _name: &str,
        ) -> Result<Option<Uuid>, AppError> {
            unimplemented!()
        }
        async fn update_department(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
            _req: crate::models::UpdateDepartmentRequest,
        ) -> Result<crate::models::Department, AppError> {
            unimplemented!()
        }
        async fn delete_department(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _id: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_departments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _page: i64,
            _limit: i64,
        ) -> Result<(Vec<crate::models::Department>, i64), AppError> {
            unimplemented!()
        }
        async fn transfer_employee(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _emp: Uuid,
            _dept: Uuid,
            _date: chrono::NaiveDate,
            _notes: Option<String>,
        ) -> Result<crate::models::EmployeeDepartmentHistory, AppError> {
            Ok(crate::models::EmployeeDepartmentHistory {
                id: Uuid::new_v4(),
                employee_id: _emp,
                department_id: _dept,
                effective_date: _date,
                notes: _notes,
                created_at: chrono::Utc::now(),
            })
        }
        async fn get_employee_department_history(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _emp: Uuid,
        ) -> Result<Vec<crate::models::EmployeeDepartmentHistory>, AppError> {
            unimplemented!()
        }
        async fn get_current_department(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_id: Uuid,
        ) -> Result<Option<crate::models::EmployeeDepartmentHistory>, AppError> {
            Ok(None)
        }

        async fn get_current_departments_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employee_ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<Uuid, crate::models::EmployeeDepartmentHistory>,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }

        async fn upsert_department_payroll_config(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _dept: Uuid,
            _std_wd: rust_decimal::Decimal,
            _hpd: rust_decimal::Decimal,
            _date: chrono::NaiveDate,
        ) -> Result<crate::models::DepartmentPayrollConfig, AppError> {
            unimplemented!()
        }
        async fn get_department_payroll_config_at(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _dept: Uuid,
            _date: chrono::NaiveDate,
        ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_department_payroll_configs(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _dept: Uuid,
        ) -> Result<Vec<crate::models::DepartmentPayrollConfig>, AppError> {
            unimplemented!()
        }
        async fn get_department_payroll_configs_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _department_ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<Uuid, Vec<crate::models::DepartmentPayrollConfig>>,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
        }
    }

    struct NopUserRepo;
    #[async_trait::async_trait]
    impl crate::repositories::user_repository::UserRepository for NopUserRepo {
        async fn create_user(&self, _user: &crate::models::User) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn create_user_with_admin_check(
            &self,
            _user: &crate::models::User,
        ) -> Result<bool, AppError> {
            unimplemented!()
        }
        async fn get_user_count(&self) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn assign_platform_admin(&self, _user_id: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_user_by_email(
            &self,
            _email: &str,
        ) -> Result<Option<crate::models::User>, AppError> {
            unimplemented!()
        }
        async fn get_user_by_id(&self, _id: Uuid) -> Result<Option<crate::models::User>, AppError> {
            unimplemented!()
        }
        async fn update_password_hash(
            &self,
            _user_id: Uuid,
            _password_hash: &str,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn get_system_roles_for_user(&self, _user_id: Uuid) -> Result<Vec<String>, AppError> {
            unimplemented!()
        }
        async fn update_user_profile(
            &self,
            _user_id: Uuid,
            _full_name: Option<String>,
            _avatar_url: Option<String>,
            _bio: Option<String>,
        ) -> Result<crate::models::User, AppError> {
            unimplemented!()
        }
        async fn get_workspace_users(
            &self,
            _workspace_id: Uuid,
        ) -> Result<Vec<crate::models::User>, AppError> {
            unimplemented!()
        }
        async fn get_dashboard_preferences(
            &self,
            _user_id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
        ) -> Result<Option<crate::handlers::dashboard_preferences::DashboardPreferences>, AppError>
        {
            unimplemented!()
        }
        async fn update_dashboard_preferences(
            &self,
            _user_id: Uuid,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _req: crate::handlers::dashboard_preferences::UpdateDashboardPreferences,
        ) -> Result<crate::handlers::dashboard_preferences::DashboardPreferences, AppError>
        {
            unimplemented!()
        }
        async fn get_notifications(
            &self,
            _user_id: Uuid,
            _limit: i64,
            _cursor: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<Vec<crate::models::Notification>, AppError> {
            unimplemented!()
        }
        async fn get_unread_notification_count(&self, _user_id: Uuid) -> Result<i64, AppError> {
            unimplemented!()
        }
        async fn mark_notification_as_read(
            &self,
            _user_id: Uuid,
            _notification_id: Uuid,
        ) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn mark_all_notifications_as_read(&self, _user_id: Uuid) -> Result<(), AppError> {
            unimplemented!()
        }
        async fn check_recent_notification_exists(
            &self,
            _user_id: Uuid,
            _notification_type: &str,
            _link: &str,
            _since_days: i32,
        ) -> Result<bool, AppError> {
            Ok(false)
        }
        async fn create_notification(
            &self,
            _notification: &crate::models::Notification,
        ) -> Result<(), AppError> {
            Ok(())
        }
    }

    /// Helper: create a service backed by the mock (pool is unused in mock-only paths).
    fn make_service(mock: MockEmployeeRepo) -> EmployeeService {
        use crate::repositories::staging_log_repository::{StagingLog, StagingLogRepository};
        struct NopStagingRepo;
        #[async_trait::async_trait]
        impl StagingLogRepository for NopStagingRepo {
            async fn create_staging_log(
                &self,
                _workspace_id: uuid::Uuid,
                _device_id: Option<String>,
                _raw_user_id: &str,
                _punch_time: chrono::DateTime<chrono::Utc>,
                _raw_data: Option<serde_json::Value>,
                _fingerprint: &str,
            ) -> Result<bool, crate::errors::AppError> {
                Ok(false)
            }
            async fn get_pending_staged_logs(
                &self,
                _workspace_id: uuid::Uuid,
                _employee_id_number: &str,
            ) -> Result<Vec<StagingLog>, crate::errors::AppError> {
                Ok(vec![])
            }
            async fn update_staging_status(
                &self,
                _id: uuid::Uuid,
                _workspace_id: uuid::Uuid,
                _status: &str,
            ) -> Result<bool, crate::errors::AppError> {
                Ok(false)
            }
            async fn get_pending_staging_logs(
                &self,
                _workspace_id: uuid::Uuid,
                _limit: i64,
            ) -> Result<Vec<StagingLog>, crate::errors::AppError> {
                Ok(vec![])
            }
            async fn count_pending_staging_logs(
                &self,
                _workspace_id: uuid::Uuid,
            ) -> Result<i64, crate::errors::AppError> {
                Ok(0)
            }
            async fn claim_staged_logs(
                &self,
                _workspace_id: uuid::Uuid,
                _raw_user_id: &str,
                _employee_id: uuid::Uuid,
            ) -> Result<u64, crate::errors::AppError> {
                Ok(0)
            }
            async fn resolve_staging_log(
                &self,
                _log_id: uuid::Uuid,
                _workspace_id: uuid::Uuid,
                _employee_id: uuid::Uuid,
            ) -> Result<crate::models::RawAttendanceLog, crate::errors::AppError> {
                Err(crate::errors::AppError::NotFound("nop".into()))
            }
        }
        EmployeeService::new(
            Arc::new(mock),
            Arc::new(NopDepartmentRepo),
            Arc::new(NopUserRepo),
            Arc::new(NopStagingRepo),
        )
    }

    // ── Tests: get_employee_by_id ────────────────────────────────────

    #[tokio::test]
    async fn get_employee_by_id_found() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let emp_id = emp.id();
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));

        let result = svc
            .get_employee_by_id(
                emp_id,
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
            )
            .await;
        assert!(result.is_ok());
        assert_eq!(result.expect("ok").id(), emp_id);
    }

    #[tokio::test]
    async fn get_employee_by_id_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .get_employee_by_id(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            )
            .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: get_all_employees ─────────────────────────────────────

    #[tokio::test]
    async fn get_all_employees_returns_list_and_count() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let mock = MockEmployeeRepo::empty();
        *mock.employees.lock().expect("lock") = vec![emp.clone()];
        *mock.total.lock().expect("lock") = 1;

        let svc = make_service(mock);
        let (list, total) = svc
            .get_all_employees(
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                EmployeeFilterOptions::default(),
            )
            .await
            .expect("ok");

        assert_eq!(list.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(list[0].full_name(), "Test Employee");
    }

    #[tokio::test]
    async fn get_all_employees_empty() {
        let svc = make_service(MockEmployeeRepo::empty());
        let (list, total) = svc
            .get_all_employees(
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                EmployeeFilterOptions::default(),
            )
            .await
            .expect("ok");

        assert_eq!(list.len(), 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn create_employee_success() {
        let ws = Uuid::new_v4();
        let svc = make_service(MockEmployeeRepo::empty());
        let emp = crate::models::CreateEmployeeRequest {
            user_id: None,
            full_name: "New Employee".into(),
            employee_id_number: crate::models::SanitizedEmployeeId::new("EMP001"),
            hire_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            tags: None,
            recruitment_source_id: None,
            primary_location_id: Uuid::new_v4(),
            employer_id: None,
        };
        let result = svc
            .create_employee(
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                emp,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn transfer_employee_success() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let emp_id = emp.id();
        let mock = MockEmployeeRepo::empty();
        *mock.employee.lock().expect("lock") = Some(emp.clone());
        let svc = make_service(mock);

        let req = crate::models::department::TransferEmployeeRequest {
            department_id: Uuid::new_v4(),
            effective_date: chrono::NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            notes: None,
        };
        let result = svc
            .transfer_employee(
                emp_id,
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(result.is_ok());
    }

    // ── Tests: get_working_conditions ────────────────────────────────

    #[tokio::test]
    async fn get_working_conditions_returns_conditions() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let cond = make_working_condition(emp.id());

        let mock = MockEmployeeRepo::with_employee(emp.clone());
        *mock.working_conditions.lock().expect("lock") = vec![cond.clone()];

        let svc = make_service(mock);
        let result = svc
            .get_working_conditions(
                emp.id(),
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                1,
                50,
            )
            .await
            .expect("ok");

        assert_eq!(result.0.len(), 1);
        assert_eq!(result.0[0].type_(), "late_allowance");
        assert_eq!(result.0[0].minute_allowance(), 15);
    }

    #[tokio::test]
    async fn get_working_conditions_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .get_working_conditions(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                1,
                50,
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: create_working_condition ──────────────────────────────

    #[tokio::test]
    async fn create_working_condition_success() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));

        let req = CreateWorkingConditionRequest {
            type_: "early_leave".into(),
            minute_allowance: 30,
            is_paid: false,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).expect("valid date"),
            end_date: None,
        };

        let result = svc
            .create_working_condition(
                emp.id(),
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(result.is_ok());
        let wc = result.expect("ok");
        assert_eq!(wc.type_(), "early_leave");
        assert_eq!(wc.minute_allowance(), 30);
        assert!(!wc.is_paid());
        assert_eq!(wc.employee_id(), emp.id());
    }

    // ── Tests: delete_working_condition ──────────────────────────────

    #[tokio::test]
    async fn delete_working_condition_success() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let mock = MockEmployeeRepo::with_employee(emp.clone());
        *mock.delete_wc_result.lock().expect("lock") = true;

        let svc = make_service(mock);
        let result = svc
            .delete_working_condition(
                emp.id(),
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                Uuid::new_v4(),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_working_condition_not_found() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws);
        let mock = MockEmployeeRepo::with_employee(emp.clone());
        // delete_wc_result defaults to false

        let svc = make_service(mock);
        let result = svc
            .delete_working_condition(
                emp.id(),
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                Uuid::new_v4(),
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn delete_working_condition_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .delete_working_condition(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                Uuid::new_v4(),
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Tests: delete_employee ───────────────────────────────────────

    #[tokio::test]
    async fn delete_employee_success() {
        let mock = MockEmployeeRepo::empty();
        *mock.delete_emp_rows.lock().expect("lock") = 1;
        let svc = make_service(mock);

        let result = svc
            .delete_employee(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .delete_employee(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    // ── Additional Tests for Coverage ──────────────────────────────

    #[tokio::test]
    async fn create_working_condition_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = CreateWorkingConditionRequest {
            type_: "early_leave".into(),
            minute_allowance: 30,
            is_paid: false,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).expect("valid date"),
            end_date: None,
        };
        let result = svc
            .create_working_condition(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                req,
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn rehire_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::RehireEmployeeRequest {
            hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        };
        let result = svc
            .rehire_employee(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                req,
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn rehire_employee_already_active() {
        let ws = Uuid::new_v4();
        let emp = make_employee(ws); // active by default
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));
        let req = crate::models::RehireEmployeeRequest {
            hire_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        };
        let result = svc
            .rehire_employee(
                emp.id(),
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(
            matches!(result.unwrap_err(), AppError::Validation(e) if e == "Employee is already active")
        );
    }

    #[tokio::test]
    async fn rehire_employee_invalid_date() {
        let ws = Uuid::new_v4();
        let mut emp = make_employee(ws);
        emp.set_is_active(false);
        emp.set_termination_date(Some(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()));
        let svc = make_service(MockEmployeeRepo::with_employee(emp.clone()));
        let req = crate::models::RehireEmployeeRequest {
            hire_date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(), // Before termination
        };
        let result = svc
            .rehire_employee(
                emp.id(),
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(
            matches!(result.unwrap_err(), AppError::Validation(e) if e == "Rehire date must be after termination date")
        );
    }

    #[tokio::test]
    async fn update_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::UpdateEmployeeRequest {
            user_id: None,
            full_name: None,
            employee_id_number: None,
            hire_date: None,
            termination_date: None,
            tags: None,
            is_active: None,
            max_concurrent_workers: None,
            recruitment_source_id: None,
            primary_location_id: None,
            employer_id: None,
        };
        let result = svc
            .update_employee(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                req,
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn update_employee_success() {
        let ws = Uuid::new_v4();
        let mock = MockEmployeeRepo::empty();
        let updated = make_employee(ws);
        let updated_id = updated.id();
        *mock.updated_employee.lock().unwrap() = Some(updated.clone());
        let svc = make_service(mock);
        let req = crate::models::UpdateEmployeeRequest {
            user_id: None,
            full_name: Some("Updated Name".into()),
            employee_id_number: None,
            hire_date: None,
            termination_date: None,
            tags: None,
            is_active: None,
            max_concurrent_workers: None,
            recruitment_source_id: None, // No source ID, no id_number update
            primary_location_id: None,
            employer_id: None,
        };
        let result = svc
            .update_employee(
                updated_id,
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn get_department_history_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let result = svc
            .get_department_history(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }

    #[tokio::test]
    async fn transfer_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::TransferEmployeeRequest {
            department_id: Uuid::new_v4(),
            effective_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            notes: None,
        };
        let result = svc
            .transfer_employee(
                Uuid::new_v4(),
                &crate::diesel_pool::TenantAuthToken::system_override(Uuid::new_v4()),
                req,
            )
            .await;
        assert!(matches!(result.unwrap_err(), AppError::NotFound(_)));
    }
}
