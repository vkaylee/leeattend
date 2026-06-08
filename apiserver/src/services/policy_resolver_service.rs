use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides, StandardMonthConfig};
use crate::errors::AppError;
use crate::models::{AttendanceRules, PolicyRules, WorkingCondition};
use crate::repositories::AttendancePolicyRepository;
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PolicyResolverService;

impl PolicyResolverService {
    pub async fn resolve_standard_month(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        payroll_repo: &dyn crate::repositories::payroll_repository::PayrollRepository,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<StandardMonthConfig, AppError> {
        let emp_config = employee_repo
            .get_employee_payroll_config_at(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await?;

        let dept_config = if let Some(dept_id) = department_id {
            payroll_repo
                .get_department_payroll_config_at(workspace_id, dept_id, date)
                .await?
        } else {
            None
        };

        let ws_settings = payroll_repo
            .get_payroll_settings_at(workspace_id, date)
            .await?;

        Ok(PolicyResolver::resolve_standard_month_pure(
            emp_config,
            dept_config,
            ws_settings,
        ))
    }

    pub async fn resolve_attendance_policy(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<AttendanceRules, AppError> {
        // Convert NaiveDate to UTC start of day
        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let policy_version = policy_repo
            .get_employee_effective_policy(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                effective_time,
            )
            .await?;

        // Fallback if no policy found
        if let Some(pv) = policy_version {
            Ok(pv.rules)
        } else {
            Ok(AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: "floor".to_string(),
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
            })
        }
    }

    pub async fn resolve_attendance_policy_batch(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_ids: &[Uuid],
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<HashMap<Uuid, AttendanceRules>, AppError> {
        if employee_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let versions = policy_repo
            .get_employee_effective_policies_batch(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_ids,
                effective_time,
            )
            .await?;

        let mut result = HashMap::with_capacity(employee_ids.len());
        for &id in employee_ids {
            if let Some(pv) = versions.get(&id) {
                result.insert(id, pv.rules.clone());
            } else {
                result.insert(
                    id,
                    AttendanceRules {
                        work_unit_minutes: 30,
                        rounding_policy: "floor".to_string(),
                        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                    },
                );
            }
        }

        Ok(result)
    }

    pub async fn resolve_attendance_rules(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<PolicyRules, AppError> {
        // 1. Get Attendance Policy rules
        let attendance_rules =
            Self::resolve_attendance_policy(policy_repo, employee_id, workspace_id, date).await?;

        // 2. Get Employee Special Conditions
        let conditions = employee_repo
            .get_working_conditions_at(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                date,
            )
            .await?;

        Ok(PolicyResolver::resolve_attendance_rules_pure(
            &attendance_rules,
            conditions,
            date,
        ))
    }

    pub async fn resolve_attendance_rules_batch(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employee_ids: &[Uuid],
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);
        let employees = employee_repo
            .get_employees_by_ids(employee_ids, &auth_token)
            .await?;

        let mut emp_to_employment = HashMap::new();
        let mut employment_ids = Vec::new();
        for emp in &employees {
            if let Some(employment_id) = emp.active_employment_id() {
                emp_to_employment.insert(emp.id(), employment_id);
                employment_ids.push(employment_id);
            }
        }

        let conditions = employee_repo
            .get_working_conditions_batch_by_employments(
                &auth_token,
                &employment_ids,
                start_date,
                end_date,
            )
            .await?;

        let mut result = HashMap::new();
        for (emp_id, employment_id) in &emp_to_employment {
            if let Some(list) = conditions.get(employment_id) {
                result.insert(*emp_id, list.clone());
            }
        }

        Ok(result)
    }

    pub async fn resolve_with_shift_overrides(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
        overrides: &ShiftPolicyOverrides,
    ) -> PolicyRules {
        let rules_res = Self::resolve_attendance_rules(
            policy_repo,
            employee_repo,
            employee_id,
            workspace_id,
            date,
        )
        .await;

        let rules = rules_res.unwrap_or_else(|_| {
            PolicyRules::new(
                overrides.grace_period_minutes().unwrap_or(0),
                0,
                false,
                overrides.early_arrival_strategy(),
                overrides.is_break_auto_deduct().unwrap_or(false),
                overrides.is_break_overtime(),
                overrides.allow_early_ot().unwrap_or(false),
            )
        });

        PolicyResolver::apply_shift_overrides(rules, overrides)
    }
}
