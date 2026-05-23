use serde_json::json;

pub struct TenantPayloadOptions {
    pub tenant_id: Option<String>,
    pub pool_size: Option<u32>,
    pub is_master: bool,
}

impl Default for TenantPayloadOptions {
    fn default() -> Self {
        Self {
            tenant_id: None,
            pool_size: None,
            is_master: false,
        }
    }
}

pub fn build_tenant_payload(db_url: &str, opts: TenantPayloadOptions) -> Option<(String, serde_json::Value)> {
    let Ok(parsed_url) = reqwest::Url::parse(db_url) else {
        tracing::warn!("Failed to parse DATABASE_URL: {db_url}");
        return None;
    };

    let db_user = if opts.is_master {
        std::env::var("SUPAVISOR_POOL_USER")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| parsed_url.username().to_string())
    } else {
        parsed_url.username().to_string()
    };

    let db_password = if opts.is_master {
        std::env::var("SUPAVISOR_POOL_PASSWORD")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| parsed_url.password().unwrap_or("").to_string())
    } else {
        parsed_url.password().unwrap_or("").to_string()
    };

    let db_host = parsed_url.host_str().unwrap_or("db_system").to_string();
    let db_port = parsed_url.port().unwrap_or(5432);
    let db_name = parsed_url.path().trim_start_matches('/').to_string();

    if db_name.is_empty() || db_user.is_empty() || db_password.is_empty() {
        tracing::warn!("DATABASE_URL is missing required components (user/password/name). Cannot initialize Supavisor.");
        return None;
    }

    let tenant_id = opts.tenant_id.unwrap_or_else(|| {
        if opts.is_master {
            std::env::var("SUPAVISOR_TENANT_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| db_name.clone())
        } else {
            db_name.clone()
        }
    });

    let enforce_ssl = std::env::var("DATABASE_REQUIRE_SSL")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    let pool_size: u32 = opts.pool_size.unwrap_or_else(|| {
        std::env::var("SUPAVISOR_POOL_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20)
    });

    let mode_type = std::env::var("SUPAVISOR_MODE_TYPE").unwrap_or_else(|_| "session".to_string());

    let ip_version = std::env::var("SUPAVISOR_IP_VERSION").unwrap_or_else(|_| "auto".to_string());

    let mut tenant_map = serde_json::Map::new();
    tenant_map.insert("db_host".to_string(), json!(db_host));
    tenant_map.insert("db_port".to_string(), json!(db_port));
    tenant_map.insert("db_database".to_string(), json!(db_name));
    tenant_map.insert("ip_version".to_string(), json!(ip_version));
    tenant_map.insert("enforce_ssl".to_string(), json!(enforce_ssl));
    tenant_map.insert("default_pool_size".to_string(), json!(pool_size));
    tenant_map.insert("default_mode".to_string(), json!(&mode_type));

    let auth_query = std::env::var("SUPAVISOR_AUTH_QUERY").ok();
    let has_auth_query = auth_query
        .as_ref()
        .map(|v| !v.is_empty() && v != "none")
        .unwrap_or(false);

    if has_auth_query {
        tenant_map.insert("require_user".to_string(), json!(false));
        if let Some(query) = auth_query {
            tenant_map.insert("auth_query".to_string(), json!(query));
        }
    } else {
        tenant_map.insert("require_user".to_string(), json!(true));
    }

    tenant_map.insert(
        "users".to_string(),
        json!([
            {
                "db_user": &db_user,
                "db_password": &db_password,
                "pool_size": pool_size,
                "mode_type": &mode_type,
                "is_manager": true
            }
        ]),
    );

    let payload = json!({
        "tenant": tenant_map
    });

    Some((tenant_id, payload))
}

use crate::errors::{AppError, AppResult};
/// Initializes the Supavisor connection pooler by registering the database tenant via its HTTP API.
/// This runs on startup if the `SUPAVISOR_API_URL` environment variable is set.
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TenantManager: Send + Sync {
    async fn init_tenant(&self);
    async fn delete_tenant(&self, tenant_id: &str) -> AppResult<()>;
    async fn register_tenant_dynamic(
        &self,
        tenant_id: &str,
        payload: &serde_json::Value,
    ) -> AppResult<()>;
    async fn get_tenant(&self, tenant_id: &str) -> AppResult<serde_json::Value>;
    async fn update_tenant_mode(&self, tenant_id: &str, mode_type: &str) -> AppResult<()>;
}

pub struct SupavisorTenantManager {
    client: reqwest::Client,
    api_url: String,
    jwt_secret: String,
    master_db_url: String,
}

impl SupavisorTenantManager {
    pub fn new() -> Option<Self> {
        let api_url = std::env::var("SUPAVISOR_API_URL").ok()?;
        let jwt_secret = std::env::var("API_JWT_SECRET").ok()?;
        let master_db_url = std::env::var("DATABASE_URL").unwrap_or_default();
        Some(Self {
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            api_url,
            jwt_secret,
            master_db_url,
        })
    }

    fn generate_token(&self) -> Result<String, jsonwebtoken::errors::Error> {
        generate_supavisor_token(&self.jwt_secret)
    }
}

#[async_trait]
impl TenantManager for SupavisorTenantManager {
    async fn init_tenant(&self) {
        if self.master_db_url.is_empty() {
            tracing::warn!("DATABASE_URL is not set. Cannot initialize Supavisor.");
            return;
        }

        let Some((tenant_id, payload)) = build_tenant_payload(&self.master_db_url, TenantPayloadOptions { is_master: true, ..Default::default() }) else {
            return;
        };

        let Ok(token) = self.generate_token() else {
            tracing::error!("Failed to generate Supavisor JWT token");
            return;
        };

        let url = format!(
            "{}/api/tenants/{tenant_id}",
            self.api_url.trim_end_matches('/')
        );

        // Check if tenant already exists to prevent overwriting and dropping connections
        match self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    tracing::info!(
                        "Supavisor tenant '{tenant_id}' already exists, skipping initialization"
                    );
                    return;
                }
            }
            Err(e) => {
                tracing::warn!("Failed to check if Supavisor tenant exists: {e:?}");
            }
        }

        tracing::info!(
            "Initializing Supavisor tenant '{tenant_id}' at {}",
            self.api_url
        );

        match self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    tracing::info!("Supavisor tenant initialized successfully (Status: {status})");
                } else if status == reqwest::StatusCode::BAD_REQUEST {
                    let text = response.text().await.unwrap_or_default();
                    tracing::warn!("Supavisor initialization returned 400: {text}");
                } else {
                    let text = response.text().await.unwrap_or_default();
                    tracing::warn!("Supavisor initialization returned status {status}: {text}");
                }
            }
            Err(e) => {
                tracing::warn!("Failed to reach Supavisor API: {e:?}");
            }
        }
    }

    async fn delete_tenant(&self, tenant_id: &str) -> AppResult<()> {
        let token = self
            .generate_token()
            .map_err(|e| AppError::Internal(format!("Failed to generate token: {e}")))?;

        let url = format!(
            "{}/api/tenants/{}",
            self.api_url.trim_end_matches('/'),
            tenant_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to reach Supavisor API: {e}")))?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(AppError::Internal(format!(
                "Supavisor returned {status}: {text}"
            )))
        }
    }

    async fn register_tenant_dynamic(
        &self,
        tenant_id: &str,
        payload: &serde_json::Value,
    ) -> AppResult<()> {
        let token = self
            .generate_token()
            .map_err(|e| AppError::Internal(format!("Failed to generate token: {e}")))?;

        let url = format!(
            "{}/api/tenants/{}",
            self.api_url.trim_end_matches('/'),
            tenant_id
        );

        let response_get = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await;

        if let Ok(res) = response_get {
            if res.status().is_success() {
                tracing::debug!("Supavisor tenant '{}' already exists, skipping dynamic registration", tenant_id);
                return Ok(());
            }
        }

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(payload)
            .send()
            .await
            .map_err(|e| {
                println!("Failed to reach Supavisor API: {e:?}");
                AppError::Internal(format!("Failed to reach Supavisor API: {e:?}"))
            })?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            println!("Supavisor returned {status}: {text}");
            Err(AppError::Internal(format!(
                "Supavisor returned {status}: {text}"
            )))
        }
    }

    async fn get_tenant(&self, tenant_id: &str) -> AppResult<serde_json::Value> {
        let token = self
            .generate_token()
            .map_err(|e| AppError::Internal(format!("Failed to generate token: {e}")))?;

        let url = format!(
            "{}/api/tenants/{}",
            self.api_url.trim_end_matches('/'),
            tenant_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to reach Supavisor API: {e}")))?;

        let status = response.status();
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await.map_err(|e| {
                AppError::Internal(format!("Failed to parse Supavisor response: {e}"))
            })?;
            Ok(body)
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(AppError::Internal(format!(
                "Supavisor returned {status}: {text}"
            )))
        }
    }

    async fn update_tenant_mode(&self, tenant_id: &str, mode_type: &str) -> AppResult<()> {
        let token = self
            .generate_token()
            .map_err(|e| AppError::Internal(format!("Failed to generate token: {e}")))?;

        let base_url = self.api_url.trim_end_matches('/');
        let url = format!("{base_url}/api/tenants/{tenant_id}");

        let current = self.get_tenant(tenant_id).await?;
        let payload = update_tenant_mode_json(current, mode_type)?;

        let response = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to reach Supavisor API: {e}")))?;

        let status = response.status();
        if status.is_success() {
            tracing::info!(
                tenant_id = %tenant_id,
                mode_type = %mode_type,
                "Updated Supavisor tenant pool mode"
            );
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(AppError::Internal(format!(
                "Supavisor returned {status}: {text}"
            )))
        }
    }
}

pub(crate) fn update_tenant_mode_json(
    mut current: serde_json::Value,
    mode_type: &str,
) -> AppResult<serde_json::Value> {
    let tenant_map = if let Some(data_val) = current.get_mut("data") {
        data_val.as_object_mut()
    } else if let Some(tenant_val) = current.get_mut("tenant") {
        tenant_val.as_object_mut()
    } else {
        current.as_object_mut()
    }
    .ok_or_else(|| AppError::Internal("Unexpected Supavisor response shape".into()))?;

    tenant_map.insert(
        "default_mode".to_string(),
        serde_json::Value::String(mode_type.to_string()),
    );

    if let Some(users) = tenant_map.get_mut("users").and_then(|u| u.as_array_mut()) {
        for user in users {
            if let Some(obj) = user.as_object_mut() {
                obj.insert(
                    "mode_type".to_string(),
                    serde_json::Value::String(mode_type.to_string()),
                );
            }
        }
    }

    Ok(json!({
        "tenant": tenant_map
    }))
}

pub struct NoopTenantManager;

#[async_trait]
impl TenantManager for NoopTenantManager {
    async fn init_tenant(&self) {
        tracing::debug!("NoopTenantManager: skipping init");
    }

    async fn delete_tenant(&self, _tenant_id: &str) -> AppResult<()> {
        tracing::debug!("NoopTenantManager: skipping delete_tenant");
        Ok(())
    }

    async fn register_tenant_dynamic(
        &self,
        _tenant_id: &str,
        _payload: &serde_json::Value,
    ) -> AppResult<()> {
        tracing::debug!("NoopTenantManager: skipping register_tenant");
        Ok(())
    }

    async fn get_tenant(&self, _tenant_id: &str) -> AppResult<serde_json::Value> {
        tracing::debug!("NoopTenantManager: skipping get_tenant");
        Ok(serde_json::json!({}))
    }

    async fn update_tenant_mode(&self, _tenant_id: &str, _mode_type: &str) -> AppResult<()> {
        tracing::debug!("NoopTenantManager: skipping update_tenant_mode");
        Ok(())
    }
}

fn generate_supavisor_token(secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        iat: usize,
        exp: usize,
    }

    let iat = chrono::Utc::now().timestamp() as usize;
    let exp = iat + 3600; // 1 hour

    let claims = Claims { iat, exp };
    let header = Header::new(Algorithm::HS256);

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Deletes a tenant from Supavisor.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tenant_payload_valid() {
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (tenant_id, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        assert_eq!(tenant_id, "my_company_db");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["db_host"], "my_db_host");
        assert_eq!(tenant["db_port"], 5433);
        assert_eq!(tenant["db_database"], "my_company_db");

        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["db_user"], "my_user");
        assert_eq!(users[0]["db_password"], "my_secret");
        assert_eq!(users[0]["is_manager"], true);
    }

    #[test]
    fn test_build_tenant_payload_auth_query_config() {
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";

        // 1. By default (env not set), auth_query should be absent, require_user = true, 1 user
        std::env::remove_var("SUPAVISOR_AUTH_QUERY");
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        let tenant = payload.get("tenant").unwrap();
        assert!(
            tenant.get("auth_query").is_none(),
            "auth_query should be absent by default"
        );
        assert_eq!(tenant["require_user"], true);
        assert_eq!(tenant["users"].as_array().unwrap().len(), 1);

        // 2. When set to a custom query, require_user = false, 1 user
        std::env::set_var("SUPAVISOR_AUTH_QUERY", "SELECT 1;");
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        let tenant = payload.get("tenant").unwrap();
        assert_eq!(tenant["auth_query"], "SELECT 1;");
        assert_eq!(tenant["require_user"], false);
        assert_eq!(tenant["users"].as_array().unwrap().len(), 1);

        // 3. When set to "none"
        std::env::set_var("SUPAVISOR_AUTH_QUERY", "none");
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        let tenant = payload.get("tenant").unwrap();
        assert!(
            tenant.get("auth_query").is_none(),
            "auth_query should be absent when set to 'none'"
        );
        assert_eq!(tenant["require_user"], true);
        assert_eq!(tenant["users"].as_array().unwrap().len(), 1);

        // 4. When set to empty string
        std::env::set_var("SUPAVISOR_AUTH_QUERY", "");
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        let tenant = payload.get("tenant").unwrap();
        assert!(
            tenant.get("auth_query").is_none(),
            "auth_query should be absent when set to empty string"
        );
        assert_eq!(tenant["require_user"], true);
        assert_eq!(tenant["users"].as_array().unwrap().len(), 1);

        std::env::remove_var("SUPAVISOR_AUTH_QUERY");
    }

    #[test]
    fn test_build_tenant_payload_enforce_ssl_from_env() {
        std::env::set_var("DATABASE_REQUIRE_SSL", "true");
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");
        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["enforce_ssl"], true);
        std::env::remove_var("DATABASE_REQUIRE_SSL");

        // Test false fallback
        let (_, payload2) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");
        let tenant2 = payload2.get("tenant").expect("missing tenant key");
        assert_eq!(tenant2["enforce_ssl"], false);
    }

    #[test]
    fn test_build_tenant_payload_with_custom_tenant_id() {
        std::env::set_var("SUPAVISOR_TENANT_ID", "custom_tenant_name");
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (tenant_id, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        assert_eq!(tenant_id, "custom_tenant_name");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "my_company_db",
            "DB name should still be the original"
        );

        std::env::remove_var("SUPAVISOR_TENANT_ID");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_user_and_password() {
        std::env::set_var("SUPAVISOR_POOL_USER", "custom_pool_user");
        std::env::set_var("SUPAVISOR_POOL_PASSWORD", "custom_pool_secret");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["db_user"], "custom_pool_user");
        assert_eq!(users[0]["db_password"], "custom_pool_secret");
        assert_eq!(users[0]["pool_size"], 20); // Default
        assert_eq!(users[0]["mode_type"], "session"); // Default
        assert_eq!(users[0]["is_manager"], true);

        std::env::remove_var("SUPAVISOR_POOL_USER");
        std::env::remove_var("SUPAVISOR_POOL_PASSWORD");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_pool_settings() {
        std::env::set_var("SUPAVISOR_POOL_SIZE", "50");
        std::env::set_var("SUPAVISOR_MODE_TYPE", "session");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["pool_size"], 50);
        assert_eq!(users[0]["mode_type"], "session");
        assert_eq!(users[0]["is_manager"], true);

        std::env::remove_var("SUPAVISOR_POOL_SIZE");
        std::env::remove_var("SUPAVISOR_MODE_TYPE");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_ip_version() {
        std::env::set_var("SUPAVISOR_IP_VERSION", "v4");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["ip_version"], "v4");

        std::env::remove_var("SUPAVISOR_IP_VERSION");
    }

    #[test]
    fn test_build_tenant_payload_missing_password() {
        let url = "postgres://my_user@my_db_host:5432/my_company_db";
        let result = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() });
        assert!(
            result.is_none(),
            "Should return None if password is missing"
        );
    }

    #[test]
    fn test_build_tenant_payload_missing_user() {
        let url = "postgres://:my_secret@my_db_host:5432/my_company_db";
        let result = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() });
        assert!(result.is_none(), "Should return None if user is missing");
    }

    #[test]
    fn test_build_tenant_payload_missing_db_name() {
        let url = "postgres://my_user:my_secret@my_db_host:5432/";
        let result = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() });
        assert!(result.is_none(), "Should return None if db name is missing");
    }

    #[test]
    fn test_build_tenant_payload_invalid_url() {
        let url = "not_a_url";
        let result = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() });
        assert!(result.is_none(), "Should return None for invalid URL");
    }

    #[test]
    fn test_build_tenant_payload_with_query_params() {
        // Query parameters like sslmode or application_name should not break the path extraction
        let url = "postgres://my_user:my_secret@my_db_host:5432/my_company_db?sslmode=disable&application_name=leeattend";
        let (tenant_id, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() })
            .expect("Should build payload correctly even with query params");

        assert_eq!(
            tenant_id, "my_company_db",
            "DB name should not include query params"
        );

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["db_database"], "my_company_db");
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_matches_connection_url_format() {
        // The tenant_id must match the format produced by diesel_pool::get_tenant_url():
        // tenant_<uuid-without-hyphens>. This contract prevents "Tenant or user not found"
        // errors when the API server connects through Supavisor.
        let url = "postgres://admin:secret@db_system:5432/tenant_019e40ee11927492bc2985dad4675f34";
        let (tenant_id, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        assert_eq!(
            tenant_id, "tenant_019e40ee11927492bc2985dad4675f34",
            "tenant_id must be tenant_<uuid-no-hyphens> to match get_tenant_url() format"
        );

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "tenant_019e40ee11927492bc2985dad4675f34",
            "db_database should match the database name from URL"
        );

        // Verify that the tenant_id is NOT a raw UUID with hyphens
        assert!(
            !tenant_id.contains('-'),
            "tenant_id must NOT contain hyphens — raw UUID format is incompatible with get_tenant_url()"
        );
        assert!(
            tenant_id.starts_with("tenant_"),
            "tenant_id must start with 'tenant_' prefix"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Contract: build_tenant_payload(, TenantPayloadOptions { is_master: true, ..Default::default() }) MUST return a tenant_id that matches
    // the tenant identifier embedded in the connection URL produced by
    // diesel_pool::get_tenant_url(). The format is:
    //
    //   tenant_<uuid-without-hyphens>
    //
    // These tests lock down the registration side of the contract so it
    // cannot drift from the connection side. See diesel_pool.rs for the
    // connection-side contract tests.
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_tenant_payload_tenant_id_standard_db_name() {
        // When the DB name follows tenant_<uuid-no-hyphens>, tenant_id
        // defaults to the database name from the URL.
        let url = "postgres://admin:secret@db_system:5432/tenant_a1b2c3d4e5f67890abcdef1234567890";
        let (tenant_id, _) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build");
        assert_eq!(tenant_id, "tenant_a1b2c3d4e5f67890abcdef1234567890");
        assert_eq!(tenant_id.len(), 39); // "tenant_" + 32 hex
        assert!(!tenant_id.contains('-'));
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_nil_uuid_db_name() {
        let url = "postgres://admin:secret@db_system:5432/tenant_00000000000000000000000000000000";
        let (tenant_id, _) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build");
        assert_eq!(tenant_id, "tenant_00000000000000000000000000000000");
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_max_uuid_db_name() {
        let url = "postgres://admin:secret@db_system:5432/tenant_ffffffffffffffffffffffffffffffff";
        let (tenant_id, _) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build");
        assert_eq!(tenant_id, "tenant_ffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_build_tenant_payload_supavisor_tenant_id_overrides_db_name() {
        // When SUPAVISOR_TENANT_ID is set, tenant_id takes that value,
        // but db_database in the payload stays as the actual PG database name.
        // This is the BYOD scenario — custom DB, but Supavisor routing
        // still uses tenant_<uuid-no-hyphens>.
        std::env::set_var(
            "SUPAVISOR_TENANT_ID",
            "tenant_019e40ee11927492bc2985dad4675f34",
        );
        let url = "postgres://admin:secret@db_system:5432/my_custom_db";
        let (tenant_id, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build");
        std::env::remove_var("SUPAVISOR_TENANT_ID");

        assert_eq!(
            tenant_id, "tenant_019e40ee11927492bc2985dad4675f34",
            "tenant_id must come from SUPAVISOR_TENANT_ID for BYOD"
        );
        // The payload must still point to the actual PG database
        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "my_custom_db",
            "db_database must be the actual PG database, not the tenant_id"
        );
        assert!(
            !tenant_id.contains('-'),
            "BYOD tenant_id must NOT contain hyphens"
        );
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_never_contains_hyphens() {
        // Regardless of URL format or env vars, the tenant_id must NEVER
        // contain hyphens — the connection URL strips them and Supavisor
        // would not find the tenant.

        // Case 1: standard tenant_ DB name
        let url1 = "postgres://u:p@h:5432/tenant_019e40ee11927492bc2985dad4675f34";
        let (tid1, _) = build_tenant_payload(url1, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        assert!(
            !tid1.contains('-'),
            "standard DB: tenant_id must not have hyphens"
        );

        // Case 2: custom DB name with hyphenated override
        std::env::set_var(
            "SUPAVISOR_TENANT_ID",
            "tenant_a1b2c3d4e5f67890abcdef1234567890",
        );
        let url2 = "postgres://u:p@h:5432/some-custom-db-name";
        let (tid2, _) = build_tenant_payload(url2, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        std::env::remove_var("SUPAVISOR_TENANT_ID");
        assert!(!tid2.contains('-'), "BYOD: tenant_id must not have hyphens");

        // Case 3: custom DB name without SUPAVISOR_TENANT_ID override
        // (falls back to DB name from URL — caller is responsible for ensuring
        // the correct format via SUPAVISOR_TENANT_ID for non-standard names)
        let url3 = "postgres://u:p@h:5432/custom_db_name";
        let (tid3, _) = build_tenant_payload(url3, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        assert!(
            !tid3.contains('-'),
            "fallback: tenant_id must not have hyphens"
        );
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_matches_connection_format_exactly() {
        // Roundtrip verification: given a standard DB URL, the tenant_id
        // must match EXACTLY what diesel_pool::get_tenant_url() would embed
        // in the Supavisor connection URL for the same workspace.
        //
        // This test duplicates the format from diesel_pool.rs to ensure
        // both sides agree. If either test fails, the contract is broken.

        // Simulate what provision_tenant_db + get_tenant_url would produce
        let ws_id = uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        let db_name = format!("tenant_{}", ws_id.to_string().replace('-', ""));
        let provisioned_url = format!("postgres://admin:secret@db_system:5432/{db_name}");

        let (tenant_id, _) = build_tenant_payload(&provisioned_url, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();

        // This is the SAME format as in get_tenant_url() — the contract
        assert_eq!(
            tenant_id, db_name,
            "tenant_id must equal the db_name from provision_tenant_db, \
             which matches the tenant identifier in get_tenant_url()"
        );
        assert_eq!(tenant_id, "tenant_a1b2c3d4e5f67890abcdef1234567890");
        assert_eq!(tenant_id.len(), 39);
        assert!(!tenant_id.contains('-'));
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_format_length_invariant() {
        // The tenant_id for standard workspace DBs is always exactly
        // 39 characters: "tenant_" (7) + 32 hex digits.
        let url = "postgres://u:p@h:5432/tenant_0123456789abcdef0123456789abcdef";
        let (tenant_id, _) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).unwrap();
        assert_eq!(
            tenant_id.len(),
            39,
            "tenant_id must be 39 chars, got {}: '{tenant_id}'",
            tenant_id.len()
        );
    }

    #[test]
    fn test_generate_supavisor_token_valid() {
        let secret = "super_secret_test_key_with_at_least_32_bytes_length";
        let token = generate_supavisor_token(secret).expect("Should generate token");

        // Ensure we can decode it back with the same secret
        use jsonwebtoken::{decode, DecodingKey, Validation};
        #[derive(serde::Deserialize)]
        struct Claims {
            iat: usize,
            exp: usize,
        }

        let mut validation = Validation::default();
        validation.leeway = 60;
        validation.validate_exp = false; // We just want to check the structure

        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .expect("Should decode successfully");

        // Check expiration is 1 hour ahead
        assert_eq!(token_data.claims.exp - token_data.claims.iat, 3600);
    }

    #[test]
    fn test_build_tenant_payload_includes_default_mode_and_pool_size() {
        // Test that Supavisor payload correctly injects default_mode and default_pool_size
        // which are required by Supavisor 2.7+ when require_user is false.
        std::env::set_var("SUPAVISOR_MODE_TYPE", "session");
        std::env::set_var("SUPAVISOR_POOL_SIZE", "42");
        std::env::set_var("SUPAVISOR_TENANT_ID", "test_tenant");
        std::env::set_var("SUPAVISOR_AUTH_QUERY", "SELECT 1;");

        let url = "postgres://user:pass@host:5432/dbname";
        let (tenant_id, payload) = build_tenant_payload(url, TenantPayloadOptions { is_master: true, ..Default::default() }).expect("Should build payload");

        assert_eq!(tenant_id, "test_tenant");

        let tenant_obj = payload.get("tenant").expect("Missing tenant object");

        assert_eq!(
            tenant_obj.get("require_user").unwrap().as_bool(),
            Some(false)
        );

        // Assert the new edge-case fields are present
        assert_eq!(
            tenant_obj.get("default_mode").unwrap().as_str(),
            Some("session")
        );
        assert_eq!(
            tenant_obj.get("default_pool_size").unwrap().as_u64(),
            Some(42)
        );

        // Cleanup
        std::env::remove_var("SUPAVISOR_MODE_TYPE");
        std::env::remove_var("SUPAVISOR_POOL_SIZE");
        std::env::remove_var("SUPAVISOR_TENANT_ID");
        std::env::remove_var("SUPAVISOR_AUTH_QUERY");
    }

    #[test]
    fn test_update_tenant_mode_json_shapes() {
        // Shape 1: Wrapped in "data" (returned by live Supavisor API)
        let input_data = json!({
            "data": {
                "db_database": "company_db",
                "default_mode": "session",
                "users": [
                    {
                        "db_user": "admin",
                        "mode_type": "session"
                    }
                ]
            }
        });
        let result_data = update_tenant_mode_json(input_data, "transaction").expect("success");
        let tenant = result_data.get("tenant").expect("should have tenant key");
        assert_eq!(tenant["default_mode"], "transaction");
        assert_eq!(tenant["users"][0]["mode_type"], "transaction");

        // Shape 2: Wrapped in "tenant" (used by some mocks)
        let input_tenant = json!({
            "tenant": {
                "db_database": "company_db",
                "default_mode": "session",
                "users": [
                    {
                        "db_user": "admin",
                        "mode_type": "session"
                    }
                ]
            }
        });
        let result_tenant = update_tenant_mode_json(input_tenant, "transaction").expect("success");
        let tenant = result_tenant.get("tenant").expect("should have tenant key");
        assert_eq!(tenant["default_mode"], "transaction");
        assert_eq!(tenant["users"][0]["mode_type"], "transaction");

        // Shape 3: Unwrapped (direct root object)
        let input_direct = json!({
            "db_database": "company_db",
            "default_mode": "session",
            "users": [
                {
                    "db_user": "admin",
                    "mode_type": "session"
                }
            ]
        });
        let result_direct = update_tenant_mode_json(input_direct, "transaction").expect("success");
        let tenant = result_direct.get("tenant").expect("should have tenant key");
        assert_eq!(tenant["default_mode"], "transaction");
        assert_eq!(tenant["users"][0]["mode_type"], "transaction");
    }
}
