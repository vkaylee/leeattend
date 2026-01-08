use sqlx::{PgPool, query_as};
use uuid::Uuid;
use chrono::Utc;
use rust_decimal::Decimal;

use crate::models::{SalarySettings, Payslip};

pub async fn get_salary_settings(pool: &PgPool, employee_id: Uuid) -> Result<Option<SalarySettings>, sqlx::Error> {
    query_as!(
        SalarySettings,
        r#"SELECT employee_id, base_salary, salary_type, currency, created_at as "created_at!", updated_at as "updated_at!" FROM salary_settings WHERE employee_id = $1"#,
        employee_id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_salary_settings(pool: &PgPool, employee_id: Uuid, base_salary: Decimal, salary_type: String, currency: String) -> Result<SalarySettings, sqlx::Error> {
    query_as!(
        SalarySettings,
        r#"INSERT INTO salary_settings (employee_id, base_salary, salary_type, currency, updated_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (employee_id) DO UPDATE SET
            base_salary = EXCLUDED.base_salary,
            salary_type = EXCLUDED.salary_type,
            currency = EXCLUDED.currency,
            updated_at = EXCLUDED.updated_at
         RETURNING employee_id, base_salary, salary_type, currency, created_at as "created_at!", updated_at as "updated_at!""#,
        employee_id,
        base_salary,
        salary_type,
        currency,
        Utc::now()
    )
    .fetch_one(pool)
    .await
}

pub async fn create_payslip(pool: &PgPool, payslip: &Payslip) -> Result<Payslip, sqlx::Error> {
    query_as!(
        Payslip,
        r#"INSERT INTO payslips (id, employee_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING id, employee_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, status, created_at as "created_at!", updated_at as "updated_at!""#,
        payslip.id,
        payslip.employee_id,
        payslip.period_start,
        payslip.period_end,
        payslip.base_pay_amount,
        payslip.overtime_pay_amount,
        payslip.deductions_amount,
        payslip.net_pay_amount,
        payslip.status
    )
    .fetch_one(pool)
    .await
}

pub async fn get_payslips_by_employee(pool: &PgPool, employee_id: Uuid) -> Result<Vec<Payslip>, sqlx::Error> {
    query_as!(
        Payslip,
        r#"SELECT id, employee_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, status, created_at as "created_at!", updated_at as "updated_at!" FROM payslips WHERE employee_id = $1 ORDER BY period_start DESC"#,
        employee_id
    )
    .fetch_all(pool)
    .await
}