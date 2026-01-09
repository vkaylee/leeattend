use crate::models::{MemberExtraRole, Workspace};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait WorkspaceRepository: Send + Sync {
    async fn create_workspace(&self, name: &str) -> Result<Workspace, String>;
    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, String>;
    async fn add_member(&self, workspace_id: Uuid, user_id: Uuid, role: &str)
        -> Result<(), String>;
    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<(), String>;
    async fn remove_member(&self, workspace_id: Uuid, user_id: Uuid) -> Result<(), String>;
    async fn find_primary_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<String>, String>;
    async fn find_extra_roles(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<MemberExtraRole>, String>;
}

pub struct SqlxWorkspaceRepository {
    pub pool: PgPool,
}

#[async_trait]
impl WorkspaceRepository for SqlxWorkspaceRepository {
    async fn create_workspace(&self, name: &str) -> Result<Workspace, String> {
        let ws = sqlx::query_as::<_, Workspace>(
            "INSERT INTO workspaces (id, name, created_at, updated_at) 
             VALUES ($1, $2, NOW(), NOW()) 
             RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(ws)
    }

    async fn get_workspace_by_id(&self, id: Uuid) -> Result<Option<Workspace>, String> {
        let ws = sqlx::query_as::<_, Workspace>(
            "SELECT * FROM workspaces WHERE id = $1",
        )
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
        role: &str,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO workspace_members (workspace_id, user_id, role, created_at, updated_at) 
             VALUES ($1, $2, $3, NOW(), NOW())",
        )
        .bind(workspace_id)
        .bind(user_id)
        .bind(role)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn update_member_role(
        &self,
        workspace_id: Uuid,
        user_id: Uuid,
        role: &str,
    ) -> Result<(), String> {
        sqlx::query(
            "UPDATE workspace_members SET role = $1, updated_at = NOW() WHERE workspace_id = $2 AND user_id = $3",
        )
        .bind(role)
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
    ) -> Result<Option<String>, String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT role FROM workspace_members WHERE workspace_id = $1 AND user_id = $2",
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
}
