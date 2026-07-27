//! Layer contribution payload — Option<T> per field.
//!
//! Every layer L2a→L3 in the ADR-001 cascade contributes a `PartialPolicyRules`.
//! Numeric fields carry BOTH an absolute `override_*` and a signed `delta_*`;
//! bool/enum fields carry only an `override_*`. See ADR-002 for the merge formula.
//!
//! `None` on every field means "layer contributes nothing" — legal and common.

use crate::models::enums::EarlyArrivalStrategy;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct PartialPolicyRules {
    #[serde(skip_serializing_if = "Option::is_none")]
    override_grace_period_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_grace_period_minutes: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    override_authorized_early_leave_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delta_authorized_early_leave_minutes: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    override_is_early_leave_paid: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    override_early_arrival_strategy: Option<EarlyArrivalStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    override_is_break_auto_deduct: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    override_is_break_overtime: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    override_allow_early_ot: Option<bool>,
}

impl PartialPolicyRules {
    pub fn is_empty(&self) -> bool {
        self.override_grace_period_minutes.is_none()
            && self.delta_grace_period_minutes.is_none()
            && self.override_authorized_early_leave_minutes.is_none()
            && self.delta_authorized_early_leave_minutes.is_none()
            && self.override_is_early_leave_paid.is_none()
            && self.override_early_arrival_strategy.is_none()
            && self.override_is_break_auto_deduct.is_none()
            && self.override_is_break_overtime.is_none()
            && self.override_allow_early_ot.is_none()
    }

    // Getters — all fields are Copy so return by value.
    pub fn override_grace_period_minutes(&self) -> Option<i32> {
        self.override_grace_period_minutes
    }
    pub fn delta_grace_period_minutes(&self) -> Option<i32> {
        self.delta_grace_period_minutes
    }
    pub fn override_authorized_early_leave_minutes(&self) -> Option<i32> {
        self.override_authorized_early_leave_minutes
    }
    pub fn delta_authorized_early_leave_minutes(&self) -> Option<i32> {
        self.delta_authorized_early_leave_minutes
    }
    pub fn override_is_early_leave_paid(&self) -> Option<bool> {
        self.override_is_early_leave_paid
    }
    pub fn override_early_arrival_strategy(&self) -> Option<EarlyArrivalStrategy> {
        self.override_early_arrival_strategy
    }
    pub fn override_is_break_auto_deduct(&self) -> Option<bool> {
        self.override_is_break_auto_deduct
    }
    pub fn override_is_break_overtime(&self) -> Option<bool> {
        self.override_is_break_overtime
    }
    pub fn override_allow_early_ot(&self) -> Option<bool> {
        self.override_allow_early_ot
    }

    // Fluent builders — `impl Into<Option<T>>` so callers can pass either `T` or `Option<T>`.
    pub fn with_override_grace_period_minutes(mut self, v: impl Into<Option<i32>>) -> Self {
        self.override_grace_period_minutes = v.into();
        self
    }
    pub fn with_delta_grace_period_minutes(mut self, v: impl Into<Option<i32>>) -> Self {
        self.delta_grace_period_minutes = v.into();
        self
    }
    pub fn with_override_authorized_early_leave_minutes(
        mut self,
        v: impl Into<Option<i32>>,
    ) -> Self {
        self.override_authorized_early_leave_minutes = v.into();
        self
    }
    pub fn with_delta_authorized_early_leave_minutes(
        mut self,
        v: impl Into<Option<i32>>,
    ) -> Self {
        self.delta_authorized_early_leave_minutes = v.into();
        self
    }
    pub fn with_override_is_early_leave_paid(mut self, v: impl Into<Option<bool>>) -> Self {
        self.override_is_early_leave_paid = v.into();
        self
    }
    pub fn with_override_early_arrival_strategy(
        mut self,
        v: impl Into<Option<EarlyArrivalStrategy>>,
    ) -> Self {
        self.override_early_arrival_strategy = v.into();
        self
    }
    pub fn with_override_is_break_auto_deduct(mut self, v: impl Into<Option<bool>>) -> Self {
        self.override_is_break_auto_deduct = v.into();
        self
    }
    pub fn with_override_is_break_overtime(mut self, v: impl Into<Option<bool>>) -> Self {
        self.override_is_break_overtime = v.into();
        self
    }
    pub fn with_override_allow_early_ot(mut self, v: impl Into<Option<bool>>) -> Self {
        self.override_allow_early_ot = v.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_partial_serializes_to_empty_object() {
        let p = PartialPolicyRules::default();
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, "{}");
        assert!(p.is_empty());
    }

    #[test]
    fn only_set_fields_are_serialized() {
        let p = PartialPolicyRules::default()
            .with_override_grace_period_minutes(20)
            .with_delta_grace_period_minutes(-5);
        let json = serde_json::to_value(&p).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 2);
        assert_eq!(obj["override_grace_period_minutes"], 20);
        assert_eq!(obj["delta_grace_period_minutes"], -5);
    }

    #[test]
    fn deserialize_ignores_missing_fields() {
        let p: PartialPolicyRules = serde_json::from_str("{}").unwrap();
        assert!(p.is_empty());
    }

    #[test]
    fn deserialize_rejects_unknown_field() {
        let err = serde_json::from_str::<PartialPolicyRules>(r#"{"typo_field": 1}"#).unwrap_err();
        assert!(err.to_string().contains("typo_field"));
    }

    #[test]
    fn roundtrip_preserves_all_populated_fields() {
        let p = PartialPolicyRules::default()
            .with_override_grace_period_minutes(30)
            .with_delta_grace_period_minutes(10)
            .with_override_authorized_early_leave_minutes(45)
            .with_delta_authorized_early_leave_minutes(-15)
            .with_override_is_early_leave_paid(true)
            .with_override_early_arrival_strategy(EarlyArrivalStrategy::Clamp)
            .with_override_is_break_auto_deduct(false)
            .with_override_is_break_overtime(true)
            .with_override_allow_early_ot(true);
        let json = serde_json::to_string(&p).unwrap();
        let back: PartialPolicyRules = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }

    #[test]
    fn getters_reflect_builder_state() {
        let p = PartialPolicyRules::default()
            .with_override_grace_period_minutes(20)
            .with_delta_grace_period_minutes(-5);
        assert_eq!(p.override_grace_period_minutes(), Some(20));
        assert_eq!(p.delta_grace_period_minutes(), Some(-5));
        assert_eq!(p.override_is_early_leave_paid(), None);
    }
}
