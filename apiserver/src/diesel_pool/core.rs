use crate::diesel_pool::config::DbPoolConfig;
use crate::diesel_pool::tenant_manager::{
    TenantAuthToken, TenantConnectionProvider, TenantPoolManager, TestTenantPoolManager,
};
use crate::errors::AppError;
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::sync::Arc;
use std::time::Duration;

pub type DieselPool = Pool<AsyncPgConnection>;
pub type DieselConnection = Object<AsyncPgConnection>;

pub struct TransactionConn(pub DieselConnection);

impl std::ops::Deref for TransactionConn {
    type Target = DieselConnection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TransactionConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct SessionConn(pub DieselConnection);

impl std::ops::Deref for SessionConn {
    type Target = DieselConnection;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SessionConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct DieselPools {
    pub api: DieselPool,
    pub worker: DieselPool,
    pub tenants_tx: Arc<dyn TenantConnectionProvider>,
    pub tenants_sess: Arc<dyn TenantConnectionProvider>,
}

impl DieselPools {
    // Hàm an toàn (Khuyên dùng)
    pub async fn tenant_transaction_conn(
        &self,
        token: &TenantAuthToken,
    ) -> Result<TransactionConn, AppError> {
        let conn = self
            .tenants_tx
            .get_connection(&token.workspace_id())
            .await?;
        Ok(TransactionConn(conn))
    }

    pub async fn tenant_session_conn(
        &self,
        token: &TenantAuthToken,
    ) -> Result<SessionConn, AppError> {
        let conn = self
            .tenants_sess
            .get_connection(&token.workspace_id())
            .await?;
        Ok(SessionConn(conn))
    }

    pub fn connect(config: &DbPoolConfig) -> Result<Self, AppError> {
        tracing::info!(
            api_max = config.api_max_conn,
            worker_max = config.worker_max_conn,
            "Initializing Diesel pools"
        );

        let (api_tx_url, worker_sess_url, _) = config.resolve_pool_urls();

        let mut api_config =
            diesel_async::pooled_connection::ManagerConfig::<AsyncPgConnection>::default();
        api_config.recycling_method = diesel_async::pooled_connection::RecyclingMethod::Fast;
        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            api_tx_url, api_config,
        );
        let api_pool = Pool::builder(api_manager)
            .max_size(config.api_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(config.api_acquire_timeout_secs)))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create API Diesel pool: {e}")))?;

        let mut worker_config =
            diesel_async::pooled_connection::ManagerConfig::<AsyncPgConnection>::default();
        worker_config.recycling_method = diesel_async::pooled_connection::RecyclingMethod::Fast;
        let worker_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            worker_sess_url,
            worker_config,
        );
        let worker_pool = Pool::builder(worker_manager)
            .max_size(config.worker_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(
                config.worker_acquire_timeout_secs,
            )))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create Worker Diesel pool: {e}")))?;

        #[allow(clippy::clone_on_ref_ptr)]
        let tenant_tx_manager = Arc::new(TenantPoolManager::new(
            config.tenant_pooler_tx_url.clone(),
            config.primary_url.clone(),
            Some(api_pool.clone()),
            config.tenant_max_conn as usize,
        ));
        tenant_tx_manager.start_eviction_task();

        let tenant_sess_manager = Arc::new(TenantPoolManager::new(
            config.tenant_pooler_sess_url.clone(),
            config.primary_url.clone(),
            Some(api_pool.clone()),
            config.tenant_max_conn as usize,
        ));
        tenant_sess_manager.start_eviction_task();

        tracing::info!("Diesel pools connected successfully");

        Ok(Self {
            api: api_pool,
            worker: worker_pool,
            tenants_tx: tenant_tx_manager,
            tenants_sess: tenant_sess_manager,
        })
    }

    pub fn for_test(url: &str) -> Self {
        let mut api_config =
            diesel_async::pooled_connection::ManagerConfig::<AsyncPgConnection>::default();
        api_config.recycling_method = diesel_async::pooled_connection::RecyclingMethod::Fast;
        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            url.to_string(),
            api_config,
        );
        let api_pool = Pool::builder(api_manager)
            .max_size(5)
            .build()
            .unwrap_or_else(|e| {
                std::panic::resume_unwind(Box::new(format!(
                    "Failed to create test API Diesel pool: {e}"
                )))
            });

        let mut worker_config =
            diesel_async::pooled_connection::ManagerConfig::<AsyncPgConnection>::default();
        worker_config.recycling_method = diesel_async::pooled_connection::RecyclingMethod::Fast;
        let worker_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            url.to_string(),
            worker_config,
        );
        let worker_pool = Pool::builder(worker_manager)
            .max_size(5)
            .build()
            .unwrap_or_else(|e| {
                std::panic::resume_unwind(Box::new(format!(
                    "Failed to create test Worker Diesel pool: {e}"
                )))
            });

        let tenant_manager = Arc::new(TestTenantPoolManager::new(api_pool.clone()));

        Self {
            api: api_pool,
            worker: worker_pool,
            #[allow(clippy::clone_on_ref_ptr)]
            tenants_tx: tenant_manager.clone(),
            tenants_sess: tenant_manager,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_conn_success() {
        let pools = DieselPools::for_test("postgres://user:pass@localhost:5432/leeattend_test_db");
        let token = TenantAuthToken::system_override(uuid::Uuid::new_v4());
        let _tx_conn = pools.tenant_transaction_conn(&token).await;
        let _sess_conn = pools.tenant_session_conn(&token).await;
    }

    #[tokio::test]
    async fn test_connect() {
        // This won't actually try to connect if the driver parses the URL successfully,
        // but it will initialize the pool builder. We just use a valid-looking URL.
        std::env::set_var(
            "DATABASE_URL",
            "postgres://user:pass@localhost:5432/leeattend_test_db",
        );

        let config = DbPoolConfig {
            primary_url: "postgres://postgres:postgres@localhost:5432/test_db".to_string(),
            pooler_tx_url: String::new(),
            pooler_sess_url: String::new(),
            tenant_pooler_tx_url: String::new(),
            tenant_pooler_sess_url: String::new(),
            reader_url: None,
            api_max_conn: 1,
            worker_max_conn: 1,
            tenant_max_conn: 1,
            api_acquire_timeout_secs: 1,
            worker_acquire_timeout_secs: 1,
        };

        let result = DieselPools::connect(&config);

        // As long as the URLs are well-formed, it will return Ok
        assert!(result.is_ok());
    }
}
