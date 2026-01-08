pub mod agent_repository;
pub mod workspace_repository;

pub use agent_repository::AgentRepository;
pub use workspace_repository::WorkspaceRepository;

use async_trait::async_trait;
use sqlx::PgPool;
use crate::models::{AuthorizedAgent, Device, Workspace, MemberExtraRole};
use uuid::Uuid;

pub struct SqlxAgentRepository {
    pub pool: PgPool,
}

#[async_trait]
impl AgentRepository for SqlxAgentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String> {
        sqlx::query_as::<_, AuthorizedAgent>("SELECT * FROM authorized_agents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }

    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String> {
        sqlx::query("UPDATE authorized_agents SET last_ip = $1 WHERE id = $2")
            .bind(ip)
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    async fn find_devices_by_agent_id(&self, agent_id: &str) -> Result<Vec<Device>, String> {
        sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE agent_id = $1 AND is_enabled = true")
            .bind(agent_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }
}

pub struct SqlxWorkspaceRepository {
    pub pool: PgPool,
}

#[async_trait]
impl WorkspaceRepository for SqlxWorkspaceRepository {
    async fn create_workspace(&self, name: &str) -> Result<Workspace, String> {
        sqlx::query_as::<_, Workspace>("INSERT INTO workspaces (name) VALUES ($1) RETURNING *")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }

    async fn add_member(&self, workspace_id: Uuid, user_id: Uuid, role: &str) -> Result<(), String> {
        sqlx::query("INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3)")
            .bind(workspace_id)
            .bind(user_id)
            .bind(role)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    async fn find_primary_role(&self, workspace_id: Uuid, user_id: Uuid) -> Result<Option<String>, String> {
        sqlx::query_scalar("SELECT role FROM workspace_members WHERE workspace_id = $1 AND user_id = $2")
            .bind(workspace_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }

    async fn find_extra_roles(&self, workspace_id: Uuid, user_id: Uuid) -> Result<Vec<MemberExtraRole>, String> {
        sqlx::query_as::<_, MemberExtraRole>("SELECT * FROM member_extra_roles WHERE workspace_id = $1 AND user_id = $2")
            .bind(workspace_id)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())
    }
}
