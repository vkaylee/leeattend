use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::{query_as, PgPool, Row};
use uuid::Uuid;

use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipSegment, SalaryComponentDefinition,
};

pub async fn get_active_incentive_policies(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<PayrollIncentivePolicy>, sqlx::Error> {
    let rows = sqlx::query(
        r#"SELECT id, workspace_id, name, base_amount, payout_type, is_active, created_at
           FROM payroll_incentive_policies 
           WHERE workspace_id = $1 AND is_active = TRUE"#,
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| PayrollIncentivePolicy {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            name: row.get("name"),
            base_amount: row.get("base_amount"),
            payout_type: row.get("payout_type"),
            is_active: row.get("is_active"),
            created_at: DateTime::from_naive_utc_and_offset(row.get("created_at"), Utc),
        })
        .collect())
}

pub async fn get_policy_tiers(
    pool: &PgPool,
    policy_id: Uuid,
) -> Result<Vec<IncentiveTier>, sqlx::Error> {
    let rows = sqlx::query(
        r#"SELECT id, policy_id, priority, max_late_early_count, max_unpaid_leave_days, max_sick_leave_days, max_total_leave_days, payout_amount
           FROM payroll_incentive_tiers 
           WHERE policy_id = $1
           ORDER BY priority ASC"#,
    )
    .bind(policy_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IncentiveTier {
            id: row.get("id"),
            policy_id: row.get("policy_id"),
            priority: row.get("priority"),
            max_late_early_count: row.get("max_late_early_count"),
            max_unpaid_leave_days: row.get::<Option<f64>, _>("max_unpaid_leave_days"),
            max_sick_leave_days: row.get::<Option<f64>, _>("max_sick_leave_days"),
            max_total_leave_days: row.get::<Option<f64>, _>("max_total_leave_days"),
            payout_amount: row.get::<f64, _>("payout_amount"),
        })
        .collect())
}

pub async fn create_payslip_incentive(
    pool: &PgPool,
    payslip_id: Uuid,
    result: &IncentiveEvaluationResult,
) -> Result<(), sqlx::Error> {
    let amount_f64 = result.amount.to_f64().unwrap_or_default();
    let standard_amount_f64 = result.standard_amount.to_f64().unwrap_or_default();
    sqlx::query(
        r#"INSERT INTO payslip_incentives (payslip_id, policy_name, amount, tier_id, standard_amount)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(payslip_id)
    .bind(&result.policy_name)
    .bind(amount_f64)
    .bind(result.tier_id)
    .bind(standard_amount_f64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_payslip_incentives(
    pool: &PgPool,
    payslip_id: Uuid,
) -> Result<Vec<IncentiveEvaluationResult>, sqlx::Error> {
    let rows = sqlx::query(
        r#"SELECT policy_name, amount, tier_id, standard_amount FROM payslip_incentives WHERE payslip_id = $1"#,
    )
    .bind(payslip_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| IncentiveEvaluationResult {
            policy_name: row.get("policy_name"),
            amount: Decimal::from_f64(row.get("amount")).unwrap_or_default(),
            standard_amount: Decimal::from_f64(row.get("standard_amount")).unwrap_or_default(),
            tier_id: row.get("tier_id"),
        })
        .collect())
}

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
        r#"SELECT workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, max_ot_hours_per_day, max_ot_hours_per_month, effective_date, created_at as "created_at!", updated_at as "updated_at!" 
           FROM payroll_settings 
           WHERE workspace_id = $1 AND effective_date <= $2
           ORDER BY effective_date DESC LIMIT 1"#,
        workspace_id,
        date
    )
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn update_payroll_settings(
    pool: &PgPool,
    workspace_id: Uuid,
    std_working_day_mode: String,
    fixed_std_days: Decimal,
    working_days_per_week: i32,
    hours_per_day: Decimal,
    max_ot_hours_per_day: Option<Decimal>,
    max_ot_hours_per_month: Option<Decimal>,
    effective_date: NaiveDate,
) -> Result<PayrollSettings, sqlx::Error> {
    query_as!(
        PayrollSettings,
        r#"INSERT INTO payroll_settings (workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, max_ot_hours_per_day, max_ot_hours_per_month, effective_date, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
           ON CONFLICT (workspace_id, effective_date) DO UPDATE 
           SET std_working_day_mode = EXCLUDED.std_working_day_mode,
               fixed_std_days = EXCLUDED.fixed_std_days,
               working_days_per_week = EXCLUDED.working_days_per_week,
               hours_per_day = EXCLUDED.hours_per_day,
               max_ot_hours_per_day = EXCLUDED.max_ot_hours_per_day,
               max_ot_hours_per_month = EXCLUDED.max_ot_hours_per_month,
               updated_at = EXCLUDED.updated_at
           RETURNING workspace_id, std_working_day_mode, fixed_std_days, working_days_per_week, hours_per_day, max_ot_hours_per_day, max_ot_hours_per_month, effective_date, created_at as "created_at!", updated_at as "updated_at!""#,
        workspace_id,
        std_working_day_mode,
        fixed_std_days,
        working_days_per_week,
        hours_per_day,
        max_ot_hours_per_day,
        max_ot_hours_per_month,
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
    let rows: Vec<_> = sqlx::query!(
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

pub async fn get_payslips_by_workspace_period(
    pool: &PgPool,
    workspace_id: Uuid,
    period_start: NaiveDate,
    period_end: NaiveDate,
) -> Result<Vec<Payslip>, sqlx::Error> {
    query_as!(
        Payslip,
        r#"SELECT p.id, p.employee_id, p.period_start, p.period_end, p.base_pay_amount, p.overtime_pay_amount, p.deductions_amount, p.net_pay_amount, p.standard_units, p.overtime_units, p.status, p.created_at as "created_at!", p.updated_at as "updated_at!" 
           FROM payslips p
           JOIN employees e ON e.id = p.employee_id
           WHERE e.workspace_id = $1 AND p.period_start = $2 AND p.period_end = $3
           ORDER BY p.created_at DESC"#,
        workspace_id,
        period_start,
        period_end
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
    let rows: Vec<_> = sqlx::query!(
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
    hours_per_day: Decimal,
    actual_work_days: Decimal,
    paid_work_days: Decimal,
    amount: Decimal,
    std_days_source: Option<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO payslip_segments (
            payslip_id, start_date, end_date, department_id,
            applied_base_salary, applied_std_days, applied_hours_per_day,
            actual_work_days, paid_work_days, amount, std_days_source
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(payslip_id)
    .bind(start_date)
    .bind(end_date)
    .bind(department_id)
    .bind(applied_base_salary)
    .bind(applied_std_days)
    .bind(hours_per_day)
    .bind(actual_work_days)
    .bind(paid_work_days)
    .bind(amount)
    .bind(std_days_source)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_payslip_segments(
    pool: &PgPool,
    payslip_id: Uuid,
) -> Result<Vec<PayslipSegment>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, payslip_id, start_date, end_date, department_id, applied_base_salary, applied_std_days, applied_hours_per_day, actual_work_days, paid_work_days, amount, NULL as reason, std_days_source
           FROM payslip_segments
           WHERE payslip_id = $1
           ORDER BY start_date ASC"#,
    )
    .bind(payslip_id)
    .fetch_all(pool)
    .await
}
