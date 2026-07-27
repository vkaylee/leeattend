//! Cascade engine — pure evaluator.
//!
//! Runs the 6-layer cascade defined in [[ADR-001]]:
//! L1 base → L2a day_type → L2b shift_template → L2c shift_version
//! → L2.5 employee_modifier (each modifier, ordered by priority_rank DESC)
//! → L3 exception.
//!
//! Every layer that runs emits exactly one `LayerContribution`, even if its
//! contribution is empty — the trace shape stays stable so downstream audit
//! tooling can rely on layer indices.
//!
//! After L3, numeric fields are clamped once against `BaseFieldSpec` bounds
//! from `BaseRules`; every clamp produces a `ClampWarning`.

use crate::domain::policy::cascade::caps::{ClampWarning, clamp_i32};
use crate::domain::policy::cascade::context::EvaluationContext;
use crate::domain::policy::cascade::merge::merge_layer;
use crate::domain::policy::cascade::partial::PartialPolicyRules;
use crate::domain::policy::cascade::trace::{
    EffectiveRulesSnapshot, LayerContribution, LayerId, LayerProvenance,
};

/// Run the cascade on `ctx` and return a snapshot ready to serialize into
/// `attendance_daily_summaries.effective_rules_snapshot`.
///
/// Modifiers on the context are sorted internally — the caller doesn't need
/// to pre-sort. The context itself is consumed by-value: contribution payloads
/// are moved into the trace to avoid a redundant clone.
pub fn evaluate(mut ctx: EvaluationContext) -> EffectiveRulesSnapshot {
    ctx.sort_modifiers();

    let (
        policy_version_id,
        base_rules,
        day_type,
        day_type_contribution,
        shift_template_id,
        shift_template_contribution,
        shift_version_id,
        shift_version_contribution,
        modifiers,
        exception,
    ) = ctx.into_parts();

    let mut rules = base_rules.seed_rules();
    let mut trace = Vec::with_capacity(4 + modifiers.len());

    // L1 — base contributes only through seed_rules(); its trace entry records
    // the policy_version_id that produced the seed, no PartialPolicyRules.
    trace.push(LayerContribution::new(
        LayerId::L1Base,
        LayerProvenance::empty().with_policy_version_id(policy_version_id),
        PartialPolicyRules::default(),
    ));

    // L2a — day_type
    let l2a = day_type_contribution.unwrap_or_default();
    merge_layer(&mut rules, &l2a);
    trace.push(LayerContribution::new(
        LayerId::L2aDayType,
        LayerProvenance::empty().with_day_type(day_type.unwrap_or_default()),
        l2a,
    ));

    // L2b — shift_template
    let l2b = shift_template_contribution.unwrap_or_default();
    merge_layer(&mut rules, &l2b);
    trace.push(LayerContribution::new(
        LayerId::L2bShiftTemplate,
        LayerProvenance::empty().with_shift_template_id(shift_template_id.unwrap_or_default()),
        l2b,
    ));

    // L2c — shift_version
    let l2c = shift_version_contribution.unwrap_or_default();
    merge_layer(&mut rules, &l2c);
    trace.push(LayerContribution::new(
        LayerId::L2cShiftVersion,
        LayerProvenance::empty().with_shift_version_id(shift_version_id.unwrap_or_default()),
        l2c,
    ));

    // L2.5 — every modifier, ordered high-rank first, so lowest runs last.
    for m in modifiers {
        let contributions = m.contributions().clone();
        merge_layer(&mut rules, &contributions);
        trace.push(LayerContribution::new(
            LayerId::L2_5Employee,
            LayerProvenance::empty()
                .with_modifier_id(m.modifier_id())
                .with_priority_rank(m.priority_rank()),
            contributions,
        ));
    }

    // L3 — exception (top of cascade)
    let (l3_partial, l3_prov) = match exception {
        Some(e) => (
            e.contributions().clone(),
            LayerProvenance::empty().with_exception_id(e.exception_id()),
        ),
        None => (PartialPolicyRules::default(), LayerProvenance::empty()),
    };
    merge_layer(&mut rules, &l3_partial);
    trace.push(LayerContribution::new(LayerId::L3Exception, l3_prov, l3_partial));

    // Clamp numeric fields against the base absolute caps.
    let mut warnings: Vec<ClampWarning> = Vec::new();
    let (grace, w) = clamp_i32(
        "grace_period_minutes",
        rules.grace_period_minutes(),
        base_rules.grace_period_minutes(),
    );
    rules.set_grace_period_minutes(grace);
    if let Some(w) = w {
        warnings.push(w);
    }
    let (ael, w) = clamp_i32(
        "authorized_early_leave_minutes",
        rules.authorized_early_leave_minutes(),
        base_rules.authorized_early_leave_minutes(),
    );
    rules.set_authorized_early_leave_minutes(ael);
    if let Some(w) = w {
        warnings.push(w);
    }

    EffectiveRulesSnapshot::new(rules, trace, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::policy::cascade::base::BaseRules;
    use crate::domain::policy::cascade::caps::{BaseFieldSpec, ClampReason};
    use crate::domain::policy::cascade::context::{AttendanceException, EmployeeModifier};
    use crate::models::enums::EarlyArrivalStrategy;
    use uuid::Uuid;

    fn spec(default: i32, min: Option<i32>, max: Option<i32>) -> BaseFieldSpec<i32> {
        BaseFieldSpec::new(default, min, max)
    }

    fn base(grace_default: i32, grace_max: Option<i32>) -> BaseRules {
        BaseRules::new(
            spec(grace_default, Some(0), grace_max),
            spec(0, Some(0), Some(240)),
            false,
            EarlyArrivalStrategy::Allow,
            true,
            false,
            false,
        )
    }

    fn empty_ctx() -> EvaluationContext {
        EvaluationContext::new(Uuid::nil(), base(10, Some(120)))
    }

    #[test]
    fn empty_cascade_returns_base_defaults_with_full_trace() {
        let snap = evaluate(empty_ctx());
        assert_eq!(snap.final_rules().grace_period_minutes(), 10);
        assert_eq!(snap.cascade_trace().len(), 5); // L1, L2a, L2b, L2c, L3 (no modifiers)
        assert_eq!(snap.cascade_trace()[0].layer(), LayerId::L1Base);
        assert_eq!(snap.cascade_trace()[4].layer(), LayerId::L3Exception);
        assert!(snap.clamp_warnings().is_empty());
    }

    /// ADR-002 example: L1=10, L2a override=20, L2.5 delta=+30, L3 override=15 → 15.
    #[test]
    fn adr002_full_cascade_example() {
        let ctx = empty_ctx()
            .with_day_type(
                "weekday",
                PartialPolicyRules::default().with_override_grace_period_minutes(20),
            )
            .push_modifier(EmployeeModifier::new(
                Uuid::new_v4(),
                1,
                PartialPolicyRules::default().with_delta_grace_period_minutes(30),
            ))
            .with_exception(AttendanceException::new(
                Uuid::new_v4(),
                PartialPolicyRules::default().with_override_grace_period_minutes(15),
            ));

        let snap = evaluate(ctx);
        assert_eq!(snap.final_rules().grace_period_minutes(), 15);
        // 5 fixed layer entries + 1 modifier
        assert_eq!(snap.cascade_trace().len(), 6);
    }

    #[test]
    fn multiple_modifiers_lowest_rank_wins_per_field() {
        let ctx = empty_ctx()
            .push_modifier(EmployeeModifier::new(
                Uuid::new_v4(),
                2,
                PartialPolicyRules::default().with_override_grace_period_minutes(80),
            ))
            .push_modifier(EmployeeModifier::new(
                Uuid::new_v4(),
                1,
                PartialPolicyRules::default().with_override_grace_period_minutes(45),
            ));

        let snap = evaluate(ctx);
        assert_eq!(snap.final_rules().grace_period_minutes(), 45);

        // Cascade trace order: L1, L2a, L2b, L2c, L2.5(rank=2), L2.5(rank=1), L3
        let ranks: Vec<i32> = snap
            .cascade_trace()
            .iter()
            .filter_map(|e| e.provenance().priority_rank())
            .collect();
        assert_eq!(ranks, vec![2, 1]);
    }

    #[test]
    fn value_above_cap_is_clamped_and_warned() {
        let base = base(10, Some(60));
        let ctx = EvaluationContext::new(Uuid::nil(), base)
            .with_exception(AttendanceException::new(
                Uuid::new_v4(),
                PartialPolicyRules::default().with_override_grace_period_minutes(200),
            ));

        let snap = evaluate(ctx);
        assert_eq!(snap.final_rules().grace_period_minutes(), 60);
        assert_eq!(snap.clamp_warnings().len(), 1);
        assert_eq!(snap.clamp_warnings()[0].field(), "grace_period_minutes");
        assert_eq!(snap.clamp_warnings()[0].before(), 200);
        assert_eq!(snap.clamp_warnings()[0].after(), 60);
        assert_eq!(snap.clamp_warnings()[0].reason(), ClampReason::AboveMax);
    }

    #[test]
    fn snapshot_serializes_with_stable_shape() {
        let snap = evaluate(empty_ctx());
        let json = serde_json::to_value(&snap).unwrap();
        assert!(json.get("final_rules").is_some());
        let trace = json.get("cascade_trace").unwrap().as_array().unwrap();
        assert_eq!(trace[0]["layer"], "L1_base");
        assert_eq!(trace[4]["layer"], "L3_exception");
        // clamp_warnings omitted when empty
        assert!(json.get("clamp_warnings").is_none());
    }
}
