use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::payroll::*;
use crate::models::{
    EmployeeSalaryComponent, IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy,
    PayrollSettings, Payslip, PayslipRuleEvaluation, PayslipSegment, SalaryComponentDefinition,
};
use crate::repositories::payroll_repository::*;

use crate::models::diesel::payroll::{
    EmployeeSalaryComponentEntity, SalaryComponentDefinitionEntity,
};
use crate::models::diesel::payroll_rule::PayrollRuleTierEntity;
use crate::repositories::diesel_department_repository::DepartmentPayrollConfigEntity;
use crate::schema::*;
use async_trait::async_trait;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

pub struct DieselPayrollRepository {
    pub diesel_pools: DieselPools,
}

impl DieselPayrollRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
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

    async fn create_incentive_policy(
        &self,
        workspace_id: Uuid,
        req: &crate::models::CreatePayrollIncentivePolicyRequest,
    ) -> Result<PayrollIncentivePolicy, AppError> {
        use crate::schema::{payroll_incentive_policies, payroll_incentive_tiers};

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let policy_id = crate::utils::id::new_id();
        let entity = crate::models::diesel::payroll::PayrollIncentivePolicyEntity {
            id: policy_id,
            workspace_id,
            name: req.name.clone(),
            base_amount: req.base_amount.to_f64().unwrap_or_default(),
            payout_type: req.payout_type.clone(),
            is_active: true,
            effective_from: req.effective_from,
            effective_to: None,
            created_at: Utc::now(),
        };

        let entity = conn
            .transaction::<_, diesel::result::Error, _>(|mut conn| {
                Box::pin(async move {
                    diesel::insert_into(payroll_incentive_policies::table)
                        .values(&entity)
                        .execute(&mut conn)
                        .await?;

                    for tier in &req.tiers {
                        let tier_id = crate::utils::id::new_id();
                        let tier_entity = crate::models::diesel::payroll::IncentiveTierEntity {
                            id: tier_id,
                            policy_id,
                            priority: tier.priority,
                            rule_config: serde_json::to_value(&tier.rule_config)
                                .unwrap_or_default(),
                            payout_amount: tier.payout_amount.to_f64().unwrap_or_default(),
                        };
                        diesel::insert_into(payroll_incentive_tiers::table)
                            .values(&tier_entity)
                            .execute(&mut conn)
                            .await?;
                    }
                    Ok(entity)
                })
            })
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut policy: PayrollIncentivePolicy = entity.into();
        policy.effective_from = req.effective_from;
        Ok(policy)
    }

    async fn delete_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<(), AppError> {
        use crate::schema::payroll_incentive_policies;

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(
            payroll_incentive_policies::table
                .filter(payroll_incentive_policies::id.eq(policy_id))
                .filter(payroll_incentive_policies::workspace_id.eq(workspace_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn clone_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<PayrollIncentivePolicy, AppError> {
        use crate::schema::{payroll_incentive_policies, payroll_incentive_tiers};

        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let (cloned_entity, _tiers) = conn
            .transaction::<_, AppError, _>(|mut conn| {
                Box::pin(async move {
                    // 1. Fetch original
                    let original_policy = payroll_incentive_policies::table
                        .filter(payroll_incentive_policies::id.eq(policy_id))
                        .filter(payroll_incentive_policies::workspace_id.eq(workspace_id))
                        .first::<crate::models::diesel::payroll::PayrollIncentivePolicyEntity>(
                            &mut conn,
                        )
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let original_tiers = payroll_incentive_tiers::table
                        .filter(payroll_incentive_tiers::policy_id.eq(policy_id))
                        .load::<crate::models::diesel::payroll::IncentiveTierEntity>(&mut conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 2. Insert new policy
                    let new_policy_id = crate::utils::id::new_id();
                    let new_name = format!("{} (Copy)", original_policy.name);

                    let new_policy_entity =
                        crate::models::diesel::payroll::PayrollIncentivePolicyEntity {
                            id: new_policy_id,
                            workspace_id,
                            name: new_name,
                            base_amount: original_policy.base_amount,
                            payout_type: original_policy.payout_type,
                            is_active: false,
                            effective_from: chrono::Utc::now().date_naive(),
                            effective_to: None,
                            created_at: Utc::now(),
                        };

                    diesel::insert_into(payroll_incentive_policies::table)
                        .values(&new_policy_entity)
                        .execute(&mut conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    // 3. Insert tiers
                    let new_tiers: Vec<_> = original_tiers
                        .into_iter()
                        .map(|tier| crate::models::diesel::payroll::IncentiveTierEntity {
                            id: crate::utils::id::new_id(),
                            policy_id: new_policy_id,
                            priority: tier.priority,
                            rule_config: tier.rule_config,
                            payout_amount: tier.payout_amount,
                        })
                        .collect();

                    if !new_tiers.is_empty() {
                        diesel::insert_into(payroll_incentive_tiers::table)
                            .values(&new_tiers)
                            .execute(&mut conn)
                            .await
                            .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                    }

                    Ok((new_policy_entity, new_tiers))
                })
            })
            .await?;

        Ok(cloned_entity.into())
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

    async fn get_payroll_settings_history(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollSettings>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payroll_settings::table
            .filter(payroll_settings::workspace_id.eq(workspace_id))
            .order(payroll_settings::effective_date.desc())
            .load::<PayrollSettingsEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employee_id.eq(employee_id))
            .filter(employee_salary_components::effective_date.le(reference_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .distinct_on((
                employee_salary_components::employee_id,
                employee_salary_components::component_definition_id,
            ))
            .order((
                employee_salary_components::employee_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Last component before start_date
        let last_before = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employee_id.eq(employee_id))
            .filter(employee_salary_components::effective_date.lt(start_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .distinct_on((
                employee_salary_components::employee_id,
                employee_salary_components::component_definition_id,
            ))
            .order((
                employee_salary_components::employee_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Components within range
        let within_range = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employee_id.eq(employee_id))
            .filter(employee_salary_components::effective_date.between(start_date, end_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order(employee_salary_components::effective_date.asc())
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut all = last_before;
        all.extend(within_range);
        all.sort_by_key(|(esc, _)| esc.effective_date);

        Ok(all
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
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

    async fn get_resolved_rule_assignments(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ResolvedRuleAssignment>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let sql = r#"
            WITH resolved_assignments AS (
                SELECT DISTINCT ON (rule_definition_id)
                    rule_definition_id, override_config, assignee_type
                FROM payroll_rule_assignments
                WHERE workspace_id = $1
                  AND effective_from <= $5
                  AND (effective_to IS NULL OR effective_to >= $4)
                  AND (
                      (assignee_type = 'employee' AND assignee_id = $2)
                      OR (assignee_type = 'department' AND assignee_id = $3)
                      OR (assignee_type = 'workspace')
                  )
                ORDER BY rule_definition_id, 
                         CASE assignee_type 
                            WHEN 'employee' THEN 1 
                            WHEN 'department' THEN 2 
                            WHEN 'workspace' THEN 3 
                         END
            )
            SELECT 
                r.rule_definition_id,
                r.override_config,
                d.code,
                d.calculation_method as "calculation_method",
                d.rule_config as default_config,
                d.category as "category"
            FROM resolved_assignments r
            JOIN payroll_rule_definitions d ON d.id = r.rule_definition_id
            WHERE d.is_active = true
            ORDER BY d.display_order
        "#;

        #[derive(QueryableByName)]
        struct ResolvedRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            pub rule_definition_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            pub override_config: Option<serde_json::Value>,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub code: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub calculation_method: String,
            #[diesel(sql_type = diesel::sql_types::Jsonb)]
            pub default_config: serde_json::Value,
            #[diesel(sql_type = diesel::sql_types::Text)]
            pub category: String,
        }

        let rows = diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(workspace_id)
            .bind::<diesel::sql_types::Uuid, _>(employee_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(department_id)
            .bind::<diesel::sql_types::Date, _>(start_date)
            .bind::<diesel::sql_types::Date, _>(end_date)
            .load::<ResolvedRow>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|r| ResolvedRuleAssignment {
                rule_definition_id: r.rule_definition_id,
                override_config: r.override_config,
                code: r.code,
                calculation_method: r.calculation_method.parse().unwrap_or_default(),
                default_config: r.default_config,
                category: r.category.parse().unwrap_or_default(),
            })
            .collect())
    }

    async fn get_payroll_rule_tiers(
        &self,
        rule_definition_id: Uuid,
    ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = payroll_rule_tiers::table
            .filter(payroll_rule_tiers::rule_definition_id.eq(rule_definition_id))
            .order(payroll_rule_tiers::tier_order.asc())
            .load::<PayrollRuleTierEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_department_payroll_config_at(
        &self,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let result = crate::schema::department_payroll_configs::table
            .filter(crate::schema::department_payroll_configs::department_id.eq(department_id))
            .filter(crate::schema::department_payroll_configs::effective_date.le(date))
            .order(crate::schema::department_payroll_configs::effective_date.desc())
            .select(DepartmentPayrollConfigEntity::as_select())
            .first::<DepartmentPayrollConfigEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.map(Into::into))
    }
}
