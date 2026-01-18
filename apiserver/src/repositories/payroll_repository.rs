use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use sqlx::{query_as, PgPool};
use uuid::Uuid;

use crate::models::{
    EmployeeSalaryComponent, PayrollSettings, Payslip, PayslipSegment, SalaryComponentDefinition,
};

pub async fn get_payroll_settings(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Option<PayrollSettings>, sqlx::Error> {
    // Get the latest settings version
    get_payroll_settings_at(pool, workspace_id, Utc::now().date_naive()).await
}

pub async fn get_payroll_settings_at(
    pool: &PgPool,
    workspace_id: Uuid,
    date: NaiveDate,
) -> Result<Option<PayrollSettings>, sqlx::Error> {
    query_as!(
        PayrollSettings,
        r#"SELECT workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, effective_date, created_at as "created_at!", updated_at as "updated_at!" 
           FROM payroll_settings 
           WHERE workspace_id = $1 AND effective_date <= $2
           ORDER BY effective_date DESC LIMIT 1"#,
        workspace_id,
        date
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_payroll_settings(
    pool: &PgPool,
    workspace_id: Uuid,
    std_working_day_mode: String,
    fixed_std_days: Decimal,
    working_days_per_week: i32,
    hours_per_day: Decimal,
    effective_date: NaiveDate,
) -> Result<PayrollSettings, sqlx::Error> {
    query_as!(
        PayrollSettings,
        r#"INSERT INTO payroll_settings (workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, effective_date, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           ON CONFLICT (workspace_id, effective_date) DO UPDATE 
           SET std_working_day_mode = EXCLUDED.std_working_day_mode,
               fixed_std_days = EXCLUDED.fixed_std_days,
               working_days_per_week = EXCLUDED.working_days_per_week,
               hours_per_day = EXCLUDED.hours_per_day,
               updated_at = EXCLUDED.updated_at
           RETURNING workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, effective_date, created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        std_working_day_mode,
        fixed_std_days,
        working_days_per_week,
        hours_per_day,
        effective_date,
        Utc::now()
    )
    .fetch_one(pool)
    .await
}

pub async fn get_salary_component_definitions(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<SalaryComponentDefinition>, sqlx::Error> {
    query_as!(
        SalaryComponentDefinition,
        r#"SELECT id, workspace_id, name, code, is_base_for_ot, is_taxable, created_at as "created_at!", updated_at as "updated_at!" 
           FROM salary_component_definitions 
           WHERE workspace_id = $1"#,
        workspace_id
    )
    .fetch_all(pool)
    .await
}

pub async fn create_salary_component_definition(
    pool: &PgPool,
    workspace_id: Uuid,
    name: String,
    code: String,
    is_base_for_ot: bool,
    is_taxable: bool,
) -> Result<SalaryComponentDefinition, sqlx::Error> {
    query_as!(
        SalaryComponentDefinition,
        r#"INSERT INTO salary_component_definitions (workspace_id, name, code, is_base_for_ot, is_taxable, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING id, workspace_id, name, code, is_base_for_ot, is_taxable, created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        name,
        code,
        is_base_for_ot,
        is_taxable,
        Utc::now()
    )
    .fetch_one(pool)
    .await
}

pub async fn assign_employee_salary_component(
    pool: &PgPool,
    employee_id: Uuid,
    component_definition_id: Uuid,
    amount: Decimal,
    effective_date: NaiveDate,
) -> Result<EmployeeSalaryComponent, sqlx::Error> {
    query_as!(
        EmployeeSalaryComponent,
        r#"INSERT INTO employee_salary_components (employee_id, component_definition_id, amount, effective_date, updated_at)
           VALUES ($1, $2, $3, $4, $5)
           RETURNING id, employee_id, component_definition_id, amount, effective_date, created_at as "created_at!", updated_at as "updated_at!""#,
        employee_id,
        component_definition_id,
        amount,
        effective_date,
        Utc::now()
    )
    .fetch_one(pool)
    .await
}

pub async fn get_employee_salary_components(
    pool: &PgPool,
    employee_id: Uuid,
    reference_date: NaiveDate,
) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT 
            esc.id as esc_id, esc.employee_id, esc.component_definition_id, esc.amount, esc.effective_date, esc.created_at as "esc_created_at!", esc.updated_at as "esc_updated_at!",
            scd.id as scd_id, scd.workspace_id, scd.name, scd.code, scd.is_base_for_ot, scd.is_taxable, scd.created_at as "scd_created_at!", scd.updated_at as "scd_updated_at!"
           FROM employee_salary_components esc
           JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
           WHERE esc.employee_id = $1 AND esc.effective_date <= $2
           AND NOT EXISTS (
               SELECT 1 FROM employee_salary_components esc2 
               WHERE esc2.employee_id = esc.employee_id 
               AND esc2.component_definition_id = esc.component_definition_id 
               AND esc2.effective_date <= $2 
               AND (esc2.effective_date > esc.effective_date OR (esc2.effective_date = esc.effective_date AND esc2.created_at > esc.created_at))
           )"#,
        employee_id,
        reference_date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                EmployeeSalaryComponent {
                    id: row.esc_id,
                    employee_id: row.employee_id,
                    component_definition_id: row.component_definition_id,
                    amount: row.amount,
                    effective_date: row.effective_date,
                    created_at: row.esc_created_at,
                    updated_at: row.esc_updated_at,
                },
                SalaryComponentDefinition {
                    id: row.scd_id,
                    workspace_id: row.workspace_id,
                    name: row.name,
                    code: row.code,
                    is_base_for_ot: row.is_base_for_ot,
                    is_taxable: row.is_taxable,
                    created_at: row.scd_created_at,
                    updated_at: row.scd_updated_at,
                },
            )
        })
        .collect())
}

pub async fn create_payslip(pool: &PgPool, payslip: &Payslip) -> Result<Payslip, sqlx::Error> {
    query_as!(
        Payslip,
        r#"INSERT INTO payslips (id, employee_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         RETURNING id, employee_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status, created_at as "created_at!", updated_at as "updated_at!""#,
        payslip.id,
        payslip.employee_id,
        payslip.period_start,
        payslip.period_end,
        payslip.base_pay_amount,
        payslip.overtime_pay_amount,
        payslip.deductions_amount,
        payslip.net_pay_amount,
        payslip.standard_units,
        payslip.overtime_units,
        payslip.status
    )
    .fetch_one(pool)
    .await
}

pub async fn get_payslips_by_employee(
    pool: &PgPool,
    employee_id: Uuid,
) -> Result<Vec<Payslip>, sqlx::Error> {
    query_as!(
        Payslip,
        r#"SELECT id, employee_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status, created_at as "created_at!", updated_at as "updated_at!" FROM payslips WHERE employee_id = $1 ORDER BY period_start DESC"#,
        employee_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_salary_components_history(
    pool: &PgPool,
    employee_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT 
            esc.id as esc_id, esc.employee_id, esc.component_definition_id, esc.amount, esc.effective_date, esc.created_at as "esc_created_at!", esc.updated_at as "esc_updated_at!",
            scd.id as scd_id, scd.workspace_id, scd.name, scd.code, scd.is_base_for_ot, scd.is_taxable, scd.created_at as "scd_created_at!", scd.updated_at as "scd_updated_at!"
           FROM employee_salary_components esc
           JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
           WHERE esc.employee_id = $1
           AND (
               esc.effective_date BETWEEN $2 AND $3
               OR esc.effective_date = (
                   SELECT MAX(effective_date) 
                   FROM employee_salary_components esc2 
                   WHERE esc2.employee_id = esc.employee_id 
                   AND esc2.component_definition_id = esc.component_definition_id
                   AND esc2.effective_date < $2
               )
           )
           ORDER BY esc.effective_date ASC"#,
        employee_id,
        start_date,
        end_date
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            (
                EmployeeSalaryComponent {
                    id: row.esc_id,
                    employee_id: row.employee_id,
                    component_definition_id: row.component_definition_id,
                    amount: row.amount,
                    effective_date: row.effective_date,
                    created_at: row.esc_created_at,
                    updated_at: row.esc_updated_at,
                },
                SalaryComponentDefinition {
                    id: row.scd_id,
                    workspace_id: row.workspace_id,
                    name: row.name,
                    code: row.code,
                    is_base_for_ot: row.is_base_for_ot,
                    is_taxable: row.is_taxable,
                    created_at: row.scd_created_at,
                    updated_at: row.scd_updated_at,
                },
            )
        })
        .collect())
}

#[allow(clippy::too_many_arguments)]
pub async fn create_payslip_segment(
    pool: &PgPool,
    payslip_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
    department_id: Option<Uuid>,
    applied_base_salary: Decimal,
    applied_std_days: Decimal,
    applied_hours_per_day: Decimal,
    actual_work_days: Decimal,
    paid_work_days: Decimal,
    amount: Decimal,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO payslip_segments (id, payslip_id, start_date, end_date, department_id, applied_base_salary, applied_std_days, applied_hours_per_day, actual_work_days, paid_work_days, amount)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"#, 
        id,
        payslip_id,
        start_date,
        end_date,
        department_id,
        applied_base_salary,
        applied_std_days,
        applied_hours_per_day,
        actual_work_days,
        paid_work_days,
        amount
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn get_payslip_segments(
    pool: &PgPool,
    payslip_id: Uuid,
) -> Result<Vec<PayslipSegment>, sqlx::Error> {
    // Note: We don't have 'reason' column yet, so we select NULL as reason
    query_as!(
        PayslipSegment,
        r#"SELECT id, payslip_id, start_date, end_date, department_id, applied_base_salary, applied_std_days, applied_hours_per_day, actual_work_days, paid_work_days, amount, NULL::text as reason
           FROM payslip_segments
           WHERE payslip_id = $1
           ORDER BY start_date ASC"#,
        payslip_id
    )
    .fetch_all(pool)
    .await
}
