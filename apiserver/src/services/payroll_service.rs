use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::{
    employee_repository, payroll_repository, shift_repository, time_entry_repository, roster_repository, workspace_repository::{WorkspaceRepository, SqlxWorkspaceRepository}
};

use crate::models::Payslip;
use crate::services::AttendanceCalculator;

pub struct PayrollService;

impl PayrollService {
    pub async fn calculate_monthly_payslip(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Payslip, StatusCode> {
        // 0. Get workspace settings for Timezone & Rounding
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo.get_workspace_by_id(workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. Get employee and salary settings
        let _employee = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(StatusCode::BAD_REQUEST)?;

        let end_date = next_month_date
            .pred_opt()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let settings = payroll_repository::get_latest_salary(pool, employee_id, end_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?;

        // 2. Define period
        let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or(StatusCode::BAD_REQUEST)?;

        let start_dt = tz
            .from_local_datetime(
                &start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?
            .with_timezone(&Utc);

        let end_dt = tz
            .from_local_datetime(
                &end_date
                    .and_hms_opt(23, 59, 59)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?
            .with_timezone(&Utc);

        // 3. Get worked entries
        let time_entries =
            time_entry_repository::get_employee_time_entries(pool, employee_id, start_dt, end_dt)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut total_standard_minutes = 0;
        let mut total_overtime_minutes = 0;

        // Group entries by local date
        use std::collections::HashMap;
        let mut daily_entries: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> = HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time.with_timezone(&tz).date_naive();
            daily_entries.entry(local_date).or_default().push(entry);
        }

        for (entry_date, entries) in daily_entries {
            let shift_id = roster_repository::get_effective_shift(pool, employee_id, entry_date)
                .await
                .unwrap_or(None);

            let work_entries: Vec<_> = entries.iter().filter(|e| e.entry_type == "work" || e.entry_type == "").collect();
            let break_entries: Vec<_> = entries.iter().filter(|e| e.entry_type == "break").collect();

            let actual_break_minutes: i32 = break_entries.iter().map(|e| {
                e.duration_minutes.unwrap_or_else(|| {
                    if let Some(out) = e.punch_out_time {
                        out.signed_duration_since(e.punch_in_time).num_minutes() as i32
                    } else {
                        0
                    }
                })
            }).sum();

            let mut day_standard = 0;
            let mut day_overtime = 0;

            for entry in work_entries {
                let punch_in = entry.punch_in_time.with_timezone(&tz);
                let punch_out = entry.punch_out_time.map(|t| t.with_timezone(&tz));

                if let Some(punch_out) = punch_out {
                    if let Some(sid) = shift_id {
                        let shift = shift_repository::get_shift_by_id(pool, sid)
                            .await
                            .unwrap_or(None);

                        if let Some(s) = shift {
                            let (std, ot) = AttendanceCalculator::calculate_daily_net_hours(
                                punch_in,
                                punch_out,
                                tz.from_local_datetime(&entry_date.and_time(s.start_time)).single(),
                                tz.from_local_datetime(&entry_date.and_time(s.end_time)).single(),
                                s.break_duration_minutes,
                                s.early_punch_in_grace_period_minutes,
                                s.is_break_auto_deduct,
                                entry.is_break_overtime,
                                actual_break_minutes
                            );
                            day_standard += std;
                            day_overtime += ot;
                        } else {
                            day_standard += (punch_out - punch_in).num_minutes() as i32;
                        }
                    } else {
                        day_standard += (punch_out - punch_in).num_minutes() as i32;
                    }
                }
            }

            // Apply rounding to daily totals
            total_standard_minutes += AttendanceCalculator::apply_rounding(day_standard, workspace.work_unit_minutes, &workspace.rounding_policy);
            total_overtime_minutes += AttendanceCalculator::apply_rounding(day_overtime, workspace.work_unit_minutes, &workspace.rounding_policy);
        }

        let worked_hours = total_standard_minutes as f64 / 60.0;


        let overtime_hours = total_overtime_minutes as f64 / 60.0;

        // 4. Calculate pay

        // Base pay

        let base_pay = if settings.salary_type == "monthly" {
            settings.base_salary
        } else {
            let rate = settings.base_salary;

            rate * Decimal::from_f64(worked_hours).unwrap_or(Decimal::ZERO)
        };

        // Overtime pay (calculated as 1.5x hourly rate)

        let hourly_rate = if settings.salary_type == "monthly" {
            // Assume 176 hours per month for salary calculation if monthly

            settings.base_salary / Decimal::from(176)
        } else {
            settings.base_salary
        };

        let overtime_pay =
            hourly_rate * Decimal::from_f64(overtime_hours * 1.5).unwrap_or(Decimal::ZERO);

        let deductions = Decimal::ZERO;

        let net_pay = base_pay + overtime_pay - deductions;

        let payslip = Payslip {
            id: Uuid::new_v4(),

            employee_id,

            period_start: start_date,

            period_end: end_date,

            base_pay_amount: base_pay,

            overtime_pay_amount: overtime_pay,

            deductions_amount: deductions,

            net_pay_amount: net_pay,

            status: "draft".to_string(),

            created_at: chrono::Utc::now(),

            updated_at: chrono::Utc::now(),
        };

        payroll_repository::create_payslip(pool, &payslip)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
