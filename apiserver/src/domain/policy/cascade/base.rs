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
    pub grace_period_minutes: BaseFieldSpec<i32>,
    pub authorized_early_leave_minutes: BaseFieldSpec<i32>,
    pub is_early_leave_paid: bool,
    pub early_arrival_strategy: EarlyArrivalStrategy,
    pub is_break_auto_deduct: bool,
    pub is_break_overtime: bool,
    pub allow_early_ot: bool,
}

impl BaseRules {
    /// Seed PolicyRules from base defaults, before any cascade layer runs.
    pub fn seed_rules(&self) -> PolicyRules {
        PolicyRules::new(
            self.grace_period_minutes.default_value,
            self.authorized_early_leave_minutes.default_value,
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
        BaseFieldSpec {
            default_value: default,
            absolute_min: min,
            absolute_max: max,
        }
    }

    #[test]
    fn seed_rules_copies_defaults_into_policyrules() {
        let base = BaseRules {
            grace_period_minutes: spec(10, Some(0), Some(120)),
            authorized_early_leave_minutes: spec(0, Some(0), Some(240)),
            is_early_leave_paid: false,
            early_arrival_strategy: EarlyArrivalStrategy::Block,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        };
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
        let base = BaseRules {
            grace_period_minutes: spec(15, Some(0), Some(120)),
            authorized_early_leave_minutes: spec(30, None, Some(240)),
            is_early_leave_paid: true,
            early_arrival_strategy: EarlyArrivalStrategy::Clamp,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: true,
        };
        let json = serde_json::to_string(&base).unwrap();
        let back: BaseRules = serde_json::from_str(&json).unwrap();
        assert_eq!(base, back);
    }
}
