use super::calculator::DEFAULT_QUALIFYING_DAYS_BYPASS;
use crate::errors::AppError;
use crate::models::LeavePolicyConfig;
use crate::utils::chrono_helpers::local_to_utc;
use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

impl super::LeaveAccrualService {
    fn qualifying_day_set_from_sources<I, J>(work_days: I, leave_days: J) -> BTreeSet<NaiveDate>
    where
        I: IntoIterator<Item = NaiveDate>,
        J: IntoIterator<Item = NaiveDate>,
    {
        work_days.into_iter().chain(leave_days).collect()
    }

    fn group_qualifying_days_by_month(
        qualifying_days: &BTreeSet<NaiveDate>,
    ) -> HashMap<NaiveDate, i32> {
        qualifying_days
            .iter()
            .fold(HashMap::new(), |mut result, date| {
                let month_key = *date - chrono::Duration::days(i64::from(date.day0()));
                *result.entry(month_key).or_insert(0) += 1;
                result
            })
    }

    async fn qualifying_day_set(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        range_start: NaiveDate,
        range_end: NaiveDate,
        tz: Tz,
    ) -> Result<BTreeSet<NaiveDate>, AppError> {
        let start_dt = local_to_utc(&tz, range_start, 0, 0, 0)?;
        let end_dt = local_to_utc(&tz, range_end, 23, 59, 59)?;
        let auth_token = crate::diesel_pool::TenantAuthToken::system_override(workspace_id);

        let work_days = self
            .time_entry_repo
            .get_distinct_work_days(&auth_token, employee_id, start_dt, end_dt, &tz.to_string())
            .await?;

        let leave_periods = self
            .leave_request_repo
            .get_approved_leave_periods(&auth_token, employee_id, range_start, range_end)
            .await?;

        let leave_days = leave_periods.into_iter().flat_map(|(req_start, req_end)| {
            let overlap_start = std::cmp::max(req_start, range_start);
            let overlap_end = std::cmp::min(req_end, range_end);
            std::iter::successors(Some(overlap_start), NaiveDate::succ_opt)
                .take_while(move |date| *date <= overlap_end)
        });

        Ok(Self::qualifying_day_set_from_sources(work_days, leave_days))
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
        let qualifying_days = self
            .qualifying_day_set(workspace_id, employee_id, start_date, end_date, tz)
            .await?;

        Ok(i32::try_from(qualifying_days.len()).unwrap_or(0))
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
        let qualifying_days = self
            .qualifying_day_set(workspace_id, employee_id, range_start, range_end, tz)
            .await?;

        Ok(Self::group_qualifying_days_by_month(&qualifying_days))
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

        let employees = self
            .employee_repo
            .get_employees_by_ids(employee_ids, &auth_token)
            .await?;

        let mut emp_to_employment = HashMap::new();
        let mut employment_ids = Vec::new();
        for emp in &employees {
            if let Some(employment_id) = emp.active_employment_id() {
                emp_to_employment.insert(emp.id(), employment_id);
                employment_ids.push(employment_id);
            }
        }

        let work_entries = self
            .time_entry_repo
            .get_workspace_time_entries_batch_by_employments(
                &auth_token,
                &employment_ids,
                start_dt,
                end_dt,
            )
            .await?;

        let approved_leaves = self
            .leave_request_repo
            .get_approved_leave_dates(&auth_token, employee_ids, range_start, range_end)
            .await?;

        let mut leave_days_by_employee: HashMap<Uuid, BTreeSet<NaiveDate>> = HashMap::new();
        for (emp_id, date) in approved_leaves {
            leave_days_by_employee
                .entry(emp_id)
                .or_default()
                .insert(date);
        }

        let mut result_map: HashMap<Uuid, HashMap<NaiveDate, i32>> = HashMap::new();
        for (emp_id, employment_id) in &emp_to_employment {
            let work_days = work_entries
                .get(employment_id)
                .into_iter()
                .flatten()
                .filter(|entry| {
                    entry.entry_type() == crate::services::pairing::types::EntryType::Work
                })
                .map(|entry| entry.punch_in_time().with_timezone(&tz).date_naive());
            let leave_days = leave_days_by_employee.remove(emp_id).unwrap_or_default();
            let qualifying_days = Self::qualifying_day_set_from_sources(work_days, leave_days);
            let month_counts = Self::group_qualifying_days_by_month(&qualifying_days);
            if !month_counts.is_empty() {
                result_map.insert(*emp_id, month_counts);
            }
        }

        Ok(result_map)
    }

    /// Compute the start of the qualifying-days query range.
    /// For start_of_period timing, we need to look back one month.
    pub(super) fn compute_qualifying_range_start(
        config: &LeavePolicyConfig,
        effective_from: NaiveDate,
    ) -> NaiveDate {
        use chrono::Datelike;
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
    pub(super) fn resolve_qualifying_days(
        config: &LeavePolicyConfig,
        accrual_date: NaiveDate,
        qualifying_days_map: &HashMap<NaiveDate, i32>,
    ) -> f64 {
        use chrono::Datelike;
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
