use std::sync::Arc;

use crate::models::{AccrualFrequency, LeaveAccrualLog, LeavePolicyConfig};
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository;
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::repositories::EmployeeRepository;
use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

/// Default qualifying work days when threshold checking is disabled.
/// This value ensures `calculate_accrual_amount` always passes the
/// threshold check when no policy threshold is configured.
const DEFAULT_QUALIFYING_DAYS_BYPASS: f64 = 20.0;

/// Context IDs that flow together through the accrual pipeline.
struct AccrualContext {
    workspace_id: Uuid,
    employee_id: Uuid,
    assignment_id: Uuid,
    leave_type_id: Uuid,
}

pub struct LeaveAccrualService {
    employee_repo: Arc<dyn EmployeeRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    leave_policy_repo: Arc<dyn LeavePolicyRepository>,
    accrual_log_repo: Arc<dyn LeaveAccrualLogRepository>,
    pool: PgPool,
}

impl LeaveAccrualService {
    pub fn new(
        employee_repo: Arc<dyn EmployeeRepository>,
        department_repo: Arc<dyn DepartmentRepository>,
        leave_policy_repo: Arc<dyn LeavePolicyRepository>,
        accrual_log_repo: Arc<dyn LeaveAccrualLogRepository>,
        pool: PgPool,
    ) -> Self {
        Self {
            employee_repo,
            department_repo,
            leave_policy_repo,
            accrual_log_repo,
            pool,
        }
    }

    /// Pure function to calculate how much leave should be accrued for a given policy and employee state.
    pub fn calculate_accrual_amount(
        config: &LeavePolicyConfig,
        months_service: i32,
        work_days_in_period: f64,
    ) -> f64 {
        // Check working days threshold if configured
        if let Some(threshold) = config.min_work_days_threshold {
            if work_days_in_period < threshold {
                return 0.0;
            }
        }

        let mut amount = config.accrual_amount;

        // Apply seniority bonus (highest milestone reached)
        let mut bonus_to_apply = 0.0;
        let mut max_milestone_months = -1;

        for bonus in &config.seniority_bonus {
            if months_service >= bonus.months && bonus.months > max_milestone_months {
                bonus_to_apply = bonus.bonus_amount;
                max_milestone_months = bonus.months;
            }
        }
        amount += bonus_to_apply;

        amount
    }

    /// Calculate how many days can be carried over to the next year
    pub fn calculate_carry_over_amount(remaining_days: f64, carry_over_limit: i32) -> f64 {
        let limit = carry_over_limit as f64;
        if remaining_days > limit {
            limit
        } else if remaining_days < 0.0 {
            0.0
        } else {
            remaining_days
        }
    }

    /// Helper to count "Qualifying Days" for performance-based accrual.
    #[tracing::instrument(skip_all, err)]
    pub async fn count_qualifying_days(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        tz: Tz,
    ) -> Result<i32, sqlx::Error> {
        let pool = &self.pool;
        use std::collections::HashSet;
        let mut qualifying_days = HashSet::new();

        // Days with actual work entries
        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)?;

        let work_days = sqlx::query!(
            r#"
            SELECT DISTINCT(te.punch_in_time AT TIME ZONE $2)::date as "date!"
            FROM time_entries te
            JOIN employees e ON te.employee_id = e.id
            WHERE te.employee_id = $1
              AND e.workspace_id = $5
              AND te.punch_in_time >= $3
              AND te.punch_in_time <= $4
              AND te.entry_type = 'work'
            "#,
            employee_id,
            tz.to_string(),
            start_dt,
            end_dt,
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        for row in work_days {
            qualifying_days.insert(row.date);
        }

        // Approved Leave days
        let leave_days = sqlx::query!(
            r#"
            SELECT lr.start_date, lr.end_date
            FROM leave_requests lr
            JOIN employees e ON lr.employee_id = e.id
            WHERE lr.employee_id = $1
              AND e.workspace_id = $4
              AND lr.status IN ('approved', 'pre_approved')
              AND lr.start_date <= $3
              AND lr.end_date >= $2
            "#,
            employee_id,
            start_date,
            end_date,
            workspace_id
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

    /// Helper to calculate full months of service across multiple periods
    pub fn calculate_total_months_seniority(
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

        (total_days / 30) as i32 // Approximate month
    }

    /// Determines if an accrual should run on this date based on frequency.
    pub fn should_run_accrual(frequency: &AccrualFrequency, timing: &str, date: NaiveDate) -> bool {
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

    /// Pre-compute all accrual dates in a range instead of iterating day-by-day.
    /// For Monthly start_of_period: returns 1st of each month in [from, to].
    /// For Monthly end_of_period: returns last day of each month in [from, to].
    /// For Quarterly/Yearly: returns corresponding period boundaries.
    pub fn get_accrual_dates(
        frequency: &AccrualFrequency,
        timing: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Vec<NaiveDate> {
        let mut dates = Vec::new();
        let is_start = timing == "start_of_period";

        match frequency {
            AccrualFrequency::Monthly => {
                let mut y = from.year();
                let mut m = from.month();
                if is_start {
                    // First day of each month
                    // If from is after the 1st, start from 1st of from's month only if it's >= from
                    while let Some(d) = NaiveDate::from_ymd_opt(y, m, 1) {
                        if d > to {
                            break;
                        }
                        if d >= from {
                            dates.push(d);
                        }
                        // Next month
                        if m == 12 {
                            m = 1;
                            y += 1;
                        } else {
                            m += 1;
                        }
                    }
                } else {
                    // Last day of each month
                    loop {
                        let next_m = if m == 12 { 1 } else { m + 1 };
                        let next_y = if m == 12 { y + 1 } else { y };
                        if let Some(first_next) = NaiveDate::from_ymd_opt(next_y, next_m, 1) {
                            if let Some(last_day) = first_next.pred_opt() {
                                if last_day > to {
                                    break;
                                }
                                if last_day >= from {
                                    dates.push(last_day);
                                }
                            }
                        }
                        m = next_m;
                        y = next_y;
                        if NaiveDate::from_ymd_opt(y, m, 1).is_none_or(|d| d > to) {
                            break;
                        }
                    }
                }
            }
            AccrualFrequency::Quarterly => {
                let quarter_months: &[u32] = if is_start {
                    &[1, 4, 7, 10]
                } else {
                    &[3, 6, 9, 12]
                };
                let mut y = from.year();
                loop {
                    for &qm in quarter_months {
                        let d = if is_start {
                            NaiveDate::from_ymd_opt(y, qm, 1)
                        } else {
                            // Last day of quarter month
                            let next_m = if qm == 12 { 1 } else { qm + 1 };
                            let next_y = if qm == 12 { y + 1 } else { y };
                            NaiveDate::from_ymd_opt(next_y, next_m, 1).and_then(|d| d.pred_opt())
                        };
                        if let Some(d) = d {
                            if d > to {
                                return dates;
                            }
                            if d >= from {
                                dates.push(d);
                            }
                        }
                    }
                    y += 1;
                    if y > to.year() + 1 {
                        break;
                    }
                }
            }
            AccrualFrequency::Yearly => {
                let mut y = from.year();
                loop {
                    let d = if is_start {
                        NaiveDate::from_ymd_opt(y, 1, 1)
                    } else {
                        NaiveDate::from_ymd_opt(y, 12, 31)
                    };
                    if let Some(d) = d {
                        if d > to {
                            break;
                        }
                        if d >= from {
                            dates.push(d);
                        }
                    }
                    y += 1;
                    if y > to.year() + 1 {
                        break;
                    }
                }
            }
        }
        dates
    }

    /// Batch-count qualifying work days per month for an employee in a single SQL query.
    /// Returns a HashMap<NaiveDate, i32> where key = first of month, value = qualifying days.
    #[tracing::instrument(skip_all, err)]
    pub async fn batch_count_qualifying_days(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        range_start: NaiveDate,
        range_end: NaiveDate,
        tz: Tz,
    ) -> Result<HashMap<NaiveDate, i32>, sqlx::Error> {
        let pool = &self.pool;
        let start_dt = local_to_utc(&tz, range_start, 0, 0, 0)?;
        let end_dt = local_to_utc(&tz, range_end, 23, 59, 59)?;

        // Count distinct work days per month from time_entries
        let work_rows = sqlx::query!(
            r#"
            SELECT date_trunc('month', (te.punch_in_time AT TIME ZONE $2))::date as "month!",
                   COUNT(DISTINCT (te.punch_in_time AT TIME ZONE $2)::date) as "count!"
            FROM time_entries te
            JOIN employees e ON te.employee_id = e.id
            WHERE te.employee_id = $1
              AND e.workspace_id = $5
              AND te.punch_in_time >= $3
              AND te.punch_in_time <= $4
              AND te.entry_type = 'work'
            GROUP BY 1
            "#,
            employee_id,
            tz.to_string(),
            start_dt,
            end_dt,
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        let mut result: HashMap<NaiveDate, i32> = HashMap::new();
        for row in work_rows {
            *result.entry(row.month).or_insert(0) += row.count as i32;
        }

        // Also count approved leave days per month
        let leave_rows = sqlx::query!(
            r#"
            SELECT lr.start_date, lr.end_date
            FROM leave_requests lr
            JOIN employees e ON lr.employee_id = e.id
            WHERE lr.employee_id = $1
              AND e.workspace_id = $4
              AND lr.status IN ('approved', 'pre_approved')
              AND lr.start_date <= $3
              AND lr.end_date >= $2
            "#,
            employee_id,
            range_start,
            range_end,
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        for req in leave_rows {
            let overlap_start = std::cmp::max(req.start_date, range_start);
            let overlap_end = std::cmp::min(req.end_date, range_end);
            let mut curr = overlap_start;
            while curr <= overlap_end {
                let month_key =
                    NaiveDate::from_ymd_opt(curr.year(), curr.month(), 1).unwrap_or(curr);
                *result.entry(month_key).or_insert(0) += 1;
                if let Some(next) = curr.succ_opt() {
                    curr = next;
                } else {
                    break;
                }
            }
        }

        Ok(result)
    }

    /// Main orchestrator to process accruals for a workspace on a specific date.
    /// Optimized: uses pre-computed accrual dates and batched qualifying-days queries.
    /// All persistence is delegated to `accrual_log_repo` (Repository pattern).
    #[tracing::instrument(skip_all, err)]
    pub async fn process_workspace_accruals(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
        force: bool,
    ) -> Result<Vec<LeaveAccrualLog>, sqlx::Error> {
        // Get workspace timezone via repository
        let tz_str = self
            .accrual_log_repo
            .get_workspace_timezone(workspace_id)
            .await?;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(&tz_str);

        // Get all active employees
        let employees = self
            .employee_repo
            .get_active_employees(workspace_id)
            .await?;
        let mut all_logs = Vec::new();

        for emp in employees {
            self.process_employee_accruals(workspace_id, &emp, date, force, tz, &mut all_logs)
                .await?;
        }

        Ok(all_logs)
    }

    /// Process accruals for a single employee. Extracted to reduce orchestrator complexity.
    async fn process_employee_accruals(
        &self,
        workspace_id: Uuid,
        emp: &crate::models::Employee,
        date: NaiveDate,
        force: bool,
        tz: Tz,
        all_logs: &mut Vec<LeaveAccrualLog>,
    ) -> Result<(), sqlx::Error> {
        let dept_hist = self
            .department_repo
            .get_current_department(workspace_id, emp.id)
            .await
            .ok()
            .flatten();

        let Some((policy, assignment_id, _leave_type_code)) = self
            .leave_policy_repo
            .find_active_assignment_for_employee(
                workspace_id,
                emp.id,
                dept_hist.map(|d| d.department_id),
                date,
            )
            .await?
        else {
            return Ok(());
        };

        let config = &policy.rule_config.0;
        let leave_type_id = policy.leave_type_id;

        // Fetch effective_from via repository
        let effective_from = self
            .accrual_log_repo
            .get_assignment_effective_from(assignment_id, workspace_id)
            .await?;

        // Pre-compute accrual dates instead of day-by-day iteration
        let accrual_dates = Self::get_accrual_dates(
            &config.frequency,
            &config.accrual_timing,
            effective_from,
            date,
        );

        if accrual_dates.is_empty() {
            return Ok(());
        }

        // Batch-fetch qualifying days for all months at once
        let qualifying_range_start = Self::compute_qualifying_range_start(config, effective_from);

        let qualifying_days_map = if config.min_work_days_threshold.is_some() {
            self.batch_count_qualifying_days(workspace_id, emp.id, qualifying_range_start, date, tz)
                .await?
        } else {
            HashMap::new()
        };

        // Pre-fetch employment history once per employee
        let periods = self
            .employee_repo
            .get_employment_history(emp.workspace_id, emp.id)
            .await?;

        let ctx = AccrualContext {
            workspace_id,
            employee_id: emp.id,
            assignment_id,
            leave_type_id,
        };

        for accrual_date in accrual_dates {
            let months_service = Self::calculate_total_months_seniority(&periods, accrual_date);
            let work_days =
                Self::resolve_qualifying_days(config, accrual_date, &qualifying_days_map);
            let amount = Self::calculate_accrual_amount(config, months_service, work_days);

            if amount > 0.0 {
                self.persist_accrual(&ctx, accrual_date, amount, force, all_logs)
                    .await?;
            }
        }

        Ok(())
    }

    /// Compute the start of the qualifying-days query range.
    /// For start_of_period timing, we need to look back one month.
    fn compute_qualifying_range_start(
        config: &LeavePolicyConfig,
        effective_from: NaiveDate,
    ) -> NaiveDate {
        if config.min_work_days_threshold.is_some() && config.accrual_timing == "start_of_period" {
            if effective_from.month() == 1 {
                NaiveDate::from_ymd_opt(effective_from.year() - 1, 12, 1).unwrap_or(effective_from)
            } else {
                NaiveDate::from_ymd_opt(effective_from.year(), effective_from.month() - 1, 1)
                    .unwrap_or(effective_from)
            }
        } else {
            effective_from
        }
    }

    /// Look up qualifying days from the batch result for a specific accrual date.
    fn resolve_qualifying_days(
        config: &LeavePolicyConfig,
        accrual_date: NaiveDate,
        qualifying_days_map: &HashMap<NaiveDate, i32>,
    ) -> f64 {
        if config.min_work_days_threshold.is_some() {
            let check_month = if config.accrual_timing == "start_of_period" {
                if accrual_date.month() == 1 {
                    NaiveDate::from_ymd_opt(accrual_date.year() - 1, 12, 1)
                } else {
                    NaiveDate::from_ymd_opt(accrual_date.year(), accrual_date.month() - 1, 1)
                }
            } else {
                NaiveDate::from_ymd_opt(accrual_date.year(), accrual_date.month(), 1)
            };
            check_month
                .and_then(|m| qualifying_days_map.get(&m).copied())
                .unwrap_or(0) as f64
        } else {
            DEFAULT_QUALIFYING_DAYS_BYPASS
        }
    }

    /// Persist an accrual log and update the leave balance via the repository.
    async fn persist_accrual(
        &self,
        ctx: &AccrualContext,
        accrual_date: NaiveDate,
        amount: f64,
        force: bool,
        all_logs: &mut Vec<LeaveAccrualLog>,
    ) -> Result<(), sqlx::Error> {
        use rust_decimal::prelude::FromPrimitive;
        let decimal_amount = rust_decimal::Decimal::from_f64(amount).ok_or_else(|| {
            tracing::error!(amount, "Invalid f64 for leave accrual Decimal conversion");
            sqlx::Error::Protocol(format!("Cannot convert {amount} to Decimal"))
        })?;

        let existing = self
            .accrual_log_repo
            .find_by_employee_date_type(
                ctx.employee_id,
                ctx.leave_type_id,
                accrual_date,
                ctx.workspace_id,
            )
            .await?;

        if let Some(old_log) = existing {
            // If force is true and amount has changed, re-calculate
            if force && old_log.amount != decimal_amount {
                let diff = decimal_amount - old_log.amount;

                let mut tx = self.pool.begin().await?;

                let updated_log =
                    crate::repositories::leave_accrual_log_repository::update_accrual_amount(
                        &mut *tx,
                        old_log.id,
                        ctx.workspace_id,
                        decimal_amount,
                    )
                    .await?;

                crate::repositories::leave_accrual_log_repository::adjust_balance_accrual(
                    &mut *tx,
                    ctx.employee_id,
                    accrual_date.year(),
                    ctx.leave_type_id,
                    diff,
                )
                .await?;

                tx.commit().await?;

                all_logs.push(updated_log);
            }
        } else {
            // No log exists, perform insertion
            let mut tx = self.pool.begin().await?;

            let log = crate::repositories::leave_accrual_log_repository::insert_accrual_log(
                &mut *tx,
                ctx.workspace_id,
                ctx.employee_id,
                Some(ctx.assignment_id),
                accrual_date,
                decimal_amount,
                ctx.leave_type_id,
            )
            .await?;

            if let Some(l) = log {
                crate::repositories::leave_accrual_log_repository::upsert_balance_accrual(
                    &mut *tx,
                    ctx.employee_id,
                    accrual_date.year(),
                    ctx.leave_type_id,
                    decimal_amount,
                )
                .await?;

                tx.commit().await?;

                all_logs.push(l);
            } else {
                tx.commit().await?;
            }
        }

        Ok(())
    }
}

use crate::utils::chrono_helpers::local_to_utc_sqlx as local_to_utc;

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;
    use crate::models::{AccrualFrequency, EmploymentPeriod, LeavePolicyConfig, SeniorityBonus};
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    fn mock_period(hire: NaiveDate, term: Option<NaiveDate>) -> EmploymentPeriod {
        EmploymentPeriod {
            id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            hire_date: hire,
            termination_date: term,
            is_rehire: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_calculate_accrual_amount() {
        let config = LeavePolicyConfig {
            frequency: AccrualFrequency::Monthly,
            accrual_amount: 1.0,
            accrual_timing: "start_of_period".to_string(),
            seniority_bonus: vec![
                SeniorityBonus {
                    months: 12,
                    bonus_amount: 0.5,
                },
                SeniorityBonus {
                    months: 60,
                    bonus_amount: 1.0,
                },
            ],
            carry_over_limit: Some(5),
            max_balance: None,
            reset_month: None,
            min_work_days_threshold: Some(14.0),
        };

        // 0 months service, enough work days
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 0, 15.0),
            1.0
        );

        // 0 months service, NOT enough work days
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 0, 10.0),
            0.0
        );

        // 12 months service milestone
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 12, 15.0),
            1.5
        );
    }

    #[test]
    fn test_carry_over_limit_logic() {
        assert_eq!(
            LeaveAccrualService::calculate_carry_over_amount(10.0, 5),
            5.0
        );
        assert_eq!(
            LeaveAccrualService::calculate_carry_over_amount(3.0, 5),
            3.0
        );
        assert_eq!(
            LeaveAccrualService::calculate_carry_over_amount(0.0, 5),
            0.0
        );
    }

    // -------------------------------------------------------------------------
    // should_run_accrual
    // -------------------------------------------------------------------------

    #[test]
    fn test_should_run_accrual_monthly_start() {
        // Monthly start_of_period → day == 1
        assert!(LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Monthly,
            "start_of_period",
            nd(2026, 3, 1)
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Monthly,
            "start_of_period",
            nd(2026, 3, 15)
        ));
    }

    #[test]
    fn test_should_run_accrual_monthly_end() {
        // Monthly end_of_period → last day of month (next day is a different month)
        assert!(LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Monthly,
            "end_of_period",
            nd(2026, 3, 31) // last day of March
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Monthly,
            "end_of_period",
            nd(2026, 3, 30)
        ));
        // February non-leap year
        assert!(LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Monthly,
            "end_of_period",
            nd(2026, 2, 28)
        ));
    }

    #[test]
    fn test_should_run_accrual_quarterly_start() {
        // Quarter starts: Jan 1, Apr 1, Jul 1, Oct 1
        for &(m, d, expected) in &[
            (1u32, 1u32, true),
            (4, 1, true),
            (7, 1, true),
            (10, 1, true),
            (2, 1, false),
            (1, 5, false),
        ] {
            assert_eq!(
                LeaveAccrualService::should_run_accrual(
                    &AccrualFrequency::Quarterly,
                    "start_of_period",
                    nd(2026, m, d)
                ),
                expected,
                "Quarterly start m={m} d={d}"
            );
        }
    }

    #[test]
    fn test_should_run_accrual_yearly() {
        // Yearly start → Jan 1
        assert!(LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Yearly,
            "start_of_period",
            nd(2026, 1, 1)
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Yearly,
            "start_of_period",
            nd(2026, 6, 1)
        ));
        // Yearly end → Dec 31
        assert!(LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Yearly,
            "end_of_period",
            nd(2026, 12, 31)
        ));
        assert!(!LeaveAccrualService::should_run_accrual(
            &AccrualFrequency::Yearly,
            "end_of_period",
            nd(2026, 12, 30)
        ));
    }

    // -------------------------------------------------------------------------
    // calculate_total_months_seniority
    // -------------------------------------------------------------------------

    #[test]
    fn test_seniority_single_period_no_termination() {
        // Hired 2025-01-01, as_of 2026-01-01 → ~12 months (365 days / 30 ≈ 12)
        let periods = vec![mock_period(nd(2025, 1, 1), None)];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
        // 366 days (2025 not a leap year but 2025-01-01 to 2026-01-01 = 365 days +1 = 366 / 30 = 12)
        assert_eq!(months, 12);
    }

    #[test]
    fn test_seniority_terminated_before_as_of() {
        // Hired 2024-01-01, terminated 2024-06-30 → 181 days / 30 = 6 months
        let periods = vec![mock_period(nd(2024, 1, 1), Some(nd(2024, 6, 30)))];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
        assert_eq!(months, 6);
    }

    #[test]
    fn test_seniority_future_hire_date_excluded() {
        // Hired in the future → 0 months
        let periods = vec![mock_period(nd(2030, 1, 1), None)];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
        assert_eq!(months, 0);
    }

    #[test]
    fn test_seniority_rehire_gap_accumulates() {
        // Period 1: 2020-01-01 → 2021-01-01 (12 months)
        // Period 2: 2022-01-01 → unspecified, as_of = 2022-07-01 (6 months)
        // Total ≈ 18 months
        let periods = vec![
            mock_period(nd(2020, 1, 1), Some(nd(2021, 1, 1))),
            mock_period(nd(2022, 1, 1), None),
        ];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2022, 7, 1));
        // Phase 1: 366+1 = 367 days ≈ 12 months
        // Phase 2: 181 days ≈ 6 months
        assert!(
            (17..=18).contains(&months),
            "Expected 17-18 months, got {months}"
        );
    }

    // -------------------------------------------------------------------------
    // get_accrual_dates
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_accrual_dates_monthly_start() {
        // Monthly start_of_period: 1st of each month
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Monthly,
            "start_of_period",
            nd(2024, 1, 1),
            nd(2026, 2, 28),
        );
        // Jan 2024 through Feb 2026 = 26 months
        assert_eq!(dates.len(), 26);
        assert_eq!(dates[0], nd(2024, 1, 1));
        assert_eq!(dates[25], nd(2026, 2, 1));
    }

    #[test]
    fn test_get_accrual_dates_monthly_start_mid_month_from() {
        // When from is mid-month, skip that month's 1st (it's before from)
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Monthly,
            "start_of_period",
            nd(2026, 1, 15),
            nd(2026, 3, 31),
        );
        assert_eq!(dates.len(), 2); // Feb 1, Mar 1
        assert_eq!(dates[0], nd(2026, 2, 1));
        assert_eq!(dates[1], nd(2026, 3, 1));
    }

    #[test]
    fn test_get_accrual_dates_monthly_end() {
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Monthly,
            "end_of_period",
            nd(2026, 1, 1),
            nd(2026, 3, 31),
        );
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], nd(2026, 1, 31));
        assert_eq!(dates[1], nd(2026, 2, 28)); // 2026 is not leap year
        assert_eq!(dates[2], nd(2026, 3, 31));
    }

    #[test]
    fn test_get_accrual_dates_quarterly_start() {
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Quarterly,
            "start_of_period",
            nd(2025, 1, 1),
            nd(2026, 6, 30),
        );
        // Q1-Q4 2025 + Q1-Q2 2026 = 6
        assert_eq!(dates.len(), 6);
        assert_eq!(dates[0], nd(2025, 1, 1));
        assert_eq!(dates[3], nd(2025, 10, 1));
        assert_eq!(dates[5], nd(2026, 4, 1));
    }

    #[test]
    fn test_get_accrual_dates_yearly_start() {
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Yearly,
            "start_of_period",
            nd(2024, 1, 1),
            nd(2026, 6, 15),
        );
        assert_eq!(dates.len(), 3);
        assert_eq!(dates[0], nd(2024, 1, 1));
        assert_eq!(dates[1], nd(2025, 1, 1));
        assert_eq!(dates[2], nd(2026, 1, 1));
    }

    #[test]
    fn test_get_accrual_dates_yearly_end() {
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Yearly,
            "end_of_period",
            nd(2024, 1, 1),
            nd(2026, 6, 15),
        );
        assert_eq!(dates.len(), 2); // Dec 31 2024, Dec 31 2025
        assert_eq!(dates[0], nd(2024, 12, 31));
        assert_eq!(dates[1], nd(2025, 12, 31));
    }

    #[test]
    fn test_get_accrual_dates_empty_range() {
        // To < first possible accrual date
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Monthly,
            "start_of_period",
            nd(2026, 3, 15),
            nd(2026, 3, 20),
        );
        assert!(dates.is_empty());
    }

    // =====================================================================
    // Additional edge case tests
    // =====================================================================

    #[test]
    fn test_carry_over_negative_remaining() {
        assert_eq!(
            LeaveAccrualService::calculate_carry_over_amount(-5.0, 10),
            0.0
        );
    }

    #[test]
    fn test_carry_over_zero_limit() {
        assert_eq!(
            LeaveAccrualService::calculate_carry_over_amount(10.0, 0),
            0.0
        );
    }

    #[test]
    fn test_carry_over_exact_limit() {
        assert_eq!(
            LeaveAccrualService::calculate_carry_over_amount(5.0, 5),
            5.0
        );
    }

    #[test]
    fn test_seniority_empty_periods() {
        let months = LeaveAccrualService::calculate_total_months_seniority(&[], nd(2026, 1, 1));
        assert_eq!(months, 0);
    }

    #[test]
    fn test_seniority_same_day_hire_and_as_of() {
        // Hired today, as_of today → 1 day / 30 = 0 months
        let periods = vec![mock_period(nd(2026, 1, 1), None)];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
        assert_eq!(months, 0); // 1 day / 30 = 0
    }

    #[test]
    fn test_accrual_amount_no_threshold() {
        let config = LeavePolicyConfig {
            frequency: AccrualFrequency::Monthly,
            accrual_amount: 1.5,
            accrual_timing: "start_of_period".to_string(),
            seniority_bonus: vec![],
            carry_over_limit: None,
            max_balance: None,
            reset_month: None,
            min_work_days_threshold: None,
        };

        // No threshold → work_days_in_period doesn't matter
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 0, 0.0),
            1.5
        );
    }

    #[test]
    fn test_accrual_amount_highest_seniority_wins() {
        let config = LeavePolicyConfig {
            frequency: AccrualFrequency::Monthly,
            accrual_amount: 1.0,
            accrual_timing: "start_of_period".to_string(),
            seniority_bonus: vec![
                SeniorityBonus {
                    months: 6,
                    bonus_amount: 0.25,
                },
                SeniorityBonus {
                    months: 12,
                    bonus_amount: 0.5,
                },
                SeniorityBonus {
                    months: 24,
                    bonus_amount: 1.0,
                },
            ],
            carry_over_limit: None,
            max_balance: None,
            reset_month: None,
            min_work_days_threshold: None,
        };

        // 36 months > all milestones → highest (24 months → 1.0 bonus)
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 36, 20.0),
            2.0 // 1.0 base + 1.0 bonus
        );

        // 18 months → 12 months milestone (0.5 bonus)
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 18, 20.0),
            1.5
        );

        // 3 months → no milestone
        assert_eq!(
            LeaveAccrualService::calculate_accrual_amount(&config, 3, 20.0),
            1.0
        );
    }

    #[test]
    fn test_get_accrual_dates_quarterly_end() {
        let dates = LeaveAccrualService::get_accrual_dates(
            &AccrualFrequency::Quarterly,
            "end_of_period",
            nd(2026, 1, 1),
            nd(2026, 12, 31),
        );
        assert_eq!(dates.len(), 4);
        assert_eq!(dates[0], nd(2026, 3, 31));
        assert_eq!(dates[1], nd(2026, 6, 30));
        assert_eq!(dates[2], nd(2026, 9, 30));
        assert_eq!(dates[3], nd(2026, 12, 31));
    }
}
