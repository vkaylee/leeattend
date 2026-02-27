use crate::models::{IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy};
use rust_decimal::prelude::*;

pub struct PayrollEvaluator;

pub struct IncentiveContext {
    pub total_work_units: i32,
    pub total_leave_days: f64,
    pub unpaid_leave_days: f64,
    pub sick_leave_days: f64,
    pub late_early_count: i32,
}

impl PayrollEvaluator {
    pub fn evaluate_policy(
        policy: &PayrollIncentivePolicy,
        tiers: &[IncentiveTier],
        context: &IncentiveContext,
    ) -> IncentiveEvaluationResult {
        // Find the first matching tier based on priority
        let mut sorted_tiers = tiers.to_vec();
        sorted_tiers.sort_by_key(|t| t.priority);

        for tier in sorted_tiers {
            if Self::matches_tier(&tier, context) {
                let amount = if policy.payout_type == "percentage" {
                    rust_decimal::Decimal::from_f64(policy.base_amount * tier.payout_amount)
                } else {
                    rust_decimal::Decimal::from_f64(tier.payout_amount)
                }
                .unwrap_or_default();

                let standard_amount =
                    rust_decimal::Decimal::from_f64(policy.base_amount).unwrap_or_default();

                return IncentiveEvaluationResult {
                    policy_name: policy.name.clone(),
                    amount,
                    standard_amount,
                    tier_id: Some(tier.id),
                };
            }
        }

        // If no tier matches, return zero
        IncentiveEvaluationResult {
            policy_name: policy.name.clone(),
            amount: rust_decimal::Decimal::ZERO,
            standard_amount: rust_decimal::Decimal::from_f64(policy.base_amount)
                .unwrap_or_default(),
            tier_id: None,
        }
    }

    fn matches_tier(tier: &IncentiveTier, ctx: &IncentiveContext) -> bool {
        // Check late/early count
        if let Some(max) = tier.max_late_early_count {
            if ctx.late_early_count > max {
                return false;
            }
        }

        // Check unpaid leave
        if let Some(max) = tier.max_unpaid_leave_days {
            if ctx.unpaid_leave_days > max {
                return false;
            }
        }

        // Check sick leave
        if let Some(max) = tier.max_sick_leave_days {
            if ctx.sick_leave_days > max {
                return false;
            }
        }

        // Check total leave days
        if let Some(max) = tier.max_total_leave_days {
            if ctx.total_leave_days > max {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_evaluate_policy_payout() {
        let p_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let t1_id = Uuid::new_v4();
        let t2_id = Uuid::new_v4();

        // Policy Base: 300,000 (Standard)
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: ws_id,
            name: "Diligence".into(),
            base_amount: 300000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };

        let tiers = vec![
            IncentiveTier {
                id: t1_id,
                policy_id: p_id,
                priority: 1,
                max_late_early_count: Some(0),
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 150000.0, // Actual
            },
            IncentiveTier {
                id: t2_id,
                policy_id: p_id,
                priority: 2,
                max_late_early_count: Some(2),
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 50000.0,
            },
        ];

        // Case 1: Perfect (0 late) -> Tier 1 (150k)
        let ctx1 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };
        let res1 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx1);
        assert_eq!(res1.amount, Decimal::from(150000));
        assert_eq!(res1.standard_amount, Decimal::from(300000)); // Base

        // Case 2: Late (1 late) -> Tier 2 (50k)
        let ctx2 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 1,
        };
        let res2 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx2);
        assert_eq!(res2.amount, Decimal::from(50000));
        assert_eq!(res2.standard_amount, Decimal::from(300000));

        // Case 3: Too Late (3 late) -> 0
        let ctx3 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 3,
        };
        let res3 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx3);
        assert_eq!(res3.amount, Decimal::ZERO);
        assert_eq!(res3.standard_amount, Decimal::from(300000));
    }

    /// WF7: Demonstrate that excluding holidays from late_early_count
    /// affects which incentive tier is matched.
    /// Without holiday exclusion: 1 late → Tier 2 (50k)
    /// With holiday exclusion: 0 late → Tier 1 (150k)
    #[test]
    fn test_holiday_exclusion_affects_incentive_tier() {
        let p_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();

        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: ws_id,
            name: "Diligence".into(),
            base_amount: 300000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };

        let tiers = vec![
            IncentiveTier {
                id: Uuid::new_v4(),
                policy_id: p_id,
                priority: 1,
                max_late_early_count: Some(0), // Perfect attendance
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 150000.0,
            },
            IncentiveTier {
                id: Uuid::new_v4(),
                policy_id: p_id,
                priority: 2,
                max_late_early_count: Some(2),
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 50000.0,
            },
        ];

        // Scenario: Employee was "late" 1 time, but it was on a holiday.
        // Before WF7: late_early_count = 1 (holiday late counted) → Tier 2 (50k)
        let ctx_without_exclusion = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 1, // Holiday late NOT excluded
        };
        let result_old = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx_without_exclusion);
        assert_eq!(result_old.amount, Decimal::from(50000), "Without exclusion: Tier 2 (50k)");

        // After WF7: late_early_count = 0 (holiday late excluded) → Tier 1 (150k)
        let ctx_with_exclusion = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0, // Holiday late excluded by payroll_service.rs
        };
        let result_new = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx_with_exclusion);
        assert_eq!(result_new.amount, Decimal::from(150000), "With exclusion: Tier 1 (150k)");

        // The difference is 100k — excluding holidays from late count matters!
        assert_eq!(
            result_new.amount - result_old.amount,
            Decimal::from(100000),
            "Holiday exclusion saves employee 100k in incentive"
        );
    }
}
