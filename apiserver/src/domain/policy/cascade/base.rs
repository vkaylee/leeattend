//! L1 base layer — concrete defaults + absolute caps for numeric fields.
//!
//! Deserializes from `attendance_policy_versions.base_rules` JSONB.
//! Single source of truth for the numeric ceilings enforced after L3.
//! Bool / enum fields carry only a concrete default (no caps).

use crate::domain::policy::cascade::caps::BaseFieldSpec;
use crate::models::PolicyRules;
use crate::models::enums::EarlyArrivalStrategy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseRules {
    grace_period_minutes: BaseFieldSpec<i32>,
    authorized_early_leave_minutes: BaseFieldSpec<i32>,
    is_early_leave_paid: bool,
    early_arrival_strategy: EarlyArrivalStrategy,
    is_break_auto_deduct: bool,
    is_break_overtime: bool,
    allow_early_ot: bool,
}

impl BaseRules {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        grace_period_minutes: BaseFieldSpec<i32>,
        authorized_early_leave_minutes: BaseFieldSpec<i32>,
        is_early_leave_paid: bool,
        early_arrival_strategy: EarlyArrivalStrategy,
        is_break_auto_deduct: bool,
        is_break_overtime: bool,
        allow_early_ot: bool,
    ) -> Self {
        Self {
            grace_period_minutes,
            authorized_early_leave_minutes,
            is_early_leave_paid,
            early_arrival_strategy,
            is_break_auto_deduct,
            is_break_overtime,
            allow_early_ot,
        }
    }

    pub fn grace_period_minutes(&self) -> &BaseFieldSpec<i32> {
        &self.grace_period_minutes
    }
    pub fn authorized_early_leave_minutes(&self) -> &BaseFieldSpec<i32> {
        &self.authorized_early_leave_minutes
    }
    pub fn is_early_leave_paid(&self) -> bool {
        self.is_early_leave_paid
    }
    pub fn early_arrival_strategy(&self) -> EarlyArrivalStrategy {
        self.early_arrival_strategy
    }
    pub fn is_break_auto_deduct(&self) -> bool {
        self.is_break_auto_deduct
    }
    pub fn is_break_overtime(&self) -> bool {
        self.is_break_overtime
    }
    pub fn allow_early_ot(&self) -> bool {
        self.allow_early_ot
    }

    /// Setter used by tests that need to swap the cap spec in-place.
    #[cfg(test)]
    pub fn set_grace_period_minutes(&mut self, spec: BaseFieldSpec<i32>) {
        self.grace_period_minutes = spec;
    }

    /// Seed PolicyRules from base defaults, before any cascade layer runs.
    pub fn seed_rules(&self) -> PolicyRules {
        PolicyRules::new(
            self.grace_period_minutes.default_value(),
            self.authorized_early_leave_minutes.default_value(),
            self.is_early_leave_paid,
            self.early_arrival_strategy,
            self.is_break_auto_deduct,
            self.is_break_overtime,
            self.allow_early_ot,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(default: i32, min: Option<i32>, max: Option<i32>) -> BaseFieldSpec<i32> {
        BaseFieldSpec::new(default, min, max)
    }

    #[test]
    fn seed_rules_copies_defaults_into_policyrules() {
        let base = BaseRules::new(
            spec(10, Some(0), Some(120)),
            spec(0, Some(0), Some(240)),
            false,
            EarlyArrivalStrategy::Block,
            true,
            false,
            false,
        );
        let rules = base.seed_rules();
        assert_eq!(rules.grace_period_minutes(), 10);
        assert_eq!(rules.authorized_early_leave_minutes(), 0);
        assert!(!rules.is_early_leave_paid());
        assert_eq!(rules.early_arrival_strategy(), EarlyArrivalStrategy::Block);
        assert!(rules.is_break_auto_deduct());
        assert!(!rules.is_break_overtime());
        assert!(!rules.allow_early_ot());
    }

    #[test]
    fn base_rules_roundtrips_through_json() {
        let base = BaseRules::new(
            spec(15, Some(0), Some(120)),
            spec(30, None, Some(240)),
            true,
            EarlyArrivalStrategy::Clamp,
            true,
            false,
            true,
        );
        let json = serde_json::to_string(&base).unwrap();
        let back: BaseRules = serde_json::from_str(&json).unwrap();
        assert_eq!(base, back);
    }
}
