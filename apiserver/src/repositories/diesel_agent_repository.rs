use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools,
    errors::AppError,
    models::{
        diesel::agent::{AuthorizedAgentEntity, DeviceEntity, UpdateDieselDevice},
        AuthorizedAgent, Device,
    },
    repositories::agent_repository::{
        CreateAuthorizedAgentArgs, CreateDeviceArgs, UpdateDeviceArgs,
    },
    schema::{authorized_agents, devices},
};

use super::AgentRepository;

pub struct DieselAgentRepository {
    diesel_pools: DieselPools,
}

impl DieselAgentRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl AgentRepository for DieselAgentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<AuthorizedAgent>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?,
        );

        let agent = authorized_agents::table
            .filter(authorized_agents::id.eq(id))
            .filter(authorized_agents::is_active.eq(true))
            .first::<AuthorizedAgentEntity>(&mut *conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(agent.map(Into::into))
    }

    async fn update_last_ip(&self, id: &str, ip: &str) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?,
        );

        diesel::update(authorized_agents::table.filter(authorized_agents::id.eq(id)))
            .set(authorized_agents::last_ip.eq(ip))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn update_last_seen(&self, id: &str) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?,
        );

        diesel::update(authorized_agents::table.filter(authorized_agents::id.eq(id)))
            .set(authorized_agents::last_seen_at.eq(Some(Utc::now())))
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }

    async fn find_devices_by_agent_id(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        agent_id: &str,
    ) -> Result<Vec<Device>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let devices_result = devices::table
            .filter(devices::agent_id.eq(agent_id))
            .filter(devices::workspace_id.eq(auth_token.workspace_id()))
            .order(devices::created_at.desc())
            .load::<DeviceEntity>(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(devices_result.into_iter().map(Into::into).collect())
    }

    async fn get_workspace_agents(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        include_inactive: bool,
    ) -> Result<Vec<AuthorizedAgent>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let mut query = authorized_agents::table
            .filter(authorized_agents::workspace_id.eq(auth_token.workspace_id()))
            .into_boxed();

        if !include_inactive {
            query = query.filter(authorized_agents::is_active.eq(true));
        }

        let agents = query
            .order(authorized_agents::created_at.desc())
            .load::<AuthorizedAgentEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(agents.into_iter().map(Into::into).collect())
    }

    async fn create_authorized_agent(
        &self,
        args: CreateAuthorizedAgentArgs<'_>,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let agent = AuthorizedAgentEntity {
            id: args.id.to_string(),
            name: args.name.to_string(),
            token: crate::crypto::hash_agent_token(args.token),
            is_trusted: None,
            last_ip: None,
            created_at: Some(Utc::now()),
            is_active: true,
            workspace_id: args.auth_token.workspace_id(),
            last_seen_at: None,
            max_concurrent_workers: args.max_concurrent_workers,
            success_cooldown_secs: args.success_cooldown_secs,
            retention_days: args.retention_days,
        };

        diesel::insert_into(authorized_agents::table)
            .values(&agent)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn agent_exists_in_workspace(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<bool, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let count: i64 = authorized_agents::table
            .filter(authorized_agents::id.eq(agent_id))
            .filter(authorized_agents::workspace_id.eq(auth_token.workspace_id()))
            .filter(authorized_agents::is_active.eq(true))
            .count()
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(count > 0)
    }

    async fn get_workspace_devices(
        &self,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Device>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let devices_result = devices::table
            .filter(devices::workspace_id.eq(auth_token.workspace_id()))
            .order(devices::created_at.desc())
            .load::<DeviceEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(devices_result.into_iter().map(Into::into).collect())
    }

    async fn get_devices_by_agent_ids(
        &self,
        agent_ids: &[String],
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Vec<Device>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let devices_result = devices::table
            .filter(devices::agent_id.eq_any(agent_ids))
            .filter(devices::workspace_id.eq(auth_token.workspace_id()))
            .order(devices::created_at.desc())
            .load::<DeviceEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(devices_result.into_iter().map(Into::into).collect())
    }

    async fn create_device(&self, args: CreateDeviceArgs<'_>) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(args.auth_token)
            .await?;

        let device = DeviceEntity {
            id: args.id,
            agent_id: args.agent_id.to_string(),
            name: args.name.to_string(),
            ip: args.ip.to_string(),
            port: Some(args.port),
            is_enabled: Some(true),
            last_status: None,
            created_at: Some(Utc::now()),
            workspace_id: args.auth_token.workspace_id(),
            brand: args.brand.to_string(),
            password: Some(args.password.to_string()),
            timezone: Some(args.timezone.to_string()),
            location_id: None,
        };

        diesel::insert_into(devices::table)
            .values(&device)
            .execute(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_device_by_id(
        &self,
        id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<Device>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let device = devices::table
            .filter(devices::id.eq(id))
            .filter(devices::workspace_id.eq(auth_token.workspace_id()))
            .first::<DeviceEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(device.map(Into::into))
    }

    async fn update_device(&self, args: UpdateDeviceArgs<'_>) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(args.auth_token)
            .await?;

        // Note: Password update logic. In sqlx it was: password = COALESCE($6, password).
        // Since we use AsChangeset, we can fetch existing if password is None, or we can just fetch and update.
        let mut update_model = UpdateDieselDevice {
            name: args.name.to_string(),
            ip: args.ip.to_string(),
            port: Some(args.port),
            is_enabled: Some(args.is_enabled),
            brand: args.brand.to_string(),
            password: args.password.map(ToString::to_string),
            timezone: Some(args.timezone.to_string()),
            location_id: None,
        };

        if update_model.password.is_none() {
            // Fetch existing password to preserve it
            let existing: Option<String> = devices::table
                .select(devices::password)
                .filter(devices::id.eq(args.id))
                .filter(devices::workspace_id.eq(args.auth_token.workspace_id()))
                .first(&mut *conn)
                .await
                .map_err(AppError::from)?;
            update_model.password = existing;
        }

        diesel::update(
            devices::table
                .filter(devices::id.eq(args.id))
                .filter(devices::workspace_id.eq(args.auth_token.workspace_id())),
        )
        .set(&update_model)
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(())
    }

    async fn get_agent_id_for_device(
        &self,
        device_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<String>, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let agent_id = devices::table
            .select(devices::agent_id)
            .filter(devices::id.eq(device_id))
            .filter(devices::workspace_id.eq(auth_token.workspace_id()))
            .first::<String>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(agent_id)
    }

    async fn delete_device(
        &self,
        device_id: Uuid,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        let mut conn = self
            .diesel_pools
            .tenant_transaction_conn(auth_token)
            .await?;

        let affected = diesel::delete(
            devices::table
                .filter(devices::id.eq(device_id))
                .filter(devices::workspace_id.eq(auth_token.workspace_id())),
        )
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(affected as u64)
    }

    async fn update_agent_trust(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
        is_trusted: bool,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::update(
            authorized_agents::table
                .filter(authorized_agents::id.eq(agent_id))
                .filter(authorized_agents::workspace_id.eq(auth_token.workspace_id())),
        )
        .set(authorized_agents::is_trusted.eq(Some(is_trusted)))
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(affected as u64)
    }

    async fn get_agent_by_id_in_workspace(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<Option<AuthorizedAgent>, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let agent = authorized_agents::table
            .filter(authorized_agents::id.eq(agent_id))
            .filter(authorized_agents::workspace_id.eq(auth_token.workspace_id()))
            .first::<AuthorizedAgentEntity>(&mut *conn)
            .await
            .optional()
            .map_err(AppError::from)?;

        Ok(agent.map(Into::into))
    }

    async fn deactivate_agent(
        &self,
        agent_id: &str,
        auth_token: &crate::diesel_pool::TenantAuthToken,
    ) -> Result<u64, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let affected = diesel::update(
            authorized_agents::table
                .filter(authorized_agents::id.eq(agent_id))
                .filter(authorized_agents::workspace_id.eq(auth_token.workspace_id())),
        )
        .set(authorized_agents::is_active.eq(false))
        .execute(&mut *conn)
        .await
        .map_err(AppError::from)?;

        Ok(affected as u64)
    }

    async fn update_authorized_agent(&self, agent: &AuthorizedAgent) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?,
        );

        diesel::update(
            authorized_agents::table
                .filter(authorized_agents::id.eq(&agent.id))
                .filter(authorized_agents::workspace_id.eq(agent.workspace_id)),
        )
        .set((
            authorized_agents::name.eq(&agent.name),
            authorized_agents::token.eq(&agent.token),
            authorized_agents::is_trusted.eq(Some(agent.is_trusted)),
            authorized_agents::is_active.eq(agent.is_active),
            authorized_agents::max_concurrent_workers.eq(agent.max_concurrent_workers),
            authorized_agents::success_cooldown_secs.eq(agent.success_cooldown_secs),
            authorized_agents::retention_days.eq(agent.retention_days),
        ))
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselAgentRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
