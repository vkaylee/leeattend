use crate::diesel_pool::config::DbPoolConfig;
use crate::diesel_pool::tenant_manager::{
    SupavisorTenantPoolManager, TenantAuthToken, TenantConnectionProvider, TestTenantPoolManager,
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

        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(api_tx_url);
        let api_pool = Pool::builder(api_manager)
            .max_size(config.api_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(config.api_acquire_timeout_secs)))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create API Diesel pool: {e}")))?;

        let worker_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(worker_sess_url);
        let worker_pool = Pool::builder(worker_manager)
            .max_size(config.worker_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(
                config.worker_acquire_timeout_secs,
            )))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create Worker Diesel pool: {e}")))?;

        let tenant_tx_manager = Arc::new(SupavisorTenantPoolManager::new(
            api_tx_url.to_string(),
            config.tenant_max_conn as usize,
        ));

        let tenant_sess_manager = Arc::new(SupavisorTenantPoolManager::new(
            worker_sess_url.to_string(), // sess_url was returned as the second element in resolve_pool_urls
            config.tenant_max_conn as usize,
        ));

        tracing::info!("Diesel pools connected successfully");

        Ok(Self {
            api: api_pool,
            worker: worker_pool,
            tenants_tx: tenant_tx_manager,
            tenants_sess: tenant_sess_manager,
        })
    }

    #[allow(clippy::panic)]
    pub fn for_test(url: &str) -> Self {
        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url.to_string());
        let api_pool = Pool::builder(api_manager)
            .max_size(5)
            .build()
            .unwrap_or_else(|e| panic!("Failed to create test API Diesel pool: {e}"));

        let worker_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(url.to_string());
        let worker_pool = Pool::builder(worker_manager)
            .max_size(5)
            .build()
            .unwrap_or_else(|e| panic!("Failed to create test Worker Diesel pool: {e}"));

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
