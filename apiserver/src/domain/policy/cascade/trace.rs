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
    pub policy_version_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub day_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift_template_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift_version_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_rank: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exception_id: Option<Uuid>,
}

/// One entry in the cascade trace. `contributions` is the layer's raw
/// `PartialPolicyRules` payload — untouched so audit tooling can replay the
/// merge deterministically.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayerContribution {
    pub layer: LayerId,
    #[serde(flatten, default, skip_serializing_if = "provenance_is_empty")]
    pub provenance: LayerProvenance,
    pub contributions: PartialPolicyRules,
}

fn provenance_is_empty(p: &LayerProvenance) -> bool {
    p.policy_version_id.is_none()
        && p.day_type.is_none()
        && p.shift_template_id.is_none()
        && p.shift_version_id.is_none()
        && p.modifier_id.is_none()
        && p.priority_rank.is_none()
        && p.exception_id.is_none()
}

/// The full snapshot stored on `attendance_daily_summaries.effective_rules_snapshot`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectiveRulesSnapshot {
    pub final_rules: PolicyRules,
    pub cascade_trace: Vec<LayerContribution>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clamp_warnings: Vec<ClampWarning>,
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
        let entry = LayerContribution {
            layer: LayerId::L1Base,
            provenance: LayerProvenance::default(),
            contributions: PartialPolicyRules::default(),
        };
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
        let entry = LayerContribution {
            layer: LayerId::L2_5Employee,
            provenance: LayerProvenance {
                modifier_id: Some(mid),
                priority_rank: Some(1),
                ..Default::default()
            },
            contributions: PartialPolicyRules {
                delta_grace_period_minutes: Some(20),
                ..Default::default()
            },
        };
        let json = serde_json::to_value(&entry).unwrap();
        assert_eq!(json["modifier_id"], serde_json::json!(mid));
        assert_eq!(json["priority_rank"], 1);
        assert_eq!(
            json["contributions"]["delta_grace_period_minutes"],
            20
        );
    }

    #[test]
    fn snapshot_roundtrip_preserves_shape() {
        let rules = PolicyRules::new(15, 30, true, EarlyArrivalStrategy::Clamp, true, false, false);
        let snap = EffectiveRulesSnapshot {
            final_rules: rules,
            cascade_trace: vec![LayerContribution {
                layer: LayerId::L1Base,
                provenance: LayerProvenance {
                    policy_version_id: Some(Uuid::nil()),
                    ..Default::default()
                },
                contributions: PartialPolicyRules::default(),
            }],
            clamp_warnings: vec![],
        };
        let json = serde_json::to_string(&snap).unwrap();
        let back: EffectiveRulesSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snap, back);
        // clamp_warnings [] must be omitted
        assert!(!json.contains("clamp_warnings"));
    }
}
