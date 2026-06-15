use crate::repositories::{
    department_repository::DepartmentRepository, EmployeeRepository, EmploymentRepository,
};
use std::sync::Arc;

mod compensation;
mod core;
mod create;
mod employment;
mod lifecycle;
mod working_conditions;

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
}

#[cfg(test)]
mod tests;
