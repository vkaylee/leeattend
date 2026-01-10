use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{LeavePolicyConfig, AccrualFrequency, LeaveAccrualLog};
use crate::repositories::{leave_policy_repository, employee_repository, department_repository};

pub struct LeaveAccrualService;

impl LeaveAccrualService {
    
    /// Pure function to calculate how much leave should be accrued for a given policy and employee state.
    pub fn calculate_accrual_amount(
        config: &LeavePolicyConfig,
        hire_date: NaiveDate,
        accrual_date: NaiveDate,
    ) -> f64 {
        let mut amount = config.accrual_amount;
        
        let years_service = Self::years_between(hire_date, accrual_date);
        
        // Apply seniority bonuses (additive)
        // Assumption: extra_days is "Days added per accrual period"
        for bonus in &config.seniority_bonus {
            if years_service >= bonus.min_years {
                amount += bonus.extra_days;
            }
        }
        
        amount
    }

    /// Helper to calculate full years of service
    fn years_between(start: NaiveDate, end: NaiveDate) -> i32 {
        if end < start {
            return 0;
        }
        let mut years = end.year() - start.year();
        if end.month() < start.month() || (end.month() == start.month() && end.day() < start.day()) {
            years -= 1;
        }
        years
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
            },
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
            },
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
    ) -> Result<Vec<LeaveAccrualLog>, sqlx::Error> {
        // 1. Get all active employees
        let employees = employee_repository::get_active_employees(pool, workspace_id).await?;
        let mut logs = Vec::new();

        for emp in employees {
            // 2. Resolve policy for employee
            // Note: `employee_repository::get_active_employees` returns `Employee` struct.
            // It doesn't have department_id directly. 
            // We need to fetch it.
            let dept_hist = department_repository::get_current_department(pool, emp.id).await.ok().flatten();
            
            if let Some(policy) = leave_policy_repository::find_active_policy_for_employee(
                pool, 
                workspace_id, 
                emp.id, 
                dept_hist.map(|d| d.department_id), 
                date
            ).await? {
                let config = &policy.rule_config.0;
                
                // 3. Check if accrual should run today
                if Self::should_run_accrual(&config.frequency, &config.accrual_timing, date) {
                    // 4. Calculate amount
                    let amount = Self::calculate_accrual_amount(config, emp.hire_date, date);
                    
                    if amount > 0.0 {
                        use rust_decimal::prelude::FromPrimitive;
                        let decimal_amount = rust_decimal::Decimal::from_f64(amount).unwrap_or_default();
                        
                        // 5. Update DB (Transactionally)
                        // Create Accrual Log
                        let log = sqlx::query_as!(
                            LeaveAccrualLog,
                            r#"
                            INSERT INTO leave_accrual_logs (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type)
                            VALUES ($1, $2, NULL, $3, $4, 'vacation')
                            ON CONFLICT (employee_id, leave_type, accrual_date) DO NOTHING
                            RETURNING id, workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type, notes, created_at as "created_at!"
                            "#,
                            workspace_id,
                            emp.id,
                            date,
                            decimal_amount
                        )
                        .fetch_optional(pool) // Optional because ON CONFLICT DO NOTHING
                        .await?;
                        
                        if let Some(l) = log {
                            // Update Balance
                            // Note: We need to handle year. Accrual date year.
                            sqlx::query!(
                                r#"
                                UPDATE leave_balances 
                                SET accrued_amount = accrued_amount + $3, updated_at = NOW()
                                WHERE employee_id = $1 AND leave_type = 'vacation' AND year = $2
                                "#,
                                emp.id,
                                date.year(),
                                decimal_amount
                            )
                            .execute(pool)
                            .await?;
                            
                            logs.push(l);
                        }
                    }
                }
            }
        }
        
        Ok(logs)
    }
}