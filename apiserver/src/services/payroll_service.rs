use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
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

        // 0.1 Get Payroll Settings
        let payroll_settings = payroll_repository::get_payroll_settings(pool, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .unwrap_or(crate::models::PayrollSettings {
                workspace_id,
                std_working_day_mode: "fixed".to_string(),
                fixed_std_days: Decimal::from(26),
                working_days_per_week: 6,
                hours_per_day: Decimal::from(8),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });

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

        // Fetch dynamic salary components
        let salary_components = payroll_repository::get_employee_salary_components(pool, employee_id, end_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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

        let mut total_standard_units = 0;
        let mut total_overtime_units = 0;

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

            let work_entries: Vec<_> = entries.iter().filter(|e| e.entry_type == "work" || e.entry_type.is_empty()).collect();
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

            let mut day_standard_units = 0;
            let mut day_overtime_units = 0;

            for entry in work_entries {
                let punch_in = entry.punch_in_time.with_timezone(&tz);
                let punch_out = entry.punch_out_time.map(|t| t.with_timezone(&tz));

                if let Some(punch_out) = punch_out {
                    if let Some(sid) = shift_id {
                        let shift = shift_repository::get_shift_by_id(pool, sid)
                            .await
                            .unwrap_or(None);

                        if let Some(s) = shift {
                            let start_dt = tz.from_local_datetime(&entry_date.and_time(s.start_time)).single();
                            let end_dt = tz.from_local_datetime(&entry_date.and_time(s.end_time)).single();
                            
                            if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                                if s.end_time < s.start_time {
                                    end += chrono::Duration::days(1);
                                }

                                let (std_units, ot_units) = AttendanceCalculator::calculate_daily_units(
                                    punch_in,
                                    punch_out,
                                    Some(start),
                                    Some(end),
                                    s.break_duration_minutes,
                                    s.early_punch_in_grace_period_minutes,
                                    s.is_break_auto_deduct,
                                    entry.is_break_overtime,
                                    actual_break_minutes,
                                    s.allow_early_ot,
                                    workspace.work_unit_minutes,
                                    &workspace.rounding_policy
                                );
                                day_standard_units += std_units;
                                day_overtime_units += ot_units;
                            } else {
                                day_standard_units += AttendanceCalculator::calculate_units(
                                    (punch_out - punch_in).num_minutes() as i32,
                                    workspace.work_unit_minutes,
                                    &workspace.rounding_policy
                                );
                            }
                        } else {
                            day_standard_units += AttendanceCalculator::calculate_units(
                                (punch_out - punch_in).num_minutes() as i32,
                                workspace.work_unit_minutes,
                                &workspace.rounding_policy
                            );
                        }
                    } else {
                        day_standard_units += AttendanceCalculator::calculate_units(
                            (punch_out - punch_in).num_minutes() as i32,
                            workspace.work_unit_minutes,
                            &workspace.rounding_policy
                        );
                    }
                }
            }

            total_standard_units += day_standard_units;
            total_overtime_units += day_overtime_units;
        }

        // 4. Calculate pay

        // 4.1 Determine standard working days for this month
        let std_days = if payroll_settings.std_working_day_mode == "actual_month" {
            AttendanceCalculator::count_working_days(year, month, payroll_settings.working_days_per_week)
        } else {
            payroll_settings.fixed_std_days.to_f64().unwrap_or(26.0)
        };

        // Standard units per day (HoursPerDay / WorkUnitMinutes)
        let units_per_day = (payroll_settings.hours_per_day * Decimal::from(60)) / Decimal::from(workspace.work_unit_minutes);
        let std_units_per_month = Decimal::from_f64(std_days).unwrap_or(Decimal::from(26)) * units_per_day;

        // 4.2 Calculate Base for OT and Total Base Pay
        let mut base_pay_for_ot = Decimal::ZERO;
        let mut total_base_pay = Decimal::ZERO;
        let mut is_hourly = false;

        for (esc, scd) in &salary_components {
            total_base_pay += esc.amount;
            if scd.is_base_for_ot {
                base_pay_for_ot += esc.amount;
            }
        }

        // If no components found, fallback to old salary_history
        if total_base_pay.is_zero() && salary_components.is_empty() {
             let old_settings = payroll_repository::get_latest_salary(pool, employee_id, end_date)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                .ok_or(StatusCode::BAD_REQUEST)?;
             
             if old_settings.salary_type == "hourly" {
                 is_hourly = true;
                 total_base_pay = old_settings.base_salary * Decimal::from(total_standard_units);
                 base_pay_for_ot = old_settings.base_salary; // This is actually unit rate if hourly
             } else {
                 total_base_pay = old_settings.base_salary;
                 base_pay_for_ot = old_settings.base_salary;
             }
        }

        // Calculate unit rate for Overtime
        let unit_rate = if is_hourly {
            base_pay_for_ot
        } else {
            base_pay_for_ot / std_units_per_month
        };

        let overtime_pay =
            unit_rate * Decimal::from(total_overtime_units) * Decimal::from_f64(1.5).unwrap();

        let deductions = Decimal::ZERO;

        let net_pay = total_base_pay + overtime_pay - deductions;

        let payslip = Payslip {
            id: Uuid::new_v4(),
            employee_id,
            period_start: start_date,
            period_end: end_date,
            base_pay_amount: total_base_pay,
            overtime_pay_amount: overtime_pay,
            deductions_amount: deductions,
            net_pay_amount: net_pay,
            standard_units: total_standard_units,
            overtime_units: total_overtime_units,
            status: "draft".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        payroll_repository::create_payslip(pool, &payslip)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
