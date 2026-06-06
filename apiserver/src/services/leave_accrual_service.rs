use std::sync::Arc;

use crate::errors::AppError;
use crate::models::{AccrualFrequency, LeaveAccrualLog, LeavePolicyConfig};
use crate::repositories::department_repository::DepartmentRepository;
use crate::repositories::leave_accrual_log_repository::LeaveAccrualLogRepository;
use crate::repositories::leave_policy_repository::LeavePolicyRepository;
use crate::repositories::EmployeeRepository;
use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

/// Default qualifying work days when threshold checking is disabled.
/// This value ensures `calculate_accrual_amount` always passes the
/// threshold check when no policy threshold is configured.
const DEFAULT_QUALIFYING_DAYS_BYPASS: f64 = 20.0;

pub struct LeaveAccrualService {
    employee_repo: Arc<dyn EmployeeRepository>,
    department_repo: Arc<dyn DepartmentRepository>,
    leave_policy_repo: Arc<dyn LeavePolicyRepository>,
    accrual_log_repo: Arc<dyn LeaveAccrualLogRepository>,
    time_entry_repo: Arc<dyn crate::repositories::TimeEntryRepository>,
    leave_request_repo:
        Arc<dyn crate::repositories::leave_request_repository::LeaveRequestRepository>,
    #[allow(dead_code)]
    daily_summary_repo: Arc<dyn crate::repositories::AttendanceDailySummaryRepository>,
    contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
}

impl LeaveAccrualService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        employee_repo: Arc<dyn EmployeeRepository>,
        department_repo: Arc<dyn DepartmentRepository>,
        leave_policy_repo: Arc<dyn LeavePolicyRepository>,
        accrual_log_repo: Arc<dyn LeaveAccrualLogRepository>,
        time_entry_repo: Arc<dyn crate::repositories::TimeEntryRepository>,
        leave_request_repo: Arc<
            dyn crate::repositories::leave_request_repository::LeaveRequestRepository,
        >,
        daily_summary_repo: Arc<dyn crate::repositories::AttendanceDailySummaryRepository>,
        contract_repo: Arc<dyn crate::repositories::EmployeeContractRepository>,
    ) -> Self {
        Self {
            employee_repo,
            department_repo,
            leave_policy_repo,
            accrual_log_repo,
            time_entry_repo,
            leave_request_repo,
            daily_summary_repo,
            contract_repo,
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
        let limit = f64::from(carry_over_limit);
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
    ) -> Result<i32, AppError> {
        use std::collections::HashSet;
        let mut qualifying_days = HashSet::new();

        // Days with actual work entries
        let start_dt = local_to_utc(&tz, start_date, 0, 0, 0)?;
        let end_dt = local_to_utc(&tz, end_date, 23, 59, 59)?;

        let work_days = self
            .time_entry_repo
            .get_distinct_work_days(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                start_dt,
                end_dt,
                &tz.to_string(),
            )
            .await?;

        for date in work_days {
            qualifying_days.insert(date);
        }

        // Approved Leave days
        let leave_days = self
            .leave_request_repo
            .get_approved_leave_periods(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                start_date,
                end_date,
            )
            .await?;

        for (req_start, req_end) in leave_days {
            let overlap_start = std::cmp::max(req_start, start_date);
            let overlap_end = std::cmp::min(req_end, end_date);
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

    /// Helper to calculate full months of service across multiple contracts
    pub fn calculate_total_months_seniority(
        contracts: &[crate::models::contract::EmployeeContract],
        as_of: NaiveDate,
    ) -> i32 {
        use crate::models::enums::ContractStatus;

        let mut valid_intervals = Vec::new();

        for c in contracts {
            if c.status() == ContractStatus::Draft {
                continue;
            }

            if c.start_date() > as_of {
                continue;
            }

            let end_date = c.end_date().unwrap_or(as_of);
            let end_date = if end_date > as_of { as_of } else { end_date };

            if end_date >= c.start_date() {
                valid_intervals.push((c.start_date(), end_date));
            }
        }

        if valid_intervals.is_empty() {
            return 0;
        }

        valid_intervals.sort_by_key(|i| i.0);

        let mut total_days = 0;
        let mut current_start = valid_intervals[0].0;
        let mut current_end = valid_intervals[0].1;

        for interval in valid_intervals.into_iter().skip(1) {
            let threshold = current_end.succ_opt().unwrap_or(current_end);
            if interval.0 <= threshold {
                if interval.1 > current_end {
                    current_end = interval.1;
                }
            } else {
                total_days += (current_end - current_start).num_days() + 1;
                current_start = interval.0;
                current_end = interval.1;
            }
        }
        total_days += (current_end - current_start).num_days() + 1;

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
    ) -> Result<HashMap<NaiveDate, i32>, AppError> {
        let start_dt = local_to_utc(&tz, range_start, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt = local_to_utc(&tz, range_end, 23, 59, 59).map_err(AppError::Internal)?;

        let work_days = self
            .time_entry_repo
            .get_distinct_work_days(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                start_dt,
                end_dt,
                &tz.to_string(),
            )
            .await?;

        let mut result: HashMap<NaiveDate, i32> = HashMap::new();
        use chrono::Datelike;

        for date in work_days {
            let month_key = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
            *result.entry(month_key).or_insert(0) += 1;
        }

        let leave_days = self
            .leave_request_repo
            .get_approved_leave_periods(
                &crate::diesel_pool::TenantAuthToken::system_override(workspace_id),
                employee_id,
                range_start,
                range_end,
            )
            .await?;

        for (req_start, req_end) in leave_days {
            let overlap_start = std::cmp::max(req_start, range_start);
            let overlap_end = std::cmp::min(req_end, range_end);
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

    pub async fn batch_count_qualifying_days_for_workspace(
        &self,
        workspace_id: Uuid,
        employee_ids: &[Uuid],
        range_start: NaiveDate,
        range_end: NaiveDate,
        tz: Tz,
    ) -> Result<HashMap<Uuid, HashMap<NaiveDate, i32>>, AppError> {
        let start_dt = local_to_utc(&tz, range_start, 0, 0, 0).map_err(AppError::Internal)?;
        let end_dt = local_to_utc(&tz, range_end, 23, 59, 59).map_err(AppError::Internal)?;
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let work_entries = self
            .time_entry_repo
            .get_workspace_time_entries_batch(&auth_token, employee_ids, start_dt, end_dt)
            .await?;

        let _leave_days_map = self
            .leave_request_repo
            .get_batch_leave_days(&auth_token, employee_ids, range_start, range_end)
            .await?;

        let approved_leaves = self
            .leave_request_repo
            .get_approved_leave_dates(&auth_token, employee_ids, range_start, range_end)
            .await?;

        let mut result_map: HashMap<Uuid, HashMap<NaiveDate, i32>> = HashMap::new();
        use chrono::Datelike;

        for (emp_id, entries) in work_entries {
            let emp_map = result_map.entry(emp_id).or_default();
            for entry in entries {
                let date = entry.punch_in_time.with_timezone(&tz).date_naive();
                let month_key =
                    NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
                *emp_map.entry(month_key).or_insert(0) += 1;
            }
        }

        for (emp_id, date) in approved_leaves {
            let emp_map = result_map.entry(emp_id).or_default();
            let month_key = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
            *emp_map.entry(month_key).or_insert(0) += 1;
        }

        Ok(result_map)
    }

    /// Main orchestrator to process accruals for a workspace on a specific date.
    /// Optimized: uses pre-computed accrual dates and batched qualifying-days queries.
    /// All persistence is delegated to `accrual_log_repo` (Repository pattern).
    #[tracing::instrument(skip_all, err)]
    #[allow(clippy::too_many_lines)]
    pub async fn process_workspace_accruals(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
        force: bool,
    ) -> Result<Vec<LeaveAccrualLog>, AppError> {
        // 1. Setup & Workspace-level queries
        let tz_str = self
            .accrual_log_repo
            .get_workspace_timezone(workspace_id)
            .await?;
        let tz: Tz = crate::utils::tz::parse_workspace_tz(&tz_str);
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let employees = self.employee_repo.get_active_employees(&auth_token).await?;

        if employees.is_empty() {
            return Ok(vec![]);
        }
        let employee_ids: Vec<Uuid> = employees.iter().map(crate::models::Employee::id).collect();

        // 2. Batch fetch all necessary data
        let dept_map = self
            .department_repo
            .get_current_departments_batch(&auth_token, &employee_ids)
            .await?;

        let all_assignments = self
            .leave_policy_repo
            .find_all_active_assignments_for_workspace(workspace_id, date)
            .await?;

        let contract_map = self
            .contract_repo
            .list_contracts_batch(&auth_token, &employee_ids)
            .await?;

        // 3. Match policies and compute overall qualifying range
        let mut emp_policy_map = HashMap::new();
        let mut accrual_plan = HashMap::new();
        let mut global_qualifying_start = date;

        for emp in &employees {
            let dept_id = dept_map
                .get(&emp.id())
                .map(crate::models::EmployeeDepartmentHistory::department_id);

            let mut matched = all_assignments.iter().find(|(a, _, _)| {
                a.assignee_type() == "employee" && a.assignee_id() == Some(emp.id())
            });
            if matched.is_none() {
                if let Some(d_id) = dept_id {
                    matched = all_assignments.iter().find(|(a, _, _)| {
                        a.assignee_type() == "department" && a.assignee_id() == Some(d_id)
                    });
                }
            }
            if matched.is_none() {
                matched = all_assignments.iter().find(|(a, _, _)| {
                    a.assignee_type() == "workspace" && a.assignee_id() == Some(workspace_id)
                });
            }

            if let Some((assignment, def, _code)) = matched {
                let config = def.rule_config();
                let effective_from = assignment.effective_from();
                let dates = Self::get_accrual_dates(
                    &config.frequency,
                    &config.accrual_timing,
                    effective_from,
                    date,
                );

                if !dates.is_empty() {
                    emp_policy_map.insert(emp.id(), (def.clone(), assignment.id()));
                    accrual_plan.insert(emp.id(), dates);

                    let q_start = Self::compute_qualifying_range_start(config, effective_from);
                    if q_start < global_qualifying_start {
                        global_qualifying_start = q_start;
                    }
                }
            }
        }

        if accrual_plan.is_empty() {
            return Ok(vec![]);
        }

        // 4. Batch fetch qualifying days
        let qualifying_days_map = self
            .batch_count_qualifying_days_for_workspace(
                workspace_id,
                &employee_ids,
                global_qualifying_start,
                date,
                tz,
            )
            .await?;

        let mut batch_entries = Vec::new();

        // 5. Collect accrual entries in memory (no DB calls)
        for emp in employees {
            let Some(dates) = accrual_plan.get(&emp.id()) else {
                continue;
            };
            let Some((def, assignment_id)) = emp_policy_map.get(&emp.id()) else {
                tracing::error!("policy must exist for accrued employee {}", emp.id());
                continue;
            };
            let config = def.rule_config();
            let leave_type_id = def.leave_type_id();

            let empty_contracts = vec![];
            let contracts = contract_map.get(&emp.id()).unwrap_or(&empty_contracts);

            let empty_q_map = HashMap::new();
            let q_map = qualifying_days_map.get(&emp.id()).unwrap_or(&empty_q_map);

            for accrual_date in dates {
                let months_service =
                    Self::calculate_total_months_seniority(contracts, *accrual_date);
                let work_days = Self::resolve_qualifying_days(config, *accrual_date, q_map);
                let amount = Self::calculate_accrual_amount(config, months_service, work_days);

                if amount > 0.0 {
                    batch_entries.push(
                        crate::repositories::leave_accrual_log_repository::AccrualBatchEntry {
                            employee_id: emp.id(),
                            assignment_id: *assignment_id,
                            leave_type_id,
                            accrual_date: *accrual_date,
                            amount,
                        },
                    );
                }
            }
        }

        // 6. Single batch DB call — all entries persisted in one transaction
        let all_logs = self
            .accrual_log_repo
            .persist_accruals_batch(&auth_token, workspace_id, &batch_entries, force)
            .await?;

        Ok(all_logs)
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
            f64::from(
                check_month
                    .and_then(|m| qualifying_days_map.get(&m).copied())
                    .unwrap_or(0),
            )
        } else {
            DEFAULT_QUALIFYING_DAYS_BYPASS
        }
    }
}

use crate::utils::chrono_helpers::local_to_utc;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AccrualFrequency, LeavePolicyConfig, SeniorityBonus};
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn nd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    fn mock_contract(
        hire: NaiveDate,
        term: Option<NaiveDate>,
    ) -> crate::models::contract::EmployeeContract {
        crate::models::contract::EmployeeContract::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            None,
            crate::models::enums::ContractType::Indefinite,
            crate::models::enums::ContractStatus::Active,
            hire,
            term,
            None,
            None,
            chrono::Utc::now(),
            chrono::Utc::now(),
        )
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
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 0, 15.0) - 1.0).abs()
                < f64::EPSILON
        );

        // 0 months service, NOT enough work days
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 0, 10.0) - 0.0).abs()
                < f64::EPSILON
        );

        // 12 months service milestone
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 12, 15.0) - 1.5).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_carry_over_limit_logic() {
        assert!(
            (LeaveAccrualService::calculate_carry_over_amount(10.0, 5) - 5.0).abs() < f64::EPSILON
        );
        assert!(
            (LeaveAccrualService::calculate_carry_over_amount(3.0, 5) - 3.0).abs() < f64::EPSILON
        );
        assert!(
            (LeaveAccrualService::calculate_carry_over_amount(0.0, 5) - 0.0).abs() < f64::EPSILON
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
        let periods = vec![mock_contract(nd(2025, 1, 1), None)];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
        // 366 days (2025 not a leap year but 2025-01-01 to 2026-01-01 = 365 days +1 = 366 / 30 = 12)
        assert_eq!(months, 12);
    }

    #[test]
    fn test_seniority_terminated_before_as_of() {
        // Hired 2024-01-01, terminated 2024-06-30 → 181 days / 30 = 6 months
        let periods = vec![mock_contract(nd(2024, 1, 1), Some(nd(2024, 6, 30)))];
        let months =
            LeaveAccrualService::calculate_total_months_seniority(&periods, nd(2026, 1, 1));
        assert_eq!(months, 6);
    }

    #[test]
    fn test_seniority_future_hire_date_excluded() {
        // Hired in the future → 0 months
        let periods = vec![mock_contract(nd(2030, 1, 1), None)];
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
            mock_contract(nd(2020, 1, 1), Some(nd(2021, 1, 1))),
            mock_contract(nd(2022, 1, 1), None),
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
        assert!(
            (LeaveAccrualService::calculate_carry_over_amount(-5.0, 10) - 0.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_carry_over_zero_limit() {
        assert!(
            (LeaveAccrualService::calculate_carry_over_amount(10.0, 0) - 0.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_carry_over_exact_limit() {
        assert!(
            (LeaveAccrualService::calculate_carry_over_amount(5.0, 5) - 5.0).abs() < f64::EPSILON
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
        let periods = vec![mock_contract(nd(2026, 1, 1), None)];
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
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 0, 0.0) - 1.5).abs()
                < f64::EPSILON
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
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 36, 20.0) - 2.0).abs()
                < f64::EPSILON
        );

        // 18 months → 12 months milestone (0.5 bonus)
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 18, 20.0) - 1.5).abs()
                < f64::EPSILON
        );

        // 3 months → no milestone
        assert!(
            (LeaveAccrualService::calculate_accrual_amount(&config, 3, 20.0) - 1.0).abs()
                < f64::EPSILON
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

    #[tokio::test]
    async fn test_process_workspace_accruals_empty_employees() {
        use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
        use crate::repositories::department_repository::MockDepartmentRepository;
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
        use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
        use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
        use crate::repositories::time_entry_repository::MockTimeEntryRepository;
        use std::sync::Arc;

        let mut mock_emp_repo = MockEmployeeRepository::new();
        let mut mock_accrual_repo = MockLeaveAccrualLogRepository::new();

        mock_accrual_repo
            .expect_get_workspace_timezone()
            .returning(|_| Ok("UTC".to_string()));

        mock_emp_repo
            .expect_get_active_employees()
            .returning(|_| Ok(vec![]));

        let service = LeaveAccrualService {
            employee_repo: Arc::new(mock_emp_repo),
            department_repo: Arc::new(MockDepartmentRepository::new()),
            leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
            accrual_log_repo: Arc::new(mock_accrual_repo),
            time_entry_repo: Arc::new(MockTimeEntryRepository::new()),
            leave_request_repo: Arc::new(MockLeaveRequestRepository::new()),
            contract_repo: Arc::new(MockEmployeeContractRepository::new()),
            daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
        };

        let result = service
            .process_workspace_accruals(Uuid::new_v4(), nd(2026, 1, 1), false)
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_count_qualifying_days_success() {
        use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
        use crate::repositories::department_repository::MockDepartmentRepository;
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
        use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
        use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
        use crate::repositories::time_entry_repository::MockTimeEntryRepository;
        use std::sync::Arc;

        let mut mock_time_entry = MockTimeEntryRepository::new();
        let mut mock_leave_req = MockLeaveRequestRepository::new();

        mock_time_entry
            .expect_get_distinct_work_days()
            .returning(|_, _, _, _, _| Ok(vec![nd(2026, 1, 15), nd(2026, 1, 16)]));

        mock_leave_req
            .expect_get_approved_leave_periods()
            .returning(|_, _, _, _| {
                Box::pin(std::future::ready(Ok(vec![(
                    nd(2026, 1, 20),
                    nd(2026, 1, 21),
                )])))
            });

        let service = LeaveAccrualService {
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            department_repo: Arc::new(MockDepartmentRepository::new()),
            leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
            accrual_log_repo: Arc::new(MockLeaveAccrualLogRepository::new()),
            time_entry_repo: Arc::new(mock_time_entry),
            leave_request_repo: Arc::new(mock_leave_req),
            contract_repo: Arc::new(MockEmployeeContractRepository::new()),
            daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
        };

        let result = service
            .count_qualifying_days(
                Uuid::new_v4(),
                Uuid::new_v4(),
                nd(2026, 1, 1),
                nd(2026, 1, 31),
                "UTC".parse().unwrap(),
            )
            .await
            .unwrap();

        // 2 work days + 2 leave days = 4 qualifying days
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn test_batch_count_qualifying_days_success() {
        use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
        use crate::repositories::department_repository::MockDepartmentRepository;
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
        use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
        use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
        use crate::repositories::time_entry_repository::MockTimeEntryRepository;
        use std::sync::Arc;

        let mut mock_time_entry = MockTimeEntryRepository::new();
        let mut mock_leave_req = MockLeaveRequestRepository::new();

        mock_time_entry
            .expect_get_distinct_work_days()
            .returning(|_, _, _, _, _| Ok(vec![nd(2026, 1, 15), nd(2026, 2, 10)]));

        mock_leave_req
            .expect_get_approved_leave_periods()
            .returning(|_, _, _, _| {
                Box::pin(std::future::ready(Ok(vec![
                    (nd(2026, 1, 20), nd(2026, 1, 21)),
                    (nd(2026, 3, 5), nd(2026, 3, 5)),
                ])))
            });

        let service = LeaveAccrualService {
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            department_repo: Arc::new(MockDepartmentRepository::new()),
            leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
            accrual_log_repo: Arc::new(MockLeaveAccrualLogRepository::new()),
            time_entry_repo: Arc::new(mock_time_entry),
            leave_request_repo: Arc::new(mock_leave_req),
            contract_repo: Arc::new(MockEmployeeContractRepository::new()),
            daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
        };

        let result = service
            .batch_count_qualifying_days(
                Uuid::new_v4(),
                Uuid::new_v4(),
                nd(2026, 1, 1),
                nd(2026, 3, 31),
                "UTC".parse().unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(result.get(&nd(2026, 1, 1)), Some(&3)); // 1 work day + 2 leave days
        assert_eq!(result.get(&nd(2026, 2, 1)), Some(&1)); // 1 work day
        assert_eq!(result.get(&nd(2026, 3, 1)), Some(&1)); // 1 leave day
    }

    #[tokio::test]
    async fn test_batch_count_qualifying_days_for_workspace_success() {
        use crate::repositories::attendance_daily_summary_repository::MockAttendanceDailySummaryRepository;
        use crate::repositories::department_repository::MockDepartmentRepository;
        use crate::repositories::employee::MockEmployeeRepository;
        use crate::repositories::employee_contract_repository::MockEmployeeContractRepository;
        use crate::repositories::leave_accrual_log_repository::MockLeaveAccrualLogRepository;
        use crate::repositories::leave_policy_repository::MockLeavePolicyRepository;
        use crate::repositories::leave_request_repository::MockLeaveRequestRepository;
        use crate::repositories::time_entry_repository::MockTimeEntryRepository;
        use std::sync::Arc;

        let mut mock_time_entry = MockTimeEntryRepository::new();
        let mut mock_leave_req = MockLeaveRequestRepository::new();

        let emp1 = Uuid::new_v4();
        let emp2 = Uuid::new_v4();

        use chrono::TimeZone;
        mock_time_entry
            .expect_get_workspace_time_entries_batch()
            .returning(move |_, _, _, _| {
                let mut map = std::collections::HashMap::new();
                map.insert(
                    emp1,
                    vec![crate::models::TimeEntry {
                        id: Uuid::new_v4(),
                        employee_id: emp1,
                        workspace_id: Uuid::new_v4(),
                        punch_in_time: chrono::Utc.with_ymd_and_hms(2026, 1, 15, 8, 0, 0).unwrap(),
                        punch_out_time: None,
                        entry_type: "work".to_string(),
                        duration_minutes: None,
                        shift_id: None,
                        is_break_overtime: false,
                        note: None,
                        generation_metadata: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }],
                );
                Ok(map)
            });

        mock_leave_req
            .expect_get_batch_leave_days()
            .returning(|_, _, _, _| {
                Box::pin(std::future::ready(Ok(std::collections::HashMap::new())))
            });

        mock_leave_req
            .expect_get_approved_leave_dates()
            .returning(move |_, _, _, _| {
                Box::pin(std::future::ready(Ok(
                    std::collections::HashSet::from_iter(vec![(emp2, nd(2026, 1, 20))]),
                )))
            });

        let service = LeaveAccrualService {
            employee_repo: Arc::new(MockEmployeeRepository::new()),
            department_repo: Arc::new(MockDepartmentRepository::new()),
            leave_policy_repo: Arc::new(MockLeavePolicyRepository::new()),
            accrual_log_repo: Arc::new(MockLeaveAccrualLogRepository::new()),
            time_entry_repo: Arc::new(mock_time_entry),
            leave_request_repo: Arc::new(mock_leave_req),
            contract_repo: Arc::new(MockEmployeeContractRepository::new()),
            daily_summary_repo: Arc::new(MockAttendanceDailySummaryRepository::new()),
        };

        let result = service
            .batch_count_qualifying_days_for_workspace(
                Uuid::new_v4(),
                &[emp1, emp2],
                nd(2026, 1, 1),
                nd(2026, 1, 31),
                "UTC".parse().unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(result.get(&emp1).unwrap().get(&nd(2026, 1, 1)), Some(&1));
        assert_eq!(result.get(&emp2).unwrap().get(&nd(2026, 1, 1)), Some(&1));
    }
}
