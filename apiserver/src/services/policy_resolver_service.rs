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

    pub async fn resolve_attendance_policy_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employment_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<AttendanceRules, AppError> {
        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let policy_version = policy_repo
            .get_employment_effective_policy(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
                effective_time,
            )
            .await?;

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

    pub async fn resolve_attendance_policy_batch_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employment_ids: &[Uuid],
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<HashMap<Uuid, AttendanceRules>, AppError> {
        if employment_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let versions = policy_repo
            .get_employment_effective_policies_batch(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_ids,
                effective_time,
            )
            .await?;

        let mut result = HashMap::with_capacity(employment_ids.len());
        for &id in employment_ids {
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

    pub async fn resolve_attendance_rules_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employment_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<PolicyRules, AppError> {
        // 1. Get Attendance Policy rules by employment_id
        let attendance_rules = Self::resolve_attendance_policy_by_employment(
            policy_repo,
            employment_id,
            workspace_id,
            date,
        )
        .await?;

        // 2. Get Employee Special Conditions by employment_id
        let conditions = employee_repo
            .get_working_conditions_at(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employment_id,
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

    pub async fn resolve_attendance_rules_batch_by_employment(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employment_ids: &[Uuid],
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);
        employee_repo
            .get_working_conditions_batch_by_employments(
                &auth_token,
                employment_ids,
                start_date,
                end_date,
            )
            .await
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

    pub async fn resolve_with_shift_overrides_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employment_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
        overrides: &ShiftPolicyOverrides,
    ) -> PolicyRules {
        let rules_res = Self::resolve_attendance_rules_by_employment(
            policy_repo,
            employee_repo,
            employment_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EarlyArrivalStrategy;
    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::employee::MockEmployeeRepository;
    use crate::repositories::payroll_repository::MockPayrollRepository;
    use chrono::NaiveDate;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_resolve_standard_month() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let dept_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employee_payroll_config_at()
            .returning(|_, _, _| Ok(None));

        let mut payroll_repo = MockPayrollRepository::new();
        payroll_repo
            .expect_get_department_payroll_config_at()
            .returning(|_, _, _| Ok(None));
        payroll_repo
            .expect_get_payroll_settings_at()
            .returning(|_, _| Ok(None));

        let res = PolicyResolverService::resolve_standard_month(
            &employee_repo,
            &payroll_repo,
            emp_id,
            Some(dept_id),
            ws_id,
            date,
        )
        .await;
        assert!(res.is_ok());

        // Test without department
        let res_no_dept = PolicyResolverService::resolve_standard_month(
            &employee_repo,
            &payroll_repo,
            emp_id,
            None,
            ws_id,
            date,
        )
        .await;
        assert!(res_no_dept.is_ok());
    }

    #[tokio::test]
    async fn test_resolve_attendance_policy() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        policy_repo
            .expect_get_employee_effective_policy()
            .returning(|_, _, _| Ok(None));

        let rules =
            PolicyResolverService::resolve_attendance_policy(&policy_repo, emp_id, ws_id, date)
                .await
                .unwrap();

        assert_eq!(rules.work_unit_minutes, 30);
        assert_eq!(rules.rounding_policy, "floor");
    }

    #[tokio::test]
    async fn test_resolve_attendance_policy_by_employment() {
        let ws_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        policy_repo
            .expect_get_employment_effective_policy()
            .returning(|_, _, _| Ok(None));

        let rules = PolicyResolverService::resolve_attendance_policy_by_employment(
            &policy_repo,
            employment_id,
            ws_id,
            date,
        )
        .await
        .unwrap();

        assert_eq!(rules.work_unit_minutes, 30);
    }

    #[tokio::test]
    async fn test_resolve_attendance_policy_batch() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        // Empty batch
        let res_empty =
            PolicyResolverService::resolve_attendance_policy_batch(&policy_repo, &[], ws_id, date)
                .await
                .unwrap();
        assert!(res_empty.is_empty());

        policy_repo
            .expect_get_employee_effective_policies_batch()
            .returning(|_, _, _| Ok(HashMap::new()));

        let res = PolicyResolverService::resolve_attendance_policy_batch(
            &policy_repo,
            &[emp_id],
            ws_id,
            date,
        )
        .await
        .unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res.get(&emp_id).unwrap().work_unit_minutes, 30);
    }

    #[tokio::test]
    async fn test_resolve_attendance_policy_batch_by_employment() {
        let ws_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        // Empty batch
        let res_empty = PolicyResolverService::resolve_attendance_policy_batch_by_employment(
            &policy_repo,
            &[],
            ws_id,
            date,
        )
        .await
        .unwrap();
        assert!(res_empty.is_empty());

        policy_repo
            .expect_get_employment_effective_policies_batch()
            .returning(|_, _, _| Ok(HashMap::new()));

        let res = PolicyResolverService::resolve_attendance_policy_batch_by_employment(
            &policy_repo,
            &[employment_id],
            ws_id,
            date,
        )
        .await
        .unwrap();
        assert_eq!(res.len(), 1);
    }

    #[tokio::test]
    async fn test_resolve_attendance_rules() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        policy_repo
            .expect_get_employee_effective_policy()
            .returning(|_, _, _| Ok(None));

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_working_conditions_at()
            .returning(|_, _, _| Ok(vec![]));

        let rules = PolicyResolverService::resolve_attendance_rules(
            &policy_repo,
            &employee_repo,
            emp_id,
            ws_id,
            date,
        )
        .await
        .unwrap();

        assert_eq!(rules.grace_period_minutes(), 15);
    }

    #[tokio::test]
    async fn test_resolve_attendance_rules_by_employment() {
        let ws_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        policy_repo
            .expect_get_employment_effective_policy()
            .returning(|_, _, _| Ok(None));

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_working_conditions_at()
            .returning(|_, _, _| Ok(vec![]));

        let rules = PolicyResolverService::resolve_attendance_rules_by_employment(
            &policy_repo,
            &employee_repo,
            employment_id,
            ws_id,
            date,
        )
        .await
        .unwrap();

        assert_eq!(rules.grace_period_minutes(), 15);
    }

    #[tokio::test]
    async fn test_resolve_attendance_rules_batch() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_employees_by_ids()
            .returning(|_, _| Ok(vec![]));
        employee_repo
            .expect_get_working_conditions_batch_by_employments()
            .returning(|_, _, _, _| Ok(HashMap::new()));

        let res = PolicyResolverService::resolve_attendance_rules_batch(
            &employee_repo,
            &[emp_id],
            ws_id,
            date,
            date,
        )
        .await
        .unwrap();

        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_attendance_rules_batch_by_employment() {
        let ws_id = Uuid::new_v4();
        let employment_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_working_conditions_batch_by_employments()
            .returning(|_, _, _, _| Ok(HashMap::new()));

        let res = PolicyResolverService::resolve_attendance_rules_batch_by_employment(
            &employee_repo,
            &[employment_id],
            ws_id,
            date,
            date,
        )
        .await
        .unwrap();

        assert!(res.is_empty());
    }

    #[tokio::test]
    async fn test_resolve_with_shift_overrides() {
        let ws_id = Uuid::new_v4();
        let emp_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        policy_repo
            .expect_get_employee_effective_policy()
            .returning(|_, _, _| Ok(None));

        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_working_conditions_at()
            .returning(|_, _, _| Ok(vec![]));

        let overrides =
            ShiftPolicyOverrides::new(Some(10), None, None, EarlyArrivalStrategy::Allow, false);
        let rules = PolicyResolverService::resolve_with_shift_overrides(
            &policy_repo,
            &employee_repo,
            emp_id,
            ws_id,
            date,
            &overrides,
        )
        .await;

        assert_eq!(rules.grace_period_minutes(), 10);

        // Test resolve_with_shift_overrides_by_employment
        let mut policy_repo_emp = MockAttendancePolicyRepository::new();
        policy_repo_emp
            .expect_get_employment_effective_policy()
            .returning(|_, _, _| Ok(None));

        let rules_emp = PolicyResolverService::resolve_with_shift_overrides_by_employment(
            &policy_repo_emp,
            &employee_repo,
            Uuid::new_v4(),
            ws_id,
            date,
            &overrides,
        )
        .await;
        assert_eq!(rules_emp.grace_period_minutes(), 10);
    }
}
