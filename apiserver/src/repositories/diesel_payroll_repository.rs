use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::payroll::*;
use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipRuleEvaluation, PayslipSegment, SalaryComponentDefinition,
};
use crate::repositories::payroll_repository::*;

use crate::schema::*;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselPayrollRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselPayrollRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl PayrollRepository for DieselPayrollRepository {
    async fn get_active_incentive_policies(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payroll_incentive_policies::table
            .filter(payroll_incentive_policies::workspace_id.eq(workspace_id))
            .filter(payroll_incentive_policies::is_active.eq(true))
            .load::<PayrollIncentivePolicyEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_active_incentive_policies_for_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payroll_incentive_policies::table
            .filter(payroll_incentive_policies::workspace_id.eq(workspace_id))
            .filter(payroll_incentive_policies::is_active.eq(true))
            .filter(payroll_incentive_policies::effective_from.le(period_end))
            .filter(
                payroll_incentive_policies::effective_to
                    .is_null()
                    .or(payroll_incentive_policies::effective_to.ge(period_start)),
            )
            .load::<PayrollIncentivePolicyEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_policy_tiers(&self, policy_id: Uuid) -> Result<Vec<IncentiveTier>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payroll_incentive_tiers::table
            .filter(payroll_incentive_tiers::policy_id.eq(policy_id))
            .order(payroll_incentive_tiers::priority.asc())
            .load::<IncentiveTierEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn create_payslip_incentive(
        &self,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = PayslipIncentiveEntity {
            id: Uuid::new_v4(),
            payslip_id,
            policy_name: result.policy_name.clone(),
            amount: result.amount.to_f64().unwrap_or_default(),
            tier_id: result.tier_id,
            standard_amount: result.standard_amount.to_f64().unwrap_or_default(),
            created_at: Utc::now(),
        };

        diesel::insert_into(payslip_incentives::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_payslip_incentives(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payslip_incentives::table
            .filter(payslip_incentives::payslip_id.eq(payslip_id))
            .load::<PayslipIncentiveEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities
            .into_iter()
            .map(|e| IncentiveEvaluationResult {
                policy_name: e.policy_name,
                amount: Decimal::from_f64(e.amount).unwrap_or_default(),
                tier_id: e.tier_id,
                standard_amount: Decimal::from_f64(e.standard_amount).unwrap_or_default(),
            })
            .collect())
    }

    async fn create_payslip_rule_evaluation(
        &self,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = PayslipRuleEvaluationEntity {
            id: Uuid::new_v4(),
            payslip_id,
            rule_code,
            amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
            created_at: Utc::now(),
        };

        diesel::insert_into(payslip_rule_evaluations::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_payslip_rule_evaluations(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipRuleEvaluation>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payslip_rule_evaluations::table
            .filter(payslip_rule_evaluations::payslip_id.eq(payslip_id))
            .select(PayslipRuleEvaluationEntity::as_select())
            .load::<PayslipRuleEvaluationEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_payroll_settings(
        &self,
        workspace_id: Uuid,
    ) -> Result<Option<PayrollSettings>, AppError> {
        self.get_payroll_settings_at(workspace_id, Utc::now().date_naive())
            .await
    }

    async fn get_payroll_settings_at(
        &self,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<PayrollSettings>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = payroll_settings::table
            .filter(payroll_settings::workspace_id.eq(workspace_id))
            .filter(payroll_settings::effective_date.le(date))
            .order(payroll_settings::effective_date.desc())
            .first::<PayrollSettingsEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = PayrollSettingsEntity {
            workspace_id,
            std_working_day_mode: req.std_working_day_mode.clone(),
            fixed_std_days: bigdecimal::BigDecimal::from_str(&req.fixed_std_days.to_string())
                .unwrap_or_default(),
            working_days_per_week: req.working_days_per_week,
            hours_per_day: bigdecimal::BigDecimal::from_str(&req.hours_per_day.to_string())
                .unwrap_or_default(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            effective_date: req.effective_date,
            max_ot_hours_per_day: req
                .max_ot_hours_per_day
                .map(|v| bigdecimal::BigDecimal::from_str(&v.to_string()).unwrap_or_default()),
            max_ot_hours_per_month: req
                .max_ot_hours_per_month
                .map(|v| bigdecimal::BigDecimal::from_str(&v.to_string()).unwrap_or_default()),
        };

        let result = diesel::insert_into(payroll_settings::table)
            .values(&entity)
            .on_conflict((
                payroll_settings::workspace_id,
                payroll_settings::effective_date,
            ))
            .do_update()
            .set(&entity)
            .get_result::<PayrollSettingsEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = salary_component_definitions::table
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .select(SalaryComponentDefinitionEntity::as_select())
            .load::<SalaryComponentDefinitionEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
    ) -> Result<SalaryComponentDefinition, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = SalaryComponentDefinitionEntity {
            id: Uuid::new_v4(),
            workspace_id,
            name,
            code,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            component_type: "earning".to_string(), // Default or from schema?
            linked_rule_id: None,
        };

        let result = diesel::insert_into(salary_component_definitions::table)
            .values(&entity)
            .get_result::<SalaryComponentDefinitionEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn assign_employee_salary_component(
        &self,
        employee_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = EmployeeSalaryComponentEntity {
            id: Uuid::new_v4(),
            employee_id,
            component_definition_id,
            amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
            effective_date,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let result = diesel::insert_into(employee_salary_components::table)
            .values(&entity)
            .get_result::<EmployeeSalaryComponentEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_employee_salary_components(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        get_employee_salary_components(&self.sqlx_pool, workspace_id, employee_id, reference_date)
            .await
            .map_err(Into::into)
    }

    async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, AppError> {
        use std::str::FromStr;
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = PayslipEntity {
            id: payslip.id,
            employee_id: payslip.employee_id,
            period_start: payslip.period_start,
            period_end: payslip.period_end,
            base_pay_amount: bigdecimal::BigDecimal::from_str(&payslip.base_pay_amount.to_string())
                .unwrap_or_default(),
            overtime_pay_amount: bigdecimal::BigDecimal::from_str(
                &payslip.overtime_pay_amount.to_string(),
            )
            .unwrap_or_default(),
            deductions_amount: bigdecimal::BigDecimal::from_str(
                &payslip.deductions_amount.to_string(),
            )
            .unwrap_or_default(),
            net_pay_amount: bigdecimal::BigDecimal::from_str(&payslip.net_pay_amount.to_string())
                .unwrap_or_default(),
            status: payslip.status.clone(),
            created_at: Some(payslip.created_at),
            updated_at: Some(payslip.updated_at),
            standard_units: payslip.standard_units,
            overtime_units: payslip.overtime_units,
            holiday_work_premium_amount: bigdecimal::BigDecimal::from_str(
                &payslip.holiday_work_premium_amount.to_string(),
            )
            .unwrap_or_default(),
            workspace_id: payslip.workspace_id,
        };

        let result = diesel::insert_into(payslips::table)
            .values(&entity)
            .get_result::<PayslipEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payslips::table
            .filter(payslips::workspace_id.eq(workspace_id))
            .filter(payslips::employee_id.eq(employee_id))
            .order(payslips::period_start.desc())
            .load::<PayslipEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_payslips_by_workspace_period(
        &self,
        workspace_id: Uuid,
        period_start: NaiveDate,
        period_end: NaiveDate,
    ) -> Result<Vec<Payslip>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payslips::table
            .filter(payslips::workspace_id.eq(workspace_id))
            .filter(payslips::period_start.eq(period_start))
            .filter(payslips::period_end.eq(period_end))
            .order(payslips::created_at.desc())
            .load::<PayslipEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        get_salary_components_history(
            &self.sqlx_pool,
            workspace_id,
            employee_id,
            start_date,
            end_date,
        )
        .await
        .map_err(Into::into)
    }

    async fn create_payslip_segment(
        &self,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), AppError> {
        use std::str::FromStr;
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = PayslipSegmentEntity {
            id: Uuid::new_v4(),
            payslip_id,
            start_date: args.start_date,
            end_date: args.end_date,
            department_id: args.department_id,
            applied_base_salary: bigdecimal::BigDecimal::from_str(
                &args.applied_base_salary.to_string(),
            )
            .unwrap_or_default(),
            applied_std_days: bigdecimal::BigDecimal::from_str(&args.applied_std_days.to_string())
                .unwrap_or_default(),
            actual_work_days: bigdecimal::BigDecimal::from_str(&args.actual_work_days.to_string())
                .unwrap_or_default(),
            paid_work_days: bigdecimal::BigDecimal::from_str(&args.paid_work_days.to_string())
                .unwrap_or_default(),
            amount: bigdecimal::BigDecimal::from_str(&args.amount.to_string()).unwrap_or_default(),
            created_at: Some(Utc::now()),
            applied_hours_per_day: bigdecimal::BigDecimal::from_str(
                &args.hours_per_day.to_string(),
            )
            .unwrap_or_default(),
            std_days_source: args.std_days_source.clone(),
        };

        diesel::insert_into(payslip_segments::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_payslip_segments(
        &self,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payslip_segments::table
            .filter(payslip_segments::payslip_id.eq(payslip_id))
            .load::<PayslipSegmentEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }
}
