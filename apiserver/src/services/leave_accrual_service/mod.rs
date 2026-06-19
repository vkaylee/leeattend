pub mod calculator;
pub mod qualifying_days;
#[cfg(test)]
mod tests;

use crate::errors::AppError;
use crate::models::LeaveAccrualLog;
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository;
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::repositories::EmployeeRepository;
use chrono::NaiveDate;
use chrono_tz::Tz;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct LeaveAccrualService {
    employee_repo: Arc<dyn EmployeeRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    leave_policy_repo: Arc<dyn LeavePolicyRepository>,
    accrual_log_repo: Arc<dyn LeaveAccrualLogRepository>,
    time_entry_repo: Arc<dyn crate::repositories::TimeEntryRepository>,
    leave_request_repo:
        Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    #[allow(dead_code)]
    daily_summary_repo: Arc<dyn crate::repositories::AttendanceDailySummaryRepository>,
    contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
}

impl LeaveAccrualService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        employee_repo: Arc<dyn EmployeeRepository>,
        department_repo: Arc<dyn DepartmentRepository>,
        leave_policy_repo: Arc<dyn LeavePolicyRepository>,
        accrual_log_repo: Arc<dyn LeaveAccrualLogRepository>,
        time_entry_repo: Arc<dyn crate::repositories::TimeEntryRepository>,
        leave_request_repo: Arc<
            dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
        >,
        daily_summary_repo: Arc<dyn crate::repositories::AttendanceDailySummaryRepository>,
        contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
    ) -> Self {
        Self {
            employee_repo,
            department_repo,
            leave_policy_repo,
            accrual_log_repo,
            time_entry_repo,
            leave_request_repo,
            daily_summary_repo,
            contract_repo,
        }
    }

    /// Main orchestrator to process accruals for a workspace on a specific date.
    /// Optimized: uses pre-computed accrual dates and batched qualifying-days queries.
    /// All persistence is delegated to `accrual_log_repo` (Repository pattern).
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_workspace_accruals(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
        force: bool,
        target_employee_ids: Option<&[Uuid]>,
    ) -> Result<Vec<LeaveAccrualLog>, AppError> {
        // 1. Setup & Workspace-level queries
        let tz_str = self
            .accrual_log_repo
            .get_workspace_timezone(workspace_id)
            .await?;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(&tz_str);
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let employees = if let Some(ids) = target_employee_ids {
            if ids.is_empty() {
                return Ok(vec![]);
            }
            self.employee_repo
                .get_employees_by_ids(ids, &auth_token)
                .await?
        } else {
            self.employee_repo.get_active_employees(&auth_token).await?
        };

        if employees.is_empty() {
            return Ok(vec![]);
        }
        let employee_ids: Vec<Uuid> = employees.iter().map(crate::models::Employee::id).collect();

        // 2. Batch fetch all necessary data
        let dept_map = self
            .department_repo
            .get_current_departments_batch(&auth_token, &employee_ids)
            .await?;

        let all_assignments = self
            .leave_policy_repo
            .find_all_active_assignments_for_workspace(workspace_id, date)
            .await?;

        let contract_map = self
            .contract_repo
            .list_contracts_batch(&auth_token, &employee_ids)
            .await?;

        // 3. Match policies and compute overall qualifying range
        let mut emp_policy_map = HashMap::new();
        let mut accrual_plan = HashMap::new();
        let mut global_qualifying_start = date;

        for emp in &employees {
            let dept_id = dept_map
                .get(&emp.id())
                .map(crate::models::EmployeeDepartmentHistory::department_id);

            let mut matched = all_assignments.iter().find(|(a, _, _)| {
                a.assignee_type() == "employee" && a.assignee_id() == Some(emp.id())
            });
            if matched.is_none() {
                if let Some(d_id) = dept_id {
                    matched = all_assignments.iter().find(|(a, _, _)| {
                        a.assignee_type() == "department" && a.assignee_id() == Some(d_id)
                    });
                }
            }
            if matched.is_none() {
                matched = all_assignments.iter().find(|(a, _, _)| {
                    a.assignee_type() == "workspace" && a.assignee_id() == Some(workspace_id)
                });
            }

            if let Some((assignment, def, _code)) = matched {
                let config = def.rule_config();
                let effective_from = assignment.effective_from();
                let dates = Self::get_accrual_dates(
                    &config.frequency,
                    &config.accrual_timing,
                    effective_from,
                    date,
                );

                if !dates.is_empty() {
                    emp_policy_map.insert(emp.id(), (def.clone(), assignment.id()));
                    accrual_plan.insert(emp.id(), dates);

                    let q_start = Self::compute_qualifying_range_start(config, effective_from);
                    if q_start < global_qualifying_start {
                        global_qualifying_start = q_start;
                    }
                }
            }
        }

        if accrual_plan.is_empty() {
            return Ok(vec![]);
        }

        // 4. Batch fetch qualifying days
        let qualifying_days_map = self
            .batch_count_qualifying_days_for_workspace(
                workspace_id,
                &employee_ids,
                global_qualifying_start,
                date,
                tz,
            )
            .await?;

        let mut batch_entries = Vec::new();

        // 5. Collect accrual entries in memory (no DB calls)
        for emp in employees {
            let Some(dates) = accrual_plan.get(&emp.id()) else {
                continue;
            };
            let Some((def, assignment_id)) = emp_policy_map.get(&emp.id()) else {
                tracing::error!("policy must exist for accrued employee {}", emp.id());
                continue;
            };
            let Some(active_emp_id) = emp.active_employment_id() else {
                tracing::error!(
                    "active employment must exist for accrued employee {}",
                    emp.id()
                );
                continue;
            };
            let config = def.rule_config();
            let leave_type_id = def.leave_type_id();

            let empty_contracts = vec![];
            let contracts = contract_map.get(&emp.id()).unwrap_or(&empty_contracts);

            let empty_q_map = HashMap::new();
            let q_map = qualifying_days_map.get(&emp.id()).unwrap_or(&empty_q_map);

            for accrual_date in dates {
                let months_service =
                    Self::calculate_total_months_seniority(contracts, *accrual_date);
                let work_days = Self::resolve_qualifying_days(config, *accrual_date, q_map);
                let amount = Self::calculate_accrual_amount(config, months_service, work_days);

                if amount > 0.0 {
                    batch_entries.push(
                        crate::repositories::leave_accrual_log_repository::AccrualBatchEntry {
                            employment_id: active_emp_id,
                            assignment_id: *assignment_id,
                            leave_type_id,
                            accrual_date: *accrual_date,
                            amount,
                        },
                    );
                }
            }
        }

        // 6. Single batch DB call — all entries persisted in one transaction
        let all_logs = self
            .accrual_log_repo
            .persist_accruals_batch(&auth_token, workspace_id, &batch_entries, force)
            .await?;

        Ok(all_logs)
    }
}
