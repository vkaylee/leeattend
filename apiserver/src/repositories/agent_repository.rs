use crate::models::{AuthorizedAgent, Device};
use async_trait::async_trait;
use sqlx::PgPool;

#[async_trait]
pub trait AgentRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, String>;
    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), String>;
    async fn update_last_seen(&self, id: &str) -> Result<(), String>;
    async fn find_devices_by_agent_id(&self, agent_id: &str) -> Result<Vec<Device>, String>;
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
        sqlx::query("UPDATE authorized_agents SET last_seen_at = CURRENT_TIMESTAMP WHERE id = $1")
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
}
