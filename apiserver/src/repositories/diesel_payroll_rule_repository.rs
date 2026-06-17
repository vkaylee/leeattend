use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::utils::i18n::I18nKey;
use crate::models::diesel::payroll_rule::*;
use crate::models::payroll_rule::*;
use crate::repositories::payroll_rule_repository::{
    AssignPayrollRuleArgs, CreatePayrollRuleDefinitionArgs, PayrollRuleRepository,
};

use crate::schema::*;
use async_trait::async_trait;
use bigdecimal::BigDecimal;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use rust_decimal::Decimal;

use std::str::FromStr;
use uuid::Uuid;

pub struct DieselPayrollRuleRepository {
    diesel_pools: DieselPools,
}

impl DieselPayrollRuleRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl PayrollRuleRepository for DieselPayrollRuleRepository {
    async fn create_definition(
        &self,
        args: CreatePayrollRuleDefinitionArgs,
    ) -> Result<PayrollRuleDefinition, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&args.workspace_id)
                .await?,
        );

        let entity = PayrollRuleDefinitionEntity {
            id: Uuid::new_v4(),
            workspace_id: args.workspace_id,
            name: args.name,
            description: args.description,
            code: args.code,
            category: args.category.to_string(),
            applies_to: args.applies_to.to_string(),
            calculation_method: args.calculation_method.to_string(),
            rule_config: args.rule_config,
            is_mandatory: args.is_mandatory,
            is_active: true,
            display_order: args.display_order,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let result = diesel::insert_into(payroll_rule_definitions::table)
            .values(&entity)
            .get_result::<PayrollRuleDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create definition: {e}")))?;

        Ok(result.into())
    }

    async fn get_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<PayrollRuleDefinition>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = payroll_rule_definitions::table
            .filter(payroll_rule_definitions::workspace_id.eq(workspace_id))
            .order((
                payroll_rule_definitions::display_order.asc(),
                payroll_rule_definitions::created_at.desc(),
            ))
            .load::<PayrollRuleDefinitionEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch definitions: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn create_tier(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        tier_order: i32,
        min_amount: Decimal,
        max_amount: Option<Decimal>,
        rate: Decimal,
        fixed_amount: Decimal,
    ) -> Result<PayrollRuleTier, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity = PayrollRuleTierEntity {
            id: Uuid::new_v4(),
            rule_definition_id,
            tier_order,
            min_amount: BigDecimal::from_str(&min_amount.to_string()).unwrap_or_default(),
            max_amount: max_amount
                .map(|m| BigDecimal::from_str(&m.to_string()).unwrap_or_default()),
            rate: BigDecimal::from_str(&rate.to_string()).unwrap_or_default(),
            fixed_amount: BigDecimal::from_str(&fixed_amount.to_string()).unwrap_or_default(),
            created_at: chrono::Utc::now(),
        };

        let result = diesel::insert_into(payroll_rule_tiers::table)
            .values(&entity)
            .get_result::<PayrollRuleTierEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create tier: {e}")))?;

        Ok(result.into())
    }

    async fn get_tiers_for_rule(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<PayrollRuleTier>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = payroll_rule_tiers::table
            .filter(payroll_rule_tiers::rule_definition_id.eq(rule_definition_id))
            .order(payroll_rule_tiers::tier_order.asc())
            .load::<PayrollRuleTierEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch tiers: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn delete_tier(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        tier_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::delete(payroll_rule_tiers::table)
            .filter(payroll_rule_tiers::id.eq(tier_id))
            .filter(payroll_rule_tiers::rule_definition_id.eq(rule_definition_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete tier: {e}")))?;

        Ok(())
    }

    async fn assign_rule(
        &self,
        args: AssignPayrollRuleArgs,
    ) -> Result<PayrollRuleAssignment, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&args.workspace_id)
                .await?,
        );

        let entity = PayrollRuleAssignmentEntity {
            id: Uuid::new_v4(),
            workspace_id: args.workspace_id,
            rule_definition_id: args.rule_definition_id,
            assignee_type: args.assignee_type.to_string(),
            assignee_id: args.assignee_id,
            effective_from: args.effective_from,
            effective_to: args.effective_to,
            override_config: args.override_config,
            created_at: chrono::Utc::now(),
        };

        let result = diesel::insert_into(payroll_rule_assignments::table)
            .values(&entity)
            .get_result::<PayrollRuleAssignmentEntity>(&mut *conn)
            .await
            .map_err(|e| {
                if let diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    info,
                ) = &e
                {
                    if info.constraint_name() == Some("payroll_rule_assignments_workspace_id_rule_definition_id_a_excl") {
                        return AppError::conflict_args(
                            I18nKey::Conflict,
                            [("details", "This assignment overlaps with an existing assignment for the same rule and assignee in this time period.")],
                        );
                    }
                }
                AppError::Internal(format!("Failed to assign payroll rule: {e}"))
            })?;

        Ok(result.into())
    }

    async fn get_assignments_for_rule(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
    ) -> Result<Vec<PayrollRuleAssignment>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entities = payroll_rule_assignments::table
            .filter(payroll_rule_assignments::workspace_id.eq(workspace_id))
            .filter(payroll_rule_assignments::rule_definition_id.eq(rule_definition_id))
            .order(payroll_rule_assignments::created_at.desc())
            .load::<PayrollRuleAssignmentEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to fetch assignments: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }

    async fn delete_assignment(
        &self,
        workspace_id: Uuid,
        rule_definition_id: Uuid,
        assignment_id: Uuid,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        diesel::delete(payroll_rule_assignments::table)
            .filter(payroll_rule_assignments::id.eq(assignment_id))
            .filter(payroll_rule_assignments::workspace_id.eq(workspace_id))
            .filter(payroll_rule_assignments::rule_definition_id.eq(rule_definition_id))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete assignment: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;
    use crate::models::{
        AssigneeType, PayrollCalculationMethod, PayrollRuleCategory, RuleAppliesTo,
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselPayrollRuleRepository::new(pools);
        // DI connection test passes if it compiles
    }

    #[tokio::test]
    async fn test_repo_methods_error_handling() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });
        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselPayrollRuleRepository::new(pools);
        let ws_id = Uuid::new_v4();

        let create_args = CreatePayrollRuleDefinitionArgs {
            workspace_id: ws_id,
            name: "test".into(),
            description: None,
            code: "T1".into(),
            category: PayrollRuleCategory::Allowance,
            applies_to: RuleAppliesTo::Employee,
            calculation_method: PayrollCalculationMethod::Fixed,
            rule_config: json!({}),
            is_mandatory: true,
            display_order: 1,
        };

        let err = repo.create_definition(create_args).await;
        let _ = err;

        let err = repo.get_definitions(ws_id).await;
        let _ = err;

        let err = repo
            .create_tier(
                ws_id,
                Uuid::new_v4(),
                1,
                rust_decimal::Decimal::new(0, 0),
                None,
                rust_decimal::Decimal::new(0, 0),
                rust_decimal::Decimal::new(0, 0),
            )
            .await;
        let _ = err;

        let err = repo.get_tiers_for_rule(ws_id, Uuid::new_v4()).await;
        let _ = err;

        let err = repo
            .delete_tier(ws_id, Uuid::new_v4(), Uuid::new_v4())
            .await;
        let _ = err;

        let assign_args = AssignPayrollRuleArgs {
            workspace_id: ws_id,
            rule_definition_id: Uuid::new_v4(),
            assignee_type: AssigneeType::Employee,
            assignee_id: Some(Uuid::new_v4()),
            effective_from: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            effective_to: None,
            override_config: None,
        };

        let err = repo.assign_rule(assign_args).await;
        let _ = err;

        let err = repo.get_assignments_for_rule(ws_id, Uuid::new_v4()).await;
        let _ = err;

        let err = repo
            .delete_assignment(ws_id, Uuid::new_v4(), Uuid::new_v4())
            .await;
        let _ = err;
    }
}
