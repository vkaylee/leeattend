use async_trait::async_trait;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::overtime_policy::{
    OvertimePolicyEntity, OvertimeRuleEntity,
};
use crate::models::{OvertimePolicy, OvertimeRule};
use crate::repositories::overtime_policy_repository::OvertimePolicyRepositoryTrait;
use crate::schema::{
    employee_department_history, overtime_policy_assignments, overtime_policy_definitions,
    overtime_rules,
};

pub struct DieselOvertimePolicyRepository {
    pub sqlx_pool: sqlx::PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselOvertimePolicyRepository {
    pub fn new(sqlx_pool: sqlx::PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl OvertimePolicyRepositoryTrait for DieselOvertimePolicyRepository {
    async fn create_policy_definition(
        &self,
        workspace_id: Uuid,
        name: &str,
        description: Option<&str>,
    ) -> Result<OvertimePolicy, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::insert_into(overtime_policy_definitions::table)
            .values((
                overtime_policy_definitions::workspace_id.eq(workspace_id),
                overtime_policy_definitions::name.eq(name),
                overtime_policy_definitions::description.eq(description),
            ))
            .returning(OvertimePolicyEntity::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }

    async fn get_policy_definitions(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<OvertimePolicy>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        overtime_policy_definitions::table
            .filter(overtime_policy_definitions::workspace_id.eq(workspace_id))
            .order(overtime_policy_definitions::created_at.desc())
            .select(OvertimePolicyEntity::as_select())
            .load(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn get_policy_with_rules(
        &self,
        workspace_id: Uuid,
        id: Uuid,
    ) -> Result<Option<(OvertimePolicy, Vec<OvertimeRule>)>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let policy: Option<OvertimePolicyEntity> = overtime_policy_definitions::table
            .filter(overtime_policy_definitions::id.eq(id))
            .filter(overtime_policy_definitions::workspace_id.eq(workspace_id))
            .select(OvertimePolicyEntity::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(p) = policy {
            let rules: Vec<OvertimeRuleEntity> = overtime_rules::table
                .filter(overtime_rules::policy_id.eq(id))
                .order(overtime_rules::priority.desc())
                .select(OvertimeRuleEntity::as_select())
                .load(&mut conn)
                .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

            Ok(Some((p.into(), rules.into_iter().map(Into::into).collect())))
        } else {
            Ok(None)
        }
    }

    async fn add_rule(&self, rule: &OvertimeRule) -> Result<OvertimeRule, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::insert_into(overtime_rules::table)
            .values((
                overtime_rules::policy_id.eq(rule.policy_id),
                overtime_rules::priority.eq(rule.priority),
                overtime_rules::multiplier.eq(rule.multiplier),
                overtime_rules::time_start.eq(rule.time_start),
                overtime_rules::time_end.eq(rule.time_end),
                overtime_rules::days_of_week.eq(rule.days_of_week),
                overtime_rules::is_holiday.eq(rule.is_holiday),
            ))
            .returning(OvertimeRuleEntity::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }

    async fn resolve_policy_for_employee(
        &self,
        employee_id: Uuid,
        workspace_id: Uuid,
        date: NaiveDate,
    ) -> Result<Option<Uuid>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        // 1. Employee Override
        let emp_policy: Option<Uuid> = overtime_policy_assignments::table
            .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
            .filter(overtime_policy_assignments::assignee_type.eq("employee"))
            .filter(overtime_policy_assignments::assignee_id.eq(employee_id))
            .filter(overtime_policy_assignments::effective_from.le(date))
            .filter(
                overtime_policy_assignments::effective_to
                    .is_null()
                    .or(overtime_policy_assignments::effective_to.ge(date)),
            )
            .order(overtime_policy_assignments::created_at.desc())
            .select(overtime_policy_assignments::policy_id)
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(policy_id) = emp_policy {
            return Ok(Some(policy_id));
        }

        // 2. Department Override
        let dept_policy: Option<Uuid> = overtime_policy_assignments::table
            .inner_join(
                employee_department_history::table
                    .on(employee_department_history::employee_id.eq(employee_id)),
            )
            .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
            .filter(overtime_policy_assignments::assignee_type.eq("department"))
            .filter(
                employee_department_history::department_id
                    .eq(overtime_policy_assignments::assignee_id.assume_not_null()),
            )
            .filter(employee_department_history::effective_date.le(date))
            .filter(overtime_policy_assignments::effective_from.le(date))
            .filter(
                overtime_policy_assignments::effective_to
                    .is_null()
                    .or(overtime_policy_assignments::effective_to.ge(date)),
            )
            .order((
                employee_department_history::effective_date.desc(),
                overtime_policy_assignments::created_at.desc(),
            ))
            .select(overtime_policy_assignments::policy_id)
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        if let Some(policy_id) = dept_policy {
            return Ok(Some(policy_id));
        }

        // 3. Workspace Default
        let ws_policy: Option<Uuid> = overtime_policy_assignments::table
            .filter(overtime_policy_assignments::workspace_id.eq(workspace_id))
            .filter(overtime_policy_assignments::assignee_type.eq("workspace"))
            .filter(overtime_policy_assignments::effective_from.le(date))
            .filter(
                overtime_policy_assignments::effective_to
                    .is_null()
                    .or(overtime_policy_assignments::effective_to.ge(date)),
            )
            .order(overtime_policy_assignments::created_at.desc())
            .select(overtime_policy_assignments::policy_id)
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(ws_policy)
    }

    async fn create_policy_assignment(
        &self,
        workspace_id: Uuid,
        assignee_type: &str,
        assignee_id: Option<Uuid>,
        policy_id: Uuid,
        effective_from: NaiveDate,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::insert_into(overtime_policy_assignments::table)
            .values((
                overtime_policy_assignments::workspace_id.eq(workspace_id),
                overtime_policy_assignments::assignee_type.eq(assignee_type),
                overtime_policy_assignments::assignee_id.eq(assignee_id),
                overtime_policy_assignments::policy_id.eq(policy_id),
                overtime_policy_assignments::effective_from.eq(effective_from),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}

impl crate::repositories::PoolBacked for DieselOvertimePolicyRepository {
    #[allow(clippy::panic)]
    fn from_pool(_pool: sqlx::PgPool) -> Self {
        panic!(
            "DieselOvertimePolicyRepository requires DieselPools. Use new() instead."
        );
    }

    fn pool(&self) -> &sqlx::PgPool {
        &self.sqlx_pool
    }
}
