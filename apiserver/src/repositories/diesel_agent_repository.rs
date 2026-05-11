use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    models::{
        diesel::agent::{AuthorizedAgentEntity, DeviceEntity, UpdateDieselDevice},
        AuthorizedAgent, Device,
    },
    schema::{authorized_agents, devices},
};

use super::AgentRepository;

pub struct DieselAgentRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselAgentRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl AgentRepository for DieselAgentRepository {
    async fn find_by_id(&self, id_val: &str) -> Result<Option<AuthorizedAgent>, String> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| e.to_string())?;

        let agent = authorized_agents::table
            .filter(authorized_agents::id.eq(id_val))
            .filter(authorized_agents::is_active.eq(true))
            .first::<AuthorizedAgentEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(agent.map(Into::into))
    }

    async fn update_last_ip(&self, id_val: &str, ip_val: &str) -> Result<(), String> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| e.to_string())?;

        diesel::update(authorized_agents::table.filter(authorized_agents::id.eq(id_val)))
            .set(authorized_agents::last_ip.eq(ip_val))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn update_last_seen(&self, id_val: &str) -> Result<(), String> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| e.to_string())?;

        diesel::update(authorized_agents::table.filter(authorized_agents::id.eq(id_val)))
            .set(authorized_agents::last_seen_at.eq(Some(Utc::now())))
            .execute(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn find_devices_by_agent_id(&self, agent_id_val: &str) -> Result<Vec<Device>, String> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| e.to_string())?;

        let devices_result = devices::table
            .filter(devices::agent_id.eq(agent_id_val))
            .order(devices::created_at.desc())
            .load::<DeviceEntity>(&mut conn)
            .await
            .map_err(|e| e.to_string())?;

        Ok(devices_result.into_iter().map(Into::into).collect())
    }

    async fn get_workspace_agents(
        &self,
        workspace_id_val: Uuid,
        include_inactive: bool,
    ) -> Result<Vec<AuthorizedAgent>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let mut query = authorized_agents::table
            .filter(authorized_agents::workspace_id.eq(workspace_id_val))
            .into_boxed();

        if !include_inactive {
            query = query.filter(authorized_agents::is_active.eq(true));
        }

        let agents = query
            .order(authorized_agents::created_at.desc())
            .load::<AuthorizedAgentEntity>(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(agents.into_iter().map(Into::into).collect())
    }

    async fn create_authorized_agent(
        &self,
        id_val: &str,
        name_val: &str,
        token_val: &str,
        workspace_id_val: Uuid,
        max_concurrent_workers_val: i32,
        success_cooldown_secs_val: i32,
        retention_days_val: i32,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let agent = AuthorizedAgentEntity {
            id: id_val.to_string(),
            name: name_val.to_string(),
            token: token_val.to_string(),
            is_trusted: None,
            last_ip: None,
            created_at: Some(Utc::now()),
            is_active: true,
            workspace_id: workspace_id_val,
            last_seen_at: None,
            max_concurrent_workers: max_concurrent_workers_val,
            success_cooldown_secs: success_cooldown_secs_val,
            retention_days: retention_days_val,
        };

        diesel::insert_into(authorized_agents::table)
            .values(&agent)
            .execute(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(())
    }

    async fn agent_exists_in_workspace(
        &self,
        agent_id_val: &str,
        workspace_id_val: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let count: i64 = authorized_agents::table
            .filter(authorized_agents::id.eq(agent_id_val))
            .filter(authorized_agents::workspace_id.eq(workspace_id_val))
            .filter(authorized_agents::is_active.eq(true))
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(count > 0)
    }

    async fn get_workspace_devices(
        &self,
        workspace_id_val: Uuid,
    ) -> Result<Vec<Device>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let devices_result = devices::table
            .filter(devices::workspace_id.eq(workspace_id_val))
            .order(devices::created_at.desc())
            .load::<DeviceEntity>(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(devices_result.into_iter().map(Into::into).collect())
    }

    async fn get_devices_by_agent_ids(
        &self,
        agent_ids_val: &[String],
        workspace_id_val: Uuid,
    ) -> Result<Vec<Device>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let devices_result = devices::table
            .filter(devices::agent_id.eq_any(agent_ids_val))
            .filter(devices::workspace_id.eq(workspace_id_val))
            .order(devices::created_at.desc())
            .load::<DeviceEntity>(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(devices_result.into_iter().map(Into::into).collect())
    }

    async fn create_device(
        &self,
        id_val: Uuid,
        agent_id_val: &str,
        name_val: &str,
        ip_val: &str,
        port_val: i32,
        workspace_id_val: Uuid,
        brand_val: &str,
        password_val: &str,
        timezone_val: &str,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let device = DeviceEntity {
            id: id_val,
            agent_id: agent_id_val.to_string(),
            name: name_val.to_string(),
            ip: ip_val.to_string(),
            port: Some(port_val),
            is_enabled: Some(true),
            last_status: None,
            created_at: Some(Utc::now()),
            workspace_id: workspace_id_val,
            brand: brand_val.to_string(),
            password: Some(password_val.to_string()),
            timezone: Some(timezone_val.to_string()),
        };

        diesel::insert_into(devices::table)
            .values(&device)
            .execute(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(())
    }

    async fn get_device_by_id(
        &self,
        id_val: Uuid,
        workspace_id_val: Uuid,
    ) -> Result<Option<Device>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let device = devices::table
            .filter(devices::id.eq(id_val))
            .filter(devices::workspace_id.eq(workspace_id_val))
            .first::<DeviceEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(device.map(Into::into))
    }

    async fn update_device(
        &self,
        id_val: Uuid,
        workspace_id_val: Uuid,
        name_val: &str,
        ip_val: &str,
        port_val: i32,
        is_enabled_val: bool,
        brand_val: &str,
        password_val: Option<&str>,
        timezone_val: &str,
    ) -> Result<(), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        // Note: Password update logic. In sqlx it was: password = COALESCE($6, password).
        // Since we use AsChangeset, we can fetch existing if password is None, or we can just fetch and update.
        let mut update_model = UpdateDieselDevice {
            name: name_val.to_string(),
            ip: ip_val.to_string(),
            port: Some(port_val),
            is_enabled: Some(is_enabled_val),
            brand: brand_val.to_string(),
            password: password_val.map(ToString::to_string),
            timezone: Some(timezone_val.to_string()),
        };

        if update_model.password.is_none() {
            // Fetch existing password to preserve it
            let existing: Option<String> = devices::table
                .select(devices::password)
                .filter(devices::id.eq(id_val))
                .filter(devices::workspace_id.eq(workspace_id_val))
                .first(&mut conn)
                .await
                .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;
            update_model.password = existing;
        }

        diesel::update(
            devices::table
                .filter(devices::id.eq(id_val))
                .filter(devices::workspace_id.eq(workspace_id_val)),
        )
        .set(&update_model)
        .execute(&mut conn)
        .await
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(())
    }

    async fn get_agent_id_for_device(
        &self,
        device_id_val: Uuid,
        workspace_id_val: Uuid,
    ) -> Result<Option<String>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let agent_id = devices::table
            .select(devices::agent_id)
            .filter(devices::id.eq(device_id_val))
            .filter(devices::workspace_id.eq(workspace_id_val))
            .first::<String>(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(agent_id)
    }

    async fn delete_device(
        &self,
        device_id_val: Uuid,
        workspace_id_val: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let affected = diesel::delete(
            devices::table
                .filter(devices::id.eq(device_id_val))
                .filter(devices::workspace_id.eq(workspace_id_val)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(affected as u64)
    }

    async fn update_agent_trust(
        &self,
        agent_id_val: &str,
        workspace_id_val: Uuid,
        is_trusted_val: bool,
    ) -> Result<u64, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let affected = diesel::update(
            authorized_agents::table
                .filter(authorized_agents::id.eq(agent_id_val))
                .filter(authorized_agents::workspace_id.eq(workspace_id_val)),
        )
        .set(authorized_agents::is_trusted.eq(Some(is_trusted_val)))
        .execute(&mut conn)
        .await
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(affected as u64)
    }

    async fn get_agent_by_id_in_workspace(
        &self,
        agent_id_val: &str,
        workspace_id_val: Uuid,
    ) -> Result<Option<AuthorizedAgent>, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let agent = authorized_agents::table
            .filter(authorized_agents::id.eq(agent_id_val))
            .filter(authorized_agents::workspace_id.eq(workspace_id_val))
            .first::<AuthorizedAgentEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(agent.map(Into::into))
    }

    async fn deactivate_agent(
        &self,
        agent_id_val: &str,
        workspace_id_val: Uuid,
    ) -> Result<u64, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let affected = diesel::update(
            authorized_agents::table
                .filter(authorized_agents::id.eq(agent_id_val))
                .filter(authorized_agents::workspace_id.eq(workspace_id_val)),
        )
        .set(authorized_agents::is_active.eq(false))
        .execute(&mut conn)
        .await
        .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(affected as u64)
    }
}

impl crate::repositories::PoolBacked for DieselAgentRepository {
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!("DieselAgentRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}
