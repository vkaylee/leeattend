use crate::models::{IncentiveTier, PayrollIncentivePolicy, IncentiveEvaluationResult};
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
                }.unwrap_or_default();

                return IncentiveEvaluationResult {
                    policy_name: policy.name.clone(),
                    amount,
                    tier_id: Some(tier.id),
                };
            }
        }

        // If no tier matches, return zero
        IncentiveEvaluationResult {
            policy_name: policy.name.clone(),
            amount: rust_decimal::Decimal::ZERO,
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
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_evaluate_policy_payout() {
        let p_id = Uuid::new_v4();
        let ws_id = Uuid::new_v4();
        let t1_id = Uuid::new_v4();
        let t2_id = Uuid::new_v4();

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
                max_late_early_count: None,
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: Some(1.0),
                payout_amount: 150000.0,
            },
            IncentiveTier {
                id: t2_id,
                policy_id: p_id,
                priority: 2,
                max_late_early_count: None,
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 0.0,
            },
        ];

        // Case 1: Thỏa mãn Tier 1 (0 ngày nghỉ)
        let ctx1 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };
        let res1 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx1);
        assert_eq!(res1.amount, Decimal::from(150000));

        // Case 2: Không thỏa mãn Tier 1 (> 1 ngày nghỉ), rơi vào Tier 2
        let ctx2 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 2.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };
        let res2 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx2);
        assert_eq!(res2.amount, Decimal::ZERO);
    }

    #[test]
    fn test_evaluate_policy_multi_thresholds() {
        let policy = PayrollIncentivePolicy {
            id: Uuid::new_v4(),
            workspace_id: Uuid::new_v4(),
            name: "Complex Bonus".into(),
            base_amount: 500000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: Utc::now(),
        };

        let tiers = vec![
            IncentiveTier {
                id: Uuid::new_v4(),
                policy_id: policy.id,
                priority: 1, // Perfect: 0 late, 0 unpaid
                max_late_early_count: Some(0),
                max_unpaid_leave_days: Some(0.0),
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 500000.0,
            },
            IncentiveTier {
                id: Uuid::new_v4(),
                policy_id: policy.id,
                priority: 2, // Good: < 2 late, < 1 unpaid
                max_late_early_count: Some(2),
                max_unpaid_leave_days: Some(1.0),
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 250000.0,
            }
        ];

        // Match Tier 1
        let ctx1 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };
        assert_eq!(PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx1).amount, Decimal::from(500000));

        // Match Tier 2 (due to 1 late)
        let ctx2 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 1,
        };
        assert_eq!(PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx2).amount, Decimal::from(250000));

        // No Match (too many lates)
        let ctx3 = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 3,
        };
        assert_eq!(PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx3).amount, Decimal::ZERO);
    }
}
