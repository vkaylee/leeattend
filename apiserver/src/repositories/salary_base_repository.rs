use crate::errors::AppError;
use crate::models::{SalaryBaseGroup, SalaryBaseGroupMember};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SalaryBaseRepository;

impl SalaryBaseRepository {
    pub async fn get_groups(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<SalaryBaseGroup>, AppError> {
        let groups = sqlx::query_as!(
            SalaryBaseGroup,
            r#"
            SELECT id, workspace_id, name, slug, description, is_system, created_at, updated_at
            FROM salary_base_groups
            WHERE workspace_id = $1 OR workspace_id IS NULL
            ORDER BY is_system DESC, name ASC
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch salary bases: {}", e)))?;
        
        Ok(groups)
    }

    pub async fn get_group_members(pool: &PgPool, workspace_id: Uuid) -> Result<Vec<SalaryBaseGroupMember>, AppError> {
        let members = sqlx::query_as!(
            SalaryBaseGroupMember,
            r#"
            SELECT m.component_definition_id, m.base_group_id, m.created_at
            FROM salary_component_base_members m
            JOIN salary_base_groups b ON m.base_group_id = b.id
            WHERE b.workspace_id = $1 OR b.workspace_id IS NULL
            "#,
            workspace_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to fetch salary base members: {}", e)))?;

        Ok(members)
    }

    pub async fn assign_component_to_group(
        pool: &PgPool,
        component_id: Uuid,
        base_group_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            INSERT INTO salary_component_base_members (component_definition_id, base_group_id)
            VALUES ($1, $2)
            ON CONFLICT (component_definition_id, base_group_id) DO NOTHING
            "#,
            component_id,
            base_group_id
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to assign component to base: {}", e)))?;

        Ok(())
    }

    pub async fn unassign_component_from_group(
        pool: &PgPool,
        component_id: Uuid,
        base_group_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            r#"
            DELETE FROM salary_component_base_members
            WHERE component_definition_id = $1 AND base_group_id = $2
            "#,
            component_id,
            base_group_id
        )
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to unassign component from base: {}", e)))?;

        Ok(())
    }
}
