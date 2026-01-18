use sqlx::PgPool;
use uuid::Uuid;
#[allow(unused_imports)]
use crate::models::{AssignmentRule, CreateAssignmentRuleRequest, UpdateAssignmentRuleRequest, AssignmentRuleConditions};

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
            serde_json::to_value(&req.conditions).unwrap()
        )
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn get_rules(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, sqlx::Error> {
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

    pub async fn get_active_rules(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<AssignmentRule>, sqlx::Error> {
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
            WHERE id = $6
            RETURNING id, workspace_id, pattern_id, name, priority, conditions as "conditions: sqlx::types::Json<AssignmentRuleConditions>", is_active, created_at as "created_at!", updated_at as "updated_at!"
            "#,
            req.pattern_id,
            req.name,
            req.priority,
            req.conditions.as_ref().map(|c| serde_json::to_value(c).unwrap()),
            req.is_active,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(rule)
    }

    pub async fn delete_rule(pool: &PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM assignment_rules WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
