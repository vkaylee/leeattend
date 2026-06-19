use super::calculator::DEFAULT_QUALIFYING_DAYS_BYPASS;
use crate::errors::AppError;
use crate::models::LeavePolicyConfig;
use crate::utils::chrono_helpers::local_to_utc;
use chrono::NaiveDate;
use chrono_tz::Tz;
use std::collections::HashMap;
use uuid::Uuid;

impl super::LeaveAccrualService {
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

        for (emp_id, employment_id) in &emp_to_employment {
            if let Some(entries) = work_entries.get(employment_id) {
                let emp_map = result_map.entry(*emp_id).or_default();
                for entry in entries {
                    let date = entry.punch_in_time().with_timezone(&tz).date_naive();
                    let month_key =
                        NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
                    *emp_map.entry(month_key).or_insert(0) += 1;
                }
            }
        }

        for (emp_id, date) in approved_leaves {
            let emp_map = result_map.entry(emp_id).or_default();
            let month_key = NaiveDate::from_ymd_opt(date.year(), date.month(), 1).unwrap_or(date);
            *emp_map.entry(month_key).or_insert(0) += 1;
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
