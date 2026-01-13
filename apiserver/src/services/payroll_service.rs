use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc, Local};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::repositories::{
    employee_repository, payroll_repository, shift_repository, time_entry_repository, roster_repository, workspace_repository::{WorkspaceRepository, SqlxWorkspaceRepository}
};

use crate::models::Payslip;

pub struct PayrollService;

impl PayrollService {
    fn apply_rounding(minutes: i32, unit: i32, policy: &str) -> i32 {
        if unit <= 1 || policy == "none" {
            return minutes;
        }

        let unit = unit as f64;
        let mins = minutes as f64;

        match policy {
            "floor" => (mins / unit).floor() as i32 * unit as i32,
            "ceil" => (mins / unit).ceil() as i32 * unit as i32,
            "round" => (mins / unit).round() as i32 * unit as i32,
            _ => minutes,
        }
    }

    pub async fn calculate_monthly_payslip(
        pool: &PgPool,

        employee_id: Uuid,

        workspace_id: Uuid,

        year: i32,

        month: u32,
    ) -> Result<Payslip, StatusCode> {
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

        let start_dt = Utc
            .from_local_datetime(
                &start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?;

        let end_dt = Utc
            .from_local_datetime(
                &end_date
                    .and_hms_opt(23, 59, 59)
                    .ok_or(StatusCode::BAD_REQUEST)?,
            )
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?;

        // 3. Get worked entries
        let time_entries =
            time_entry_repository::get_employee_time_entries(pool, employee_id, start_dt, end_dt)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 0. Get workspace settings for rounding
        let workspace_repo = SqlxWorkspaceRepository { pool: pool.clone() };
        let workspace = workspace_repo.get_workspace_by_id(workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let mut total_standard_minutes = 0;
        let mut total_overtime_minutes = 0;

        // Group entries by local date to handle breaks within the same day
        use std::collections::HashMap;
        let mut daily_entries: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> = HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time.with_timezone(&Local).date_naive();
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
                let mut duration = entry.duration_minutes.unwrap_or_else(|| {
                    if let Some(out) = entry.punch_out_time {
                        out.signed_duration_since(entry.punch_in_time).num_minutes() as i32
                    } else {
                        0
                    }
                });

                if let Some(sid) = shift_id {
                    let shift = shift_repository::get_shift_by_id(pool, sid)
                        .await
                        .unwrap_or(None);

                    if let Some(s) = shift {
                        // Effective punch in logic (Grace Period)
                        let local_punch_in = entry.punch_in_time.with_timezone(&Local);
                        let shift_start_dt = Local.from_local_datetime(&entry_date.and_time(s.start_time)).single();
                        
                        let mut effective_punch_in = local_punch_in;

                        if let Some(shift_start) = shift_start_dt {
                            if s.early_punch_in_grace_period_minutes > 0 {
                                let grace_start = shift_start - chrono::Duration::minutes(s.early_punch_in_grace_period_minutes as i64);
                                if local_punch_in >= grace_start && local_punch_in < shift_start {
                                    effective_punch_in = shift_start;
                                }
                            }
                        }

                        // Recalculate duration if punch_in was adjusted (effective punch-in)
                        if let Some(punch_out_time) = entry.punch_out_time {
                            let local_punch_out = punch_out_time.with_timezone(&Local);
                            let adjusted_duration = local_punch_out
                                .signed_duration_since(effective_punch_in)
                                .num_minutes() as i32;
                            duration = i32::max(0, adjusted_duration);
                        }

                        // Standard work duration
                        let shift_duration = (s.end_time - s.start_time).num_minutes() as i32;
                        let work_hours_standard = shift_duration - s.break_duration_minutes;

                        // Break handling logic
                        // Rule: If actual break entries exist, use them. 
                        // If not, fallback to auto-deduct if enabled.
                        let break_to_deduct = if actual_break_minutes > 0 {
                            actual_break_minutes
                        } else if s.is_break_auto_deduct && !entry.is_break_overtime {
                            s.break_duration_minutes
                        } else {
                            0
                        };

                        let worked_minutes = i32::max(0, duration - break_to_deduct);

                        if worked_minutes > work_hours_standard {
                            day_standard += work_hours_standard;
                            day_overtime += worked_minutes - work_hours_standard;
                        } else {
                            day_standard += worked_minutes;
                        }
                    } else {
                        day_standard += duration;
                    }
                } else {
                    day_standard += duration;
                }
            }

            // Apply rounding to daily totals
            total_standard_minutes += Self::apply_rounding(day_standard, workspace.work_unit_minutes, &workspace.rounding_policy);
            total_overtime_minutes += Self::apply_rounding(day_overtime, workspace.work_unit_minutes, &workspace.rounding_policy);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_rounding_none() {
        assert_eq!(PayrollService::apply_rounding(17, 15, "none"), 17);
        assert_eq!(PayrollService::apply_rounding(17, 1, "floor"), 17); // unit 1 should be no-op effectively or exact minutes
    }

    #[test]
    fn test_apply_rounding_floor() {
        // 17 mins, 15 unit -> 15
        assert_eq!(PayrollService::apply_rounding(17, 15, "floor"), 15);
        // 29 mins, 15 unit -> 15
        assert_eq!(PayrollService::apply_rounding(29, 15, "floor"), 15);
        // 30 mins, 15 unit -> 30
        assert_eq!(PayrollService::apply_rounding(30, 15, "floor"), 30);
    }

    #[test]
    fn test_apply_rounding_ceil() {
        // 16 mins, 15 unit -> 30
        assert_eq!(PayrollService::apply_rounding(16, 15, "ceil"), 30);
        // 15 mins, 15 unit -> 15
        assert_eq!(PayrollService::apply_rounding(15, 15, "ceil"), 15);
        // 1 min, 15 unit -> 15
        assert_eq!(PayrollService::apply_rounding(1, 15, "ceil"), 15);
    }

    #[test]
    fn test_apply_rounding_round() {
        // 22 mins (7 past 15), 15 unit -> 15 (less than half way?)
        // 15/2 = 7.5.
        // 15 + 7 = 22. 22/15 = 1.466 -> round to 1 -> 15.
        assert_eq!(PayrollService::apply_rounding(22, 15, "round"), 15);
        
        // 23 mins (8 past 15), 15 unit -> 30
        // 15 + 8 = 23. 23/15 = 1.533 -> round to 2 -> 30.
        assert_eq!(PayrollService::apply_rounding(23, 15, "round"), 30);
    }
}
