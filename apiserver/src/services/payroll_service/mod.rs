use crate::diesel_pool::TenantAuthToken;
use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;

use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use chrono::NaiveDate;
use chrono_tz::Tz;
use futures_util::stream::{self, StreamExt};
use rust_decimal::Decimal;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::Payslip;
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::holiday_repository::HolidayRepository;
use crate::repositories::leave_request_repository::LeaveRequestRepository;
use crate::repositories::payroll_repository::PayrollRepository;
use crate::repositories::roster::RosterRepository;
use crate::repositories::workspace_repository::WorkspaceRepository;

use super::attendance_matcher::AttendanceMatcher;
use super::financial_calculator::{
    FinancialCalculator, IncentivePolicyWithTiers, RuleAssignmentWithTiers,
};
use super::PolicyResolverService;

pub mod context;
pub(crate) mod helpers;
pub mod segments;
use context::ChunkSharedData;

pub use context::{DirectRosterDataProvider, PayrollContext, RosterDataProvider, SegmentSnapshot};

pub struct PayrollServiceArgs {
    pub payroll_repo: Arc<dyn PayrollRepository>,
    pub department_repo: Arc<dyn DepartmentRepository>,
    pub time_entry_repo: Arc<dyn crate::repositories::time_entry_repository::TimeEntryRepository>,
    pub workspace_repo: Arc<dyn WorkspaceRepository>,
    pub holiday_repo: Arc<dyn HolidayRepository>,
    pub roster_repo: Arc<dyn RosterRepository>,
    pub leave_repo: Arc<dyn LeaveRequestRepository>,
    pub policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    pub condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    pub employee_repo: Arc<dyn crate::repositories::EmployeeRepository>,
    pub overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
    pub salary_base_repo:
        Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    pub roster_provider: Arc<dyn RosterDataProvider>,
    pub employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
}

pub struct PayrollService {
    payroll_repo: Arc<dyn PayrollRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    time_entry_repo: Arc<dyn crate::repositories::time_entry_repository::TimeEntryRepository>,
    workspace_repo: Arc<dyn WorkspaceRepository>,
    holiday_repo: Arc<dyn HolidayRepository>,
    roster_repo: Arc<dyn RosterRepository>,
    leave_repo: Arc<dyn LeaveRequestRepository>,
    policy_repo: Arc<dyn crate::repositories::AttendancePolicyRepository>,
    condition_type_repo: Arc<dyn crate::repositories::ConditionTypeRepository>,
    employee_repo: Arc<dyn crate::repositories::EmployeeRepository>,
    overtime_policy_repo: Arc<dyn OvertimePolicyRepositoryTrait>,
    salary_base_repo: Arc<dyn crate::repositories::salary_base_repository::SalaryBaseRepository>,
    roster_provider: Arc<dyn RosterDataProvider>,
    employment_repo: Arc<dyn crate::repositories::EmploymentRepository>,
}

impl PayrollService {
    pub fn new(args: PayrollServiceArgs) -> Self {
        Self {
            payroll_repo: args.payroll_repo,
            department_repo: args.department_repo,
            time_entry_repo: args.time_entry_repo,
            workspace_repo: args.workspace_repo,
            holiday_repo: args.holiday_repo,
            roster_repo: args.roster_repo,
            leave_repo: args.leave_repo,
            policy_repo: args.policy_repo,
            condition_type_repo: args.condition_type_repo,
            employee_repo: args.employee_repo,
            overtime_policy_repo: args.overtime_policy_repo,
            salary_base_repo: args.salary_base_repo,
            roster_provider: args.roster_provider,
            employment_repo: args.employment_repo,
        }
    }
}

impl PayrollService {
    fn build_segments(ctx: &PayrollContext) -> Vec<(NaiveDate, NaiveDate)> {
        segments::build_segments(ctx)
    }

    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn calculate_monthly_payslip(
        &self,
        employee_id: Uuid,
        auth_token: &TenantAuthToken,
        year: i32,
        month: u32,
    ) -> Result<Payslip, AppError> {
        // Phase 1: Fetch all context data
        let ctx = self
            .fetch_context(employee_id, auth_token, year, month)
            .await?;

        self.calculate_payslip_with_context(ctx).await
    }
}

mod batch_chunk;
mod batch_helpers;
mod batch_orchestrator;
mod build_contexts;
mod fetch;
mod payslip_calc;
#[cfg(test)]
mod tests_mod;

pub(crate) use build_contexts::{build_payroll_contexts, BatchMaps};
