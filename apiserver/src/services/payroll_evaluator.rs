use crate::errors::AppError;
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
    ) -> Result<IncentiveEvaluationResult, AppError> {
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
                .ok_or_else(|| {
                    tracing::error!(policy_name = %policy.name, "Invalid f64 for Decimal conversion in tier payout");
                    AppError::Internal("Invalid monetary amount in incentive tier".into())
                })?;

                let standard_amount =
                    rust_decimal::Decimal::from_f64(policy.base_amount).ok_or_else(|| {
                        tracing::error!(policy_name = %policy.name, base_amount = policy.base_amount, "Invalid f64 for Decimal conversion in base_amount");
                        AppError::Internal("Invalid monetary amount in incentive policy".into())
                    })?;

                return Ok(IncentiveEvaluationResult {
                    policy_name: policy.name.clone(),
                    amount,
                    standard_amount,
                    tier_id: Some(tier.id),
                });
            }
        }

        // If no tier matches, return zero
        let standard_amount = rust_decimal::Decimal::from_f64(policy.base_amount)
            .ok_or_else(|| {
                tracing::error!(policy_name = %policy.name, base_amount = policy.base_amount, "Invalid f64 for Decimal conversion in base_amount");
                AppError::Internal("Invalid monetary amount in incentive policy".into())
            })?;

        Ok(IncentiveEvaluationResult {
            policy_name: policy.name.clone(),
            amount: rust_decimal::Decimal::ZERO,
            standard_amount,
            tier_id: None,
        })
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
        let res1 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx1).unwrap();
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
        let res2 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx2).unwrap();
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
        let res3 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx3).unwrap();
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
        let result_old = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx_without_exclusion).unwrap();
        assert_eq!(result_old.amount, Decimal::from(50000), "Without exclusion: Tier 2 (50k)");

        // After WF7: late_early_count = 0 (holiday late excluded) → Tier 1 (150k)
        let ctx_with_exclusion = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0, // Holiday late excluded by payroll_service.rs
        };
        let result_new = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx_with_exclusion).unwrap();
        assert_eq!(result_new.amount, Decimal::from(150000), "With exclusion: Tier 1 (150k)");

        // The difference is 100k — excluding holidays from late count matters!
        assert_eq!(
            result_new.amount - result_old.amount,
            Decimal::from(100000),
            "Holiday exclusion saves employee 100k in incentive"
        );
    }

    // --- Additional tests ---

    #[test]
    fn test_percentage_payout_type() {
        let p_id = Uuid::new_v4();
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: Uuid::new_v4(),
            name: "Bonus".into(),
            base_amount: 1_000_000.0, // 1M base
            payout_type: "percentage".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };
        let tiers = vec![IncentiveTier {
            id: Uuid::new_v4(),
            policy_id: p_id,
            priority: 1,
            max_late_early_count: None,
            max_unpaid_leave_days: None,
            max_sick_leave_days: None,
            max_total_leave_days: None,
            payout_amount: 0.1, // 10%
        }];
        let ctx = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };
        let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx).unwrap();
        // percentage: base_amount * payout_amount = 1_000_000 * 0.1 = 100_000
        assert_eq!(res.amount, Decimal::from(100_000));
    }

    #[test]
    fn test_unpaid_leave_exceeds_threshold() {
        let p_id = Uuid::new_v4();
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: Uuid::new_v4(),
            name: "Attendance Bonus".into(),
            base_amount: 200_000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };
        let tiers = vec![IncentiveTier {
            id: Uuid::new_v4(),
            policy_id: p_id,
            priority: 1,
            max_late_early_count: None,
            max_unpaid_leave_days: Some(1.0), // Max 1 unpaid leave day
            max_sick_leave_days: None,
            max_total_leave_days: None,
            payout_amount: 200_000.0,
        }];
        // Employee has 2 unpaid leave days → exceeds threshold
        let ctx = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 2.0,
            unpaid_leave_days: 2.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };
        let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx).unwrap();
        assert_eq!(res.amount, Decimal::ZERO);
    }

    #[test]
    fn test_sick_leave_threshold() {
        let p_id = Uuid::new_v4();
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: Uuid::new_v4(),
            name: "Health Bonus".into(),
            base_amount: 100_000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };
        let tiers = vec![IncentiveTier {
            id: Uuid::new_v4(),
            policy_id: p_id,
            priority: 1,
            max_late_early_count: None,
            max_unpaid_leave_days: None,
            max_sick_leave_days: Some(0.0), // No sick days allowed
            max_total_leave_days: None,
            payout_amount: 100_000.0,
        }];
        // Employee has 1 sick day
        let ctx = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 1.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 1.0,
            late_early_count: 0,
        };
        let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx).unwrap();
        assert_eq!(res.amount, Decimal::ZERO);
    }

    #[test]
    fn test_total_leave_threshold() {
        let p_id = Uuid::new_v4();
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: Uuid::new_v4(),
            name: "Full Bonus".into(),
            base_amount: 500_000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };
        let tiers = vec![IncentiveTier {
            id: Uuid::new_v4(),
            policy_id: p_id,
            priority: 1,
            max_late_early_count: None,
            max_unpaid_leave_days: None,
            max_sick_leave_days: None,
            max_total_leave_days: Some(3.0), // Max 3 total leave days
            payout_amount: 500_000.0,
        }];
        // Exactly at limit → should pass
        let ctx_pass = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 3.0,
            unpaid_leave_days: 1.0,
            sick_leave_days: 2.0,
            late_early_count: 0,
        };
        let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx_pass).unwrap();
        assert_eq!(res.amount, Decimal::from(500_000));

        // Above limit → should fail
        let ctx_fail = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 4.0,
            unpaid_leave_days: 2.0,
            sick_leave_days: 2.0,
            late_early_count: 0,
        };
        let res2 = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx_fail).unwrap();
        assert_eq!(res2.amount, Decimal::ZERO);
    }

    #[test]
    fn test_tier_priority_selects_best_match() {
        let p_id = Uuid::new_v4();
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: Uuid::new_v4(),
            name: "Multi-Tier".into(),
            base_amount: 300_000.0,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };
        let t2_id = Uuid::new_v4();
        let tiers = vec![
            // Tier 1: strict (priority 1) — requires 0 late
            IncentiveTier {
                id: Uuid::new_v4(),
                policy_id: p_id,
                priority: 1,
                max_late_early_count: Some(0),
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 300_000.0,
            },
            // Tier 2: lenient (priority 2) — allows up to 5 late
            IncentiveTier {
                id: t2_id,
                policy_id: p_id,
                priority: 2,
                max_late_early_count: Some(5),
                max_unpaid_leave_days: None,
                max_sick_leave_days: None,
                max_total_leave_days: None,
                payout_amount: 100_000.0,
            },
        ];
        // Employee is late 2 times → doesn't match Tier 1, matches Tier 2
        let ctx = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 2,
        };
        let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx).unwrap();
        assert_eq!(res.amount, Decimal::from(100_000));
        assert_eq!(res.tier_id, Some(t2_id));
    }

    #[test]
    fn test_evaluate_policy_invalid_f64_returns_error() {
        let p_id = Uuid::new_v4();
        // Policy with NaN base_amount
        let policy = PayrollIncentivePolicy {
            id: p_id,
            workspace_id: Uuid::new_v4(),
            name: "Invalid Policy".into(),
            base_amount: f64::NAN,
            payout_type: "fixed".into(),
            is_active: true,
            created_at: chrono::Utc::now(),
        };

        let tiers = vec![IncentiveTier {
            id: Uuid::new_v4(),
            policy_id: p_id,
            priority: 1,
            max_late_early_count: Some(0),
            max_unpaid_leave_days: None,
            max_sick_leave_days: None,
            max_total_leave_days: None,
            payout_amount: 100_000.0,
        }];

        let ctx = IncentiveContext {
            total_work_units: 160,
            total_leave_days: 0.0,
            unpaid_leave_days: 0.0,
            sick_leave_days: 0.0,
            late_early_count: 0,
        };

        // Should return an Err because base_amount is NaN
        let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &ctx);
        assert!(res.is_err(), "Evaluating an invalid float should return an error");
        let err = res.unwrap_err();
        assert!(matches!(err, crate::errors::AppError::Internal(_)));
        
        // Also test invalid tier payout_amount
        let policy2 = PayrollIncentivePolicy {
            base_amount: 100_000.0,
            ..policy.clone()
        };
        
        let tiers_invalid = vec![IncentiveTier {
            payout_amount: f64::INFINITY,
            ..tiers[0].clone()
        }];
        
        let res2 = PayrollEvaluator::evaluate_policy(&policy2, &tiers_invalid, &ctx);
        assert!(res2.is_err(), "Evaluating an invalid tier float should return an error");
    }
}
