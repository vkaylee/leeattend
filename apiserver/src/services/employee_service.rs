use std::sync::Arc;

use chrono::{Datelike, Utc};
use uuid::Uuid;

use crate::utils::id::new_id;

use crate::errors::AppError;
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, RehireEmployeeRequest,
    UpdateEmployeeRequest, WorkingCondition,
};
use crate::repositories::{
    department_repository::DepartmentRepository, EmployeeRepository, EmploymentRepository,
};

pub struct EmployeeService {
    repo: Arc<dyn EmployeeRepository>,
    employment_repo: Arc<dyn EmploymentRepository>,
    dept_repo: Arc<dyn DepartmentRepository>,
    user_repo: Arc<dyn crate::repositories::user_repository::UserRepository>,
    staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
    leave_balance_repo:
        Arc<dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository>,
    lifecycle_service: Option<Arc<crate::services::lifecycle_service::LifecycleService>>,
    payroll_repo: Option<Arc<dyn crate::repositories::payroll_repository::PayrollRepository>>,
    employer_repo: Option<Arc<dyn crate::repositories::EmployerRepository>>,
}

fn dates_overlap(
    s1: chrono::NaiveDate,
    e1: Option<chrono::NaiveDate>,
    s2: chrono::NaiveDate,
    e2: Option<chrono::NaiveDate>,
) -> bool {
    let overlap_start = std::cmp::max(s1, s2);
    let overlap_end = match (e1, e2) {
        (Some(d1), Some(d2)) => Some(std::cmp::min(d1, d2)),
        (Some(d1), None) => Some(d1),
        (None, Some(d2)) => Some(d2),
        (None, None) => None,
    };

    match overlap_end {
        Some(end) => overlap_start <= end,
        None => true,
    }
}

impl EmployeeService {
    pub fn new(
        repo: Arc<dyn EmployeeRepository>,
        employment_repo: Arc<dyn EmploymentRepository>,
        dept_repo: Arc<dyn DepartmentRepository>,
        user_repo: Arc<dyn crate::repositories::user_repository::UserRepository>,
        staging_repo: Arc<dyn crate::repositories::staging_log_repository::StagingLogRepository>,
        leave_balance_repo: Arc<
            dyn crate::repositories::leave_balance_repository::LeaveBalanceRepository,
        >,
    ) -> Self {
        Self {
            repo,
            employment_repo,
            dept_repo,
            user_repo,
            staging_repo,
            leave_balance_repo,
            lifecycle_service: None,
            payroll_repo: None,
            employer_repo: None,
        }
    }

    #[must_use]
    pub fn with_lifecycle(
        mut self,
        lifecycle_service: Arc<crate::services::lifecycle_service::LifecycleService>,
    ) -> Self {
        self.lifecycle_service = Some(lifecycle_service);
        self
    }

    #[must_use]
    pub fn with_payroll(
        mut self,
        payroll_repo: Arc<dyn crate::repositories::payroll_repository::PayrollRepository>,
    ) -> Self {
        self.payroll_repo = Some(payroll_repo);
        self
    }

    #[must_use]
    pub fn with_employer(
        mut self,
        employer_repo: Arc<dyn crate::repositories::EmployerRepository>,
    ) -> Self {
        self.employer_repo = Some(employer_repo);
        self
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn create_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        req: &crate::models::employment::CreateEmploymentRequest,
    ) -> Result<crate::models::employment::Employment, AppError> {
        let _workspace_id = auth_token.workspace_id();

        // 1. Conflict check for primary employment
        if req.employment_type == "primary" {
            let existing = self
                .employment_repo
                .list_employee_employments(auth_token, req.employee_id)
                .await?;

            for emp in existing {
                if emp.employment_type() == "primary"
                    && dates_overlap(
                        req.start_date,
                        req.end_date,
                        emp.start_date(),
                        emp.end_date(),
                    )
                {
                    return Err(AppError::conflict(
                        crate::utils::i18n::I18nKey::OverlappingEmployment,
                    ));
                }
            }
        }

        // 2. Delegate to repo
        self.employment_repo
            .create_employment(auth_token, req)
            .await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn update_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        req: &crate::models::employment::UpdateEmploymentRequest,
    ) -> Result<crate::models::employment::Employment, AppError> {
        let _workspace_id = auth_token.workspace_id();

        let current_emp = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmploymentNotFound))?;

        let target_type = req
            .employment_type
            .as_deref()
            .unwrap_or_else(|| current_emp.employment_type());
        let target_start = req.start_date.unwrap_or_else(|| current_emp.start_date());
        let target_end = match &req.end_date {
            Some(opt_d) => *opt_d,
            None => current_emp.end_date(),
        };

        if target_type == "primary" {
            let existing = self
                .employment_repo
                .list_employee_employments(auth_token, current_emp.employee_id())
                .await?;

            for emp in existing {
                if emp.id() == employment_id {
                    continue;
                }
                if emp.employment_type() == "primary"
                    && dates_overlap(target_start, target_end, emp.start_date(), emp.end_date())
                {
                    return Err(AppError::conflict(
                        crate::utils::i18n::I18nKey::OverlappingEmployment,
                    ));
                }
            }
        }

        self.employment_repo
            .update_employment(auth_token, employment_id, req)
            .await
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

        let active_employment_id = emp
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        self.repo
            .get_working_conditions(auth_token, active_employment_id, page, limit)
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

        let active_employment_id = emp
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let condition = WorkingCondition::new(
            new_id(),
            active_employment_id,
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

        let active_employment_id = emp
            .active_employment_id()
            .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

        let deleted = self
            .repo
            .delete_working_condition(auth_token, condition_id, active_employment_id)
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
            req.tags.unwrap_or_default(),
            Utc::now(),
            Utc::now(),
            None,
        );

        // Use repository to create employee and set recruitment source transactionally
        self.repo
            .create_employee_transactional(
                auth_token,
                &employee,
                req.hire_date,
                req.recruitment_source_id,
                req.employer_id,
                req.primary_location_id,
                req.department_id,
            )
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

        // Trigger onboarding checklist if lifecycle service is registered
        if let Some(ref lifecycle_service) = self.lifecycle_service {
            if let Ok(employments) = self
                .employment_repo
                .list_employee_employments(auth_token, employee.id())
                .await
            {
                if let Some(emp) = employments.first() {
                    if let Err(e) = lifecycle_service
                        .auto_trigger_onboarding(workspace_id, emp.id(), None)
                        .await
                    {
                        tracing::warn!(
                            "Failed to auto-trigger onboarding checklist for employee {}: {:?}",
                            employee.id(),
                            e
                        );
                    }
                }
            }
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
        self.repo
            .rehire_employee(
                auth_token,
                employee_id,
                req.hire_date,
                req.employer_id,
                req.primary_location_id,
                req.department_id,
            )
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
        // Optional: Check if user_id exists if provided, and check duplicate
        if let Some(Some(user_id)) = req.user_id {
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
                if existing_emp_id != id && req.force_transfer != Some(true) {
                    let existing_emp = self
                        .repo
                        .get_employee_by_id(existing_emp_id, auth_token)
                        .await?
                        .ok_or_else(|| AppError::NotFound("Existing employee not found".into()))?;

                    return Err(AppError::Validation(format!(
                        "error.user_already_linked:name={}",
                        existing_emp.full_name()
                    )));
                }
            }
        }

        let employee = self
            .repo
            .update_employee_transactional(id, auth_token, &req)
            .await?
            .ok_or(AppError::NotFound("Employee not found".into()))?;

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

    #[tracing::instrument(skip_all, err)]
    pub async fn terminate_employment(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employment_id: Uuid,
        termination_date: chrono::NaiveDate,
    ) -> Result<crate::models::employment::Employment, AppError> {
        // 1. Get employment details
        let employment = self
            .employment_repo
            .get_employment_by_id(auth_token, employment_id)
            .await?
            .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmploymentNotFound))?;

        // 2. Validate termination state & date
        if employment.status() == "terminated" {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::Validation,
            ));
        }

        if termination_date < employment.start_date() {
            return Err(AppError::validation(
                crate::utils::i18n::I18nKey::Validation,
            ));
        }

        // 3. Update the employment status and end date to database
        let update_req = crate::models::employment::UpdateEmploymentRequest {
            employer_id: None,
            primary_location_id: None,
            department_id: None,
            status: Some("terminated".to_string()),
            start_date: None,
            end_date: Some(Some(termination_date)),
            job_title: None,
            employment_type: None,
            manager_employment_id: None,
        };

        let updated_employment = self
            .employment_repo
            .update_employment(auth_token, employment_id, &update_req)
            .await?;

        // Trigger offboarding checklist if lifecycle service is registered
        if let Some(ref lifecycle_service) = self.lifecycle_service {
            if let Err(e) = lifecycle_service
                .auto_trigger_offboarding(auth_token.workspace_id(), employment_id, None)
                .await
            {
                tracing::warn!(
                    "Failed to auto-trigger offboarding checklist for employment {}: {:?}",
                    employment_id,
                    e
                );
            }
        }

        // 4. Calculate prorated leave balance for the year of termination
        let year = termination_date.year();
        let start_of_year = chrono::NaiveDate::from_ymd_opt(year, 1, 1)
            .ok_or_else(|| AppError::Internal("Invalid start of year".to_string()))?;
        let end_of_year = chrono::NaiveDate::from_ymd_opt(year, 12, 31)
            .ok_or_else(|| AppError::Internal("Invalid end of year".to_string()))?;

        // Effective start date is the maximum of start of year or employment start date
        let eff_start = std::cmp::max(start_of_year, employment.start_date());

        // Days worked in the year
        let days_worked = (termination_date - eff_start).num_days() + 1;
        let total_days = (end_of_year - start_of_year).num_days() + 1;

        let proration_fraction = days_worked as f64 / total_days as f64;

        // Fetch existing leave balances for this employee
        let balances = self
            .leave_balance_repo
            .get_employee_balances(auth_token, employment.employee_id(), year)
            .await?;

        // Prorate entitlement for each leave balance and persist
        for emp_bal in balances {
            let prorated_entitlement =
                (f64::from(emp_bal.entitlement) * proration_fraction).round() as i32;

            let balance = crate::models::LeaveBalance::new(
                Uuid::new_v4(), // The on_conflict logic will match employment_id/leave_type_id/year
                employment_id,
                emp_bal.leave_type_id,
                year,
                prorated_entitlement,
                rust_decimal::Decimal::from_f64_retain(emp_bal.accrued_amount).unwrap_or_default(),
                emp_bal.carried_over,
                chrono::Utc::now(),
                chrono::Utc::now(),
            );

            self.leave_balance_repo
                .upsert_leave_balance(auth_token, &balance)
                .await?;
        }

        Ok(updated_employment)
    }

    #[allow(clippy::too_many_lines)]
    #[tracing::instrument(skip_all, err)]
    pub async fn get_compensation_history(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        filter_employment_id: Option<Uuid>,
    ) -> Result<Vec<crate::models::CompensationTimelineEvent>, AppError> {
        let workspace_id = auth_token.workspace_id();
        let payroll_repo = self.payroll_repo.as_ref().ok_or_else(|| {
            AppError::Internal(
                "Payroll repository is not configured on EmployeeService".to_string(),
            )
        })?;
        let employer_repo = self.employer_repo.as_ref().ok_or_else(|| {
            AppError::Internal(
                "Employer repository is not configured on EmployeeService".to_string(),
            )
        })?;

        // 1. Fetch all employments for this employee
        let mut employments = self
            .employment_repo
            .list_employee_employments(auth_token, employee_id)
            .await?;

        // 2. Filter employments if requested
        if let Some(target_id) = filter_employment_id {
            employments.retain(|e| e.id() == target_id);
        }

        if employments.is_empty() {
            return Ok(Vec::new());
        }

        let employment_ids: Vec<Uuid> = employments
            .iter()
            .map(crate::models::employment::Employment::id)
            .collect();

        // 3. Fetch all salary component changes for these employments
        let components_history = payroll_repo
            .get_salary_components_history_for_employments(workspace_id, &employment_ids)
            .await?;

        // 4. Resolve employer names to avoid N+1 queries
        let mut employer_ids: Vec<Uuid> = employments
            .iter()
            .map(crate::models::employment::Employment::employer_id)
            .collect();
        employer_ids.sort();
        employer_ids.dedup();

        let mut employer_names = std::collections::HashMap::new();
        for emp_id in employer_ids {
            if let Some(employer) = employer_repo.get_employer_by_id(auth_token, emp_id).await? {
                employer_names.insert(emp_id, employer.name.clone());
            }
        }

        // 5. Resolve department names to avoid N+1 queries
        let mut dept_ids: Vec<Uuid> = employments
            .iter()
            .filter_map(crate::models::employment::Employment::department_id)
            .collect();
        dept_ids.sort();
        dept_ids.dedup();

        let mut department_names = std::collections::HashMap::new();
        for d_id in dept_ids {
            if let Some(dept) = self
                .dept_repo
                .get_department_by_id(auth_token, d_id)
                .await?
            {
                department_names.insert(d_id, dept.name().to_string());
            }
        }

        // Map employments by ID for quick lookup
        let employments_map: std::collections::HashMap<
            Uuid,
            &crate::models::employment::Employment,
        > = employments.iter().map(|e| (e.id(), e)).collect();

        // Identify distinct chronological keys (employment_id, effective_date)
        let mut timeline_keys: Vec<(Uuid, chrono::NaiveDate)> = components_history
            .iter()
            .map(|(esc, _)| (esc.employment_id, esc.effective_date))
            .collect();
        timeline_keys.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
        timeline_keys.dedup();

        let mut active_components: std::collections::HashMap<
            Uuid,
            std::collections::HashMap<
                Uuid,
                (
                    crate::models::EmployeeSalaryComponent,
                    crate::models::SalaryComponentDefinition,
                ),
            >,
        > = std::collections::HashMap::new();

        let mut timeline_events = Vec::new();
        let mut previous_total: Option<rust_decimal::Decimal> = None;
        let mut previous_employment_id: Option<Uuid> = None;

        for (emp_id, eff_date) in timeline_keys {
            let Some(emp) = employments_map.get(&emp_id) else {
                continue;
            };

            let emp_components = active_components.entry(emp_id).or_default();

            for (esc, scd) in &components_history {
                if esc.employment_id == emp_id && esc.effective_date == eff_date {
                    emp_components.insert(esc.component_definition_id, (esc.clone(), scd.clone()));
                }
            }

            let mut event_components: Vec<crate::models::SalaryComponentWithDefinition> =
                emp_components
                    .values()
                    .map(|(esc, scd)| crate::models::SalaryComponentWithDefinition {
                        component: esc.clone(),
                        definition: scd.clone(),
                    })
                    .collect();

            event_components.sort_by(|a, b| a.definition.code.cmp(&b.definition.code));

            let total_compensation: rust_decimal::Decimal =
                event_components.iter().map(|c| c.component.amount).sum();

            let change_type = if previous_total.is_none() {
                "initial".to_string()
            } else if Some(emp_id) != previous_employment_id {
                "transfer".to_string()
            } else {
                "adjustment".to_string()
            };

            let delta = match previous_total {
                Some(prev) => total_compensation - prev,
                None => total_compensation,
            };

            let employer_name = employer_names
                .get(&emp.employer_id())
                .cloned()
                .unwrap_or_else(|| "Unknown Employer".to_string());
            let department_name = emp
                .department_id()
                .and_then(|id| department_names.get(&id).cloned());

            let event = crate::models::CompensationTimelineEvent {
                effective_date: eff_date,
                employment: crate::models::CompensationEmploymentContext {
                    id: emp_id,
                    employer_name,
                    department_name,
                    job_title: emp.job_title().map(std::string::ToString::to_string),
                },
                components: event_components,
                total_compensation,
                change_type,
                change_details: crate::models::CompensationChangeDetails {
                    previous_total,
                    delta,
                },
            };

            timeline_events.push(event);

            previous_total = Some(total_compensation);
            previous_employment_id = Some(emp_id);
        }

        // Return newest first
        timeline_events.reverse();

        Ok(timeline_events)
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
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        Employee::new(
            emp_id,
            ws,
            None,
            "Test Employee".into(),
            "EMP001".into(),
            vec![],
            Utc::now(),
            Utc::now(),
            None,
        )
        .with_employment_info(Some(employment_id), None, None)
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
        employee_id_by_user_id: Mutex<Option<Uuid>>,
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
                employee_id_by_user_id: Mutex::new(None),
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

        async fn update_employee_transactional(
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
            _hire_date: chrono::NaiveDate,
            _recruitment_source_id: Option<Uuid>,
            _employer_id: Uuid,
            _primary_location_id: Option<Uuid>,
            _department_id: Option<Uuid>,
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
            _hire_date: NaiveDate,
            _employer_id: Uuid,
            _primary_location_id: Option<Uuid>,
            _department_id: Option<Uuid>,
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
            Ok(*self.employee_id_by_user_id.lock().expect("lock"))
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

        async fn get_working_conditions_batch_by_employments(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
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
            employment_id: Uuid,
            std_working_days: Decimal,
            hours_per_day: Decimal,
            effective_date: NaiveDate,
        ) -> Result<crate::models::EmployeePayrollConfig, AppError> {
            Ok(crate::models::EmployeePayrollConfig {
                id: Uuid::new_v4(),
                employment_id,
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
            _employment_id: Uuid,
        ) -> Result<Vec<crate::models::employee_assignment::EmployeeAssignment>, AppError> {
            Ok(vec![])
        }

        async fn get_employee_assignment(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_id: Uuid,
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
                employment_id: _assignment.employment_id,
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
            _employment_id: Uuid,
            _assignment_id: Uuid,
            _req: &crate::models::employee_assignment::UpdateEmployeeAssignmentRequest,
        ) -> Result<crate::models::employee_assignment::EmployeeAssignment, AppError> {
            Ok(crate::models::employee_assignment::EmployeeAssignment {
                id: _assignment_id,
                workspace_id: Uuid::new_v4(),
                employment_id: _employment_id,
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
            _employment_id: Uuid,
            _assignment_id: Uuid,
        ) -> Result<u64, AppError> {
            Ok(0)
        }

        async fn list_assignments_batch(
            &self,
            _auth_token: &crate::diesel_pool::TenantAuthToken,
            _employment_ids: &[Uuid],
        ) -> Result<
            std::collections::HashMap<
                Uuid,
                Vec<crate::models::employee_assignment::EmployeeAssignment>,
            >,
            AppError,
        > {
            Ok(std::collections::HashMap::new())
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
            Ok(crate::models::EmployeeDepartmentHistory::new(
                Uuid::new_v4(),
                _emp,
                _dept,
                _date,
                _notes,
                chrono::Utc::now(),
            ))
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
        async fn get_user_by_id(&self, id: Uuid) -> Result<Option<crate::models::User>, AppError> {
            Ok(Some(crate::models::User::new(
                id,
                "test@example.com".to_string(),
                "password_hash".to_string(),
                chrono::Utc::now(),
                Some("Test User".to_string()),
                None,
                None,
                None,
            )))
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

    /// Helper: create a service backed by the mock (pool is unused in mock-only paths).
    fn make_service(mock: MockEmployeeRepo) -> EmployeeService {
        EmployeeService::new(
            Arc::new(mock),
            Arc::new(crate::repositories::employment_repository::MockEmploymentRepository::new()),
            Arc::new(NopDepartmentRepo),
            Arc::new(NopUserRepo),
            Arc::new(NopStagingRepo),
            Arc::new(
                crate::repositories::leave_balance_repository::MockLeaveBalanceRepository::new(),
            ),
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
            primary_location_id: Some(Uuid::new_v4()),
            employer_id: Uuid::new_v4(),
            department_id: None,
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
        assert_eq!(wc.employment_id(), emp.active_employment_id().unwrap());
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
            employer_id: Uuid::new_v4(),
            primary_location_id: None,
            department_id: None,
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
    async fn update_employee_not_found() {
        let svc = make_service(MockEmployeeRepo::empty());
        let req = crate::models::UpdateEmployeeRequest {
            user_id: None,
            full_name: None,
            employee_id_number: None,
            tags: None,
            max_concurrent_workers: None,
            recruitment_source_id: None,
            force_transfer: None,
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
            tags: None,
            max_concurrent_workers: None,
            recruitment_source_id: None, // No source ID, no id_number update
            force_transfer: None,
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
    async fn update_employee_duplicate_user_conflict() {
        let ws = Uuid::new_v4();
        let target_emp_id = Uuid::new_v4();
        let existing_emp_id = Uuid::new_v4();
        let conflict_user_id = Uuid::new_v4();

        let mock = MockEmployeeRepo::empty();
        *mock.employee_id_by_user_id.lock().unwrap() = Some(existing_emp_id);
        let existing_emp = make_employee(ws);
        *mock.employee.lock().unwrap() = Some(existing_emp.clone());

        let svc = make_service(mock);
        let req = crate::models::UpdateEmployeeRequest {
            user_id: Some(Some(conflict_user_id)),
            full_name: None,
            employee_id_number: None,
            tags: None,
            max_concurrent_workers: None,
            recruitment_source_id: None,
            force_transfer: None,
        };

        let result = svc
            .update_employee(
                target_emp_id,
                &crate::diesel_pool::TenantAuthToken::system_override(ws),
                req,
            )
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        assert_eq!(
            err.to_string(),
            "Validation error: error.user_already_linked:name=Test Employee"
        );
    }

    #[tokio::test]
    async fn update_employee_duplicate_user_force() {
        let ws = Uuid::new_v4();
        let target_emp_id = Uuid::new_v4();
        let existing_emp_id = Uuid::new_v4();
        let conflict_user_id = Uuid::new_v4();

        let mock = MockEmployeeRepo::empty();
        *mock.employee_id_by_user_id.lock().unwrap() = Some(existing_emp_id);
        let updated_emp = make_employee(ws);
        *mock.updated_employee.lock().unwrap() = Some(updated_emp.clone());

        let svc = make_service(mock);
        let req = crate::models::UpdateEmployeeRequest {
            user_id: Some(Some(conflict_user_id)),
            full_name: None,
            employee_id_number: None,
            tags: None,
            max_concurrent_workers: None,
            recruitment_source_id: None,
            force_transfer: Some(true),
        };

        let result = svc
            .update_employee(
                target_emp_id,
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

    #[tokio::test]
    async fn test_terminate_employment_success() -> Result<(), Box<dyn std::error::Error>> {
        let ws = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(ws);
        let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).ok_or("invalid start_date")?;
        let termination_date =
            NaiveDate::from_ymd_opt(2026, 6, 30).ok_or("invalid termination_date")?;

        // 1. Mock Employment
        let mock_emp = crate::models::employment::Employment::new(
            employment_id,
            ws,
            emp_id,
            Uuid::new_v4(),
            None,
            None,
            "active".to_string(),
            start_date,
            None,
            None,
            "primary".to_string(),
            None,
            Utc::now(),
            Utc::now(),
        );

        let mut mock_employment_repo =
            crate::repositories::employment_repository::MockEmploymentRepository::new();
        let mock_emp_clone = mock_emp.clone();
        mock_employment_repo
            .expect_get_employment_by_id()
            .returning(move |_, _| Ok(Some(mock_emp_clone.clone())));

        let mock_emp_terminated = crate::models::employment::Employment::new(
            employment_id,
            ws,
            emp_id,
            Uuid::new_v4(),
            None,
            None,
            "terminated".to_string(),
            start_date,
            Some(termination_date),
            None,
            "primary".to_string(),
            None,
            Utc::now(),
            Utc::now(),
        );
        mock_employment_repo
            .expect_update_employment()
            .returning(move |_, _, _| Ok(mock_emp_terminated.clone()));

        // 2. Mock Leave Balances
        let mut mock_leave_balance_repo =
            crate::repositories::leave_balance_repository::MockLeaveBalanceRepository::new();
        let leave_type_id = Uuid::new_v4();
        let mock_balances = vec![crate::models::EmployeeLeaveBalance {
            leave_type_id,
            leave_type_code: "AL".to_string(),
            leave_type_name: "Annual Leave".to_string(),
            is_paid: true,
            entitlement: 12,
            accrued_amount: 0.0,
            carried_over: 0,
            used: 0,
            remaining: 12.0,
        }];
        mock_leave_balance_repo
            .expect_get_employee_balances()
            .returning(move |_, _, _| Ok(mock_balances.clone()));

        mock_leave_balance_repo
            .expect_upsert_leave_balance()
            .returning(|_, balance| {
                // Assert that entitlement has been prorated.
                // From 2026-01-01 to 2026-06-30 is 181 days. 181 / 365 * 12 = 5.95 -> round to 6.
                assert_eq!(balance.entitlement(), 6);
                Ok(())
            });

        // 3. Create service and run
        let svc = EmployeeService::new(
            Arc::new(MockEmployeeRepo::empty()),
            Arc::new(mock_employment_repo),
            Arc::new(NopDepartmentRepo),
            Arc::new(NopUserRepo),
            Arc::new(NopStagingRepo),
            Arc::new(mock_leave_balance_repo),
        );

        let res = svc
            .terminate_employment(&auth_token, employment_id, termination_date)
            .await;

        assert!(res.is_ok());
        let updated = res?;
        assert_eq!(updated.status(), "terminated");
        assert_eq!(updated.end_date(), Some(termination_date));
        Ok(())
    }
}
