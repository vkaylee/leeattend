use crate::models::{AccrualFrequency, LeaveAccrualLog, LeavePolicyConfig};
use crate::repositories::{
    department_repository, employee_repository, leave_policy_repository,
};
use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
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
        for bonus in &config.seniority_bonus {
            if years_service >= bonus.min_years {
                amount += bonus.extra_days;
            }
        }

        amount
    }

    /// Helper to count "Qualifying Days" for performance-based accrual.
    pub async fn count_qualifying_days(
        pool: &PgPool,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<i32, sqlx::Error> {
        use std::collections::HashSet;
        let mut qualifying_days = HashSet::new();

        // Days with actual work entries
        let start_dt = tz
            .from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap())
            .single()
            .unwrap()
            .with_timezone(&Utc);
        let end_dt = tz
            .from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap())
            .single()
            .unwrap()
            .with_timezone(&Utc);

        let work_days = sqlx::query!(
            r#"
            SELECT DISTINCT(punch_in_time AT TIME ZONE $2)::date as "date!"
            FROM time_entries
            WHERE employee_id = $1
              AND punch_in_time >= $3
              AND punch_in_time <= $4
              AND entry_type = 'work'
            "#,
            employee_id,
            tz.to_string(),
            start_dt,
            end_dt
        )
        .fetch_all(pool)
        .await?;

        for row in work_days {
            qualifying_days.insert(row.date);
        }

        // Approved Leave days
        let leave_days = sqlx::query!(
            r#"
            SELECT start_date, end_date
            FROM leave_requests
            WHERE employee_id = $1
              AND status = 'approved'
              AND start_date <= $3
              AND end_date >= $2
            "#,
            employee_id,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        for req in leave_days {
            let overlap_start = std::cmp::max(req.start_date, start_date);
            let overlap_end = std::cmp::min(req.end_date, end_date);
            let mut curr = overlap_start;
            while curr <= overlap_end {
                qualifying_days.insert(curr);
                if let Some(next) = curr.succ_opt() {
                    curr = next;
                } else {
                    break;
                }
            }
        }

        Ok(qualifying_days.len() as i32)
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
    pub fn should_run_accrual(
        frequency: &AccrualFrequency,
        timing: &str,
        date: NaiveDate,
    ) -> bool {
        let is_start = timing == "start_of_period";

        match frequency {
            AccrualFrequency::Monthly => {
                if is_start {
                    date.day() == 1
                } else {
                    let next_day = date.succ_opt().unwrap_or(date);
                    next_day.month() != date.month()
                }
            }
            AccrualFrequency::Quarterly => {
                let is_quarter_month = matches!(date.month(), 1 | 4 | 7 | 10);
                if is_start {
                    date.day() == 1 && is_quarter_month
                } else {
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
    pub async fn process_workspace_accruals(
        pool: &PgPool,
        workspace_id: Uuid,
        date: NaiveDate,
        force: bool,
    ) -> Result<Vec<LeaveAccrualLog>, sqlx::Error> {
        // Get Workspace settings for timezone
        let workspace = sqlx::query!(
            "SELECT timezone FROM workspaces WHERE id = $1",
            workspace_id
        )
        .fetch_one(pool)
        .await?;
        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // Get all active employees
        let employees = employee_repository::get_active_employees(pool, workspace_id).await?;
        let mut all_logs = Vec::new();

        for emp in employees {
            // Resolve policy for employee
            let dept_hist = department_repository::get_current_department(pool, emp.id)
                .await
                .ok()
                .flatten();

            if let Some((policy, assignment_id, _leave_type_code)) =
                leave_policy_repository::find_active_assignment_for_employee(
                    pool,
                    workspace_id,
                    emp.id,
                    dept_hist.map(|d| d.department_id),
                    date,
                )
                .await?
            {
                let config = &policy.rule_config.0;
                let leave_type_id = policy.leave_type_id;

                // Fetch the assignment record to get the true effective_from
                let assignment = sqlx::query!(
                    "SELECT effective_from FROM leave_policy_assignments WHERE id = $1",
                    assignment_id
                )
                .fetch_one(pool)
                .await?;

                let mut current_check_date = assignment.effective_from;

                while current_check_date <= date {
                    if Self::should_run_accrual(
                        &config.frequency,
                        &config.accrual_timing,
                        current_check_date,
                    ) {
                        let periods =
                            employee_repository::get_employment_history(pool, emp.id).await?;
                        let years_service =
                            Self::calculate_total_seniority(&periods, current_check_date);
                        let mut amount =
                            Self::calculate_accrual_amount(config, years_service, current_check_date);

                        // Performance threshold check
                        if let Some(threshold) = config.min_work_days_threshold {
                            let (check_start, check_end) = match config.frequency {
                                AccrualFrequency::Monthly => {
                                    if config.accrual_timing == "start_of_period" {
                                        let prev_month_date = if current_check_date.month() == 1 {
                                            NaiveDate::from_ymd_opt(
                                                current_check_date.year() - 1,
                                                12,
                                                1,
                                            )
                                            .unwrap()
                                        } else {
                                            NaiveDate::from_ymd_opt(
                                                current_check_date.year(),
                                                current_check_date.month() - 1,
                                                1,
                                            )
                                            .unwrap()
                                        };
                                        let end = NaiveDate::from_ymd_opt(
                                            current_check_date.year(),
                                            current_check_date.month(),
                                            1,
                                        )
                                        .unwrap()
                                        .pred_opt()
                                        .unwrap();
                                        (prev_month_date, end)
                                    } else {
                                        let start = NaiveDate::from_ymd_opt(
                                            current_check_date.year(),
                                            current_check_date.month(),
                                            1,
                                        )
                                        .unwrap();
                                        (start, current_check_date)
                                    }
                                }
                                _ => (current_check_date, current_check_date),
                            };

                            let qualifying_days =
                                Self::count_qualifying_days(pool, emp.id, check_start, check_end, tz)
                                    .await?;
                            if (qualifying_days as f64) < threshold {
                                amount = 0.0;
                            }
                        }

                        if amount > 0.0 {
                            use rust_decimal::prelude::FromPrimitive;
                            let decimal_amount =
                                rust_decimal::Decimal::from_f64(amount).unwrap_or_default();

                            // Check if a log already exists for this date/type/employee using FK
                            let existing_log = sqlx::query!(
                                r#"
                                SELECT id, amount FROM leave_accrual_logs
                                WHERE employee_id = $1 AND leave_type_id = $2 AND accrual_date = $3
                                "#,
                                emp.id,
                                leave_type_id,
                                current_check_date
                            )
                            .fetch_optional(pool)
                            .await?;

                            if let Some(old_log) = existing_log {
                                // If force is true and amount has changed, re-calculate
                                if force && old_log.amount != decimal_amount {
                                    let diff = decimal_amount - old_log.amount;

                                    // Update the log entry
                                    let updated_log = sqlx::query_as!(
                                        LeaveAccrualLog,
                                        r#"
                                        UPDATE leave_accrual_logs
                                        SET amount = $1, notes = COALESCE(notes, '') || ' (Re-calculated)'
                                        WHERE id = $2
                                        RETURNING id, workspace_id, employee_id, policy_assignment_id,
                                                  accrual_date, amount, leave_type_id, notes,
                                                  created_at as "created_at!"
                                        "#,
                                        decimal_amount,
                                        old_log.id
                                    )
                                    .fetch_one(pool)
                                    .await?;

                                    // Adjust the balance entry using FK
                                    sqlx::query!(
                                        r#"
                                        UPDATE leave_balances
                                        SET accrued_amount = accrued_amount + $3, updated_at = NOW()
                                        WHERE employee_id = $1 AND year = $2 AND leave_type_id = $4
                                        "#,
                                        emp.id,
                                        current_check_date.year(),
                                        diff,
                                        leave_type_id
                                    )
                                    .execute(pool)
                                    .await?;

                                    all_logs.push(updated_log);
                                }
                            } else {
                                // No log exists, perform insertion using FK
                                let log = sqlx::query_as!(
                                    LeaveAccrualLog,
                                    r#"
                                    INSERT INTO leave_accrual_logs
                                        (workspace_id, employee_id, policy_assignment_id, accrual_date, amount, leave_type_id)
                                    VALUES ($1, $2, $3, $4, $5, $6)
                                    ON CONFLICT (employee_id, leave_type_id, accrual_date) DO NOTHING
                                    RETURNING id, workspace_id, employee_id, policy_assignment_id,
                                              accrual_date, amount, leave_type_id, notes,
                                              created_at as "created_at!"
                                    "#,
                                    workspace_id,
                                    emp.id,
                                    Some(assignment_id),
                                    current_check_date,
                                    decimal_amount,
                                    leave_type_id
                                )
                                .fetch_optional(pool)
                                .await?;

                                if let Some(l) = log {
                                    // Update Balance using FK
                                    sqlx::query!(
                                        r#"
                                        INSERT INTO leave_balances (employee_id, year, leave_type_id, entitlement, accrued_amount, carried_over)
                                        VALUES ($1, $2, $3, 0, $4, 0)
                                        ON CONFLICT (employee_id, leave_type_id, year)
                                        DO UPDATE SET
                                            accrued_amount = leave_balances.accrued_amount + EXCLUDED.accrued_amount,
                                            updated_at = NOW()
                                        "#,
                                        emp.id,
                                        current_check_date.year(),
                                        leave_type_id,
                                        decimal_amount
                                    )
                                    .execute(pool)
                                    .await?;

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
            min_work_days_threshold: None,
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
