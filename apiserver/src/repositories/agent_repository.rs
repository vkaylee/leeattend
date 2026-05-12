use crate::errors::AppError;
use crate::models::{AuthorizedAgent, Device};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait AgentRepository: super::PoolBacked + Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String>;
    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String>;
    async fn update_last_seen(&self, id: &str) -> Result<(), String>;
    async fn find_devices_by_agent_id(&self, agent_id: &str) -> Result<Vec<Device>, String>;
    async fn get_workspace_agents(
        &self,
        workspace_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<AuthorizedAgent>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn create_authorized_agent(
        &self,
        id: &str,
        name: &str,
        token: &str,
        workspace_id: Uuid,
        max_concurrent_workers: i32,
        success_cooldown_secs: i32,
        retention_days: i32,
    ) -> Result<(), AppError>;
    async fn agent_exists_in_workspace(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<bool, AppError>;
    async fn get_workspace_devices(&self, workspace_id: Uuid) -> Result<Vec<Device>, AppError>;
    async fn get_devices_by_agent_ids(
        &self,
        agent_ids: &[String],
        workspace_id: Uuid,
    ) -> Result<Vec<Device>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn create_device(
        &self,
        id: Uuid,
        agent_id: &str,
        name: &str,
        ip: &str,
        port: i32,
        workspace_id: Uuid,
        brand: &str,
        password: &str,
        timezone: &str,
    ) -> Result<(), AppError>;
    async fn get_device_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Device>, AppError>;
    #[allow(clippy::too_many_arguments)]
    async fn update_device(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: &str,
        ip: &str,
        port: i32,
        is_enabled: bool,
        brand: &str,
        password: Option<&str>,
        timezone: &str,
    ) -> Result<(), AppError>;
    async fn get_agent_id_for_device(
        &self,
        device_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<String>, AppError>;
    async fn delete_device(&self, device_id: Uuid, workspace_id: Uuid) -> Result<u64, AppError>;
    async fn update_agent_trust(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
        is_trusted: bool,
    ) -> Result<u64, AppError>;
    async fn get_agent_by_id_in_workspace(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<Option<AuthorizedAgent>, AppError>;
    async fn deactivate_agent(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<u64, AppError>;
}

pub struct SqlxAgentRepository {
    pub pool: PgPool,
}

impl SqlxAgentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AgentRepository for SqlxAgentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String> {
        let agent = sqlx::query_as::<_, AuthorizedAgent>(
            "SELECT * FROM authorized_agents WHERE id = $1 AND is_active = true",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(agent)
    }

    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String> {
        sqlx::query("UPDATE authorized_agents SET last_ip = $1 WHERE id = $2")
            .bind(ip)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn update_last_seen(&self, id: &str) -> Result<(), String> {
        sqlx::query("UPDATE authorized_agents SET last_seen_at = $1 WHERE id = $2")
            .bind(chrono::Utc::now())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn find_devices_by_agent_id(&self, agent_id: &str) -> Result<Vec<Device>, String> {
        let devices = sqlx::query_as::<_, Device>(
            "SELECT * FROM devices WHERE agent_id = $1 ORDER BY created_at DESC",
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(devices)
    }

    async fn get_workspace_agents(
        &self,
        workspace_id: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<AuthorizedAgent>, AppError> {
        let query = if include_inactive {
            "SELECT * FROM authorized_agents WHERE workspace_id = $1 ORDER BY created_at DESC"
        } else {
            "SELECT * FROM authorized_agents WHERE workspace_id = $1 AND is_active = TRUE ORDER BY created_at DESC"
        };
        sqlx::query_as::<_, AuthorizedAgent>(query)
            .bind(workspace_id)
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn create_authorized_agent(
        &self,
        id: &str,
        name: &str,
        token: &str,
        workspace_id: Uuid,
        max_concurrent_workers: i32,
        success_cooldown_secs: i32,
        retention_days: i32,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO authorized_agents (id, name, token, workspace_id, max_concurrent_workers, success_cooldown_secs, retention_days) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(id)
        .bind(name)
        .bind(token)
        .bind(workspace_id)
        .bind(max_concurrent_workers)
        .bind(success_cooldown_secs)
        .bind(retention_days)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn agent_exists_in_workspace(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<bool, AppError> {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM authorized_agents WHERE id = $1 AND workspace_id = $2 AND is_active = TRUE)",
        )
        .bind(agent_id)
        .bind(workspace_id)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn get_workspace_devices(&self, workspace_id: Uuid) -> Result<Vec<Device>, AppError> {
        sqlx::query_as::<_, Device>(
            "SELECT * FROM devices WHERE workspace_id = $1 ORDER BY created_at DESC",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn get_devices_by_agent_ids(
        &self,
        agent_ids: &[String],
        workspace_id: Uuid,
    ) -> Result<Vec<Device>, AppError> {
        sqlx::query_as::<_, Device>(
            "SELECT * FROM devices WHERE agent_id = ANY($1) AND workspace_id = $2 ORDER BY created_at DESC",
        )
        .bind(agent_ids)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn create_device(
        &self,
        id: Uuid,
        agent_id: &str,
        name: &str,
        ip: &str,
        port: i32,
        workspace_id: Uuid,
        brand: &str,
        password: &str,
        timezone: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO devices (id, agent_id, name, ip, port, workspace_id, brand, password, timezone) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(id)
        .bind(agent_id)
        .bind(name)
        .bind(ip)
        .bind(port)
        .bind(workspace_id)
        .bind(brand)
        .bind(password)
        .bind(timezone)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_device_by_id(
        &self,
        id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<Device>, AppError> {
        sqlx::query_as::<_, Device>("SELECT * FROM devices WHERE id = $1 AND workspace_id = $2")
            .bind(id)
            .bind(workspace_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn update_device(
        &self,
        id: Uuid,
        workspace_id: Uuid,
        name: &str,
        ip: &str,
        port: i32,
        is_enabled: bool,
        brand: &str,
        password: Option<&str>,
        timezone: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE devices SET name = $1, ip = $2, port = $3, is_enabled = $4, brand = $5, password = COALESCE($6, password), timezone = $7 WHERE id = $8 AND workspace_id = $9",
        )
        .bind(name)
        .bind(ip)
        .bind(port)
        .bind(is_enabled)
        .bind(brand)
        .bind(password)
        .bind(timezone)
        .bind(id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_agent_id_for_device(
        &self,
        device_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        sqlx::query_scalar("SELECT agent_id FROM devices WHERE id = $1 AND workspace_id = $2")
            .bind(device_id)
            .bind(workspace_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    async fn delete_device(&self, device_id: Uuid, workspace_id: Uuid) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM devices WHERE id = $1 AND workspace_id = $2")
            .bind(device_id)
            .bind(workspace_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    async fn update_agent_trust(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
        is_trusted: bool,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            "UPDATE authorized_agents SET is_trusted = $1 WHERE id = $2 AND workspace_id = $3",
        )
        .bind(is_trusted)
        .bind(agent_id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    async fn get_agent_by_id_in_workspace(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<Option<AuthorizedAgent>, AppError> {
        sqlx::query_as::<_, AuthorizedAgent>(
            "SELECT * FROM authorized_agents WHERE id = $1 AND workspace_id = $2",
        )
        .bind(agent_id)
        .bind(workspace_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn deactivate_agent(
        &self,
        agent_id: &str,
        workspace_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            "UPDATE authorized_agents SET is_active = FALSE WHERE id = $1 AND workspace_id = $2",
        )
        .bind(agent_id)
        .bind(workspace_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}

impl super::PoolBacked for SqlxAgentRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
