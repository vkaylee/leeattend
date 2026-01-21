use crate::models::{AccrualFrequency, LeaveAccrualLog, LeavePolicyConfig};
use crate::repositories::{department_repository, employee_repository, leave_policy_repository};
use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;
use uuid::Uuid;

pub struct LeaveAccrualService;

impl LeaveAccrualService {
    /// Pure function to calculate how much leave should be accrued for a given policy and employee state.
    pub fn calculate_accrual_amount(
        config: &LeavePolicyConfig,
        years_service: i32,
        _accrual_date: NaiveDate,
    ) -> f64 {
        let mut amount = config.accrual_amount;

        // Apply seniority bonuses (additive)
        // Assumption: extra_days is "Days added per accrual period"
        for bonus in &config.seniority_bonus {
            if years_service >= bonus.min_years {
                amount += bonus.extra_days;
            }
        }

        amount
    }

    /// Helper to calculate full years of service across multiple periods
    pub fn calculate_total_seniority(
        periods: &[crate::models::EmploymentPeriod],
        as_of: NaiveDate,
    ) -> i32 {
        let mut total_days = 0;
        for period in periods {
            if period.hire_date > as_of {
                continue;
            }
            let end_date = period.termination_date.unwrap_or(as_of);
            let end_date = if end_date > as_of { as_of } else { end_date };

            if end_date >= period.hire_date {
                total_days += (end_date - period.hire_date).num_days() + 1;
            }
        }

        (total_days / 365) as i32
    }

    /// Determines if an accrual should run on this date based on frequency.
    /// Returns true if this date matches the schedule.
    ///
    /// Policies:
    /// - Monthly: Runs on 1st of month (if timing=start) or Last of month (if timing=end).
    /// - Quarterly: Runs on Jan 1, Apr 1, Jul 1, Oct 1.
    pub fn should_run_accrual(
        frequency: &AccrualFrequency,
        timing: &str, // "start_of_period" or "end_of_period"
        date: NaiveDate,
    ) -> bool {
        let is_start = timing == "start_of_period";

        match frequency {
            AccrualFrequency::Monthly => {
                if is_start {
                    date.day() == 1
                } else {
                    // Last day of month
                    let next_day = date.succ_opt().unwrap_or(date);
                    next_day.month() != date.month()
                }
            }
            AccrualFrequency::Quarterly => {
                // Quarters start: Jan(1), Apr(4), Jul(7), Oct(10)
                let is_quarter_month = matches!(date.month(), 1 | 4 | 7 | 10);
                if is_start {
                    date.day() == 1 && is_quarter_month
                } else {
                    // End of quarter: Mar 31, Jun 30, Sep 30, Dec 31
                    // Logic: Tomorrow is 1st of (Apr, Jul, Oct, Jan)
                    let next_day = date.succ_opt().unwrap_or(date);
                    next_day.day() == 1 && matches!(next_day.month(), 4 | 7 | 10 | 1)
                }
            }
            AccrualFrequency::Yearly => {
                if is_start {
                    date.day() == 1 && date.month() == 1
                } else {
                    date.day() == 31 && date.month() == 12
                }
            }
        }
    }

    /// Main orchestrator to process accruals for a workspace on a specific date.
    /// This is expected to be called by a daily cron job.
    pub async fn process_workspace_accruals(
        pool: &PgPool,
        workspace_id: Uuid,
        date: NaiveDate,
        force: bool,
    ) -> Result<Vec<LeaveAccrualLog>, sqlx::Error> {
        // 1. Get all active employees
        let employees = employee_repository::get_active_employees(pool, workspace_id).await?;
        let mut all_logs = Vec::new();

        for emp in employees {
            // 2. Resolve policy for employee
            let dept_hist = department_repository::get_current_department(pool, emp.id)
                .await
                .ok()
                .flatten();

            if let Some((policy, assignment_id)) = leave_policy_repository::find_active_assignment_for_employee(
                pool,
                workspace_id,
                emp.id,
                dept_hist.map(|d| d.department_id),
                date,
            )
            .await?
            {
                let config = &policy.rule_config.0;

                // --- SMART CATCH-UP LOGIC ---
                // We iterate from the start of the policy assignment up to the target date.
                // This ensures that if a policy was assigned retroactively, we backfill missing days.
                
                // Fetch the assignment record to get the true effective_from
                let assignment = sqlx::query!(
                    "SELECT effective_from FROM leave_policy_assignments WHERE id = $1",
                    assignment_id
                ).fetch_one(pool).await?;

                let mut current_check_date = assignment.effective_from;
                
                while current_check_date <= date {
                    // Check if accrual should run on THIS specific date in the past/present
                    if Self::should_run_accrual(&config.frequency, &config.accrual_timing, current_check_date) {
                        
                        let periods = employee_repository::get_employment_history(pool, emp.id).await?;
                        let years_service = Self::calculate_total_seniority(&periods, current_check_date);
                        let amount = Self::calculate_accrual_amount(config, years_service, current_check_date);

                        if amount > 0.0 {
                            use rust_decimal::prelude::FromPrimitive;
                            let decimal_amount = rust_decimal::Decimal::from_f64(amount).unwrap_or_default();

                            // Check if a log already exists for this date/type/employee
                            let existing_log = sqlx::query!(
                                r#"
                                SELECT id, amount FROM leave_accrual_logs 
                                WHERE employee_id = $1 AND leave_type = $2 AND accrual_date = $3
                                "#,
                                emp.id, policy.leave_type, current_check_date
                            ).fetch_optional(pool).await?;

                            if let Some(old_log) = existing_log {
                                // If force is true and amount has changed, RE-CALCULATE
                                if force && old_log.amount != decimal_amount {
                                    let diff = decimal_amount - old_log.amount;

                                    // 1. Update the log entry
                                    let updated_log = sqlx::query_as!(
                                        LeaveAccrualLog,
                                        r#"
                                        UPDATE leave_accrual_logs 
                                        SET amount = $1, notes = COALESCE(notes, '') || ' (Re-calculated)'
                                        WHERE id = $2
                                        RETURNING id, workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type, notes, created_at as "created_at!"
                                        "#,
                                        decimal_amount, old_log.id
                                    ).fetch_one(pool).await?;

                                    // 2. Adjust the balance entry (add the difference)
                                    sqlx::query!(
                                        r#"
                                        UPDATE leave_balances 
                                        SET accrued_amount = accrued_amount + $3, updated_at = NOW()
                                        WHERE employee_id = $1 AND year = $2 AND leave_type = $4
                                        "#,
                                        emp.id, current_check_date.year(), diff, policy.leave_type
                                    ).execute(pool).await?;

                                    all_logs.push(updated_log);
                                }
                            } else {
                                // No log exists, perform original insertion logic
                                let log = sqlx::query_as!(
                                    LeaveAccrualLog,
                                    r#"
                                    INSERT INTO leave_accrual_logs (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type)
                                    VALUES ($1, $2, $4, $3, $5, $6)
                                    ON CONFLICT (employee_id, leave_type, accrual_date) DO NOTHING
                                    RETURNING id, workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type, notes, created_at as "created_at!"
                                    "#,
                                    workspace_id,
                                    emp.id,
                                    current_check_date,
                                    Some(assignment_id),
                                    decimal_amount,
                                    policy.leave_type
                                )
                                .fetch_optional(pool)
                                .await?;

                                if let Some(l) = log {
                                    // Update Balance for the year of THIS check date
                                    sqlx::query!(
                                        r#"
                                        INSERT INTO leave_balances (employee_id, year, leave_type, entitlement, accrued_amount, carried_over)
                                        VALUES ($1, $2, $3, 0, $4, 0)
                                        ON CONFLICT (employee_id, leave_type, year)
                                        DO UPDATE SET 
                                            accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
                                            updated_at = NOW()
                                        "#,
                                        emp.id,
                                        current_check_date.year(),
                                        policy.leave_type,
                                        decimal_amount
                                    )
                                    .execute(pool).await?;

                                    all_logs.push(l);
                                }
                            }
                        }
                    }

                    // Move to next day
                    if let Some(next) = current_check_date.succ_opt() {
                        current_check_date = next;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(all_logs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccrualFrequency, LeavePolicyConfig, SeniorityBonus};

    #[test]
    fn test_calculate_accrual_amount() {
        let config = LeavePolicyConfig {
            frequency: AccrualFrequency::Monthly,
            accrual_amount: 1.0,
            accrual_timing: "start_of_period".to_string(),
            seniority_bonus: vec![
                SeniorityBonus {
                    min_years: 1,
                    extra_days: 0.5,
                },
                SeniorityBonus {
                    min_years: 5,
                    extra_days: 1.0,
                },
            ],
            max_balance: None,
            reset_month: None,
        };

        // 0 years service
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(
                &config,
                0,
                NaiveDate::from_ymd_opt(2020, 6, 1).unwrap()
            ),
            1.0
        );

        // 1 year service
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(
                &config,
                1,
                NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()
            ),
            1.5
        );

        // 5 years service
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(
                &config,
                5,
                NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
            ),
            2.5
        );
    }

    #[test]
    fn test_should_run_accrual() {
        let monthly = AccrualFrequency::Monthly;
        let quarterly = AccrualFrequency::Quarterly;
        let yearly = AccrualFrequency::Yearly;

        // Monthly start
        assert!(LeaveAccrualService::should_run_accrual(
            &monthly,
            "start_of_period",
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &monthly,
            "start_of_period",
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
        ));

        // Monthly end
        assert!(LeaveAccrualService::should_run_accrual(
            &monthly,
            "end_of_period",
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &monthly,
            "end_of_period",
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
        ));

        // Quarterly start
        assert!(LeaveAccrualService::should_run_accrual(
            &quarterly,
            "start_of_period",
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &quarterly,
            "start_of_period",
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
        ));

        // Quarterly end
        assert!(LeaveAccrualService::should_run_accrual(
            &quarterly,
            "end_of_period",
            NaiveDate::from_ymd_opt(2026, 3, 31).unwrap()
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &quarterly,
            "end_of_period",
            NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()
        ));

        // Yearly start
        assert!(LeaveAccrualService::should_run_accrual(
            &yearly,
            "start_of_period",
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
        ));

        // Yearly end
        assert!(LeaveAccrualService::should_run_accrual(
            &yearly,
            "end_of_period",
            NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()
        ));
    }
}