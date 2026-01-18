use axum::http::StatusCode;
use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::collections::{BTreeSet, HashMap};
use uuid::Uuid;

use crate::domain::policy::resolver::PolicyResolver;
use crate::models::Payslip;
use crate::repositories::{
    department_repository, payroll_repository, roster_repository,
    shift_repository, time_entry_repository,
    workspace_repository::{SqlxWorkspaceRepository, WorkspaceRepository},
};
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
        let workspace = workspace_repo
            .get_workspace_by_id(workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let tz: Tz = workspace.timezone.parse().unwrap_or(chrono_tz::UTC);

        // 1. Define Period
        let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or(StatusCode::BAD_REQUEST)?;
        let next_month_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(StatusCode::BAD_REQUEST)?;
        let end_date = next_month_date
            .pred_opt()
            .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        // 2. Fetch History Data for Segmentation
        let salary_history = payroll_repository::get_salary_components_history(
            pool,
            employee_id,
            start_date,
            end_date,
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Fetch all dept history
        let all_dept_history =
            department_repository::get_employee_department_history(pool, employee_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 4. Fetch Time Entries (All for the month)
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

        let time_entries =
            time_entry_repository::get_employee_time_entries(pool, employee_id, start_dt, end_dt)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // 5. Determine Segments
        let mut change_points = BTreeSet::new();
        change_points.insert(start_date);

        for (comp, _) in &salary_history {
            if comp.effective_date > start_date && comp.effective_date <= end_date {
                change_points.insert(comp.effective_date);
            }
        }
        for dept in &all_dept_history {
            if dept.effective_date > start_date && dept.effective_date <= end_date {
                change_points.insert(dept.effective_date);
            }
        }

        let sorted_dates: Vec<NaiveDate> = change_points.into_iter().collect();
        let mut segments = Vec::new();

        for i in 0..sorted_dates.len() {
            let seg_start = sorted_dates[i];
            let seg_end = if i + 1 < sorted_dates.len() {
                sorted_dates[i + 1].pred_opt().unwrap()
            } else {
                end_date
            };
            segments.push((seg_start, seg_end));
        }

        // 6. Calculate Each Segment
        let mut total_base_pay = Decimal::ZERO;
        let mut total_std_units = 0;
        let mut total_ot_units = 0;
        let mut segment_snapshots = Vec::new();

        // Prepare time entries grouped by date for faster access
        let mut daily_entries_map: HashMap<NaiveDate, Vec<crate::models::TimeEntry>> =
            HashMap::new();
        for entry in time_entries {
            let local_date = entry.punch_in_time.with_timezone(&tz).date_naive();
            daily_entries_map
                .entry(local_date)
                .or_default()
                .push(entry);
        }

        for (seg_start, seg_end) in segments {
            // A. Context Resolution
            let active_dept = all_dept_history
                .iter()
                .filter(|d| d.effective_date <= seg_start)
                .max_by_key(|d| d.effective_date);
            let dept_id = active_dept.map(|d| d.department_id);

            let mut segment_base_salary = Decimal::ZERO;
            let mut processed_def_ids = BTreeSet::new();
            for (esc, scd) in salary_history.iter().rev() {
                if !processed_def_ids.contains(&scd.id) && esc.effective_date <= seg_start {
                    if scd.code == "BASE" || scd.name == "Lương cơ bản" {
                        segment_base_salary += esc.amount;
                    }
                    processed_def_ids.insert(scd.id);
                }
            }

            // Resolve Standard Month Config
            let std_config = PolicyResolver::resolve_standard_month(
                pool,
                employee_id,
                dept_id,
                workspace_id,
                seg_start,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            let seg_units_per_day = (std_config.hours_per_day * Decimal::from(60))
                / Decimal::from(workspace.work_unit_minutes);

            // B. Attendance Calculation
            let mut segment_std_units = 0;
            let mut segment_ot_units = 0;

            let mut current_date = seg_start;
            while current_date <= seg_end {
                // Fetch shift and calculate
                let shift_id = roster_repository::get_effective_shift(pool, employee_id, current_date)
                    .await
                    .unwrap_or(None);

                if let Some(entries) = daily_entries_map.get(&current_date) {
                    let work_entries: Vec<_> = entries.iter().filter(|e| e.entry_type == "work" || e.entry_type.is_empty()).collect();
                    let break_entries: Vec<_> = entries.iter().filter(|e| e.entry_type == "break").collect();
                    let actual_break_minutes: i32 = break_entries.iter().map(|e| e.duration_minutes.unwrap_or(0)).sum();

                    for entry in work_entries {
                        let punch_in = entry.punch_in_time.with_timezone(&tz);
                        if let Some(punch_out_raw) = entry.punch_out_time {
                            let punch_out = punch_out_raw.with_timezone(&tz);
                            if let Some(sid) = shift_id {
                                if let Some(s) = shift_repository::get_shift_by_id(pool, sid).await.unwrap_or(None) {
                                    let start_dt = tz.from_local_datetime(&current_date.and_time(s.start_time)).single();
                                    let end_dt = tz.from_local_datetime(&current_date.and_time(s.end_time)).single();
                                    if let (Some(start), Some(mut end)) = (start_dt, end_dt) {
                                        if s.end_time < s.start_time { end += chrono::Duration::days(1); }
                                        let (std_u, ot_u) = AttendanceCalculator::calculate_daily_units(
                                            punch_in, punch_out, Some(start), Some(end),
                                            s.break_duration_minutes, s.early_punch_in_grace_period_minutes,
                                            s.is_break_auto_deduct, entry.is_break_overtime,
                                            actual_break_minutes, s.allow_early_ot,
                                            workspace.work_unit_minutes, &workspace.rounding_policy, 0, false
                                        );
                                        segment_std_units += std_u;
                                        segment_ot_units += ot_u;
                                    }
                                }
                            } else {
                                segment_std_units += AttendanceCalculator::calculate_units(
                                    (punch_out - punch_in).num_minutes() as i32,
                                    workspace.work_unit_minutes, &workspace.rounding_policy
                                );
                            }
                        }
                    }
                }
                current_date = current_date.succ_opt().unwrap();
            }

            // C. Financials
            let worked_days_dec = Decimal::from(segment_std_units) / seg_units_per_day;
            let daily_rate = if std_config.std_working_days.is_zero() { Decimal::ZERO } else { segment_base_salary / std_config.std_working_days };
            let segment_amount = daily_rate * worked_days_dec;

            total_base_pay += segment_amount;
            total_std_units += segment_std_units;
            total_ot_units += segment_ot_units;
            
            segment_snapshots.push((
                seg_start, seg_end, dept_id, segment_base_salary, std_config.std_working_days,
                std_config.hours_per_day, worked_days_dec, worked_days_dec, segment_amount
            ));
        }

        // OT Calculation
        let total_overtime_pay = if !segment_snapshots.is_empty() {
            let last = segment_snapshots.last().unwrap();
            let daily_rate = if last.4.is_zero() { Decimal::ZERO } else { last.3 / last.4 };
            let hourly_rate = if last.5.is_zero() { Decimal::ZERO } else { daily_rate / last.5 };
            hourly_rate * (Decimal::from(total_ot_units) * Decimal::from(workspace.work_unit_minutes) / Decimal::from(60)) * Decimal::from_f64(1.5).unwrap()
        } else { Decimal::ZERO };

        let payslip = Payslip {
            id: Uuid::new_v4(), employee_id, period_start: start_date, period_end: end_date,
            base_pay_amount: total_base_pay, overtime_pay_amount: total_overtime_pay,
            deductions_amount: Decimal::ZERO, net_pay_amount: total_base_pay + total_overtime_pay,
            standard_units: total_std_units, overtime_units: total_ot_units,
            status: "draft".to_string(), created_at: Utc::now(), updated_at: Utc::now(),
        };

        let created_payslip = payroll_repository::create_payslip(pool, &payslip).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            
        for (start, end, dept, base, std, hours, actual, paid, amount) in segment_snapshots {
            payroll_repository::create_payslip_segment(
                pool, created_payslip.id, start, end, dept, base, std, hours, actual, paid, amount
            ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }

        Ok(created_payslip)
    }
}
