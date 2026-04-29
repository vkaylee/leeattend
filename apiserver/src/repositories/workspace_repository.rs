use crate::models::{MemberExtraRole, Workspace};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn get_workspace_by_id(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Workspace>, sqlx::Error> {
    sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[async_trait]
pub trait WorkspaceRepository: super::PoolBacked + Send + Sync {
    async fn create_workspace(&self, name: &str) -> Result<Workspace, String>;
    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, String>;
    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), String>;
    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), String>;
    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), String>;
    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, String>;
    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, String>;
    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, String>;
}

pub struct SqlxWorkspaceRepository {
    pub pool: PgPool,
}

impl SqlxWorkspaceRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait]
impl WorkspaceRepository for SqlxWorkspaceRepository {
    async fn create_workspace(&self, name: &str) -> Result<Workspace, String> {
        let ws = sqlx::query_as::<_, Workspace>(
            "INSERT INTO workspaces (id, name, created_at) 
             VALUES ($1, $2, NOW())
             RETURNING id, name, date_format, time_format, timezone, theme_primary_color, default_language, holiday_overnight_strategy, default_holiday_ot_multiplier, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ws)
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, String> {
        let ws = sqlx::query_as::<_, Workspace>("SELECT * FROM workspaces WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(ws)
    }

    async fn add_member(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role_id) 
             VALUES ($1, $2, $3)",
        )
        .bind(workspace_id)
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE workspace_members SET role_id = $1 WHERE workspace_id = $2 AND user_id = $3",
        )
        .bind(role_id)
        .bind(workspace_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), String> {
        sqlx::query("DELETE FROM workspace_members WHERE workspace_id = $1 AND user_id = $2")
            .bind(workspace_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Uuid>, String> {
        let row: Option<(Uuid,)> = sqlx::query_as(
            "SELECT role_id FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(row.map(|r| r.0))
    }

    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, String> {
        let roles = sqlx::query_as::<_, MemberExtraRole>(
            "SELECT * FROM member_extra_roles WHERE workspace_id = $1 AND user_id = $2",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(roles)
    }

    async fn find_extra_permissions(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<String>, String> {
        let permissions: Vec<(String,)> = sqlx::query_as(
            "SELECT permission FROM workspace_member_extra_permissions WHERE workspace_id = $1 AND user_id = $2 AND expires_at > NOW()",
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(permissions.into_iter().map(|(p,)| p).collect())
    }
}

impl super::PoolBacked for SqlxWorkspaceRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self { Self { pool } }
    fn pool(&self) -> &sqlx::PgPool { &self.pool }
}
