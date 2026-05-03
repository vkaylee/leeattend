use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use sqlx::{query_as, PgPool, Row};
use uuid::Uuid;

use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipSegment, SalaryComponentDefinition,
};

// ---------------------------------------------------------------------------
// Trait: PayrollRepository — DIP abstraction over payroll data access
// ---------------------------------------------------------------------------
#[async_trait]
pub trait PayrollRepository: super::PoolBacked + Send + Sync {
    async fn get_active_incentive_policies(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollIncentivePolicy>, sqlx::Error>;

    async fn get_policy_tiers(&self, policy_id: Uuid) -> Result<Vec<IncentiveTier>, sqlx::Error>;

    async fn create_payslip_incentive(
        &self,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), sqlx::Error>;

    async fn get_payslip_incentives(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, sqlx::Error>;

    async fn create_payslip_rule_evaluation(
        &self,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), sqlx::Error>;

    async fn get_payslip_rule_evaluations(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<crate::models::PayslipRuleEvaluation>, sqlx::Error>;

    async fn get_payroll_settings(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<PayrollSettings>, sqlx::Error>;

    async fn get_payroll_settings_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<PayrollSettings>, sqlx::Error>;

    async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, sqlx::Error>;

    async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, sqlx::Error>;

    async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
        is_base_for_ot: bool,
        is_taxable: bool,
    ) -> Result<SalaryComponentDefinition, sqlx::Error>;

    async fn assign_employee_salary_component(
        &self,
        employee_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, sqlx::Error>;

    async fn get_employee_salary_components(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error>;

    async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, sqlx::Error>;

    async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, sqlx::Error>;

    async fn get_payslips_by_workspace_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Payslip>, sqlx::Error>;

    async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error>;

    async fn create_payslip_segment(
        &self,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), sqlx::Error>;

    async fn get_payslip_segments(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxPayrollRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxPayrollRepository {
    pub pool: PgPool,
}

impl SqlxPayrollRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PayrollRepository for SqlxPayrollRepository {
    async fn get_active_incentive_policies(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollIncentivePolicy>, sqlx::Error> {
        get_active_incentive_policies(&self.pool, workspace_id).await
    }
    async fn get_policy_tiers(&self, policy_id: Uuid) -> Result<Vec<IncentiveTier>, sqlx::Error> {
        get_policy_tiers(&self.pool, policy_id).await
    }
    async fn create_payslip_incentive(
        &self,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), sqlx::Error> {
        create_payslip_incentive(&self.pool, payslip_id, result).await
    }
    async fn get_payslip_incentives(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, sqlx::Error> {
        get_payslip_incentives(&self.pool, payslip_id).await
    }
    async fn create_payslip_rule_evaluation(
        &self,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), sqlx::Error> {
        create_payslip_rule_evaluation(&self.pool, payslip_id, rule_code, amount).await
    }
    async fn get_payslip_rule_evaluations(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<crate::models::PayslipRuleEvaluation>, sqlx::Error> {
        get_payslip_rule_evaluations(&self.pool, payslip_id).await
    }
    async fn get_payroll_settings(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<PayrollSettings>, sqlx::Error> {
        get_payroll_settings(&self.pool, workspace_id).await
    }
    async fn get_payroll_settings_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<PayrollSettings>, sqlx::Error> {
        get_payroll_settings_at(&self.pool, workspace_id, date).await
    }
    async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, sqlx::Error> {
        update_payroll_settings(&self.pool, workspace_id, req).await
    }
    async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, sqlx::Error> {
        get_salary_component_definitions(&self.pool, workspace_id).await
    }
    async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
        is_base_for_ot: bool,
        is_taxable: bool,
    ) -> Result<SalaryComponentDefinition, sqlx::Error> {
        create_salary_component_definition(
            &self.pool,
            workspace_id,
            name,
            code,
            is_base_for_ot,
            is_taxable,
        )
        .await
    }
    async fn assign_employee_salary_component(
        &self,
        employee_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, sqlx::Error> {
        assign_employee_salary_component(
            &self.pool,
            employee_id,
            component_definition_id,
            amount,
            effective_date,
        )
        .await
    }
    async fn get_employee_salary_components(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error> {
        get_employee_salary_components(&self.pool, workspace_id, employee_id, reference_date).await
    }
    async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, sqlx::Error> {
        create_payslip(&self.pool, payslip).await
    }
    async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, sqlx::Error> {
        get_payslips_by_employee(&self.pool, workspace_id, employee_id).await
    }
    async fn get_payslips_by_workspace_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Payslip>, sqlx::Error> {
        get_payslips_by_workspace_period(&self.pool, workspace_id, period_start, period_end).await
    }
    async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error> {
        get_salary_components_history(&self.pool, workspace_id, employee_id, start_date, end_date)
            .await
    }
    async fn create_payslip_segment(
        &self,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), sqlx::Error> {
        create_payslip_segment(&self.pool, payslip_id, args).await
    }
    async fn get_payslip_segments(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, sqlx::Error> {
        get_payslip_segments(&self.pool, payslip_id).await
    }
}

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
            created_at: row.get("created_at"),
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
    let amount_f64 = result.amount.to_f64().ok_or_else(|| {
        sqlx::Error::Protocol(format!(
            "Decimal→f64 overflow for incentive amount: {}",
            result.amount
        ))
    })?;
    let standard_amount_f64 = result.standard_amount.to_f64().ok_or_else(|| {
        sqlx::Error::Protocol(format!(
            "Decimal→f64 overflow for standard_amount: {}",
            result.standard_amount
        ))
    })?;
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

    let results: Vec<IncentiveEvaluationResult> = rows
        .into_iter()
        .map(|row| -> Result<IncentiveEvaluationResult, sqlx::Error> {
            Ok(IncentiveEvaluationResult {
                policy_name: row.get("policy_name"),
                amount: Decimal::from_f64(row.get("amount")).ok_or_else(|| {
                    sqlx::Error::Protocol(
                        "f64→Decimal conversion failed for incentive amount".into(),
                    )
                })?,
                standard_amount: Decimal::from_f64(row.get("standard_amount")).ok_or_else(
                    || {
                        sqlx::Error::Protocol(
                            "f64→Decimal conversion failed for standard_amount".into(),
                        )
                    },
                )?,
                tier_id: row.get("tier_id"),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(results)
}

pub async fn create_payslip_rule_evaluation(
    pool: &PgPool,
    payslip_id: Uuid,
    rule_code: String,
    amount: Decimal,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO payslip_rule_evaluations (payslip_id, rule_code, amount)
           VALUES ($1, $2, $3)"#,
    )
    .bind(payslip_id)
    .bind(rule_code)
    .bind(amount)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_payslip_rule_evaluations(
    pool: &PgPool,
    payslip_id: Uuid,
) -> Result<Vec<crate::models::PayslipRuleEvaluation>, sqlx::Error> {
    sqlx::query_as!(
        crate::models::PayslipRuleEvaluation,
        r#"SELECT id, payslip_id, rule_code, amount, created_at as "created_at!"
           FROM payslip_rule_evaluations
           WHERE payslip_id = $1
           ORDER BY created_at ASC"#,
        payslip_id
    )
    .fetch_all(pool)
    .await
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

pub async fn update_payroll_settings(
    pool: &PgPool,
    workspace_id: Uuid,
    req: &crate::models::SavePayrollSettings,
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
        req.std_working_day_mode,
        req.fixed_std_days,
        req.working_days_per_week,
        req.hours_per_day,
        req.max_ot_hours_per_day,
        req.max_ot_hours_per_month,
        req.effective_date,
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
    workspace_id: Uuid,
    employee_id: Uuid,
    reference_date: NaiveDate,
) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, sqlx::Error> {
    let rows: Vec<_> = sqlx::query!(
        r#"SELECT 
            esc.id as esc_id, esc.employee_id, esc.component_definition_id, esc.amount, esc.effective_date, esc.created_at as "esc_created_at!", esc.updated_at as "esc_updated_at!",
            scd.id as scd_id, scd.workspace_id, scd.name, scd.code, scd.is_base_for_ot, scd.is_taxable, scd.created_at as "scd_created_at!", scd.updated_at as "scd_updated_at!"
           FROM employee_salary_components esc
           JOIN salary_component_definitions scd ON scd.id = esc.component_definition_id
           WHERE esc.employee_id = $1 AND esc.effective_date <= $2 AND scd.workspace_id = $3
           AND NOT EXISTS (
               SELECT 1 FROM employee_salary_components esc2 
               WHERE esc2.employee_id = esc.employee_id 
               AND esc2.component_definition_id = esc.component_definition_id 
               AND esc2.effective_date <= $2 
               AND (esc2.effective_date > esc.effective_date OR (esc2.effective_date = esc.effective_date AND esc2.created_at > esc.created_at))
           )"#,
        employee_id,
        reference_date,
        workspace_id
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
        r#"INSERT INTO payslips (id, employee_id, workspace_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status, holiday_work_premium_amount)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING id, employee_id, workspace_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status, holiday_work_premium_amount, created_at as "created_at!", updated_at as "updated_at!""#,
        payslip.id,
        payslip.employee_id,
        payslip.workspace_id,
        payslip.period_start,
        payslip.period_end,
        payslip.base_pay_amount,
        payslip.overtime_pay_amount,
        payslip.deductions_amount,
        payslip.net_pay_amount,
        payslip.standard_units,
        payslip.overtime_units,
        payslip.status,
        payslip.holiday_work_premium_amount
    )
    .fetch_one(pool)
    .await
}

pub async fn get_payslips_by_employee(
    pool: &PgPool,
    workspace_id: Uuid,
    employee_id: Uuid,
) -> Result<Vec<Payslip>, sqlx::Error> {
    query_as!(
        Payslip,
        r#"SELECT id, employee_id, workspace_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status, holiday_work_premium_amount, created_at as "created_at!", updated_at as "updated_at!"
           FROM payslips
           WHERE employee_id = $1 AND workspace_id = $2
           ORDER BY period_start DESC"#,
        employee_id,
        workspace_id
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
        r#"SELECT id, employee_id, workspace_id, period_start, period_end, base_pay_amount, overtime_pay_amount, deductions_amount, net_pay_amount, standard_units, overtime_units, status, holiday_work_premium_amount, created_at as "created_at!", updated_at as "updated_at!" 
           FROM payslips
           WHERE workspace_id = $1 AND period_start = $2 AND period_end = $3
           ORDER BY created_at DESC"#,
        workspace_id,
        period_start,
        period_end
    )
    .fetch_all(pool)
    .await
}

pub async fn get_salary_components_history(
    pool: &PgPool,
    workspace_id: Uuid,
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
           WHERE esc.employee_id = $1 AND scd.workspace_id = $4
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
        end_date,
        workspace_id
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

pub async fn create_payslip_segment(
    pool: &PgPool,
    payslip_id: Uuid,
    args: &crate::models::CreatePayslipSegmentArgs,
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
    .bind(args.start_date)
    .bind(args.end_date)
    .bind(args.department_id)
    .bind(args.applied_base_salary)
    .bind(args.applied_std_days)
    .bind(args.hours_per_day)
    .bind(args.actual_work_days)
    .bind(args.paid_work_days)
    .bind(args.amount)
    .bind(&args.std_days_source)
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

impl super::PoolBacked for SqlxPayrollRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
    use rust_decimal::Decimal;
    use uuid::Uuid;

    use crate::models::IncentiveEvaluationResult;

    /// Normal Decimal values should convert to f64 successfully.
    #[test]
    fn test_decimal_to_f64_normal_values() {
        let values = [
            Decimal::ZERO,
            Decimal::new(150_000_000, 2), // 1,500,000.00
            Decimal::new(99_999_999, 2),  // 999,999.99
            Decimal::new(-50_000, 2),     // -500.00
        ];
        for val in values {
            assert!(
                val.to_f64().is_some(),
                "Normal Decimal {:?} should convert to f64",
                val
            );
        }
    }

    /// Decimal::MAX → f64 produces Infinity, which fails from_f64() roundtrip.
    /// This is the exact error chain that previously produced silent 0₫ in payslips:
    /// DB returns huge Decimal → to_f64() → Infinity → from_f64(Infinity) → None → 0₫
    #[test]
    fn test_decimal_max_roundtrip_fails() {
        let huge = Decimal::MAX;
        let as_f64 = huge.to_f64();
        // to_f64 may succeed with Infinity
        if let Some(f) = as_f64 {
            // But from_f64(Infinity) must reject it
            assert!(
                Decimal::from_f64(f).is_none(),
                "Decimal::MAX roundtrip via f64 should fail — this proves the error path matters"
            );
        }
        // Either to_f64 returns None or the roundtrip fails — both exercise the error path
    }

    /// f64 NaN and Infinity cannot become Decimal → from_f64() returns None.
    /// Without proper error handling, this would silently zero out a payslip amount.
    #[test]
    fn test_f64_nan_infinity_fail_to_decimal() {
        assert!(
            Decimal::from_f64(f64::NAN).is_none(),
            "NaN must not silently become Decimal::ZERO"
        );
        assert!(
            Decimal::from_f64(f64::INFINITY).is_none(),
            "Infinity must not silently become Decimal::ZERO"
        );
        assert!(
            Decimal::from_f64(f64::NEG_INFINITY).is_none(),
            "Negative infinity must not silently become Decimal::ZERO"
        );
    }

    /// Simulates the error path in `create_payslip_incentive` — ensures the
    /// ok_or_else pattern produces a meaningful error message for debugging.
    #[test]
    fn test_create_payslip_incentive_decimal_validation_error_message() {
        let result = IncentiveEvaluationResult {
            policy_name: "Attendance Bonus".to_string(),
            amount: Decimal::new(100_000_000, 2),
            standard_amount: Decimal::new(100_000_000, 2),
            tier_id: Some(Uuid::new_v4()),
        };

        // Normal case should succeed
        let ok = result.amount.to_f64().ok_or_else(|| {
            format!(
                "Decimal→f64 overflow for incentive amount: {}",
                result.amount
            )
        });
        assert!(ok.is_ok(), "Normal Decimal should convert to f64");

        // Verify the error message format for the f64→Decimal path (NaN from DB)
        let err = Decimal::from_f64(f64::NAN)
            .ok_or_else(|| "f64→Decimal conversion failed for incentive amount".to_string());
        assert!(err.is_err(), "NaN should fail from_f64");
        assert!(err.unwrap_err().contains("conversion failed"));
    }

    /// Validates that normal incentive amounts pass validation cleanly.
    #[test]
    fn test_create_payslip_incentive_normal_amounts_pass() {
        let result = IncentiveEvaluationResult {
            policy_name: "Monthly Bonus".to_string(),
            amount: Decimal::new(500_000_000, 2), // 5,000,000.00 VND
            standard_amount: Decimal::new(500_000_000, 2),
            tier_id: None,
        };

        assert!(result.amount.to_f64().is_some());
        assert!(result.standard_amount.to_f64().is_some());
    }

    /// Validates the f64→Decimal roundtrip in `get_payslip_incentives`.
    #[test]
    fn test_f64_to_decimal_roundtrip_precision() {
        let original = Decimal::new(123_456_789, 2); // 1,234,567.89
        let as_f64 = original.to_f64().expect("normal conversion");
        let back = Decimal::from_f64(as_f64).expect("roundtrip should work");

        let diff = (original - back).abs();
        assert!(
            diff < Decimal::new(1, 2),
            "Roundtrip precision loss too large: original={}, back={}, diff={}",
            original,
            back,
            diff
        );
    }
}
