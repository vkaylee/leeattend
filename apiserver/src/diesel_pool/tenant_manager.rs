use crate::errors::AppError;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::time::Duration;
use crate::diesel_pool::core::{DieselConnection, DieselPool};

#[async_trait::async_trait]
pub trait TenantConnectionProvider: Send + Sync {
    async fn get_connection(&self, workspace_id: &Uuid) -> Result<DieselConnection, AppError>;
}

pub struct SupavisorTenantPoolManager {
    base_url: String,
    max_size: usize,
    pools: RwLock<HashMap<Uuid, DieselPool>>,
}

impl SupavisorTenantPoolManager {
    pub fn new(base_url: String, max_size: usize) -> Self {
        Self {
            base_url,
            max_size,
            pools: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_tenant_url(&self, workspace_id: &Uuid) -> Result<String, AppError> {
        let db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));

        let base_url = &self.base_url;
        let (without_query, query) = match base_url.find('?') {
            Some(idx) => (&base_url[..idx], &base_url[idx..]),
            None => (base_url.as_str(), ""),
        };

        let (before_slash, _) = match without_query.rfind('/') {
            Some(idx) => (&without_query[..idx], &without_query[idx + 1..]),
            None => {
                return Err(AppError::Internal(
                    "Invalid base pool URL: missing path slash".into(),
                ))
            }
        };

        let mut result_url = format!("{before_slash}/{db_name}{query}");

        // Now handle username if it contains a dot
        let prefix_len = if result_url.starts_with("postgres://") {
            Some(11)
        } else if result_url.starts_with("postgresql://") {
            Some(13)
        } else {
            None
        };

        if let Some(plen) = prefix_len {
            let after_prefix = &result_url[plen..];
            // find last @ before the path
            let path_start = after_prefix.find('/').unwrap_or(after_prefix.len());
            let auth_host = &after_prefix[..path_start];

            if let Some(at_idx) = auth_host.rfind('@') {
                let auth = &auth_host[..at_idx];
                let user = match auth.find(':') {
                    Some(colon_idx) => &auth[..colon_idx],
                    None => auth,
                };

                if user.contains('.') {
                    if let Some((prefix, _old_tenant)) = user.split_once('.') {
                        let new_user = format!("{prefix}.{db_name}");
                        let replace_start = plen;
                        let replace_end = plen + user.len();
                        result_url.replace_range(replace_start..replace_end, &new_user);
                    }
                }
            }
        }

        Ok(result_url)
    }
}

impl SupavisorTenantPoolManager {
    async fn get_or_create_pool(&self, workspace_id: &Uuid) -> Result<DieselPool, AppError> {
        // Fast path
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(workspace_id) {
                return Ok(pool.clone());
            }
        }

        // Slow path
        let mut pools = self.pools.write().await;
        if let Some(pool) = pools.get(workspace_id) {
            return Ok(pool.clone());
        }

        let tenant_url = self.get_tenant_url(workspace_id)?;
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(tenant_url);
        let pool = Pool::builder(manager)
            .max_size(self.max_size)
            .wait_timeout(Some(Duration::from_secs(5)))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create tenant pool: {e}")))?;

        pools.insert(*workspace_id, pool.clone());
        Ok(pool)
    }
}

#[async_trait::async_trait]
impl TenantConnectionProvider for SupavisorTenantPoolManager {
    async fn get_connection(&self, workspace_id: &Uuid) -> Result<DieselConnection, AppError> {
        let pool = self.get_or_create_pool(workspace_id).await?;

        let mut retries = 3;
        loop {
            match pool.get().await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    let err_msg = e.to_string();
                    if retries > 0 && is_transient_supavisor_error(&err_msg) {
                        tracing::warn!(
                            workspace_id = %workspace_id,
                            error = %err_msg,
                            "Transient Supavisor connection error, retrying in 300ms..."
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        retries -= 1;
                        continue;
                    }
                    return Err(AppError::Internal(format!("db: {e}")));
                }
            }
        }
    }
}

pub struct TestTenantPoolManager {
    pub(crate) pool: DieselPool,
}

impl TestTenantPoolManager {
    pub fn new(pool: DieselPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TenantConnectionProvider for TestTenantPoolManager {
    async fn get_connection(&self, _workspace_id: &Uuid) -> Result<DieselConnection, AppError> {
        self.pool
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
    }
}

pub fn is_transient_supavisor_error(err_msg: &str) -> bool {
    // These errors indicate Supavisor is momentarily restarting the tenant process (e.g., during PUT)
    err_msg.contains("No auth_query specified")
        || err_msg.contains("Tenant or user not found")
        || err_msg.contains("terminating connection")
        || err_msg.contains("closed")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantAuthToken(pub(crate) Uuid);

impl TenantAuthToken {
    /// Chỉ được phép khởi tạo bởi hệ thống auth (Extractors) hoặc System Bypass
    pub(crate) fn new_authorized(workspace_id: Uuid) -> Self {
        Self(workspace_id)
    }

    /// Khởi tạo token cho các Background Job, Cron Job, Webhooks (không chạy trong ngữ cảnh HTTP Request)
    pub fn system_override(workspace_id: Uuid) -> Self {
        Self(workspace_id)
    }

    pub fn workspace_id(&self) -> Uuid {
        self.0
    }
}
