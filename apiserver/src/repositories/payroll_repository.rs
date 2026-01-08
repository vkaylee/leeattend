use sqlx::{PgPool, query_as};
use uuid::Uuid;
use chrono::{Utc, NaiveDate};
use rust_decimal::Decimal;

use crate::models::{SalaryHistory, Payslip};

pub async fn get_latest_salary(pool: &PgPool, employee_id: Uuid, reference_date: NaiveDate) -> Result<Option<SalaryHistory>, sqlx::Error> {
    query_as!(
        SalaryHistory,
        r#"SELECT id, employee_id, base_salary, salary_type, currency, effective_date, created_at as "created_at!", updated_at as "updated_at!" 
           FROM salary_history 
           WHERE employee_id = $1 AND effective_date <= $2 
           ORDER BY effective_date DESC, created_at DESC 
           LIMIT 1"#,
        employee_id,
        reference_date
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_salary_history(pool: &PgPool, employee_id: Uuid) -> Result<Vec<SalaryHistory>, sqlx::Error> {
    query_as!(
        SalaryHistory,
        r#"SELECT id, employee_id, base_salary, salary_type, currency, effective_date, created_at as "created_at!", updated_at as "updated_at!" 
           FROM salary_history 
           WHERE employee_id = $1 
           ORDER BY effective_date DESC"#,
        employee_id
    )
    .fetch_all(pool)
    .await
}

pub async fn add_salary_history(pool: &PgPool, employee_id: Uuid, base_salary: Decimal, salary_type: String, currency: String, effective_date: NaiveDate) -> Result<SalaryHistory, sqlx::Error> {
    query_as!(
        SalaryHistory,
        r#"INSERT INTO salary_history (employee_id, base_salary, salary_type, currency, effective_date, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id, employee_id, base_salary, salary_type, currency, effective_date, created_at as "created_at!", updated_at as "updated_at!""#,
        employee_id,
        base_salary,
        salary_type,
        currency,
        effective_date,
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