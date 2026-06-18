use crate::domain::policy::defaults::{
    DEFAULT_GRACE_PERIOD_MINUTES, DEFAULT_HOURS_PER_DAY, DEFAULT_STD_WORKING_DAYS,
};
use crate::models::{AttendanceRules, EarlyArrivalStrategy, PolicyRules, WorkingCondition};
use chrono::NaiveDate;
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct StandardMonthConfig {
    std_working_days: Decimal,
    hours_per_day: Decimal,
    source: String,
}

impl StandardMonthConfig {
    pub fn new(std_working_days: Decimal, hours_per_day: Decimal, source: String) -> Self {
        Self {
            std_working_days,
            hours_per_day,
            source,
        }
    }

    pub fn std_working_days(&self) -> Decimal {
        self.std_working_days
    }

    pub fn hours_per_day(&self) -> Decimal {
        self.hours_per_day
    }

    pub fn source(&self) -> &str {
        &self.source
    }
}

pub struct PolicyResolver;

impl PolicyResolver {
    pub fn resolve_standard_month_pure(
        emp_config: Option<crate::models::EmployeePayrollConfig>,
        dept_config: Option<crate::models::DepartmentPayrollConfig>,
        ws_settings: Option<crate::models::PayrollSettings>,
    ) -> StandardMonthConfig {
        // 1. Check Individual Override (Highest Priority)
        if let Some(config) = emp_config {
            return StandardMonthConfig::new(
                config.std_working_days,
                config.hours_per_day,
                "individual".to_string(),
            );
        }

        // 2. Check Department Override
        if let Some(config) = dept_config {
            return StandardMonthConfig::new(
                config.std_working_days,
                config.hours_per_day,
                "department".to_string(),
            );
        }

        // 3. Fallback to Workspace Settings (Temporal)
        if let Some(settings) = ws_settings {
            return StandardMonthConfig::new(
                settings.fixed_std_days,
                settings.hours_per_day,
                "workspace".to_string(),
            );
        }

        // 4. System Default fallback
        StandardMonthConfig::new(
            Decimal::from(DEFAULT_STD_WORKING_DAYS),
            Decimal::from(DEFAULT_HOURS_PER_DAY),
            "system".to_string(),
        )
    }

    pub fn resolve_attendance_rules_pure(
        attendance_rules: &AttendanceRules,
        conditions: Vec<WorkingCondition>,
        _date: NaiveDate,
    ) -> PolicyRules {
        // Merge logic
        let mut rules = PolicyRules::new(
            attendance_rules
                .grace_period_minutes
                .unwrap_or(DEFAULT_GRACE_PERIOD_MINUTES),
            0,
            false,
            attendance_rules.early_arrival_strategy,
            true, // Default
            false,
            false,
        );

        for cond in conditions {
            match cond.type_() {
                "EARLY_LEAVE" => {
                    // Take the most generous allowance
                    if cond.minute_allowance() > rules.authorized_early_leave_minutes() {
                        rules.set_authorized_early_leave_minutes(cond.minute_allowance());
                        rules.set_is_early_leave_paid(cond.is_paid());
                    }
                }
                "BREASTFEEDING" => {
                    // Example: breastfeeding might also be 60m paid early leave
                    if cond.minute_allowance() > rules.authorized_early_leave_minutes() {
                        rules.set_authorized_early_leave_minutes(cond.minute_allowance());
                        rules.set_is_early_leave_paid(true);
                    }
                }
                _ => {}
            }
        }

        rules
    }

    /// Build PolicyRules from workspace defaults and working conditions.
    /// Used after batch fetching conditions to construct rules per employee/date.
    pub fn build_rules_from_conditions(
        conditions: &[WorkingCondition],
        date: NaiveDate,
        early_arrival_strategy: EarlyArrivalStrategy,
        grace_period_minutes: i32,
    ) -> PolicyRules {
        let mut rules = PolicyRules::new(
            grace_period_minutes,
            0,
            false,
            early_arrival_strategy,
            true,
            false,
            false,
        );

        // Filter conditions active on this specific date
        for cond in conditions
            .iter()
            .filter(|c| c.start_date() <= date && c.end_date().is_none_or(|end| end >= date))
        {
            match cond.type_() {
                "EARLY_LEAVE" => {
                    if cond.minute_allowance() > rules.authorized_early_leave_minutes() {
                        rules.set_authorized_early_leave_minutes(cond.minute_allowance());
                        rules.set_is_early_leave_paid(cond.is_paid());
                    }
                }
                "BREASTFEEDING" => {
                    if cond.minute_allowance() > rules.authorized_early_leave_minutes() {
                        rules.set_authorized_early_leave_minutes(cond.minute_allowance());
                        rules.set_is_early_leave_paid(true);
                    }
                }
                _ => {}
            }
        }

        rules
    }

    pub fn apply_shift_overrides(
        mut rules: PolicyRules,
        overrides: &ShiftPolicyOverrides,
    ) -> PolicyRules {
        if let Some(bad) = overrides.is_break_auto_deduct() {
            rules.set_is_break_auto_deduct(bad);
        }
        if let Some(aeo) = overrides.allow_early_ot() {
            rules.set_allow_early_ot(aeo);
        }
        rules.set_is_break_overtime(overrides.is_break_overtime());

        rules
    }

    /// Resolve attendance rules and apply shift-specific overrides in one call (pure memory version).
    pub fn resolve_with_shift_overrides_pure(
        attendance_rules: Option<AttendanceRules>,
        conditions: Vec<WorkingCondition>,
        date: NaiveDate,
        overrides: &ShiftPolicyOverrides,
    ) -> PolicyRules {
        let base_rules = if let Some(ar) = attendance_rules {
            Self::resolve_attendance_rules_pure(&ar, conditions, date)
        } else {
            PolicyRules::new(
                DEFAULT_GRACE_PERIOD_MINUTES,
                0,
                false,
                overrides.early_arrival_strategy(),
                overrides.is_break_auto_deduct().unwrap_or(false),
                overrides.is_break_overtime(),
                overrides.allow_early_ot().unwrap_or(false),
            )
        };

        Self::apply_shift_overrides(base_rules, overrides)
    }
}

/// Shift-level policy overrides extracted from schedule/shift data.
///
/// This struct encapsulates the fields that vary per-shift and override
/// the workspace-level or working-condition-level PolicyRules.
#[derive(Debug, Clone, Default)]
pub struct ShiftPolicyOverrides {
    grace_period_minutes: Option<i32>,
    is_break_auto_deduct: Option<bool>,
    allow_early_ot: Option<bool>,
    early_arrival_strategy: EarlyArrivalStrategy,
    is_break_overtime: bool,
}

impl ShiftPolicyOverrides {
    pub fn new(
        grace_period_minutes: Option<i32>,
        is_break_auto_deduct: Option<bool>,
        allow_early_ot: Option<bool>,
        early_arrival_strategy: EarlyArrivalStrategy,
        is_break_overtime: bool,
    ) -> Self {
        Self {
            grace_period_minutes,
            is_break_auto_deduct,
            allow_early_ot,
            early_arrival_strategy,
            is_break_overtime,
        }
    }

    pub fn grace_period_minutes(&self) -> Option<i32> {
        self.grace_period_minutes
    }

    pub fn is_break_auto_deduct(&self) -> Option<bool> {
        self.is_break_auto_deduct
    }

    pub fn allow_early_ot(&self) -> Option<bool> {
        self.allow_early_ot
    }

    pub fn early_arrival_strategy(&self) -> EarlyArrivalStrategy {
        self.early_arrival_strategy
    }

    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn mock_condition(
        type_: &str,
        minute_allowance: i32,
        is_paid: bool,
        start: NaiveDate,
        end: Option<NaiveDate>,
    ) -> WorkingCondition {
        WorkingCondition::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            type_.to_string(),
            minute_allowance,
            is_paid,
            start,
            end,
        )
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
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.grace_period_minutes(), 15);
        assert_eq!(rules.authorized_early_leave_minutes(), 0);
        assert!(rules.is_break_auto_deduct());
        assert!(!rules.allow_early_ot());
    }

    #[test]
    fn test_build_rules_early_leave() {
        let conds = vec![mock_condition("EARLY_LEAVE", 60, true, d(2026, 1, 1), None)];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 60);
        assert!(rules.is_early_leave_paid());
    }

    #[test]
    fn test_build_rules_early_arrival_strategy_preserved() {
        let rules = PolicyResolver::build_rules_from_conditions(
            &[],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Clamp,
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.early_arrival_strategy(), EarlyArrivalStrategy::Clamp);

        let rules_block = PolicyResolver::build_rules_from_conditions(
            &[],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Block,
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(
            rules_block.early_arrival_strategy(),
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
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 45);
        assert!(rules.is_early_leave_paid()); // always true for BREASTFEEDING
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
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 60);
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
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 0); // expired, not applied
    }

    #[test]
    fn test_build_rules_future_condition_ignored() {
        let conds = vec![mock_condition("EARLY_LEAVE", 60, true, d(2026, 4, 1), None)];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 0); // not yet active
    }

    // --- ShiftPolicyOverrides application tests ---

    #[test]
    fn test_shift_overrides_applied_to_base_rules() {
        let rules = PolicyRules::new(
            15,
            0,
            false,
            EarlyArrivalStrategy::Allow,
            true,
            false,
            false,
        );

        let ov = ShiftPolicyOverrides::new(
            Some(30),
            Some(false),
            Some(true),
            EarlyArrivalStrategy::Clamp,
            true,
        );

        let result = PolicyResolver::apply_shift_overrides(rules, &ov);

        assert_eq!(result.grace_period_minutes(), 15); // Decoupled from shift overrides
        assert!(!result.is_break_auto_deduct());
        assert!(result.allow_early_ot());
        assert!(result.is_break_overtime());
    }

    #[test]
    fn test_shift_overrides_none_preserves_defaults() {
        let rules = PolicyRules::new(
            15,
            0,
            false,
            EarlyArrivalStrategy::Allow,
            true,
            false,
            false,
        );

        let ov = ShiftPolicyOverrides::new(None, None, None, EarlyArrivalStrategy::Allow, false);

        let result = PolicyResolver::apply_shift_overrides(rules, &ov);

        assert_eq!(result.grace_period_minutes(), 15); // unchanged
        assert!(result.is_break_auto_deduct()); // unchanged
        assert!(!result.allow_early_ot()); // unchanged
    }

    #[test]
    fn test_shift_overrides_zero_grace_not_applied() {
        let rules = PolicyRules::new(
            15,
            0,
            false,
            EarlyArrivalStrategy::Allow,
            true,
            false,
            false,
        );

        let ov = ShiftPolicyOverrides::new(Some(0), None, None, EarlyArrivalStrategy::Allow, false);

        let result = PolicyResolver::apply_shift_overrides(rules, &ov);

        assert_eq!(result.grace_period_minutes(), 15); // unchanged
    }
}
