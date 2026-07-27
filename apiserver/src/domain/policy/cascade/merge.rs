//! ADR-002 merge formula.
//!
//! ```text
//! value_new = override.unwrap_or(value_current) + delta.unwrap_or(0)
//! ```
//!
//! - `override = Some(x)` resets any delta chain accumulated below.
//! - `delta = Some(±y)` adds on top of the current value.
//! - Both `Some`: override wins, then delta stacks on top of the override.
//! - Both `None`: no change.
//!
//! Merge is applied once per cascade layer (L2a → L3). Clamp to
//! `[absolute_min, absolute_max]` happens exactly once, after L3.

use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::models::PolicyRules;

/// Apply one layer's `PartialPolicyRules` to the accumulator in-place.
pub fn merge_layer(acc: &mut PolicyRules, layer: &PartialPolicyRules) {
    // Numeric fields: override + delta.
    let grace = merge_i32(
        acc.grace_period_minutes(),
        layer.override_grace_period_minutes,
        layer.delta_grace_period_minutes,
    );
    acc.set_grace_period_minutes(grace);

    let ael = merge_i32(
        acc.authorized_early_leave_minutes(),
        layer.override_authorized_early_leave_minutes,
        layer.delta_authorized_early_leave_minutes,
    );
    acc.set_authorized_early_leave_minutes(ael);

    // Bool / enum fields: override only.
    if let Some(v) = layer.override_is_early_leave_paid {
        acc.set_is_early_leave_paid(v);
    }
    if let Some(v) = layer.override_early_arrival_strategy {
        acc.set_early_arrival_strategy(v);
    }
    if let Some(v) = layer.override_is_break_auto_deduct {
        acc.set_is_break_auto_deduct(v);
    }
    if let Some(v) = layer.override_is_break_overtime {
        acc.set_is_break_overtime(v);
    }
    if let Some(v) = layer.override_allow_early_ot {
        acc.set_allow_early_ot(v);
    }
}

/// `override.unwrap_or(current) + delta.unwrap_or(0)`, saturating on overflow.
fn merge_i32(current: i32, override_val: Option<i32>, delta: Option<i32>) -> i32 {
    override_val
        .unwrap_or(current)
        .saturating_add(delta.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::EarlyArrivalStrategy;

    fn base_rules() -> PolicyRules {
        PolicyRules::new(10, 0, false, EarlyArrivalStrategy::Allow, true, false, false)
    }

    #[test]
    fn empty_partial_leaves_accumulator_unchanged() {
        let mut acc = base_rules();
        let before = acc.clone();
        merge_layer(&mut acc, &PartialPolicyRules::default());
        assert_eq!(acc.grace_period_minutes(), before.grace_period_minutes());
        assert_eq!(acc.is_break_overtime(), before.is_break_overtime());
    }

    #[test]
    fn override_only_replaces_value() {
        let mut acc = base_rules();
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                override_grace_period_minutes: Some(20),
                ..Default::default()
            },
        );
        assert_eq!(acc.grace_period_minutes(), 20);
    }

    #[test]
    fn delta_only_adds_to_current() {
        let mut acc = base_rules();
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                delta_grace_period_minutes: Some(30),
                ..Default::default()
            },
        );
        assert_eq!(acc.grace_period_minutes(), 40);
    }

    #[test]
    fn override_and_delta_stack_override_first() {
        let mut acc = base_rules();
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                override_grace_period_minutes: Some(20),
                delta_grace_period_minutes: Some(5),
                ..Default::default()
            },
        );
        assert_eq!(acc.grace_period_minutes(), 25);
    }

    /// ADR-002 example table: 10 → 20 → 20 → 50 → 15.
    #[test]
    fn adr002_example_flow() {
        let mut acc = base_rules(); // L1: grace = 10
        // L2a: override 20
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                override_grace_period_minutes: Some(20),
                ..Default::default()
            },
        );
        assert_eq!(acc.grace_period_minutes(), 20);
        // L2b: nothing
        merge_layer(&mut acc, &PartialPolicyRules::default());
        assert_eq!(acc.grace_period_minutes(), 20);
        // L2c: nothing (skipped)
        merge_layer(&mut acc, &PartialPolicyRules::default());
        // L2.5: delta +30
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                delta_grace_period_minutes: Some(30),
                ..Default::default()
            },
        );
        assert_eq!(acc.grace_period_minutes(), 50);
        // L3: override 15 — resets the whole chain
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                override_grace_period_minutes: Some(15),
                ..Default::default()
            },
        );
        assert_eq!(acc.grace_period_minutes(), 15);
    }

    #[test]
    fn bool_and_enum_override_replaces_value() {
        let mut acc = base_rules();
        merge_layer(
            &mut acc,
            &PartialPolicyRules {
                override_is_break_overtime: Some(true),
                override_early_arrival_strategy: Some(EarlyArrivalStrategy::Clamp),
                override_allow_early_ot: Some(true),
                ..Default::default()
            },
        );
        assert!(acc.is_break_overtime());
        assert!(acc.allow_early_ot());
        assert_eq!(acc.early_arrival_strategy(), EarlyArrivalStrategy::Clamp);
    }

    #[test]
    fn delta_saturates_on_overflow() {
        let current = i32::MAX - 5;
        let out = merge_i32(current, None, Some(100));
        assert_eq!(out, i32::MAX);
    }
}
