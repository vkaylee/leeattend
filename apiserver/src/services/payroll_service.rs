use sqlx::PgPool;
use uuid::Uuid;
use chrono::{NaiveDate, TimeZone, Utc};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use axum::http::StatusCode;

use crate::repositories::{payroll_repository, employee_repository, time_entry_repository};
use crate::models::Payslip;

pub struct PayrollService;

impl PayrollService {
    pub async fn calculate_monthly_payslip(
        pool: &PgPool,
        employee_id: Uuid,
        workspace_id: Uuid,
        year: i32,
        month: u32,
    ) -> Result<Payslip, StatusCode> {
        // 1. Get employee and salary settings (use reference date as end of period)
        let _employee = employee_repository::get_employee_by_id(pool, employee_id, workspace_id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;

        let end_date = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }.unwrap().pred_opt().unwrap();

        let settings = payroll_repository::get_latest_salary(pool, employee_id, end_date)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?; // Salary settings must be configured

        // 2. Define period
        let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or(StatusCode::BAD_REQUEST)?;

        // Convert NaiveDate to DateTime<Utc> for query
        let start_dt = Utc.from_local_datetime(&start_date.and_hms_opt(0, 0, 0).unwrap()).unwrap();
        let end_dt = Utc.from_local_datetime(&end_date.and_hms_opt(23, 59, 59).unwrap()).unwrap();

        // 3. Get worked hours
        let time_entries = time_entry_repository::get_employee_time_entries(pool, employee_id, start_dt, end_dt)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let worked_minutes: i32 = time_entries.into_iter()
            .map(|te| te.duration_minutes.unwrap_or(0))
            .sum();

        let worked_hours = worked_minutes as f64 / 60.0;

        // 4. Calculate pay based on type
        let base_pay = if settings.salary_type == "monthly" {
            settings.base_salary
        } else {
            // hourly
            let rate = settings.base_salary;
            rate * Decimal::from_f64(worked_hours).unwrap_or(Decimal::ZERO)
        };

        // For MVP, overtime and deductions are 0
        let overtime_pay = Decimal::ZERO;
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