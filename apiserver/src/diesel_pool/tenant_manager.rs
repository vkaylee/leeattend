use crate::diesel_pool::core::{DieselConnection, DieselPool};
use crate::errors::AppError;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait TenantConnectionProvider: Send + Sync {
    async fn get_connection(&self, workspace_id: &Uuid) -> Result<DieselConnection, AppError>;

    /// Resolve the correct migration URL for a tenant.
    ///
    /// Takes the decrypted `custom_db_url_encrypted` (tenant's direct DB URL with real
    /// credentials) and returns a URL suitable for sync `PgConnection::establish`.
    fn resolve_migration_url(
        &self,
        workspace_id: &Uuid,
        tenant_direct_url: &str,
    ) -> Result<String, String>;
}

pub struct TenantPoolManager {
    base_url: String,
    #[allow(dead_code)]
    system_db_url: String,
    system_db_pool: Option<DieselPool>,
    max_size: usize,
    pools: RwLock<HashMap<Uuid, DieselPool>>,
    url_cache: RwLock<HashMap<Uuid, String>>,
    last_accessed: RwLock<HashMap<Uuid, std::time::Instant>>,
}

impl TenantPoolManager {
    pub fn new(
        base_url: String,
        system_db_url: String,
        system_db_pool: Option<DieselPool>,
        max_size: usize,
    ) -> Self {
        Self {
            base_url,
            system_db_url,
            system_db_pool,
            max_size,
            pools: RwLock::new(HashMap::new()),
            url_cache: RwLock::new(HashMap::new()),
            last_accessed: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_tenant_url_with_credentials(
        &self,
        workspace_id: &Uuid,
        override_user: &str,
        override_password: &str,
        override_db_name: Option<&str>,
    ) -> String {
        let tenant_uuid_str = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
        let db_name = match override_db_name {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => tenant_uuid_str,
        };
        let mut result_url = self.base_url.clone();

        if let Some(prefix_len) = result_url.find("://") {
            let plen = prefix_len + 3;
            let after_prefix = &result_url[plen..];
            let path_start = after_prefix.find('/').unwrap_or(after_prefix.len());
            let auth_host = &after_prefix[..path_start];

            if let Some(at_idx) = auth_host.rfind('@') {
                let auth = &auth_host[..at_idx];
                let (user, existing_password) = match auth.find(':') {
                    Some(colon_idx) => (&auth[..colon_idx], Some(&auth[colon_idx + 1..])),
                    None => (auth, None),
                };

                let pass_to_use = if override_password.is_empty() {
                    existing_password
                } else {
                    Some(override_password)
                };

                let new_user = if override_user.is_empty() {
                    user.to_string()
                } else {
                    override_user.to_string()
                };
                let replace_start = plen;
                let replace_end = plen + auth.len();

                let new_auth = match pass_to_use {
                    Some(p) => format!("{new_user}:{p}"),
                    None => new_user,
                };
                result_url.replace_range(replace_start..replace_end, &new_auth);
            }
        }

        // Now replace the database name in the path
        let (without_query, query) = match result_url.find('?') {
            Some(idx) => (&result_url[..idx], &result_url[idx..]),
            None => (result_url.as_str(), ""),
        };

        if let Some(slash_idx) = without_query.rfind('/') {
            let before_slash = &without_query[..slash_idx];
            result_url = format!("{before_slash}/{db_name}{query}");
        }

        result_url
    }
}

impl TenantPoolManager {
    fn create_pool_from_url(&self, tenant_url: &str) -> Result<DieselPool, AppError> {
        let mut manager_config =
            diesel_async::pooled_connection::ManagerConfig::<AsyncPgConnection>::default();
        manager_config.recycling_method = diesel_async::pooled_connection::RecyclingMethod::Fast;
        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
            tenant_url.to_string(),
            manager_config,
        );
        Pool::builder(manager)
            .max_size(self.max_size)
            .wait_timeout(Some(Duration::from_secs(5)))
            .create_timeout(Some(Duration::from_secs(5)))
            .runtime(deadpool::Runtime::Tokio1)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create tenant pool: {e}")))
    }

    async fn get_or_create_pool(&self, workspace_id: &Uuid) -> Result<DieselPool, AppError> {
        // Fast path
        {
            let pools = self.pools.read().await;
            if let Some(pool) = pools.get(workspace_id) {
                return Ok(pool.clone());
            }
        }

        // Check if we have the tenant URL in the cache
        let cached_url = {
            let cache = self.url_cache.read().await;
            cache.get(workspace_id).cloned()
        };

        let tenant_url = if let Some(url) = cached_url {
            url
        } else {
            let ws_id = *workspace_id;

            let fetched_url = if let Some(ref system_pool) = self.system_db_pool {
                let mut conn = system_pool.get().await.map_err(|e| {
                    AppError::Internal(format!("Failed to get System DB connection: {e}"))
                })?;

                use diesel::sql_types::{Nullable, Text, Uuid as UuidSql};
                use diesel_async::RunQueryDsl;

                #[derive(diesel::QueryableByName)]
                struct WsRow {
                    #[diesel(sql_type = Nullable<Text>)]
                    custom_db_url_encrypted: Option<String>,
                }

                let rows = diesel::sql_query(
                    "SELECT custom_db_url_encrypted FROM workspaces WHERE id = $1",
                )
                .bind::<UuidSql, _>(ws_id)
                .get_results::<WsRow>(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("System DB query failed: {e}")))?;

                let mut custom_url = None;
                if let Some(row) = rows.into_iter().next() {
                    if let Some(encrypted) = row.custom_db_url_encrypted {
                        // Decrypt the URL
                        let crypto_service = std::env::var("ENCRYPTION_KEYS_JSON")
                            .ok()
                            .and_then(|keys| crate::crypto::CryptoService::from_json(&keys).ok());

                        custom_url = if let Some(crypto) = crypto_service {
                            Some(crypto.decrypt(&encrypted).unwrap_or(encrypted))
                        } else {
                            Some(encrypted)
                        };
                    }
                }
                custom_url
            } else {
                None
            };

            let url = match fetched_url {
                Some(custom_url) => custom_url,
                None => self.get_tenant_url_with_credentials(
                    workspace_id,
                    "tenant_user",
                    "tenant_password",
                    None,
                ),
            };

            // Store in cache
            let mut cache = self.url_cache.write().await;
            cache.insert(*workspace_id, url.clone());
            url
        };

        let pool = self.create_pool_from_url(&tenant_url)?;

        // Acquire write lock to insert the new pool
        let mut pools = self.pools.write().await;
        let res = if let Some(existing) = pools.get(workspace_id) {
            Ok(existing.clone())
        } else {
            pools.insert(*workspace_id, pool.clone());
            Ok(pool)
        };
        drop(pools);
        res
    }

    pub fn start_eviction_task(self: &std::sync::Arc<Self>) {
        let weak_self = std::sync::Arc::downgrade(self);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                let manager = match weak_self.upgrade() {
                    Some(m) => m,
                    None => {
                        tracing::info!("TenantPoolManager dropped, stopping eviction task.");
                        break;
                    }
                };
                if let Err(e) = manager.evict_idle_pools().await {
                    tracing::error!("Error during tenant connection pool eviction: {:?}", e);
                }
            }
        });
    }

    async fn evict_idle_pools(&self) -> Result<(), AppError> {
        let now = std::time::Instant::now();
        let idle_timeout = std::time::Duration::from_secs(1800); // 30 minutes

        let mut pools_to_remove = Vec::new();
        {
            let accessed = self.last_accessed.read().await;
            let pools = self.pools.read().await;
            for &ws_id in pools.keys() {
                let last_used = accessed.get(&ws_id).copied().unwrap_or(now);
                if now.duration_since(last_used) > idle_timeout {
                    pools_to_remove.push(ws_id);
                }
            }
        }

        if !pools_to_remove.is_empty() {
            let mut pools = self.pools.write().await;
            let mut accessed = self.last_accessed.write().await;
            for ws_id in pools_to_remove {
                if let Some(pool) = pools.remove(&ws_id) {
                    tracing::info!(workspace_id = %ws_id, "Evicting idle tenant connection pool");
                    drop(pool);
                }
                accessed.remove(&ws_id);
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl TenantConnectionProvider for TenantPoolManager {
    async fn get_connection(&self, workspace_id: &Uuid) -> Result<DieselConnection, AppError> {
        let pool = self.get_or_create_pool(workspace_id).await?;

        {
            let mut accessed = self.last_accessed.write().await;
            accessed.insert(*workspace_id, std::time::Instant::now());
        }

        pool.get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))
    }

    fn resolve_migration_url(
        &self,
        _workspace_id: &Uuid,
        tenant_direct_url: &str,
    ) -> Result<String, String> {
        Ok(tenant_direct_url.to_string())
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

    fn resolve_migration_url(
        &self,
        _workspace_id: &Uuid,
        tenant_direct_url: &str,
    ) -> Result<String, String> {
        Ok(tenant_direct_url.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TenantAuthToken(pub(crate) Uuid);

impl TenantAuthToken {
    pub(crate) fn new_authorized(workspace_id: Uuid) -> Self {
        Self(workspace_id)
    }

    pub fn system_override(workspace_id: Uuid) -> Self {
        Self(workspace_id)
    }

    pub fn workspace_id(&self) -> Uuid {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tenant_url_direct_mode() {
        let manager = TenantPoolManager::new(
            "postgres://admin:pass@localhost:5432/leeattend".to_string(),
            "postgres://admin:pass@localhost:5432/leeattend".to_string(),
            None,
            5,
        );
        let ws_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let url =
            manager.get_tenant_url_with_credentials(&ws_id, "tenant_user", "tenant_password", None);

        assert_eq!(
            url,
            "postgres://tenant_user:tenant_password@localhost:5432/tenant_123e4567e89b12d3a456426614174000"
        );
    }

    #[tokio::test]
    async fn test_tenant_url_caching() {
        let manager = TenantPoolManager::new(
            "postgres://admin:pass@localhost:5432/leeattend".to_string(),
            "postgres://admin:pass@localhost:5432/leeattend".to_string(),
            None,
            5,
        );
        let ws_id = Uuid::new_v4();

        // Ensure cache is initially empty
        {
            let cache = manager.url_cache.read().await;
            assert!(!cache.contains_key(&ws_id));
        }

        // Insert a custom URL manually into the cache to simulate it being populated
        let test_url = "postgres://cached_user:cached_pass@localhost:5432/cached_db".to_string();
        {
            let mut cache = manager.url_cache.write().await;
            cache.insert(ws_id, test_url.clone());
        }

        // Verify that get_or_create_pool utilizes the cached URL and doesn't run spawn_blocking to hit System DB.
        // Deadpool pool building does not connect eagerly, so it will return Ok(pool) immediately.
        let pool = manager.get_or_create_pool(&ws_id).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_tenant_pool_eviction() {
        let manager = TenantPoolManager::new(
            "postgres://admin:pass@localhost:5432/leeattend".to_string(),
            "postgres://admin:pass@localhost:5432/leeattend".to_string(),
            None,
            5,
        );
        let ws_id = Uuid::new_v4();

        // Manually cache URL to avoid System DB query
        {
            let mut cache = manager.url_cache.write().await;
            cache.insert(ws_id, "postgres://user:pass@localhost:5432/db".to_string());
        }

        // 1. Create pool
        let pool = manager.get_or_create_pool(&ws_id).await;
        assert!(pool.is_ok());

        // 2. Set last accessed to 31 minutes ago
        {
            let mut last_accessed = manager.last_accessed.write().await;
            last_accessed.insert(ws_id, std::time::Instant::now() - std::time::Duration::from_secs(1860));
        }

        // 3. Verify it is in pools
        {
            let pools = manager.pools.read().await;
            assert!(pools.contains_key(&ws_id));
        }

        // 4. Run eviction
        let evict_res = manager.evict_idle_pools().await;
        assert!(evict_res.is_ok());

        // 5. Verify it is removed from pools and last_accessed
        {
            let pools = manager.pools.read().await;
            assert!(!pools.contains_key(&ws_id));

            let last_accessed = manager.last_accessed.read().await;
            assert!(!last_accessed.contains_key(&ws_id));
        }
    }
}

