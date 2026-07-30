use super::*;
use crate::errors::AppError;
use crate::models::diesel::payroll::*;
use crate::models::{IncentiveEvaluationResult, IncentiveTier, PayrollIncentivePolicy};
use chrono::{NaiveDate, Utc};
use diesel_async::{AsyncConnection, RunQueryDsl};
use rust_decimal::prelude::*;
use uuid::Uuid;

#[allow(clippy::too_many_lines, clippy::too_many_arguments)]
impl DieselPayrollRepository {
    pub(super) async fn get_active_incentive_policies(
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_active_incentive_policies_for_period(
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_policy_tiers(
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
            .map_err(AppError::from)?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    pub(super) async fn get_policy_tiers_batch(
        &self,
        workspace_id: Uuid,
        policy_ids: &[Uuid],
    ) -> Result<std::collections::HashMap<Uuid, Vec<IncentiveTier>>, AppError> {
        if policy_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await?,
        );

        let entities = payroll_incentive_tiers::table
            .filter(payroll_incentive_tiers::policy_id.eq_any(policy_ids))
            .order(payroll_incentive_tiers::priority.asc())
            .load::<IncentiveTierEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        let mut map = std::collections::HashMap::new();
        for &policy_id in policy_ids {
            map.insert(policy_id, Vec::new());
        }

        for ent in entities {
            let policy_id = ent.policy_id;
            map.entry(policy_id)
                .or_insert_with(Vec::new)
                .push(ent.into());
        }

        Ok(map)
    }

    pub(super) async fn create_incentive_policy(
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

        let policy: PayrollIncentivePolicy = entity.into();
        Ok(policy)
    }

    pub(super) async fn delete_incentive_policy(
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
        .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn clone_incentive_policy(
        &self,
        workspace_id: Uuid,
        policy_id: Uuid,
    ) -> Result<PayrollIncentivePolicy, AppError> {
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
                        .map_err(AppError::from)?;

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
                        .map_err(AppError::from)?;

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
                            .map_err(AppError::from)?;
                    }

                    Ok((new_policy_entity, new_tiers))
                })
            })
            .await?;

        Ok(cloned_entity.into())
    }

    pub(super) async fn create_payslip_incentive(
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
            .map_err(AppError::from)?;

        Ok(())
    }

    pub(super) async fn get_payslip_incentives(
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
            .map_err(AppError::from)?;

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
}
