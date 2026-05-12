use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::assignment_rule::AssignmentRuleEntity;
use crate::models::employee::{
    AssignmentRule, CreateAssignmentRuleRequest,
    UpdateAssignmentRuleRequest,
};
use crate::repositories::assignment_rule_repository::AssignmentRuleRepositoryTrait;
use crate::schema::assignment_rules;

pub struct DieselAssignmentRuleRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselAssignmentRuleRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl AssignmentRuleRepositoryTrait for DieselAssignmentRuleRepository {
    async fn create_rule(
        &self,
        workspace_id: Uuid,
        req: &CreateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let conditions = serde_json::to_value(&req.conditions)
            .map_err(|e| AppError::Internal(format!("serde: {e}")))?;
        let now = Utc::now();

        let entity = AssignmentRuleEntity {
            id: Uuid::new_v4(),
            workspace_id,
            pattern_id: req.pattern_id,
            name: req.name.clone(),
            priority: req.priority,
            conditions,
            is_active: true,
            created_at: Some(now),
            updated_at: Some(now),
        };

        diesel::insert_into(assignment_rules::table)
            .values(&entity)
            .returning(AssignmentRuleEntity::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }

    async fn get_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        assignment_rules::table
            .filter(assignment_rules::workspace_id.eq(workspace_id))
            .order((assignment_rules::priority.desc(), assignment_rules::created_at.asc()))
            .select(AssignmentRuleEntity::as_select())
            .load::<AssignmentRuleEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn get_active_rules(
        &self,
        workspace_id: Uuid,
    ) -> Result<Vec<AssignmentRule>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        assignment_rules::table
            .filter(assignment_rules::workspace_id.eq(workspace_id))
            .filter(assignment_rules::is_active.eq(true))
            .order((assignment_rules::priority.desc(), assignment_rules::created_at.asc()))
            .select(AssignmentRuleEntity::as_select())
            .load::<AssignmentRuleEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(|rows| rows.into_iter().map(Into::into).collect())
    }

    async fn update_rule(
        &self,
        workspace_id: Uuid,
        id: Uuid,
        req: &UpdateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let conditions_json = req
            .conditions
            .as_ref()
            .map(|c| serde_json::to_value(c).map_err(|e| AppError::Internal(format!("serde: {e}"))))
            .transpose()?;

        let target = assignment_rules::table
            .filter(assignment_rules::id.eq(id))
            .filter(assignment_rules::workspace_id.eq(workspace_id));

        diesel::update(target)
            .set((
                req.pattern_id.map(|v| assignment_rules::pattern_id.eq(v)),
                req.name.as_ref().map(|v| assignment_rules::name.eq(v.clone())),
                req.priority.map(|v| assignment_rules::priority.eq(v)),
                conditions_json.map(|v| assignment_rules::conditions.eq(v)),
                req.is_active.map(|v| assignment_rules::is_active.eq(v)),
                assignment_rules::updated_at.eq(Some(Utc::now())),
            ))
            .returning(AssignmentRuleEntity::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
            .map(Into::into)
    }

    async fn delete_rule(&self, workspace_id: Uuid, id: Uuid) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::delete(
            assignment_rules::table
                .filter(assignment_rules::id.eq(id))
                .filter(assignment_rules::workspace_id.eq(workspace_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}

#[allow(clippy::panic)]
impl crate::repositories::PoolBacked for DieselAssignmentRuleRepository {
    fn from_pool(_pool: PgPool) -> Self {
        panic!("DieselAssignmentRuleRepository requires DieselPools. Use new() instead.");
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}
