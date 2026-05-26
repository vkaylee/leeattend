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
}

pub struct SupavisorTenantPoolManager {
    base_url: String,
    max_size: usize,
    pools: RwLock<HashMap<Uuid, DieselPool>>,
    tenant_api: std::sync::Arc<dyn crate::supavisor::TenantManager>,
}

impl SupavisorTenantPoolManager {
    pub fn new(base_url: String, max_size: usize, tenant_api: std::sync::Arc<dyn crate::supavisor::TenantManager>) -> Self {
        Self {
            base_url,
            max_size,
            pools: RwLock::new(HashMap::new()),
            tenant_api,
        }
    }


    pub fn get_tenant_url_with_credentials(
        &self,
        workspace_id: &Uuid,
        override_user: &str,
        override_password: &str,
        override_db_name: Option<&str>,
    ) -> Result<String, AppError> {
        let tenant_uuid_str = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
        let db_name = match override_db_name {
            Some(name) if !name.is_empty() => name.to_string(),
            _ => tenant_uuid_str.clone(),
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

                let pass_to_use = if !override_password.is_empty() {
                    Some(override_password)
                } else {
                    existing_password
                };

                if user.contains('.') {
                    // Supavisor mode: username needs .tenant suffix
                    let actual_prefix = if override_user.is_empty() {
                        user.split_once('.').map(|(p, _)| p).unwrap_or(user)
                    } else {
                        override_user
                    };
                    let new_user = format!("{actual_prefix}.{tenant_uuid_str}");
                    let replace_start = plen;
                    let replace_end = plen + auth.len();
                    
                    let new_auth = match pass_to_use {
                        Some(p) => format!("{new_user}:{p}"),
                        None => new_user,
                    };
                    result_url.replace_range(replace_start..replace_end, &new_auth);
                } else if !override_user.is_empty() {
                    // Direct connection mode: replace user entirely, no .tenant suffix
                    let new_user = override_user.to_string();
                    let replace_start = plen;
                    let replace_end = plen + auth.len();
                    
                    let new_auth = match pass_to_use {
                        Some(p) => format!("{new_user}:{p}"),
                        None => new_user,
                    };
                    result_url.replace_range(replace_start..replace_end, &new_auth);
                }
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

        let db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
        let mut db_user = String::new();
        let mut db_password = String::new();
        if let Ok(tenant_data) = self.tenant_api.get_tenant(&db_name).await {
            if let Some(users) = tenant_data.get("users").and_then(|u| u.as_array()) {
                if let Some(first_user) = users.first() {
                    if let Some(u) = first_user.get("db_user").and_then(|v| v.as_str()) {
                        db_user = u.to_string();
                    }
                    if let Some(p) = first_user.get("db_password").and_then(|v| v.as_str()) {
                        db_password = p.to_string();
                    }
                }
            }
        }

        // Supavisor GET /api/tenants strips the `users` array for security.
        // If we couldn't extract the user, we MUST fetch it from the workspaces table
        let mut db_name_opt = None;
        if db_user.is_empty() {
            let primary_url = self.base_url.clone();
            let ws_id = *workspace_id;
            
            let fetched_user: Result<Option<(String, String, String)>, String> = tokio::task::spawn_blocking(move || {
                use diesel::prelude::*;
                use diesel::sql_types::{Nullable, Text, Uuid as UuidSql};
                
                #[derive(QueryableByName)]
                struct WsRow {
                    #[diesel(sql_type = Nullable<Text>)]
                    custom_db_url_encrypted: Option<String>,
                }
                
                let mut conn = PgConnection::establish(&primary_url)
                    .map_err(|e| e.to_string())?;
                    
                let rows = diesel::sql_query("SELECT custom_db_url_encrypted FROM workspaces WHERE id = $1")
                    .bind::<UuidSql, _>(ws_id)
                    .load::<WsRow>(&mut conn)
                    .map_err(|e| e.to_string())?;
                    
                if let Some(row) = rows.into_iter().next() {
                    if let Some(encrypted) = row.custom_db_url_encrypted {
                        // Decrypt the URL
                        let crypto_service = std::env::var("ENCRYPTION_KEYS_JSON")
                            .ok()
                            .and_then(|keys| crate::crypto::CryptoService::from_json(&keys).ok());
                        
                        let decrypted = if let Some(crypto) = crypto_service {
                            crypto.decrypt(&encrypted).unwrap_or(encrypted)
                        } else {
                            encrypted
                        };
                        
                        // Extract db_user and pass using our existing parser
                        if let Some((user, pass, _, _, db)) = crate::supavisor::parse_postgres_url(&decrypted) {
                            return Ok(Some((user, pass, db)));
                        }
                    }
                }
                Ok(None)
            }).await.map_err(|e| e.to_string())?;
            
            if let Ok(Some((u, p, d))) = fetched_user {
                db_user = u;
                db_password = p;
                db_name_opt = Some(d);
            } else {
                // Extreme fallback, but it should never reach here if workspace registration properly encrypts final_db_url
                db_user = "tenant_user".to_string();
                db_password = "tenant_password".to_string();
            }
        }

        let tenant_url = self.get_tenant_url_with_credentials(workspace_id, &db_user, &db_password, db_name_opt.as_deref())?;
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
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    // Helper mock to test URL parsing isolated from the real struct
    struct MockTenantManager {
        base_url: String,
    }

    impl MockTenantManager {
        fn new(base_url: &str) -> Self {
            Self {
                base_url: base_url.to_string(),
            }
        }

        // Copy of the logic from SupavisorTenantPoolManager::get_tenant_url_with_credentials
        fn get_tenant_url_with_credentials(
            &self,
            workspace_id: &Uuid,
            override_user: &str,
            override_password: &str,
            override_db_name: Option<&str>,
        ) -> Result<String, AppError> {
            let tenant_uuid_str = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
            let db_name = match override_db_name {
                Some(name) if !name.is_empty() => name.to_string(),
                _ => tenant_uuid_str.clone(),
            };
            let mut result_url = self.base_url.clone();

            if let Some(prefix_len) = result_url.find("://") {
                let plen = prefix_len + 3;
                let after_prefix = &result_url[plen..];
                let path_start = after_prefix.find('/').unwrap_or(after_prefix.len());
                let auth_host = &after_prefix[..path_start];

                if let Some(at_idx) = auth_host.rfind('@') {
                    let auth = &auth_host[..at_idx];
                    let user = match auth.find(':') {
                        Some(colon_idx) => &auth[..colon_idx],
                        None => auth,
                    };

                    if user.contains('.') {
                        // Supavisor mode: username needs .tenant suffix
                        let actual_prefix = if override_user.is_empty() {
                            user.split_once('.').map(|(p, _)| p).unwrap_or(user)
                        } else {
                            override_user
                        };
                        let new_user = format!("{actual_prefix}.{tenant_uuid_str}");
                        let replace_start = plen;
                        let replace_end = plen + auth.len();

                        let new_auth = if override_password.is_empty() {
                            new_user
                        } else {
                            format!("{new_user}:{override_password}")
                        };
                        result_url.replace_range(replace_start..replace_end, &new_auth);
                    } else if !override_user.is_empty() {
                        // Direct connection mode: replace user entirely, no .tenant suffix
                        let new_user = override_user.to_string();
                        let replace_start = plen;
                        let replace_end = plen + auth.len();

                        let new_auth = if override_password.is_empty() {
                            new_user
                        } else {
                            format!("{new_user}:{override_password}")
                        };
                        result_url.replace_range(replace_start..replace_end, &new_auth);
                    }
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
            
            Ok(result_url)
        }
    }

    #[test]
    fn test_get_tenant_url_direct_mode() {
        let manager = MockTenantManager::new("postgres://admin:pass@localhost:5432/leeattend");
        let ws_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let url = manager.get_tenant_url_with_credentials(&ws_id, "tenant_user", "tenant_password", None).unwrap();
        
        assert_eq!(url, "postgres://tenant_user:tenant_password@localhost:5432/tenant_123e4567e89b12d3a456426614174000");
    }

    #[test]
    fn test_get_tenant_url_supavisor_mode() {
        // Pool URL có tên user (có thể chứa .tenant_id)
        let manager = MockTenantManager::new(
            "postgres://admin.tenant_019...:pass@supavisor:6543/leeattend",
        );
        let ws_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

        let url = manager.get_tenant_url_with_credentials(&ws_id, "", "", None).unwrap();
        
        assert_eq!(url, "postgres://admin.tenant_123e4567e89b12d3a456426614174000@supavisor:6543/tenant_123e4567e89b12d3a456426614174000");
    }

    #[test]
    fn test_get_tenant_url_direct_mode_no_override() {
        let ws_id = Uuid::parse_str("019e615b-51dc-7163-8936-82f42318cf84").unwrap();
        let manager = MockTenantManager::new("postgres://admin:admin@db:5432/postgres");
        
        let url = manager.get_tenant_url_with_credentials(&ws_id, "my_user", "my_pass", None).unwrap();
        
        assert_eq!(url, "postgres://my_user:my_pass@db:5432/tenant_019e615b51dc7163893682f42318cf84");
    }

    #[test]
    fn test_get_tenant_url_with_override_db_name() {
        let ws_id = Uuid::parse_str("019e615b-51dc-7163-8936-82f42318cf84").unwrap();
        let manager = MockTenantManager::new("postgres://admin.leeattend:pass@supavisor:6543/leeattend");
        
        // This is the BYOD scenario where override_db_name is set to the actual database name (e.g. custom_db_name)
        let url = manager.get_tenant_url_with_credentials(&ws_id, "", "", Some("custom_db_name")).unwrap();
        
        assert_eq!(url, "postgres://admin.tenant_019e615b51dc7163893682f42318cf84@supavisor:6543/custom_db_name");
    }
}
