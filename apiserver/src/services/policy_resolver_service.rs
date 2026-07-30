use crate::domain::policy::resolver::{PolicyResolver, ShiftPolicyOverrides, StandardMonthConfig};
use crate::errors::AppError;
use crate::models::{AttendanceRules, ConditionTypeDefinition, PolicyRules, WorkingCondition};
use crate::repositories::{AttendancePolicyRepository, ConditionTypeRepository};
use chrono::NaiveDate;
use std::collections::HashMap;
use uuid::Uuid;

pub struct PolicyResolverService;

impl PolicyResolverService {
    pub async fn load_condition_type_catalog(
        condition_type_repo: &dyn ConditionTypeRepository,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<HashMap<String, ConditionTypeDefinition>, AppError> {
        let definitions = condition_type_repo
            .list_condition_types(auth_token, false)
            .await?;

        Ok(definitions
            .into_iter()
            .map(|definition| (definition.code().to_string(), definition))
            .collect())
    }

    pub async fn resolve_standard_month(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        payroll_repo: &dyn crate::repositories::payroll_repository::PayrollRepository,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<StandardMonthConfig, AppError> {
        let workspace_id = auth_token.workspace_id();
        let emp_config = employee_repo
            .get_employee_payroll_config_at(auth_token, employee_id, date)
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<AttendanceRules, AppError> {
        // Convert NaiveDate to UTC start of day
        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let policy_version = policy_repo
            .get_employee_effective_policy(auth_token, employee_id, effective_time)
            .await?;

        // Fallback if no policy found
        if let Some(pv) = policy_version {
            Ok(pv.rules)
        } else {
            Ok(AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            })
        }
    }

    pub async fn resolve_attendance_policy_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employment_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<AttendanceRules, AppError> {
        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let policy_version = policy_repo
            .get_employment_effective_policy(auth_token, employment_id, effective_time)
            .await?;

        if let Some(pv) = policy_version {
            Ok(pv.rules)
        } else {
            Ok(AttendanceRules {
                work_unit_minutes: 30,
                rounding_policy: crate::models::RoundingPolicy::Floor,
                early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                grace_period_minutes: None,
            })
        }
    }

    pub async fn resolve_attendance_policy_batch(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<HashMap<Uuid, AttendanceRules>, AppError> {
        if employee_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let versions = policy_repo
            .get_employee_effective_policies_batch(auth_token, employee_ids, effective_time)
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
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    },
                );
            }
        }

        Ok(result)
    }

    pub async fn resolve_attendance_policy_batch_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employment_ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<HashMap<Uuid, AttendanceRules>, AppError> {
        if employment_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let versions = policy_repo
            .get_employment_effective_policies_batch(auth_token, employment_ids, effective_time)
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
                        rounding_policy: crate::models::RoundingPolicy::Floor,
                        early_arrival_strategy: crate::models::EarlyArrivalStrategy::Block,
                        grace_period_minutes: None,
                    },
                );
            }
        }

        Ok(result)
    }

    pub async fn resolve_attendance_rules(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        condition_type_repo: &dyn ConditionTypeRepository,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<PolicyRules, AppError> {
        // 1. Get Attendance Policy rules
        let attendance_rules =
            Self::resolve_attendance_policy(policy_repo, employee_id, auth_token, date).await?;

        // 2. Get Employee Special Conditions
        let conditions = employee_repo
            .get_working_conditions_at(auth_token, employee_id, date)
            .await?;
        let catalog = Self::load_condition_type_catalog(condition_type_repo, auth_token).await?;

        Ok(PolicyResolver::resolve_attendance_rules_pure(
            &attendance_rules,
            conditions,
            date,
            Some(&catalog),
        ))
    }

    pub async fn resolve_attendance_rules_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        condition_type_repo: &dyn ConditionTypeRepository,
        employment_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
    ) -> Result<PolicyRules, AppError> {
        // 1. Get Attendance Policy rules by employment_id
        let attendance_rules = Self::resolve_attendance_policy_by_employment(
            policy_repo,
            employment_id,
            auth_token,
            date,
        )
        .await?;

        // 2. Get Employee Special Conditions by employment_id
        let conditions = employee_repo
            .get_working_conditions_at(auth_token, employment_id, date)
            .await?;
        let catalog = Self::load_condition_type_catalog(condition_type_repo, auth_token).await?;

        Ok(PolicyResolver::resolve_attendance_rules_pure(
            &attendance_rules,
            conditions,
            date,
            Some(&catalog),
        ))
    }

    pub async fn resolve_attendance_rules_batch(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employee_ids: &[Uuid],
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        let employees = employee_repo
            .get_employees_by_ids(employee_ids, auth_token)
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
                auth_token,
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
        auth_token: &crate::diesel_pool::TenantAuthToken,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        employee_repo
            .get_working_conditions_batch_by_employments(
                auth_token,
                employment_ids,
                start_date,
                end_date,
            )
            .await
    }

    pub async fn resolve_with_shift_overrides(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        condition_type_repo: &dyn ConditionTypeRepository,
        employee_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
        overrides: &ShiftPolicyOverrides,
    ) -> Result<PolicyRules, AppError> {
        let rules = Self::resolve_attendance_rules(
            policy_repo,
            employee_repo,
            condition_type_repo,
            employee_id,
            auth_token,
            date,
        )
        .await?;

        Ok(PolicyResolver::apply_shift_overrides(rules, overrides))
    }

    pub async fn resolve_with_shift_overrides_by_employment(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        condition_type_repo: &dyn ConditionTypeRepository,
        employment_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        date: NaiveDate,
        overrides: &ShiftPolicyOverrides,
    ) -> Result<PolicyRules, AppError> {
        let rules = Self::resolve_attendance_rules_by_employment(
            policy_repo,
            employee_repo,
            condition_type_repo,
            employment_id,
            auth_token,
            date,
        )
        .await?;

        Ok(PolicyResolver::apply_shift_overrides(rules, overrides))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EarlyArrivalStrategy;
    use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;
    use crate::repositories::condition_type_repository::MockConditionTypeRepository;
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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

        let rules = PolicyResolverService::resolve_attendance_policy(
            &policy_repo,
            emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
        )
        .await
        .unwrap();

        assert_eq!(rules.work_unit_minutes, 30);
        assert_eq!(rules.rounding_policy, crate::models::RoundingPolicy::Floor);
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
        let res_empty = PolicyResolverService::resolve_attendance_policy_batch(
            &policy_repo,
            &[],
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
        )
        .await
        .unwrap();
        assert!(res_empty.is_empty());

        policy_repo
            .expect_get_employee_effective_policies_batch()
            .returning(|_, _, _| Ok(HashMap::new()));

        let res = PolicyResolverService::resolve_attendance_policy_batch(
            &policy_repo,
            &[emp_id],
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
        )
        .await
        .unwrap();
        assert_eq!(res.len(), 1);
    }

    fn empty_condition_type_repo() -> MockConditionTypeRepository {
        let mut repo = MockConditionTypeRepository::new();
        repo.expect_list_condition_types()
            .returning(|_, _| Box::pin(async move { Ok(vec![]) }));
        repo
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

        let condition_type_repo = empty_condition_type_repo();
        let rules = PolicyResolverService::resolve_attendance_rules(
            &policy_repo,
            &employee_repo,
            &condition_type_repo,
            emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
        )
        .await
        .unwrap();

        assert_eq!(rules.grace_period_minutes(), 15);
    }

    #[tokio::test]
    async fn custom_condition_type_affects_direct_policy_resolution() {
        let ws_id = Uuid::new_v4();
        let employee_id = Uuid::new_v4();
        let condition_type_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        let mut policy_repo = MockAttendancePolicyRepository::new();
        policy_repo
            .expect_get_employee_effective_policy()
            .returning(|_, _, _| Ok(None));

        let condition = WorkingCondition::new(
            Uuid::new_v4(),
            employee_id,
            "CUSTOM_PAID_EARLY_EXIT".to_string(),
            Some(condition_type_id),
            45,
            true,
            date,
            None,
        );
        let mut employee_repo = MockEmployeeRepository::new();
        employee_repo
            .expect_get_working_conditions_at()
            .returning(move |_, _, _| Ok(vec![condition.clone()]));

        let definition = ConditionTypeDefinition::new(
            condition_type_id,
            ws_id,
            "CUSTOM_PAID_EARLY_EXIT".to_string(),
            serde_json::json!({"en": "Custom paid early exit"}),
            "working_hours".to_string(),
            45,
            true,
            serde_json::json!({
                "authorized_early_leave_minutes": true,
                "is_early_leave_paid": true
            }),
            None,
            None,
            false,
            0,
            false,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );
        let mut condition_type_repo = MockConditionTypeRepository::new();
        condition_type_repo
            .expect_list_condition_types()
            .withf(|_, active_only| !active_only)
            .times(1)
            .returning(move |_, _| {
                let definition = definition.clone();
                Box::pin(async move { Ok(vec![definition]) })
            });

        let rules = PolicyResolverService::resolve_attendance_rules(
            &policy_repo,
            &employee_repo,
            &condition_type_repo,
            employee_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
        )
        .await
        .unwrap();

        assert_eq!(rules.authorized_early_leave_minutes(), 45);
        assert!(rules.is_early_leave_paid());
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

        let condition_type_repo = empty_condition_type_repo();
        let rules = PolicyResolverService::resolve_attendance_rules_by_employment(
            &policy_repo,
            &employee_repo,
            &condition_type_repo,
            employment_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
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
        let condition_type_repo = empty_condition_type_repo();
        let rules = PolicyResolverService::resolve_with_shift_overrides(
            &policy_repo,
            &employee_repo,
            &condition_type_repo,
            emp_id,
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
            &overrides,
        )
        .await
        .unwrap();

        assert_eq!(rules.grace_period_minutes(), 15);

        // Test resolve_with_shift_overrides_by_employment
        let mut policy_repo_emp = MockAttendancePolicyRepository::new();
        policy_repo_emp
            .expect_get_employment_effective_policy()
            .returning(|_, _, _| Ok(None));

        let condition_type_repo_emp = empty_condition_type_repo();
        let rules_emp = PolicyResolverService::resolve_with_shift_overrides_by_employment(
            &policy_repo_emp,
            &employee_repo,
            &condition_type_repo_emp,
            Uuid::new_v4(),
            &crate::diesel_pool::TenantAuthToken::system_override(ws_id),
            date,
            &overrides,
        )
        .await
        .unwrap();
        assert_eq!(rules_emp.grace_period_minutes(), 15);
    }
}
