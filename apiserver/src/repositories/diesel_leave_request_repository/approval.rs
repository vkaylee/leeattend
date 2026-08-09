use super::*;
use crate::diesel_pool::DieselPoolsTenantTxExt;
use crate::errors::AppError;
use crate::models::LeaveStatus;
use crate::repositories::leave_request_repository::LeaveBreakdown;
use crate::schema::*;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

impl DieselLeaveRequestRepository {
    pub(super) async fn get_approved_leave_breakdown_impl(
        &self,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<LeaveBreakdown, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {

        // Step 1: Get raw leave days per type
        let rows = leave_requests::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::employee_id.eq(employee_id))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.le(end_date))
            .filter(leave_requests::end_date.ge(start_date))
            .group_by(leave_types::code)
            .select((
                leave_types::code,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
                    "SUM(LEAST(leave_requests.end_date, '{end_date}') - GREATEST(leave_requests.start_date, '{start_date}') + 1)"
                )),
            ))
            .load::<(String, i64)>(conn)
            .await
            .map_err(AppError::from)?;

        // Step 2: Count paid holidays in the period to subtract from total
        let paid_holiday_count: i64 = holidays::table
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::is_paid.eq(true))
            .filter(holidays::date.ge(start_date))
            .filter(holidays::date.le(end_date))
            .filter(diesel::dsl::exists(
                leave_requests::table
                    .inner_join(employments::table)
                    .filter(employments::employee_id.eq(employee_id))
                    .filter(employments::workspace_id.eq(workspace_id))
                    .filter(
                        leave_requests::status
                            .eq(LeaveStatus::Approved)
                            .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                    )
                    .filter(leave_requests::start_date.le(holidays::date))
                    .filter(leave_requests::end_date.ge(holidays::date)),
            ))
            .count()
            .get_result::<i64>(conn)
            .await
            .map_err(AppError::from)?;

        let mut breakdown = LeaveBreakdown {
            total_days: 0.0,
            unpaid_days: 0.0,
            sick_days: 0.0,
        };

        for (code, days) in rows {
            #[allow(clippy::cast_precision_loss)]
            let days_f = days as f64;
            breakdown.total_days += days_f;
            if code == "unpaid" || code == "personal" {
                breakdown.unpaid_days += days_f;
            } else if code == "sick" {
                breakdown.sick_days += days_f;
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let paid_holiday_count_f = paid_holiday_count as f64;
        breakdown.total_days -= paid_holiday_count_f;
        if breakdown.total_days < 0.0 {
            breakdown.total_days = 0.0;
        }

        Ok(breakdown)
                    })
            })
            .await
    }

    pub(super) async fn get_unpaid_holiday_days_without_leave_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<i64, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let count = holidays::table
                        .filter(holidays::workspace_id.eq(workspace_id))
                        .filter(holidays::is_paid.eq(false))
                        .filter(holidays::date.ge(start_date))
                        .filter(holidays::date.le(end_date))
                        .filter(diesel::dsl::not(diesel::dsl::exists(
                            leave_requests::table
                                .inner_join(employments::table)
                                .filter(employments::employee_id.eq(employee_id))
                                .filter(employments::workspace_id.eq(workspace_id))
                                .filter(
                                    leave_requests::status
                                        .eq(LeaveStatus::Approved)
                                        .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                                )
                                .filter(leave_requests::start_date.le(holidays::date))
                                .filter(leave_requests::end_date.ge(holidays::date)),
                        )))
                        .count()
                        .get_result::<i64>(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(count)
                })
            })
            .await
    }

    pub(super) async fn get_batch_leave_days_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let rows = leave_requests::table
                        .inner_join(employments::table)
                        .filter(employments::employee_id.eq_any(employee_ids))
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(
                            leave_requests::status
                                .eq(LeaveStatus::Approved)
                                .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                        )
                        .filter(leave_requests::start_date.ge(start_date))
                        .filter(leave_requests::end_date.le(end_date))
                        .group_by(employments::employee_id)
                        .select((
                            employments::employee_id,
                            diesel::dsl::sql::<diesel::sql_types::BigInt>(
                                "SUM(leave_requests.end_date - leave_requests.start_date + 1)",
                            ),
                        ))
                        .load::<(Uuid, i64)>(conn)
                        .await
                        .map_err(AppError::from)?;

                    Ok(rows.into_iter().collect())
                })
            })
            .await
    }

    pub(super) async fn get_approved_leave_dates_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashSet<(Uuid, NaiveDate)>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let rows = leave_requests::table
                        .inner_join(employments::table)
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(employments::employee_id.eq_any(employee_ids))
                        .filter(
                            leave_requests::status
                                .eq(LeaveStatus::Approved)
                                .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                        )
                        .filter(leave_requests::start_date.le(end_date))
                        .filter(leave_requests::end_date.ge(start_date))
                        .select((
                            employments::employee_id,
                            leave_requests::start_date,
                            leave_requests::end_date,
                        ))
                        .load::<(Uuid, NaiveDate, NaiveDate)>(conn)
                        .await
                        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

                    let mut set = std::collections::HashSet::new();
                    for (employee_id, start_d, end_d) in rows {
                        let mut d = start_d;
                        while d <= end_d {
                            if d >= start_date && d <= end_date {
                                set.insert((employee_id, d));
                            }
                            d = match d.succ_opt() {
                                Some(next) => next,
                                None => break,
                            };
                        }
                    }

                    Ok(set)
                })
            })
            .await
    }

    pub(super) async fn get_approved_leave_details_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<(Uuid, NaiveDate), bool>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let rows = leave_requests::table
                        .inner_join(employments::table)
                        .inner_join(leave_types::table)
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(employments::employee_id.eq_any(employee_ids))
                        .filter(
                            leave_requests::status
                                .eq(LeaveStatus::Approved)
                                .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                        )
                        .filter(leave_requests::start_date.le(end_date))
                        .filter(leave_requests::end_date.ge(start_date))
                        .select((
                            employments::employee_id,
                            leave_requests::start_date,
                            leave_requests::end_date,
                            leave_types::is_paid,
                        ))
                        .load::<(Uuid, NaiveDate, NaiveDate, bool)>(conn)
                        .await
                        .map_err(|e| crate::errors::AppError::Internal(e.to_string()))?;

                    let mut map = std::collections::HashMap::new();
                    for (employee_id, start_d, end_d, is_paid) in rows {
                        let mut d = start_d;
                        while d <= end_d {
                            if d >= start_date && d <= end_date {
                                map.insert((employee_id, d), is_paid);
                            }
                            d = match d.succ_opt() {
                                Some(next) => next,
                                None => break,
                            };
                        }
                    }

                    Ok(map)
                })
            })
            .await
    }

    pub(super) async fn get_approved_leave_periods_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(NaiveDate, NaiveDate)>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    let rows: Vec<(NaiveDate, NaiveDate)> = leave_requests::table
                        .inner_join(employments::table)
                        .filter(employments::employee_id.eq(employee_id))
                        .filter(employments::workspace_id.eq(workspace_id))
                        .filter(
                            leave_requests::status
                                .eq(LeaveStatus::Approved)
                                .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                        )
                        .filter(leave_requests::start_date.le(end_date))
                        .filter(leave_requests::end_date.ge(start_date))
                        .select((leave_requests::start_date, leave_requests::end_date))
                        .load(conn)
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?;

                    Ok(rows)
                })
            })
            .await
    }

    pub(super) async fn get_approved_leave_breakdowns_batch_impl(
        &self,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<std::collections::HashMap<Uuid, LeaveBreakdown>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {

        // Step 1: Get raw leave days per type per employee
        let rows = leave_requests::table
            .inner_join(leave_types::table)
            .inner_join(employments::table)
            .filter(employments::employee_id.eq_any(employee_ids))
            .filter(employments::workspace_id.eq(workspace_id))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.le(end_date))
            .filter(leave_requests::end_date.ge(start_date))
            .group_by((employments::employee_id, leave_types::code))
            .select((
                employments::employee_id,
                leave_types::code,
                diesel::dsl::sql::<diesel::sql_types::BigInt>(&format!(
                    "SUM(LEAST(leave_requests.end_date, '{end_date}') - GREATEST(leave_requests.start_date, '{start_date}') + 1)"
                )),
            ))
            .load::<(Uuid, String, i64)>(conn)
            .await
            .map_err(AppError::from)?;

        // Step 2: Get paid holidays count intersecting with any approved leave per employee
        let holiday_rows = holidays::table
            .inner_join(employments::table.on(employments::workspace_id.eq(holidays::workspace_id)))
            .inner_join(leave_requests::table.on(leave_requests::employment_id.eq(employments::id)))
            .filter(holidays::workspace_id.eq(workspace_id))
            .filter(holidays::is_paid.eq(true))
            .filter(holidays::date.ge(start_date))
            .filter(holidays::date.le(end_date))
            .filter(employments::employee_id.eq_any(employee_ids))
            .filter(
                leave_requests::status
                    .eq(LeaveStatus::Approved)
                    .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
            )
            .filter(leave_requests::start_date.le(holidays::date))
            .filter(leave_requests::end_date.ge(holidays::date))
            .select((employments::employee_id, holidays::date))
            .distinct()
            .load::<(Uuid, NaiveDate)>(conn)
            .await
            .map_err(|e| AppError::Internal(format!("db holiday counts: {e}")))?;

        let mut holiday_counts = std::collections::HashMap::new();
        for (employee_id, _holiday_date) in holiday_rows {
            *holiday_counts.entry(employee_id).or_insert(0) += 1;
        }

        let mut result = std::collections::HashMap::new();
        for &emp_id in employee_ids {
            result.insert(
                emp_id,
                LeaveBreakdown {
                    total_days: 0.0,
                    unpaid_days: 0.0,
                    sick_days: 0.0,
                },
            );
        }

        for (emp_id, code, days) in rows {
            if let Some(breakdown) = result.get_mut(&emp_id) {
                #[allow(clippy::cast_precision_loss)]
                let days_f = days as f64;
                breakdown.total_days += days_f;
                if code == "unpaid" || code == "personal" {
                    breakdown.unpaid_days += days_f;
                } else if code == "sick" {
                    breakdown.sick_days += days_f;
                }
            }
        }

        for &emp_id in employee_ids {
            if let Some(breakdown) = result.get_mut(&emp_id) {
                let paid_holiday_count = holiday_counts.get(&emp_id).copied().unwrap_or(0);
                breakdown.total_days -= f64::from(paid_holiday_count);
                if breakdown.total_days < 0.0 {
                    breakdown.total_days = 0.0;
                }
            }
        }

        Ok(result)
                    })
            })
            .await
    }

    pub(super) async fn get_unpaid_holiday_days_without_leave_batch_impl(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        employee_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<std::collections::HashMap<Uuid, i64>, AppError> {
        let workspace_id = auth_token.workspace_id();
        self.diesel_pools
            .with_tenant_tx(auth_token, |conn| {
                Box::pin(async move {
                    // 1. Get all unpaid holidays in the period
                    let unpaid_holidays: Vec<NaiveDate> = holidays::table
                        .filter(holidays::workspace_id.eq(workspace_id))
                        .filter(holidays::is_paid.eq(false))
                        .filter(holidays::date.ge(start_date))
                        .filter(holidays::date.le(end_date))
                        .select(holidays::date)
                        .load::<NaiveDate>(conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db unpaid holidays: {e}")))?;

                    let total_unpaid_holidays = i64::try_from(unpaid_holidays.len()).unwrap_or(0);

                    if total_unpaid_holidays == 0 {
                        let mut result = std::collections::HashMap::new();
                        for &emp_id in employee_ids {
                            result.insert(emp_id, 0);
                        }
                        return Ok(result);
                    }

                    // 2. Query which employees have approved leaves on which unpaid holidays
                    let leave_rows = holidays::table
                        .inner_join(
                            employments::table
                                .on(employments::workspace_id.eq(holidays::workspace_id)),
                        )
                        .inner_join(
                            leave_requests::table
                                .on(leave_requests::employment_id.eq(employments::id)),
                        )
                        .filter(holidays::workspace_id.eq(workspace_id))
                        .filter(holidays::is_paid.eq(false))
                        .filter(holidays::date.ge(start_date))
                        .filter(holidays::date.le(end_date))
                        .filter(employments::employee_id.eq_any(employee_ids))
                        .filter(
                            leave_requests::status
                                .eq(LeaveStatus::Approved)
                                .or(leave_requests::status.eq(LeaveStatus::PreApproved)),
                        )
                        .filter(leave_requests::start_date.le(holidays::date))
                        .filter(leave_requests::end_date.ge(holidays::date))
                        .select(employments::employee_id)
                        .load::<Uuid>(conn)
                        .await
                        .map_err(|e| {
                            AppError::Internal(format!("db leaves on unpaid holidays: {e}"))
                        })?;

                    let mut covered_counts = std::collections::HashMap::new();
                    for employee_id in leave_rows {
                        *covered_counts.entry(employee_id).or_insert(0) += 1;
                    }

                    let mut result = std::collections::HashMap::new();
                    for &emp_id in employee_ids {
                        let covered = covered_counts.get(&emp_id).copied().unwrap_or(0);
                        result.insert(emp_id, total_unpaid_holidays - covered);
                    }

                    Ok(result)
                })
            })
            .await
    }
}
