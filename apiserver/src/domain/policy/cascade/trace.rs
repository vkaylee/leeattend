//! Cascade trace types serialized into `attendance_daily_summaries.effective_rules_snapshot`.
//!
//! Shape per [[ADR-004]] §4. Every layer that runs pushes exactly one
//! `LayerContribution`; the top-level `EffectiveRulesSnapshot` bundles the
//! resolved `PolicyRules`, the trace, and any `ClampWarning`s emitted by the
//! post-L3 clamp.
//!
//! `LayerId` string values match the layer names in [[ADR-004]] §4 with one
//! rename: `L2b_shift_type` → `L2b_shift_template` and `L2c_shift_instance` →
//! `L2c_shift_version` to align with the actual migration tables
//! (`attendance_policy_shift_template_overrides`,
//! `attendance_policy_shift_version_overrides`).

use crate::domain::policy::cascade::caps::ClampWarning;
use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::models::PolicyRules;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerId {
    #[serde(rename = "L1_base")]
    L1Base,
    #[serde(rename = "L2a_day_type")]
    L2aDayType,
    #[serde(rename = "L2b_shift_template")]
    L2bShiftTemplate,
    #[serde(rename = "L2c_shift_version")]
    L2cShiftVersion,
    #[serde(rename = "L2.5_employee")]
    L2_5Employee,
    #[serde(rename = "L3_exception")]
    L3Exception,
}

/// Per-layer provenance — which DB row was consulted and any tie-breaker key.
/// Untagged so JSON stays flat inside the trace entry.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerProvenance {
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_version_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    day_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shift_template_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shift_version_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modifier_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority_rank: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exception_id: Option<Uuid>,
}

impl LayerProvenance {
    pub fn empty() -> Self {
        Self::default()
    }

    // Fluent builders — all fields are optional; setter form fits engine call sites.
    pub fn with_policy_version_id(mut self, v: Uuid) -> Self {
        self.policy_version_id = Some(v);
        self
    }
    pub fn with_day_type(mut self, v: impl Into<String>) -> Self {
        self.day_type = Some(v.into());
        self
    }
    pub fn with_shift_template_id(mut self, v: Uuid) -> Self {
        self.shift_template_id = Some(v);
        self
    }
    pub fn with_shift_version_id(mut self, v: Uuid) -> Self {
        self.shift_version_id = Some(v);
        self
    }
    pub fn with_modifier_id(mut self, v: Uuid) -> Self {
        self.modifier_id = Some(v);
        self
    }
    pub fn with_priority_rank(mut self, v: i32) -> Self {
        self.priority_rank = Some(v);
        self
    }
    pub fn with_exception_id(mut self, v: Uuid) -> Self {
        self.exception_id = Some(v);
        self
    }

    // Getters.
    pub fn policy_version_id(&self) -> Option<Uuid> {
        self.policy_version_id
    }
    pub fn day_type(&self) -> Option<&str> {
        self.day_type.as_deref()
    }
    pub fn shift_template_id(&self) -> Option<Uuid> {
        self.shift_template_id
    }
    pub fn shift_version_id(&self) -> Option<Uuid> {
        self.shift_version_id
    }
    pub fn modifier_id(&self) -> Option<Uuid> {
        self.modifier_id
    }
    pub fn priority_rank(&self) -> Option<i32> {
        self.priority_rank
    }
    pub fn exception_id(&self) -> Option<Uuid> {
        self.exception_id
    }

    pub fn is_empty(&self) -> bool {
        self.policy_version_id.is_none()
            && self.day_type.is_none()
            && self.shift_template_id.is_none()
            && self.shift_version_id.is_none()
            && self.modifier_id.is_none()
            && self.priority_rank.is_none()
            && self.exception_id.is_none()
    }
}

/// One entry in the cascade trace. `contributions` is the layer's raw
/// `PartialPolicyRules` payload — untouched so audit tooling can replay the
/// merge deterministically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerContribution {
    layer: LayerId,
    #[serde(flatten, default, skip_serializing_if = "LayerProvenance::is_empty")]
    provenance: LayerProvenance,
    contributions: PartialPolicyRules,
}

impl LayerContribution {
    pub fn new(
        layer: LayerId,
        provenance: LayerProvenance,
        contributions: PartialPolicyRules,
    ) -> Self {
        Self {
            layer,
            provenance,
            contributions,
        }
    }

    pub fn layer(&self) -> LayerId {
        self.layer
    }
    pub fn provenance(&self) -> &LayerProvenance {
        &self.provenance
    }
    pub fn contributions(&self) -> &PartialPolicyRules {
        &self.contributions
    }
}

/// The full snapshot stored on `attendance_daily_summaries.effective_rules_snapshot`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveRulesSnapshot {
    final_rules: PolicyRules,
    cascade_trace: Vec<LayerContribution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    clamp_warnings: Vec<ClampWarning>,
}

impl EffectiveRulesSnapshot {
    pub fn new(
        final_rules: PolicyRules,
        cascade_trace: Vec<LayerContribution>,
        clamp_warnings: Vec<ClampWarning>,
    ) -> Self {
        Self {
            final_rules,
            cascade_trace,
            clamp_warnings,
        }
    }

    pub fn final_rules(&self) -> &PolicyRules {
        &self.final_rules
    }
    pub fn cascade_trace(&self) -> &[LayerContribution] {
        &self.cascade_trace
    }
    pub fn clamp_warnings(&self) -> &[ClampWarning] {
        &self.clamp_warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::EarlyArrivalStrategy;

    #[test]
    fn layer_id_serializes_to_adr_names() {
        assert_eq!(serde_json::to_string(&LayerId::L1Base).unwrap(), r#""L1_base""#);
        assert_eq!(
            serde_json::to_string(&LayerId::L2aDayType).unwrap(),
            r#""L2a_day_type""#
        );
        assert_eq!(
            serde_json::to_string(&LayerId::L2bShiftTemplate).unwrap(),
            r#""L2b_shift_template""#
        );
        assert_eq!(
            serde_json::to_string(&LayerId::L2cShiftVersion).unwrap(),
            r#""L2c_shift_version""#
        );
        assert_eq!(
            serde_json::to_string(&LayerId::L2_5Employee).unwrap(),
            r#""L2.5_employee""#
        );
        assert_eq!(
            serde_json::to_string(&LayerId::L3Exception).unwrap(),
            r#""L3_exception""#
        );
    }

    #[test]
    fn empty_provenance_is_hidden_by_flatten() {
        let entry = LayerContribution::new(
            LayerId::L1Base,
            LayerProvenance::empty(),
            PartialPolicyRules::default(),
        );
        let json = serde_json::to_value(&entry).unwrap();
        let obj = json.as_object().unwrap();
        assert!(obj.contains_key("layer"));
        assert!(obj.contains_key("contributions"));
        assert!(!obj.contains_key("policy_version_id"));
        assert!(!obj.contains_key("modifier_id"));
    }

    #[test]
    fn modifier_layer_carries_priority_rank() {
        let mid = Uuid::new_v4();
        let entry = LayerContribution::new(
            LayerId::L2_5Employee,
            LayerProvenance::empty()
                .with_modifier_id(mid)
                .with_priority_rank(1),
            PartialPolicyRules::default().with_delta_grace_period_minutes(20),
        );
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["modifier_id"], serde_json::json!(mid));
        assert_eq!(json["priority_rank"], 1);
        assert_eq!(json["contributions"]["delta_grace_period_minutes"], 20);
    }

    #[test]
    fn snapshot_roundtrip_preserves_shape() {
        let rules = PolicyRules::new(
            15,
            30,
            true,
            EarlyArrivalStrategy::Clamp,
            true,
            false,
            false,
        );
        let snap = EffectiveRulesSnapshot::new(
            rules,
            vec![LayerContribution::new(
                LayerId::L1Base,
                LayerProvenance::empty().with_policy_version_id(Uuid::nil()),
                PartialPolicyRules::default(),
            )],
            vec![],
        );
        let json = serde_json::to_string(&snap).unwrap();
        let back: EffectiveRulesSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);
        // clamp_warnings [] must be omitted
        assert!(!json.contains("clamp_warnings"));
    }
}
