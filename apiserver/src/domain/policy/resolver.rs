use crate::domain::policy::defaults::{
    DEFAULT_GRACE_PERIOD_MINUTES, DEFAULT_HOURS_PER_DAY, DEFAULT_STD_WORKING_DAYS,
};
use crate::models::{AttendanceRules, ConditionTypeDefinition, EarlyArrivalStrategy, PolicyRules, WorkingCondition};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;

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

    /// Apply a working condition's effects from the workspace catalog.
    ///
    /// The catalog is authoritative: a condition without a matching definition
    /// intentionally has no effect on policy rules.
    fn apply_condition_effects(
        rules: &mut PolicyRules,
        cond: &WorkingCondition,
        catalog: Option<&HashMap<String, ConditionTypeDefinition>>,
    ) {
        let Some(catalog) = catalog else {
            return;
        };

        // Lookup by immutable definition id first, then by legacy code for rows
        // created before condition_type_id was introduced.
        let type_def = cond
            .condition_type_id()
            .and_then(|ct_id| catalog.values().find(|d| d.id() == ct_id))
            .or_else(|| catalog.get(cond.type_()));

        let Some(def) = type_def else {
            return;
        };

        let affects: HashMap<String, bool> =
            serde_json::from_value(def.affects_rules().clone()).unwrap_or_default();

        if affects.get("authorized_early_leave_minutes") == Some(&true)
            && cond.minute_allowance() > rules.authorized_early_leave_minutes()
        {
            rules.set_authorized_early_leave_minutes(cond.minute_allowance());
            rules.set_is_early_leave_paid(cond.is_paid());
        }
        if affects.get("is_early_leave_paid") == Some(&true) {
            rules.set_is_early_leave_paid(cond.is_paid());
        }
    }

    pub fn resolve_attendance_rules_pure(
        attendance_rules: &AttendanceRules,
        conditions: Vec<WorkingCondition>,
        _date: NaiveDate,
        catalog: Option<&HashMap<String, ConditionTypeDefinition>>,
    ) -> PolicyRules {
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
            Self::apply_condition_effects(&mut rules, &cond, catalog);
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
        catalog: Option<&HashMap<String, ConditionTypeDefinition>>,
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

        for cond in conditions
            .iter()
            .filter(|c| c.start_date() <= date && c.end_date().is_none_or(|end| end >= date))
        {
            Self::apply_condition_effects(&mut rules, cond, catalog);
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
        catalog: Option<&HashMap<String, ConditionTypeDefinition>>,
        overrides: &ShiftPolicyOverrides,
    ) -> PolicyRules {
        let base_rules = if let Some(ar) = attendance_rules {
            Self::resolve_attendance_rules_pure(&ar, conditions, date, catalog)
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
            None,
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
            Some(&system_catalog()),
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
            Some(&system_catalog()),
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
            Some(&system_catalog()),
        );
        assert_eq!(rules.early_arrival_strategy(), EarlyArrivalStrategy::Clamp);

        let rules_block = PolicyResolver::build_rules_from_conditions(
            &[],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Block,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&system_catalog()),
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
            Some(&system_catalog()),
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 45);
        assert!(!rules.is_early_leave_paid()); // catalog respects the assigned payment flag
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
            Some(&system_catalog()),
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
            Some(&system_catalog()),
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
            Some(&system_catalog()),
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

    // --- Data-driven catalog tests ---

    fn catalog_def(code: &str, affects: serde_json::Value) -> ConditionTypeDefinition {
        ConditionTypeDefinition::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            code.to_string(),
            serde_json::json!({"en": code}),
            "general".to_string(),
            0,
            false,
            affects,
            None,
            None,
            true,
            0,
            true,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
    }

    fn make_catalog(entries: Vec<ConditionTypeDefinition>) -> HashMap<String, ConditionTypeDefinition> {
        entries.into_iter().map(|d| (d.code().to_string(), d)).collect()
    }

    fn system_catalog() -> HashMap<String, ConditionTypeDefinition> {
        make_catalog(vec![
            catalog_def(
                "EARLY_LEAVE",
                serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
            ),
            catalog_def(
                "BREASTFEEDING",
                serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
            ),
        ])
    }

    #[test]
    fn catalog_early_leave_affects_rules() {
        let catalog = make_catalog(vec![catalog_def(
            "EARLY_LEAVE",
            serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
        )]);

        let cond = mock_condition("EARLY_LEAVE", 45, true, d(2026, 1, 1), None);
        let rules = PolicyResolver::build_rules_from_conditions(
            &[cond],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&catalog),
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 45);
        assert!(rules.is_early_leave_paid());
    }

    #[test]
    fn catalog_unknown_type_is_skipped() {
        let catalog = make_catalog(vec![]);
        let cond = mock_condition("UNKNOWN_TYPE", 99, true, d(2026, 1, 1), None);
        let rules = PolicyResolver::build_rules_from_conditions(
            &[cond],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&catalog),
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 0);
    }

    #[test]
    fn catalog_empty_affects_rules_has_no_effect() {
        let catalog = make_catalog(vec![catalog_def(
            "CUSTOM",
            serde_json::json!({}),
        )]);
        let cond = mock_condition("CUSTOM", 60, true, d(2026, 1, 1), None);
        let rules = PolicyResolver::build_rules_from_conditions(
            &[cond],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&catalog),
        );
        // No effects because affects_rules is empty
        assert_eq!(rules.authorized_early_leave_minutes(), 0);
    }

    #[test]
    fn catalog_is_authoritative_when_definition_is_missing() {
        let cond = mock_condition("EARLY_LEAVE", 30, true, d(2026, 1, 1), None);
        let rules = PolicyResolver::build_rules_from_conditions(
            &[cond],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&HashMap::new()),
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 0);
        assert!(!rules.is_early_leave_paid());
    }

    #[test]
    fn catalog_lookup_by_condition_type_id() {
        let ct_id = Uuid::new_v4();
        let catalog_def = ConditionTypeDefinition::new(
            ct_id,
            Uuid::new_v4(),
            "CUSTOM_V2".to_string(),
            serde_json::json!({"en": "Custom V2"}),
            "general".to_string(),
            0,
            false,
            serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
            None,
            None,
            false,
            0,
            true,
            chrono::Utc::now(),
            chrono::Utc::now(),
        );
        let catalog = make_catalog(vec![catalog_def]);

        // Condition with type_ string that doesn't match code but has condition_type_id
        let cond = WorkingCondition::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "OLD_NAME".to_string(),
            Some(ct_id),
            50,
            true,
            d(2026, 1, 1),
            None,
        );

        let rules = PolicyResolver::build_rules_from_conditions(
            &[cond],
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&catalog),
        );
        assert_eq!(rules.authorized_early_leave_minutes(), 50);
        assert!(rules.is_early_leave_paid());
    }

    #[test]
    fn catalog_most_generous_wins() {
        let catalog = make_catalog(vec![
            catalog_def("EARLY_LEAVE", serde_json::json!({"authorized_early_leave_minutes": true})),
            catalog_def("BREASTFEEDING", serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true})),
        ]);

        let conds = vec![
            mock_condition("EARLY_LEAVE", 30, false, d(2026, 1, 1), None),
            mock_condition("BREASTFEEDING", 60, true, d(2026, 1, 1), None),
        ];
        let rules = PolicyResolver::build_rules_from_conditions(
            &conds,
            d(2026, 3, 1),
            EarlyArrivalStrategy::Allow,
            DEFAULT_GRACE_PERIOD_MINUTES,
            Some(&catalog),
        );
        // Most generous allowance wins (60 from BREASTFEEDING)
        // BREASTFEEDING's is_paid=true takes effect via affects_rules.is_early_leave_paid
        assert_eq!(rules.authorized_early_leave_minutes(), 60);
        assert!(rules.is_early_leave_paid());
    }

    #[test]
    fn catalog_resolve_attendance_rules_pure() {
        let catalog = make_catalog(vec![catalog_def(
            "EARLY_LEAVE",
            serde_json::json!({"authorized_early_leave_minutes": true, "is_early_leave_paid": true}),
        )]);

        let attendance_rules = AttendanceRules {
            work_unit_minutes: 30,
            rounding_policy: "floor".to_string(),
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            grace_period_minutes: Some(10),
        };

        let cond = mock_condition("EARLY_LEAVE", 45, true, d(2026, 1, 1), None);
        let rules = PolicyResolver::resolve_attendance_rules_pure(
            &attendance_rules,
            vec![cond],
            d(2026, 3, 1),
            Some(&catalog),
        );
        assert_eq!(rules.grace_period_minutes(), 10); // from attendance rules
        assert_eq!(rules.authorized_early_leave_minutes(), 45); // from catalog condition
        assert!(rules.is_early_leave_paid());
    }
}
