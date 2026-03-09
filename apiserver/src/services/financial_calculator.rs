//! FinancialCalculator — Rate calculations, OT capping, incentives,
//! deductions, and Payslip assembly.
//!
//! Extracted from `PayrollService` to satisfy SRP.

use crate::errors::AppError;
use chrono::{NaiveDate, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{IncentiveEvaluationResult, Payslip};
use crate::repositories::{leave_request_repository, payroll_repository};
use crate::services::payroll_evaluator::{IncentiveContext, PayrollEvaluator};

use super::payroll_service::SegmentSnapshot;

/// Incentives calculation result.
pub struct IncentivesResult {
    pub total_amount: Decimal,
    pub results: Vec<IncentiveEvaluationResult>,
}

/// Deductions calculation result.
pub struct DeductionsResult {
    pub amount: Decimal,
}

pub struct FinancialCalculator;

impl FinancialCalculator {
    // -----------------------------------------------------------------------
    // Pure rate helpers (no DB, no side-effects)
    // -----------------------------------------------------------------------

    pub fn calculate_daily_rate(total_base: Decimal, std_working_days: Decimal) -> Decimal {
        if std_working_days.is_zero() {
            Decimal::ZERO
        } else {
            total_base / std_working_days
        }
    }

    pub fn calculate_hourly_rate(
        ot_base: Decimal,
        std_working_days: Decimal,
        hours_per_day: Decimal,
    ) -> Decimal {
        if std_working_days.is_zero() || hours_per_day.is_zero() {
            Decimal::ZERO
        } else {
            (ot_base / std_working_days) / hours_per_day
        }
    }

    /// Apply monthly OT cap, proportionally reducing OT pay.
    /// Returns `(capped_ot_units, adjusted_ot_pay)`.
    pub fn apply_ot_cap(
        total_ot_units: i32,
        total_overtime_pay: Decimal,
        max_monthly_hours: Decimal,
        work_unit_minutes: i32,
    ) -> (i32, Decimal) {
        let max_ot_minutes = max_monthly_hours * Decimal::from(60);
        let max_ot_units_dec = max_ot_minutes / Decimal::from(work_unit_minutes);
        let max_ot_units = max_ot_units_dec.to_i32().unwrap_or(i32::MAX);

        if total_ot_units > max_ot_units {
            let adjusted_pay = if total_ot_units > 0 {
                let ratio = Decimal::from(max_ot_units) / Decimal::from(total_ot_units);
                (total_overtime_pay * ratio).round_dp(0)
            } else {
                total_overtime_pay
            };
            (max_ot_units, adjusted_pay)
        } else {
            (total_ot_units, total_overtime_pay)
        }
    }

    // -----------------------------------------------------------------------
    // Incentives
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub async fn calculate_incentives(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        total_std_units: i32,
        total_late_early_count: i32,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<IncentivesResult, AppError> {
        let active_policies = payroll_repository::get_active_incentive_policies(pool, workspace_id)
            .await
            .unwrap_or_default();
        let mut incentive_results = Vec::new();
        let mut total_incentive_amount = Decimal::ZERO;

        let leave_breakdown = leave_request_repository::get_approved_leave_breakdown(
            pool,
            employee_id,
            start_date,
            end_date,
            workspace_id,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                "Failed to fetch leave breakdown for employee {}: {}",
                employee_id,
                e
            );
            AppError::Internal("Internal error".into())
        })?;

        if !active_policies.is_empty() {
            let context = IncentiveContext {
                total_work_units: total_std_units,
                total_leave_days: leave_breakdown.total_days,
                unpaid_leave_days: leave_breakdown.unpaid_days,
                sick_leave_days: leave_breakdown.sick_days,
                late_early_count: total_late_early_count,
            };

            for policy in active_policies {
                let tiers = payroll_repository::get_policy_tiers(pool, policy.id)
                    .await
                    .unwrap_or_default();
                let res = PayrollEvaluator::evaluate_policy(&policy, &tiers, &context);
                if !res.amount.is_zero() {
                    total_incentive_amount += res.amount;
                    incentive_results.push(res);
                }
            }
        }

        Ok(IncentivesResult {
            total_amount: total_incentive_amount,
            results: incentive_results,
        })
    }

    // -----------------------------------------------------------------------
    // Deductions
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub async fn calculate_deductions(
        pool: &PgPool,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        segment_snapshots: &[SegmentSnapshot],
    ) -> Result<DeductionsResult, AppError> {
        let leave_breakdown = leave_request_repository::get_approved_leave_breakdown(
            pool, employee_id, start_date, end_date, workspace_id,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch leave breakdown: {}", e);
            AppError::Internal("Internal error".into())
        })?;

        let mut deductions_amount = Decimal::ZERO;
        if leave_breakdown.unpaid_days > 0.0 {
            if let Some(last_snap) = segment_snapshots.last() {
                let daily_rate =
                    Self::calculate_daily_rate(last_snap.total_base, last_snap.std_working_days);
                deductions_amount = daily_rate
                    * Decimal::from_f64(leave_breakdown.unpaid_days).unwrap_or(Decimal::ZERO);
            }
        }

        // Deductions for unpaid holidays where employee has no leave request
        let unpaid_holiday_days: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) as "count!"
               FROM holidays h
               WHERE h.workspace_id = $1
                 AND h.is_paid = false
                 AND h.date >= $2
                 AND h.date <= $3
                 AND NOT EXISTS (
                     SELECT 1 FROM leave_requests lr
                     WHERE lr.employee_id = $4
                       AND lr.status = 'approved'
                       AND lr.start_date <= h.date
                       AND lr.end_date >= h.date
                 )"#,
            workspace_id,
            start_date,
            end_date,
            employee_id
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if unpaid_holiday_days > 0 {
            if let Some(last_snap) = segment_snapshots.last() {
                let daily_rate =
                    Self::calculate_daily_rate(last_snap.total_base, last_snap.std_working_days);
                deductions_amount += daily_rate * Decimal::from(unpaid_holiday_days);
            }
        }

        Ok(DeductionsResult {
            amount: deductions_amount,
        })
    }

    // -----------------------------------------------------------------------
    // Payslip Assembly
    // -----------------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn assemble_payslip(
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        total_base_pay: Decimal,
        total_overtime_pay: Decimal,
        total_holiday_work_premium: Decimal,
        total_incentive_amount: Decimal,
        deductions_amount: Decimal,
        total_std_units: i32,
        total_ot_units: i32,
    ) -> Payslip {
        Payslip {
            id: Uuid::new_v4(),
            employee_id,
            period_start: start_date,
            period_end: end_date,
            base_pay_amount: total_base_pay,
            overtime_pay_amount: total_overtime_pay,
            deductions_amount,
            holiday_work_premium_amount: total_holiday_work_premium,
            net_pay_amount: total_base_pay + total_overtime_pay + total_incentive_amount
                + total_holiday_work_premium - deductions_amount,
            standard_units: total_std_units,
            overtime_units: total_ot_units,
            status: "draft".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_rate_normal() {
        let rate = FinancialCalculator::calculate_daily_rate(Decimal::from(26000), Decimal::from(26));
        assert_eq!(rate, Decimal::from(1000));
    }

    #[test]
    fn test_daily_rate_zero_days() {
        let rate = FinancialCalculator::calculate_daily_rate(Decimal::from(26000), Decimal::ZERO);
        assert_eq!(rate, Decimal::ZERO);
    }

    #[test]
    fn test_hourly_rate_normal() {
        let rate = FinancialCalculator::calculate_hourly_rate(
            Decimal::from(26000), Decimal::from(26), Decimal::from(8),
        );
        assert_eq!(rate, Decimal::from(125));
    }

    #[test]
    fn test_hourly_rate_zero_days() {
        let rate = FinancialCalculator::calculate_hourly_rate(
            Decimal::from(26000), Decimal::ZERO, Decimal::from(8),
        );
        assert_eq!(rate, Decimal::ZERO);
    }

    #[test]
    fn test_hourly_rate_zero_hours() {
        let rate = FinancialCalculator::calculate_hourly_rate(
            Decimal::from(26000), Decimal::from(26), Decimal::ZERO,
        );
        assert_eq!(rate, Decimal::ZERO);
    }

    #[test]
    fn test_apply_ot_cap_no_cap() {
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            10, Decimal::from(5000), Decimal::from(100), 30,
        );
        assert_eq!(units, 10);
        assert_eq!(pay, Decimal::from(5000));
    }

    #[test]
    fn test_apply_ot_cap_capped() {
        // 10 units at 30min = 300min = 5h; cap at 2h = 120min = 4 units
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            10, Decimal::from(5000), Decimal::from(2), 30,
        );
        assert_eq!(units, 4);
        assert_eq!(pay, Decimal::from(2000));
    }

    #[test]
    fn test_apply_ot_cap_zero_units() {
        let (units, pay) = FinancialCalculator::apply_ot_cap(
            0, Decimal::ZERO, Decimal::from(10), 30,
        );
        assert_eq!(units, 0);
        assert_eq!(pay, Decimal::ZERO);
    }

    #[test]
    fn test_assemble_payslip_basic() {
        let emp = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("valid date");
        let end = NaiveDate::from_ymd_opt(2026, 1, 31).expect("valid date");

        let slip = FinancialCalculator::assemble_payslip(
            emp,
            start,
            end,
            Decimal::from(10000),  // base
            Decimal::from(2000),   // OT
            Decimal::from(500),    // holiday premium
            Decimal::from(300),    // incentive
            Decimal::from(800),    // deductions
            26,                     // std units
            10,                     // OT units
        );

        assert_eq!(slip.employee_id, emp);
        assert_eq!(slip.period_start, start);
        assert_eq!(slip.period_end, end);
        assert_eq!(slip.base_pay_amount, Decimal::from(10000));
        assert_eq!(slip.overtime_pay_amount, Decimal::from(2000));
        assert_eq!(slip.holiday_work_premium_amount, Decimal::from(500));
        assert_eq!(slip.deductions_amount, Decimal::from(800));
        // net = 10000 + 2000 + 300 + 500 - 800 = 12000
        assert_eq!(slip.net_pay_amount, Decimal::from(12000));
        assert_eq!(slip.standard_units, 26);
        assert_eq!(slip.overtime_units, 10);
        assert_eq!(slip.status, "draft");
    }

    #[test]
    fn test_assemble_payslip_zero_everything() {
        let emp = Uuid::new_v4();
        let start = NaiveDate::from_ymd_opt(2026, 3, 1).expect("valid date");
        let end = NaiveDate::from_ymd_opt(2026, 3, 31).expect("valid date");

        let slip = FinancialCalculator::assemble_payslip(
            emp, start, end,
            Decimal::ZERO, Decimal::ZERO, Decimal::ZERO,
            Decimal::ZERO, Decimal::ZERO, 0, 0,
        );

        assert_eq!(slip.net_pay_amount, Decimal::ZERO);
        assert_eq!(slip.standard_units, 0);
        assert_eq!(slip.overtime_units, 0);
    }

    #[test]
    fn test_incentives_result_construction() {
        let result = IncentivesResult {
            total_amount: Decimal::from(1500),
            results: vec![],
        };
        assert_eq!(result.total_amount, Decimal::from(1500));
        assert!(result.results.is_empty());
    }

    #[test]
    fn test_deductions_result_construction() {
        let result = DeductionsResult {
            amount: Decimal::from(500),
        };
        assert_eq!(result.amount, Decimal::from(500));
    }
}
