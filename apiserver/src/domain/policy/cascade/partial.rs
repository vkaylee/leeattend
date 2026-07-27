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
    pub override_grace_period_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_grace_period_minutes: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_authorized_early_leave_minutes: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_authorized_early_leave_minutes: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_is_early_leave_paid: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_early_arrival_strategy: Option<EarlyArrivalStrategy>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_is_break_auto_deduct: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_is_break_overtime: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_allow_early_ot: Option<bool>,
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
        let p = PartialPolicyRules {
            override_grace_period_minutes: Some(20),
            delta_grace_period_minutes: Some(-5),
            ..Default::default()
        };
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
        let p = PartialPolicyRules {
            override_grace_period_minutes: Some(30),
            delta_grace_period_minutes: Some(10),
            override_authorized_early_leave_minutes: Some(45),
            delta_authorized_early_leave_minutes: Some(-15),
            override_is_early_leave_paid: Some(true),
            override_early_arrival_strategy: Some(EarlyArrivalStrategy::Clamp),
            override_is_break_auto_deduct: Some(false),
            override_is_break_overtime: Some(true),
            override_allow_early_ot: Some(true),
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: PartialPolicyRules = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}
