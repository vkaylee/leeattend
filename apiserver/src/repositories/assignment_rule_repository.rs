#[allow(unused_imports)]
use crate::models::{
    AssignmentRule, AssignmentRuleConditions, CreateAssignmentRuleRequest,
    UpdateAssignmentRuleRequest,
};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: AssignmentRuleRepositoryTrait — DIP abstraction
// ---------------------------------------------------------------------------
#[async_trait]
pub trait AssignmentRuleRepositoryTrait: super::PoolBacked + Send + Sync {
    async fn create_rule(&self, workspace_id: Uuid, req: &CreateAssignmentRuleRequest) -> Result<AssignmentRule, sqlx::Error>;
    async fn get_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, sqlx::Error>;
    async fn get_active_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, sqlx::Error>;
    async fn update_rule(&self, workspace_id: Uuid, id: Uuid, req: &UpdateAssignmentRuleRequest) -> Result<AssignmentRule, sqlx::Error>;
    async fn delete_rule(&self, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error>;
}

// ---------------------------------------------------------------------------
// SqlxAssignmentRuleRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxAssignmentRuleRepository {
    pub pool: PgPool,
}

impl SqlxAssignmentRuleRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl AssignmentRuleRepositoryTrait for SqlxAssignmentRuleRepository {
    async fn create_rule(&self, workspace_id: Uuid, req: &CreateAssignmentRuleRequest) -> Result<AssignmentRule, sqlx::Error> {
        AssignmentRuleRepository::create_rule(&self.pool, workspace_id, req).await
    }
    async fn get_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, sqlx::Error> {
        AssignmentRuleRepository::get_rules(&self.pool, workspace_id).await
    }
    async fn get_active_rules(&self, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, sqlx::Error> {
        AssignmentRuleRepository::get_active_rules(&self.pool, workspace_id).await
    }
    async fn update_rule(&self, workspace_id: Uuid, id: Uuid, req: &UpdateAssignmentRuleRequest) -> Result<AssignmentRule, sqlx::Error> {
        AssignmentRuleRepository::update_rule(&self.pool, workspace_id, id, req).await
    }
    async fn delete_rule(&self, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error> {
        AssignmentRuleRepository::delete_rule(&self.pool, workspace_id, id).await
    }
}

/// Backwards-compatible static method repository.
pub struct AssignmentRuleRepository;

impl AssignmentRuleRepository {

    pub async fn create_rule(
        pool: &PgPool,
        workspace_id: Uuid,
        req: &CreateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, sqlx::Error> {
        let rule = sqlx::query_as!(
            AssignmentRule,
            r#"
            INSERT INTO assignment_rules (workspace_id, pattern_id, name, priority, conditions)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, workspace_id, pattern_id, name, priority, conditions as "conditions: sqlx::types::Json<AssignmentRuleConditions>", is_active, created_at as "created_at!", updated_at as "updated_at!"
            "#,
            workspace_id,
            req.pattern_id,
            req.name,
            req.priority,
            serde_json::to_value(&req.conditions).map_err(|e| sqlx::Error::Decode(Box::new(e)))?
        )
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn get_rules(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<AssignmentRule>, sqlx::Error> {
        let rules = sqlx::query_as!(
            AssignmentRule,
            r#"
            SELECT id, workspace_id, pattern_id, name, priority, conditions as "conditions: sqlx::types::Json<AssignmentRuleConditions>", is_active, created_at as "created_at!", updated_at as "updated_at!"
            FROM assignment_rules
            WHERE workspace_id = $1
            ORDER BY priority DESC, created_at ASC
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rules)
    }

    pub async fn get_active_rules(
        pool: &PgPool,
        workspace_id: Uuid,
    ) -> Result<Vec<AssignmentRule>, sqlx::Error> {
        let rules = sqlx::query_as!(
            AssignmentRule,
            r#"
            SELECT id, workspace_id, pattern_id, name, priority, conditions as "conditions: sqlx::types::Json<AssignmentRuleConditions>", is_active, created_at as "created_at!", updated_at as "updated_at!"
            FROM assignment_rules
            WHERE workspace_id = $1 AND is_active = true
            ORDER BY priority DESC, created_at ASC
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rules)
    }

    pub async fn update_rule(
        pool: &PgPool,
        workspace_id: Uuid,
        id: Uuid,
        req: &UpdateAssignmentRuleRequest,
    ) -> Result<AssignmentRule, sqlx::Error> {
        let rule = sqlx::query_as!(
            AssignmentRule,
            r#"
            UPDATE assignment_rules
            SET 
                pattern_id = COALESCE($1, pattern_id),
                name = COALESCE($2, name),
                priority = COALESCE($3, priority),
                conditions = COALESCE($4, conditions),
                is_active = COALESCE($5, is_active),
                updated_at = NOW()
            WHERE id = $6 AND workspace_id = $7
            RETURNING id, workspace_id, pattern_id, name, priority, conditions as "conditions: sqlx::types::Json<AssignmentRuleConditions>", is_active, created_at as "created_at!", updated_at as "updated_at!"
            "#,
            req.pattern_id,
            req.name,
            req.priority,
            req.conditions.as_ref().map(|c| serde_json::to_value(c).map_err(|e| sqlx::Error::Decode(Box::new(e)))).transpose()?,
            req.is_active,
            id,
            workspace_id
        )
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn delete_rule(pool: &PgPool, workspace_id: Uuid, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM assignment_rules WHERE id = $1 AND workspace_id = $2", id, workspace_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl super::PoolBacked for SqlxAssignmentRuleRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self { Self::new(pool) }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
