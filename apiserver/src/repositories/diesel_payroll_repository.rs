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
    diesel_pools: DieselPools,
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = salary_component_base_members)]
struct SalaryBaseGroupMemberRow {
    pub component_definition_id: Uuid,
    pub base_group_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl DieselPayrollRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[allow(clippy::too_many_lines)]
#[async_trait]
impl PayrollRepository for DieselPayrollRepository {
    async fn get_active_incentive_policies(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollIncentivePolicy>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_incentive_policies::table
            .filter(payroll_incentive_policies::workspace_id.eq(workspace_id))
            .filter(payroll_incentive_policies::is_active.eq(true))
            .filter(payroll_incentive_policies::deleted_at.is_null())
            .load::<PayrollIncentivePolicyEntity>(&mut *conn)
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
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_incentive_policies::table
            .filter(payroll_incentive_policies::workspace_id.eq(workspace_id))
            .filter(payroll_incentive_policies::is_active.eq(true))
            .filter(payroll_incentive_policies::deleted_at.is_null())
            .filter(payroll_incentive_policies::effective_from.le(period_end))
            .filter(
                payroll_incentive_policies::effective_to
                    .is_null()
                    .or(payroll_incentive_policies::effective_to.ge(period_start)),
            )
            .load::<PayrollIncentivePolicyEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_policy_tiers(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<Vec<IncentiveTier>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_incentive_tiers::table
            .filter(payroll_incentive_tiers::policy_id.eq(policy_id))
            .order(payroll_incentive_tiers::priority.asc())
            .load::<IncentiveTierEntity>(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

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
            deleted_at: None,
        };

        let entity = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    diesel::insert_into(payroll_incentive_policies::table)
                        .values(&entity)
                        .execute(&mut *conn)
                        .await?;

                    if !req.tiers.is_empty() {
                        let tier_entities: Vec<_> = req
                            .tiers
                            .iter()
                            .map(|tier| crate::models::diesel::payroll::IncentiveTierEntity {
                                id: crate::utils::id::new_id(),
                                policy_id,
                                priority: tier.priority,
                                rule_config: serde_json::to_value(&tier.rule_config)
                                    .unwrap_or_default(),
                                payout_amount: tier.payout_amount.to_f64().unwrap_or_default(),
                            })
                            .collect();

                        diesel::insert_into(payroll_incentive_tiers::table)
                            .values(&tier_entities)
                            .execute(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        diesel::update(
            payroll_incentive_policies::table
                .filter(payroll_incentive_policies::id.eq(policy_id))
                .filter(payroll_incentive_policies::workspace_id.eq(workspace_id)),
        )
        .set(payroll_incentive_policies::deleted_at.eq(Some(Utc::now())))
        .execute(&mut *conn)
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

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let (cloned_entity, _tiers) = conn
            .transaction::<_, AppError, _>(|conn| {
                Box::pin(async move {
                    // 1. Fetch original
                    let original_policy = payroll_incentive_policies::table
                        .filter(payroll_incentive_policies::id.eq(policy_id))
                        .filter(payroll_incentive_policies::workspace_id.eq(workspace_id))
                        .filter(payroll_incentive_policies::deleted_at.is_null())
                        .first::<crate::models::diesel::payroll::PayrollIncentivePolicyEntity>(
                            &mut *conn,
                        )
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    let original_tiers = payroll_incentive_tiers::table
                        .filter(payroll_incentive_tiers::policy_id.eq(policy_id))
                        .load::<crate::models::diesel::payroll::IncentiveTierEntity>(&mut *conn)
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
                            deleted_at: None,
                        };

                    diesel::insert_into(payroll_incentive_policies::table)
                        .values(&new_policy_entity)
                        .execute(&mut *conn)
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
                            .execute(&mut *conn)
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
        workspace_id: Uuid,
        payslip_id: Uuid,
        result: &IncentiveEvaluationResult,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

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
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_payslip_incentives(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<IncentiveEvaluationResult>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslip_incentives::table
            .filter(payslip_incentives::payslip_id.eq(payslip_id))
            .load::<PayslipIncentiveEntity>(&mut *conn)
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
        workspace_id: Uuid,
        payslip_id: Uuid,
        rule_code: String,
        amount: Decimal,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = PayslipRuleEvaluationEntity {
            id: Uuid::new_v4(),
            payslip_id,
            rule_code,
            amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
            created_at: Utc::now(),
        };

        diesel::insert_into(payslip_rule_evaluations::table)
            .values(&entity)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_payslip_rule_evaluations(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipRuleEvaluation>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslip_rule_evaluations::table
            .filter(payslip_rule_evaluations::payslip_id.eq(payslip_id))
            .select(PayslipRuleEvaluationEntity::as_select())
            .load::<PayslipRuleEvaluationEntity>(&mut *conn)
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
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::scope_type.eq("workspace"))
            .filter(workspace_config_entries::effective_from.le(date))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if entries.is_empty() {
            return Ok(None);
        }

        let resolver = crate::models::WorkspaceConfigResolver::new(entries);
        let resolved = resolver.resolve_payroll_settings(Uuid::nil(), None, None, date);

        let max_effective_from = resolver
            .resolve_key(
                "payroll.std_working_day_mode",
                Uuid::nil(),
                None,
                None,
                date,
            )
            .map(crate::models::WorkspaceConfigEntry::effective_from)
            .unwrap_or(date);

        Ok(Some(PayrollSettings {
            workspace_id,
            std_working_day_mode: resolved.std_working_day_mode,
            fixed_std_days: resolved.fixed_std_days,
            working_days_per_week: resolved.working_days_per_week,
            hours_per_day: resolved.hours_per_day,
            max_ot_hours_per_day: resolved.max_ot_hours_per_day,
            max_ot_hours_per_month: resolved.max_ot_hours_per_month,
            effective_date: max_effective_from,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }))
    }

    async fn get_payroll_settings_history(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollSettings>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::scope_type.eq("workspace"))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if entries.is_empty() {
            return Ok(Vec::new());
        }

        let mut dates: Vec<NaiveDate> = entries
            .iter()
            .map(crate::models::WorkspaceConfigEntry::effective_from)
            .collect();
        dates.sort_unstable();
        dates.dedup();
        dates.reverse();

        let mut history = Vec::new();
        let resolver = crate::models::WorkspaceConfigResolver::new(entries);
        for d in dates {
            let resolved = resolver.resolve_payroll_settings(Uuid::nil(), None, None, d);
            history.push(PayrollSettings {
                workspace_id,
                std_working_day_mode: resolved.std_working_day_mode,
                fixed_std_days: resolved.fixed_std_days,
                working_days_per_week: resolved.working_days_per_week,
                hours_per_day: resolved.hours_per_day,
                max_ot_hours_per_day: resolved.max_ot_hours_per_day,
                max_ot_hours_per_month: resolved.max_ot_hours_per_month,
                effective_date: d,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            });
        }

        Ok(history)
    }

    async fn update_payroll_settings(
        &self,
        workspace_id: Uuid,
        req: &crate::models::SavePayrollSettings,
    ) -> Result<PayrollSettings, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let keys_and_values = vec![
            (
                "payroll.std_working_day_mode",
                None,
                None,
                Some(req.std_working_day_mode.clone()),
                None,
                None,
            ),
            (
                "payroll.fixed_std_days",
                None,
                Some(req.fixed_std_days),
                None,
                None,
                None,
            ),
            (
                "payroll.working_days_per_week",
                Some(req.working_days_per_week),
                None,
                None,
                None,
                None,
            ),
            (
                "payroll.hours_per_day",
                None,
                Some(req.hours_per_day),
                None,
                None,
                None,
            ),
            (
                "payroll.max_ot_hours_per_day",
                None,
                req.max_ot_hours_per_day,
                None,
                None,
                None,
            ),
            (
                "payroll.max_ot_hours_per_month",
                None,
                req.max_ot_hours_per_month,
                None,
                None,
                None,
            ),
        ];

        for (key, val_int, val_dec, val_text, val_bool, val_json) in keys_and_values {
            let has_value = val_int.is_some()
                || val_dec.is_some()
                || val_text.is_some()
                || val_bool.is_some()
                || val_json.is_some();

            if has_value {
                let existing = workspace_config_entries::table
                    .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                    .filter(workspace_config_entries::scope_type.eq("workspace"))
                    .filter(workspace_config_entries::scope_id.is_null())
                    .filter(workspace_config_entries::config_key.eq(key))
                    .filter(workspace_config_entries::effective_from.eq(req.effective_date))
                    .first::<crate::models::WorkspaceConfigEntry>(&mut *conn)
                    .await
                    .optional()
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                if let Some(entry) = existing {
                    diesel::update(
                        workspace_config_entries::table
                            .filter(workspace_config_entries::id.eq(entry.id())),
                    )
                    .set((
                        workspace_config_entries::value_int.eq(val_int),
                        workspace_config_entries::value_decimal.eq(val_dec),
                        workspace_config_entries::value_text.eq(val_text),
                        workspace_config_entries::value_bool.eq(val_bool),
                        workspace_config_entries::value_json.eq(val_json),
                        workspace_config_entries::updated_at.eq(Utc::now()),
                    ))
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                } else {
                    let id = Uuid::new_v4();
                    let entry = crate::models::WorkspaceConfigEntry::new(
                        id,
                        workspace_id,
                        "workspace".to_string(),
                        None,
                        key.to_string(),
                        val_int,
                        val_dec,
                        val_text,
                        val_bool,
                        val_json,
                        req.effective_date,
                        None,
                        None,
                        Utc::now(),
                        Utc::now(),
                    );
                    diesel::insert_into(workspace_config_entries::table)
                        .values(&entry)
                        .execute(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                }
            } else {
                diesel::delete(
                    workspace_config_entries::table
                        .filter(workspace_config_entries::workspace_id.eq(workspace_id))
                        .filter(workspace_config_entries::scope_type.eq("workspace"))
                        .filter(workspace_config_entries::scope_id.is_null())
                        .filter(workspace_config_entries::config_key.eq(key))
                        .filter(workspace_config_entries::effective_from.eq(req.effective_date)),
                )
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
            }
        }

        let resolved = self
            .get_payroll_settings_at(workspace_id, req.effective_date)
            .await?;
        resolved.ok_or_else(|| {
            AppError::Internal("Failed to resolve newly updated payroll settings".to_string())
        })
    }

    async fn get_salary_component_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<SalaryComponentDefinition>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = salary_component_definitions::table
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .select(SalaryComponentDefinitionEntity::as_select())
            .load::<SalaryComponentDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn create_salary_component_definition(
        &self,
        workspace_id: Uuid,
        name: String,
        code: String,
        base_group_ids: Option<Vec<Uuid>>,
    ) -> Result<SalaryComponentDefinition, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = conn
            .transaction::<_, AppError, _>(|conn| {
                Box::pin(async move {
                    let entity = SalaryComponentDefinitionEntity {
                        id: Uuid::new_v4(),
                        workspace_id,
                        name,
                        code,
                        created_at: Some(Utc::now()),
                        updated_at: Some(Utc::now()),
                        component_type: "earning".to_string(),
                        linked_rule_id: None,
                    };

                    let result = diesel::insert_into(salary_component_definitions::table)
                        .values(&entity)
                        .get_result::<SalaryComponentDefinitionEntity>(&mut *conn)
                        .await
                        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

                    if let Some(base_group_ids) = base_group_ids {
                        if !base_group_ids.is_empty() {
                            let valid_groups: Vec<Uuid> = salary_base_groups::table
                                .filter(salary_base_groups::id.eq_any(&base_group_ids))
                                .filter(
                                    salary_base_groups::workspace_id
                                        .eq(workspace_id)
                                        .or(salary_base_groups::workspace_id.is_null()),
                                )
                                .select(salary_base_groups::id)
                                .load(&mut *conn)
                                .await
                                .map_err(|e| {
                                    AppError::Internal(format!("Failed to validate groups: {e}"))
                                })?;

                            if valid_groups.len() != base_group_ids.len() {
                                return Err(AppError::Forbidden("error.forbidden".to_string()));
                            }

                            let new_members: Vec<SalaryBaseGroupMemberRow> = base_group_ids
                                .into_iter()
                                .map(|group_id| SalaryBaseGroupMemberRow {
                                    component_definition_id: result.id,
                                    base_group_id: group_id,
                                    created_at: Utc::now(),
                                })
                                .collect();

                            diesel::insert_into(salary_component_base_members::table)
                                .values(&new_members)
                                .on_conflict((
                                    salary_component_base_members::component_definition_id,
                                    salary_component_base_members::base_group_id,
                                ))
                                .do_nothing()
                                .execute(&mut *conn)
                                .await
                                .map_err(|e| AppError::Internal(format!("db: {e}")))?;
                        }
                    }

                    Ok(result)
                })
            })
            .await?;

        Ok(entity.into())
    }

    async fn assign_employment_salary_component(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        component_definition_id: Uuid,
        amount: Decimal,
        effective_date: NaiveDate,
    ) -> Result<EmployeeSalaryComponent, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entity = EmployeeSalaryComponentEntity {
            id: Uuid::new_v4(),
            employment_id,
            component_definition_id,
            amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
            effective_date,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let result = diesel::insert_into(employee_salary_components::table)
            .values(&entity)
            .get_result::<EmployeeSalaryComponentEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_employment_salary_components(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        reference_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq(employment_id))
            .filter(employee_salary_components::effective_date.le(reference_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .distinct_on((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
            ))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
    }

    async fn get_salary_components_history_for_employments(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq_any(employment_ids))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::effective_date.asc(),
                employee_salary_components::created_at.asc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(rows
            .into_iter()
            .map(|(esc, scd)| (esc.into(), scd.into()))
            .collect())
    }

    async fn create_payslip(&self, payslip: &Payslip) -> Result<Payslip, AppError> {
        use std::str::FromStr;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&payslip.workspace_id)
                .await?,
        );

        let entity = PayslipEntity {
            id: payslip.id,
            employment_id: payslip.employment_id,
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
            .get_result::<PayslipEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(result.into())
    }

    async fn get_payslips_by_employee(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
    ) -> Result<Vec<Payslip>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslips::table
            .inner_join(employments::table)
            .filter(payslips::workspace_id.eq(workspace_id))
            .filter(employments::employee_id.eq(employee_id))
            .select(PayslipEntity::as_select())
            .order(payslips::period_start.desc())
            .load::<PayslipEntity>(&mut *conn)
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
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslips::table
            .filter(payslips::workspace_id.eq(workspace_id))
            .filter(payslips::period_start.eq(period_start))
            .filter(payslips::period_end.eq(period_end))
            .order(payslips::created_at.desc())
            .load::<PayslipEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_salary_components_history(
        &self,
        workspace_id: Uuid,
        employment_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        // Last component before start_date
        let last_before = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq(employment_id))
            .filter(employee_salary_components::effective_date.lt(start_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .distinct_on((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
            ))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // Components within range
        let within_range = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq(employment_id))
            .filter(employee_salary_components::effective_date.between(start_date, end_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order(employee_salary_components::effective_date.asc())
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
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

    async fn get_salary_components_history_batch(
        &self,
        workspace_id: Uuid,
        employment_ids: &[Uuid],
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<
        std::collections::HashMap<Uuid, Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>>,
        AppError,
    > {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        // Fetch all components for these employments up to end_date
        let rows = employee_salary_components::table
            .inner_join(salary_component_definitions::table)
            .filter(employee_salary_components::employment_id.eq_any(employment_ids))
            .filter(employee_salary_components::effective_date.le(end_date))
            .filter(salary_component_definitions::workspace_id.eq(workspace_id))
            .order((
                employee_salary_components::employment_id,
                employee_salary_components::component_definition_id,
                employee_salary_components::effective_date.desc(),
                employee_salary_components::created_at.desc(),
            ))
            .load::<(
                EmployeeSalaryComponentEntity,
                SalaryComponentDefinitionEntity,
            )>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let mut map: std::collections::HashMap<
            Uuid,
            Vec<(EmployeeSalaryComponent, SalaryComponentDefinition)>,
        > = std::collections::HashMap::new();

        // We need to keep only the latest before start_date, and all within range.
        // Group by employment and component id
        let mut employment_components: std::collections::HashMap<
            Uuid,
            std::collections::HashMap<
                Uuid,
                Vec<(
                    EmployeeSalaryComponentEntity,
                    SalaryComponentDefinitionEntity,
                )>,
            >,
        > = std::collections::HashMap::new();

        for row in rows {
            let emp_id = row.0.employment_id;
            let comp_id = row.0.component_definition_id;
            employment_components
                .entry(emp_id)
                .or_default()
                .entry(comp_id)
                .or_default()
                .push(row);
        }

        for (emp_id, comp_map) in employment_components {
            let mut emp_history: Vec<(
                crate::models::EmployeeSalaryComponent,
                crate::models::SalaryComponentDefinition,
            )> = Vec::new();
            for (_, history) in comp_map {
                // history is sorted desc by effective_date
                // Find all within range
                let mut within_range = Vec::new();
                let mut last_before = None;

                for item in history {
                    if item.0.effective_date >= start_date {
                        within_range.push(item);
                    } else if last_before.is_none() {
                        last_before = Some(item);
                    }
                }

                if let Some(item) = last_before {
                    emp_history.push((item.0.into(), item.1.into()));
                }
                for item in within_range {
                    emp_history.push((item.0.into(), item.1.into()));
                }
            }
            emp_history.sort_by_key(|(esc, _)| esc.effective_date);
            map.insert(emp_id, emp_history);
        }

        Ok(map)
    }

    async fn create_payslip_segment(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
        args: &crate::models::CreatePayslipSegmentArgs,
    ) -> Result<(), AppError> {
        use std::str::FromStr;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

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
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_payslip_segments(
        &self,
        workspace_id: Uuid,
        payslip_id: Uuid,
    ) -> Result<Vec<PayslipSegment>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payslip_segments::table
            .filter(payslip_segments::payslip_id.eq(payslip_id))
            .load::<PayslipSegmentEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn persist_calculated_payslip_data(
        &self,
        workspace_id: Uuid,
        payslip: &Payslip,
        segments: &[crate::models::CreatePayslipSegmentArgs],
        incentives: &[IncentiveEvaluationResult],
        rule_evaluations: &[(String, Decimal)],
    ) -> Result<Payslip, AppError> {
        use std::str::FromStr;
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let payslip_entity = PayslipEntity {
            id: payslip.id,
            employment_id: payslip.employment_id,
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

        let segments_entities: Vec<PayslipSegmentEntity> = segments
            .iter()
            .map(|args| PayslipSegmentEntity {
                id: Uuid::new_v4(),
                payslip_id: payslip.id,
                start_date: args.start_date,
                end_date: args.end_date,
                department_id: args.department_id,
                applied_base_salary: bigdecimal::BigDecimal::from_str(
                    &args.applied_base_salary.to_string(),
                )
                .unwrap_or_default(),
                applied_std_days: bigdecimal::BigDecimal::from_str(
                    &args.applied_std_days.to_string(),
                )
                .unwrap_or_default(),
                actual_work_days: bigdecimal::BigDecimal::from_str(
                    &args.actual_work_days.to_string(),
                )
                .unwrap_or_default(),
                paid_work_days: bigdecimal::BigDecimal::from_str(&args.paid_work_days.to_string())
                    .unwrap_or_default(),
                amount: bigdecimal::BigDecimal::from_str(&args.amount.to_string())
                    .unwrap_or_default(),
                created_at: Some(Utc::now()),
                applied_hours_per_day: bigdecimal::BigDecimal::from_str(
                    &args.hours_per_day.to_string(),
                )
                .unwrap_or_default(),
                std_days_source: args.std_days_source.clone(),
            })
            .collect();

        let incentives_entities: Vec<PayslipIncentiveEntity> = incentives
            .iter()
            .map(|result| PayslipIncentiveEntity {
                id: Uuid::new_v4(),
                payslip_id: payslip.id,
                policy_name: result.policy_name.clone(),
                amount: result.amount.to_f64().unwrap_or_default(),
                tier_id: result.tier_id,
                standard_amount: result.standard_amount.to_f64().unwrap_or_default(),
                created_at: Utc::now(),
            })
            .collect();

        let rules_entities: Vec<PayslipRuleEvaluationEntity> = rule_evaluations
            .iter()
            .map(|(rule_code, amount)| PayslipRuleEvaluationEntity {
                id: Uuid::new_v4(),
                payslip_id: payslip.id,
                rule_code: rule_code.clone(),
                amount: bigdecimal::BigDecimal::from_str(&amount.to_string()).unwrap_or_default(),
                created_at: Utc::now(),
            })
            .collect();

        let result = conn
            .transaction::<_, diesel::result::Error, _>(|conn| {
                Box::pin(async move {
                    let created_payslip = diesel::insert_into(payslips::table)
                        .values(&payslip_entity)
                        .get_result::<PayslipEntity>(&mut *conn)
                        .await?;

                    if !segments_entities.is_empty() {
                        diesel::insert_into(payslip_segments::table)
                            .values(&segments_entities)
                            .execute(&mut *conn)
                            .await?;
                    }

                    if !incentives_entities.is_empty() {
                        diesel::insert_into(payslip_incentives::table)
                            .values(&incentives_entities)
                            .execute(&mut *conn)
                            .await?;
                    }

                    if !rules_entities.is_empty() {
                        diesel::insert_into(payslip_rule_evaluations::table)
                            .values(&rules_entities)
                            .execute(&mut *conn)
                            .await?;
                    }

                    Ok(created_payslip)
                })
            })
            .await
            .map_err(|e| AppError::Internal(format!("db transaction error: {e}")))?;

        Ok(result.into())
    }

    async fn get_resolved_rule_assignments(
        &self,
        workspace_id: Uuid,
        employee_id: Uuid,
        department_id: Option<Uuid>,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<ResolvedRuleAssignment>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

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
            .load::<ResolvedRow>(&mut *conn)
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
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<crate::models::PayrollRuleTier>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_rule_tiers::table
            .filter(payroll_rule_tiers::rule_definition_id.eq(rule_definition_id))
            .order(payroll_rule_tiers::tier_order.asc())
            .load::<PayrollRuleTierEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn get_department_payroll_config_at(
        &self,
        workspace_id: Uuid,
        department_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<crate::models::DepartmentPayrollConfig>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        use crate::schema::workspace_config_entries;

        let entries = workspace_config_entries::table
            .filter(workspace_config_entries::workspace_id.eq(workspace_id))
            .filter(workspace_config_entries::scope_type.eq("department"))
            .filter(workspace_config_entries::scope_id.eq(department_id))
            .filter(workspace_config_entries::effective_from.le(date))
            .load::<crate::models::WorkspaceConfigEntry>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if entries.is_empty() {
            return Ok(None);
        }

        let mut fixed_std_days = None;
        let mut hours_per_day = None;
        let mut max_effective_from = None;
        let mut entry_id = Uuid::new_v4();

        for entry in &entries {
            if entry.config_key() == "payroll.fixed_std_days"
                && max_effective_from.is_none_or(|max_d| entry.effective_from() >= max_d)
            {
                fixed_std_days = entry.value_decimal();
                max_effective_from = Some(entry.effective_from());
                entry_id = entry.id();
            } else if entry.config_key() == "payroll.hours_per_day"
                && max_effective_from.is_none_or(|max_d| entry.effective_from() >= max_d)
            {
                hours_per_day = entry.value_decimal();
                max_effective_from = Some(entry.effective_from());
            }
        }

        if fixed_std_days.is_none() && hours_per_day.is_none() {
            return Ok(None);
        }

        let effective_date = max_effective_from.unwrap_or(date);

        Ok(Some(crate::models::DepartmentPayrollConfig {
            id: entry_id,
            department_id,
            std_working_days: fixed_std_days.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            hours_per_day: hours_per_day.unwrap_or_else(|| rust_decimal::Decimal::from(0)),
            effective_date,
            created_at: Utc::now(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselPayrollRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_create_salary_component_definition() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselPayrollRepository::new(pools.clone());
        let ws_id = Uuid::new_v4();
        let token = crate::diesel_pool::TenantAuthToken::system_override(ws_id);

        let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
            c
        } else {
            println!("Database not available, skipping integration assertions");
            return;
        };

        // Insert Workspace to satisfy FK constraints
        let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
            id: ws_id,
            name: "Test Workspace".to_string(),
            created_at: Some(chrono::Utc::now()),
            date_format: "YYYY-MM-DD".to_string(),
            time_format: "HH:mm".to_string(),
            timezone: "UTC".to_string(),
            theme_primary_color: None,
            default_language: "en".to_string(),
            logo_url: None,
            tagline: None,
            custom_db_url_encrypted: None,
        };
        diesel::insert_into(crate::schema::workspaces::table)
            .values(&workspace_entity)
            .execute(&mut *conn)
            .await
            .expect("insert workspace");

        // Insert a base group
        let group_id1 = Uuid::new_v4();
        let group_id2 = Uuid::new_v4();

        diesel::sql_query("INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind::<diesel::sql_types::Uuid, _>(group_id1)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(ws_id))
            .bind::<diesel::sql_types::Text, _>("Test Group 1")
            .bind::<diesel::sql_types::Text, _>("test-group-1")
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
            .bind::<diesel::sql_types::Bool, _>(false)
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert group 1");

        diesel::sql_query("INSERT INTO salary_base_groups (id, workspace_id, name, slug, description, is_system, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
            .bind::<diesel::sql_types::Uuid, _>(group_id2)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(ws_id))
            .bind::<diesel::sql_types::Text, _>("Test Group 2")
            .bind::<diesel::sql_types::Text, _>("test-group-2")
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(None::<String>)
            .bind::<diesel::sql_types::Bool, _>(false)
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
            .execute(&mut *conn)
            .await
            .expect("insert group 2");

        drop(conn);

        // 1. Success Path: create definition with valid groups
        let res = repo
            .create_salary_component_definition(
                ws_id,
                "Base Salary".to_string(),
                "BASE".to_string(),
                Some(vec![group_id1, group_id2]),
            )
            .await;
        assert!(res.is_ok());
        let def = res.unwrap();
        assert_eq!(def.name, "Base Salary");
        assert_eq!(def.code, "BASE");

        // Verify members are created in DB
        let mut conn = pools.tenant_transaction_conn(&token).await.unwrap();
        let count: i64 = crate::schema::salary_component_base_members::table
            .filter(
                crate::schema::salary_component_base_members::component_definition_id.eq(def.id),
            )
            .count()
            .get_result(&mut *conn)
            .await
            .unwrap();
        assert_eq!(count, 2);

        // 2. Failure Path: invalid base group (Forbidden error)
        let bad_group = Uuid::new_v4();
        let res_err = repo
            .create_salary_component_definition(
                ws_id,
                "Allowance".to_string(),
                "ALLOW".to_string(),
                Some(vec![bad_group]),
            )
            .await;
        assert!(res_err.is_err());
        assert!(matches!(res_err.unwrap_err(), AppError::Forbidden(_)));
    }
}
