use crate::domain::policy::defaults::{
    DEFAULT_GRACE_PERIOD_MINUTES, DEFAULT_HOURS_PER_DAY, DEFAULT_STD_WORKING_DAYS,
};
use crate::errors::AppError;
use crate::models::{AttendanceRules, EarlyArrivalStrategy, PolicyRules, WorkingCondition};
use crate::repositories::AttendancePolicyRepository;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StandardMonthConfig {
    pub std_working_days: Decimal,
    pub hours_per_day: Decimal,
    pub source: String,
}

pub struct PolicyResolver;

impl PolicyResolver {
    pub async fn resolve_standard_month(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        payroll_repo: &dyn crate::repositories::payroll_repository::PayrollRepository,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<StandardMonthConfig, AppError> {
        // 1. Check Individual Override (Highest Priority)
        let emp_config = employee_repo
            .get_employee_payroll_config_at(employee_id, date)
            .await?;

        if let Some(config) = emp_config {
            return Ok(StandardMonthConfig {
                std_working_days: config.std_working_days,
                hours_per_day: config.hours_per_day,
                source: "individual".to_string(),
            });
        }

        // 2. Check Department Override
        if let Some(dept_id) = department_id {
            let dept_config = payroll_repo
                .get_department_payroll_config_at(dept_id, date)
                .await?;

            if let Some(config) = dept_config {
                return Ok(StandardMonthConfig {
                    std_working_days: config.std_working_days,
                    hours_per_day: config.hours_per_day,
                    source: "department".to_string(),
                });
            }
        }

        // 3. Fallback to Workspace Settings (Temporal)
        let ws_settings = payroll_repo
            .get_payroll_settings_at(workspace_id, date)
            .await?;

        if let Some(settings) = ws_settings {
            return Ok(StandardMonthConfig {
                std_working_days: settings.fixed_std_days,
                hours_per_day: settings.hours_per_day,
                source: "workspace".to_string(),
            });
        }

        // 4. System Default fallback
        Ok(StandardMonthConfig {
            std_working_days: Decimal::from(DEFAULT_STD_WORKING_DAYS),
            hours_per_day: Decimal::from(DEFAULT_HOURS_PER_DAY),
            source: "system".to_string(),
        })
    }

    // Deprecated in favor of resolve_standard_month, keeping for small logic pieces if needed
    pub async fn resolve_standard_days(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        payroll_repo: &dyn crate::repositories::payroll_repository::PayrollRepository,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Decimal, AppError> {
        let config = Self::resolve_standard_month(
            employee_repo,
            payroll_repo,
            employee_id,
            department_id,
            workspace_id,
            date,
        )
        .await?;
        Ok(config.std_working_days)
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
            .get_employee_effective_policy(workspace_id, employee_id, effective_time)
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
    ) -> Result<std::collections::HashMap<Uuid, AttendanceRules>, AppError> {
        if employee_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let effective_time = date.and_hms_opt(0, 0, 0).unwrap_or_default().and_utc();

        let versions = policy_repo
            .get_employee_effective_policies_batch(workspace_id, employee_ids, effective_time)
            .await?;

        let mut result = std::collections::HashMap::with_capacity(employee_ids.len());
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
            .get_working_conditions_at(workspace_id, employee_id, date)
            .await?;

        // 3. Merge logic
        let mut rules = PolicyRules {
            grace_period_minutes: DEFAULT_GRACE_PERIOD_MINUTES,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: attendance_rules.early_arrival_strategy,
            is_break_auto_deduct: true, // Default
            is_break_overtime: false,
            allow_early_ot: false,
        };

        for cond in conditions {
            match cond.type_.as_str() {
                "EARLY_LEAVE" => {
                    // Take the most generous allowance
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = cond.is_paid;
                    }
                }
                "BREASTFEEDING" => {
                    // Example: breastfeeding might also be 60m paid early leave
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = true;
                    }
                }
                _ => {}
            }
        }

        Ok(rules)
    }

    /// Batch resolve attendance rules for multiple employees.
    /// Returns a HashMap keyed by employee_id.
    /// This eliminates N+1 queries when processing batches.
    pub async fn resolve_attendance_rules_batch(
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employee_ids: &[Uuid],
        workspace_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<HashMap<Uuid, Vec<WorkingCondition>>, AppError> {
        employee_repo
            .get_working_conditions_batch(workspace_id, employee_ids, start_date, end_date)
            .await
    }

    /// Build PolicyRules from workspace defaults and working conditions.
    /// Used after batch fetching conditions to construct rules per employee/date.
    pub fn build_rules_from_conditions(
        conditions: &[WorkingCondition],
        date: NaiveDate,
        early_arrival_strategy: EarlyArrivalStrategy,
    ) -> PolicyRules {
        let mut rules = PolicyRules {
            grace_period_minutes: DEFAULT_GRACE_PERIOD_MINUTES,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        };

        // Filter conditions active on this specific date
        for cond in conditions
            .iter()
            .filter(|c| c.start_date <= date && c.end_date.is_none_or(|end| end >= date))
        {
            match cond.type_.as_str() {
                "EARLY_LEAVE" => {
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = cond.is_paid;
                    }
                }
                "BREASTFEEDING" => {
                    if cond.minute_allowance > rules.authorized_early_leave_minutes {
                        rules.authorized_early_leave_minutes = cond.minute_allowance;
                        rules.is_early_leave_paid = true;
                    }
                }
                _ => {}
            }
        }

        rules
    }

    /// Resolve attendance rules and apply shift-specific overrides in one call.
    ///
    /// This consolidates the repeated pattern of:
    /// 1. `PolicyResolver::resolve_attendance_rules()`
    /// 2. Fallback to shift defaults if resolution fails
    /// 3. Override `grace_period_minutes`, `is_break_auto_deduct`, `allow_early_ot`
    ///    from schedule-level fields
    ///
    /// Used by: `report_service`, `payroll_service`, `attendance_summary_service`
    pub async fn resolve_with_shift_overrides(
        policy_repo: &dyn AttendancePolicyRepository,
        employee_repo: &dyn crate::repositories::employee::EmployeeRepository,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
        overrides: &ShiftPolicyOverrides,
    ) -> PolicyRules {
        let mut rules = Self::resolve_attendance_rules(
            policy_repo,
            employee_repo,
            employee_id,
            workspace_id,
            date,
        )
        .await
        .unwrap_or(PolicyRules {
            grace_period_minutes: overrides.grace_period_minutes.unwrap_or(0),
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: overrides.early_arrival_strategy,
            is_break_auto_deduct: overrides.is_break_auto_deduct.unwrap_or(false),
            is_break_overtime: overrides.is_break_overtime,
            allow_early_ot: overrides.allow_early_ot.unwrap_or(false),
        });

        // Apply shift-specific overrides (shift config takes precedence)
        if let Some(gp) = overrides.grace_period_minutes {
            if gp > 0 {
                rules.grace_period_minutes = gp;
            }
        }
        if let Some(bad) = overrides.is_break_auto_deduct {
            rules.is_break_auto_deduct = bad;
        }
        if let Some(aeo) = overrides.allow_early_ot {
            rules.allow_early_ot = aeo;
        }
        rules.is_break_overtime = overrides.is_break_overtime;

        rules
    }
}

/// Shift-level policy overrides extracted from schedule/shift data.
///
/// This struct encapsulates the fields that vary per-shift and override
/// the workspace-level or working-condition-level PolicyRules.
#[derive(Debug, Clone, Default)]
pub struct ShiftPolicyOverrides {
    pub grace_period_minutes: Option<i32>,
    pub is_break_auto_deduct: Option<bool>,
    pub allow_early_ot: Option<bool>,
    pub early_arrival_strategy: EarlyArrivalStrategy,
    pub is_break_overtime: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_condition(
        type_: &str,
        minute_allowance: i32,
        is_paid: bool,
        start: NaiveDate,
        end: Option<NaiveDate>,
    ) -> WorkingCondition {
        WorkingCondition {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            type_: type_.to_string(),
            minute_allowance,
            is_paid,
            start_date: start,
            end_date: end,
        }
    }

    fn d(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    // --- build_rules_from_conditions tests ---

    #[test]
    fn test_build_rules_no_conditions() {
        let rules = PolicyResolver::build_rules_from_conditions(
            &[],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
        );
        assert_eq!(rules.grace_period_minutes, 15);
        assert_eq!(rules.authorized_early_leave_minutes, 0);
        assert!(rules.is_break_auto_deduct);
        assert!(!rules.allow_early_ot);
    }

    #[test]
    fn test_build_rules_early_leave() {
        let conds = vec![mock_condition("EARLY_LEAVE", 60, true, d(2026, 1, 1), None)];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
        );
        assert_eq!(rules.authorized_early_leave_minutes, 60);
        assert!(rules.is_early_leave_paid);
    }

    #[test]
    fn test_build_rules_early_arrival_strategy_preserved() {
        let rules = PolicyResolver::build_rules_from_conditions(
            &[],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Clamp,
        );
        assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Clamp);

        let rules_block = PolicyResolver::build_rules_from_conditions(
            &[],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Block,
        );
        assert_eq!(
            rules_block.early_arrival_strategy,
            EarlyArrivalStrategy::Block
        );
    }

    #[test]
    fn test_build_rules_breastfeeding_always_paid() {
        let conds = vec![mock_condition(
            "BREASTFEEDING",
            45,
            false,
            d(2026, 1, 1),
            None,
        )];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
        );
        assert_eq!(rules.authorized_early_leave_minutes, 45);
        assert!(rules.is_early_leave_paid); // always true for BREASTFEEDING
    }

    #[test]
    fn test_build_rules_most_generous_wins() {
        let conds = vec![
            mock_condition("EARLY_LEAVE", 30, false, d(2026, 1, 1), None),
            mock_condition("BREASTFEEDING", 60, false, d(2026, 1, 1), None),
        ];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
        );
        assert_eq!(rules.authorized_early_leave_minutes, 60);
    }

    #[test]
    fn test_build_rules_expired_condition_ignored() {
        let conds = vec![mock_condition(
            "EARLY_LEAVE",
            60,
            true,
            d(2026, 1, 1),
            Some(d(2026, 2, 28)),
        )];
        // Date after expiry
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
        );
        assert_eq!(rules.authorized_early_leave_minutes, 0); // expired, not applied
    }

    #[test]
    fn test_build_rules_future_condition_ignored() {
        let conds = vec![mock_condition("EARLY_LEAVE", 60, true, d(2026, 4, 1), None)];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
        );
        assert_eq!(rules.authorized_early_leave_minutes, 0); // not yet active
    }

    // --- ShiftPolicyOverrides application tests ---

    #[test]
    fn test_shift_overrides_applied_to_base_rules() {
        let mut rules = PolicyRules {
            grace_period_minutes: 15,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        };

        let ov = ShiftPolicyOverrides {
            grace_period_minutes: Some(30),
            is_break_auto_deduct: Some(false),
            allow_early_ot: Some(true),
            early_arrival_strategy: EarlyArrivalStrategy::Clamp,
            is_break_overtime: true,
        };

        // Simulate the override logic from resolve_with_shift_overrides
        if let Some(gp) = ov.grace_period_minutes {
            if gp > 0 {
                rules.grace_period_minutes = gp;
            }
        }
        if let Some(bad) = ov.is_break_auto_deduct {
            rules.is_break_auto_deduct = bad;
        }
        if let Some(aeo) = ov.allow_early_ot {
            rules.allow_early_ot = aeo;
        }
        rules.is_break_overtime = ov.is_break_overtime;

        assert_eq!(rules.grace_period_minutes, 30);
        assert!(!rules.is_break_auto_deduct);
        assert!(rules.allow_early_ot);
        assert!(rules.is_break_overtime);
    }

    #[test]
    fn test_shift_overrides_none_preserves_defaults() {
        let mut rules = PolicyRules {
            grace_period_minutes: 15,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        };

        let ov = ShiftPolicyOverrides {
            grace_period_minutes: None,
            is_break_auto_deduct: None,
            allow_early_ot: None,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_overtime: false,
        };

        if let Some(gp) = ov.grace_period_minutes {
            if gp > 0 {
                rules.grace_period_minutes = gp;
            }
        }
        if let Some(bad) = ov.is_break_auto_deduct {
            rules.is_break_auto_deduct = bad;
        }
        if let Some(aeo) = ov.allow_early_ot {
            rules.allow_early_ot = aeo;
        }
        rules.is_break_overtime = ov.is_break_overtime;

        assert_eq!(rules.grace_period_minutes, 15); // unchanged
        assert!(rules.is_break_auto_deduct); // unchanged
        assert!(!rules.allow_early_ot); // unchanged
    }

    #[test]
    fn test_shift_overrides_zero_grace_not_applied() {
        let mut rules = PolicyRules {
            grace_period_minutes: 15,
            authorized_early_leave_minutes: 0,
            is_early_leave_paid: false,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        };

        let ov = ShiftPolicyOverrides {
            grace_period_minutes: Some(0),
            is_break_auto_deduct: None,
            allow_early_ot: None,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_overtime: false,
        };

        if let Some(gp) = ov.grace_period_minutes {
            if gp > 0 {
                rules.grace_period_minutes = gp;
            }
        }

        assert_eq!(rules.grace_period_minutes, 15); // zero not applied
    }

    #[tokio::test]
    async fn test_resolve_attendance_policy_with_mock() {
        use crate::models::{AttendancePolicyVersion, AttendanceRules, EarlyArrivalStrategy};
        use crate::repositories::attendance_policy_repository::MockAttendancePolicyRepository;

        let mut mock_repo = MockAttendancePolicyRepository::new();
        mock_repo
            .expect_get_employee_effective_policy()
            .returning(|_, _, _| {
                Ok(Some(AttendancePolicyVersion {
                    id: Uuid::new_v4(),
                    policy_id: Uuid::new_v4(),
                    rules: AttendanceRules {
                        work_unit_minutes: 15,
                        rounding_policy: "ceil".to_string(),
                        early_arrival_strategy: EarlyArrivalStrategy::Block,
                    },
                    valid_from: chrono::Utc::now(),
                    valid_to: None,
                    created_by: None,
                    created_at: chrono::Utc::now(),
                }))
            });

        let employee_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();
        let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();

        let rules =
            PolicyResolver::resolve_attendance_policy(&mock_repo, employee_id, workspace_id, date)
                .await
                .unwrap();

        assert_eq!(rules.work_unit_minutes, 15);
        assert_eq!(rules.rounding_policy, "ceil");
        assert_eq!(rules.early_arrival_strategy, EarlyArrivalStrategy::Block);
    }
}
