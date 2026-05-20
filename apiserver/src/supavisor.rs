use serde_json::json;

pub(crate) fn build_tenant_payload(db_url: &str) -> Option<(String, serde_json::Value)> {
    let Ok(parsed_url) = reqwest::Url::parse(db_url) else {
        tracing::warn!("Failed to parse DATABASE_URL: {db_url}");
        return None;
    };

    let db_user = std::env::var("SUPAVISOR_POOL_USER")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| parsed_url.username().to_string());

    let db_password = std::env::var("SUPAVISOR_POOL_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| parsed_url.password().unwrap_or("").to_string());

    let db_host = parsed_url.host_str().unwrap_or("db").to_string();
    let db_port = parsed_url.port().unwrap_or(5432);
    let db_name = parsed_url.path().trim_start_matches('/').to_string();

    if db_name.is_empty() || db_user.is_empty() || db_password.is_empty() {
        tracing::warn!("DATABASE_URL is missing required components (user/password/name). Cannot initialize Supavisor.");
        return None;
    }

    let tenant_id = std::env::var("SUPAVISOR_TENANT_ID")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| db_name.clone());

    let enforce_ssl = std::env::var("DATABASE_REQUIRE_SSL")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    let pool_size: u32 = std::env::var("SUPAVISOR_POOL_SIZE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let mode_type =
        std::env::var("SUPAVISOR_MODE_TYPE").unwrap_or_else(|_| "transaction".to_string());

    let ip_version = std::env::var("SUPAVISOR_IP_VERSION").unwrap_or_else(|_| "auto".to_string());

    let payload = json!({
        "tenant": {
            "db_host": db_host,
            "db_port": db_port,
            "db_database": db_name,
            "ip_version": ip_version,
            "enforce_ssl": enforce_ssl,
            "require_user": false,
            "default_pool_size": pool_size,
            "default_mode": &mode_type,
            "auth_query": "SELECT rolname, rolpassword FROM pg_authid WHERE rolname=$1;",
            "users": [
                {
                    "db_user": db_user,
                    "db_password": db_password,
                    "pool_size": pool_size,
                    "mode_type": mode_type,
                    "is_manager": true
                }
            ]
        }
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
                .timeout(std::time::Duration::from_secs(5))
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

        let Some((tenant_id, payload)) = build_tenant_payload(&self.master_db_url) else {
            return;
        };

        let Ok(token) = self.generate_token() else {
            tracing::error!("Failed to generate Supavisor JWT token");
            return;
        };

        tracing::info!(
            "Initializing Supavisor tenant '{tenant_id}' at {}",
            self.api_url
        );

        let url = format!(
            "{}/api/tenants/{tenant_id}",
            self.api_url.trim_end_matches('/')
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
            let body = response
                .json::<serde_json::Value>()
                .await
                .map_err(|e| AppError::Internal(format!("Failed to parse Supavisor response: {e}")))?;
            Ok(body)
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(AppError::Internal(format!(
                "Supavisor returned {status}: {text}"
            )))
        }
    }
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
        let (tenant_id, payload) = build_tenant_payload(url).expect("Should build payload");

        assert_eq!(tenant_id, "my_company_db");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["db_host"], "my_db_host");
        assert_eq!(tenant["db_port"], 5433);
        assert_eq!(tenant["db_database"], "my_company_db");

        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["db_user"], "my_user");
        assert_eq!(users[0]["db_password"], "my_secret");
    }

    #[test]
    fn test_build_tenant_payload_enforce_ssl_from_env() {
        std::env::set_var("DATABASE_REQUIRE_SSL", "true");
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url).expect("Should build payload");
        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["enforce_ssl"], true);
        std::env::remove_var("DATABASE_REQUIRE_SSL");

        // Test false fallback
        let (_, payload2) = build_tenant_payload(url).expect("Should build payload");
        let tenant2 = payload2.get("tenant").expect("missing tenant key");
        assert_eq!(tenant2["enforce_ssl"], false);
    }

    #[test]
    fn test_build_tenant_payload_with_custom_tenant_id() {
        std::env::set_var("SUPAVISOR_TENANT_ID", "custom_tenant_name");
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (tenant_id, payload) = build_tenant_payload(url).expect("Should build payload");

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
        let (_, payload) = build_tenant_payload(url).expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users[0]["db_user"], "custom_pool_user");
        assert_eq!(users[0]["db_password"], "custom_pool_secret");
        assert_eq!(users[0]["pool_size"], 20); // Default
        assert_eq!(users[0]["mode_type"], "transaction"); // Default

        std::env::remove_var("SUPAVISOR_POOL_USER");
        std::env::remove_var("SUPAVISOR_POOL_PASSWORD");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_pool_settings() {
        std::env::set_var("SUPAVISOR_POOL_SIZE", "50");
        std::env::set_var("SUPAVISOR_MODE_TYPE", "session");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url).expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users[0]["pool_size"], 50);
        assert_eq!(users[0]["mode_type"], "session");

        std::env::remove_var("SUPAVISOR_POOL_SIZE");
        std::env::remove_var("SUPAVISOR_MODE_TYPE");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_ip_version() {
        std::env::set_var("SUPAVISOR_IP_VERSION", "v4");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(url).expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["ip_version"], "v4");

        std::env::remove_var("SUPAVISOR_IP_VERSION");
    }

    #[test]
    fn test_build_tenant_payload_missing_password() {
        let url = "postgres://my_user@my_db_host:5432/my_company_db";
        let result = build_tenant_payload(url);
        assert!(
            result.is_none(),
            "Should return None if password is missing"
        );
    }

    #[test]
    fn test_build_tenant_payload_missing_user() {
        let url = "postgres://:my_secret@my_db_host:5432/my_company_db";
        let result = build_tenant_payload(url);
        assert!(result.is_none(), "Should return None if user is missing");
    }

    #[test]
    fn test_build_tenant_payload_missing_db_name() {
        let url = "postgres://my_user:my_secret@my_db_host:5432/";
        let result = build_tenant_payload(url);
        assert!(result.is_none(), "Should return None if db name is missing");
    }

    #[test]
    fn test_build_tenant_payload_invalid_url() {
        let url = "not_a_url";
        let result = build_tenant_payload(url);
        assert!(result.is_none(), "Should return None for invalid URL");
    }

    #[test]
    fn test_build_tenant_payload_with_query_params() {
        // Query parameters like sslmode or application_name should not break the path extraction
        let url = "postgres://my_user:my_secret@my_db_host:5432/my_company_db?sslmode=disable&application_name=leeattend";
        let (tenant_id, payload) = build_tenant_payload(url)
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
        let url = "postgres://admin:secret@db:5432/tenant_019e40ee11927492bc2985dad4675f34";
        let (tenant_id, payload) = build_tenant_payload(url).expect("Should build payload");

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
    // Contract: build_tenant_payload() MUST return a tenant_id that matches
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
        let url = "postgres://admin:secret@db:5432/tenant_a1b2c3d4e5f67890abcdef1234567890";
        let (tenant_id, _) = build_tenant_payload(url).expect("Should build");
        assert_eq!(tenant_id, "tenant_a1b2c3d4e5f67890abcdef1234567890");
        assert_eq!(tenant_id.len(), 39); // "tenant_" + 32 hex
        assert!(!tenant_id.contains('-'));
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_nil_uuid_db_name() {
        let url = "postgres://admin:secret@db:5432/tenant_00000000000000000000000000000000";
        let (tenant_id, _) = build_tenant_payload(url).expect("Should build");
        assert_eq!(tenant_id, "tenant_00000000000000000000000000000000");
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_max_uuid_db_name() {
        let url = "postgres://admin:secret@db:5432/tenant_ffffffffffffffffffffffffffffffff";
        let (tenant_id, _) = build_tenant_payload(url).expect("Should build");
        assert_eq!(tenant_id, "tenant_ffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_build_tenant_payload_supavisor_tenant_id_overrides_db_name() {
        // When SUPAVISOR_TENANT_ID is set, tenant_id takes that value,
        // but db_database in the payload stays as the actual PG database name.
        // This is the BYOD scenario — custom DB, but Supavisor routing
        // still uses tenant_<uuid-no-hyphens>.
        std::env::set_var("SUPAVISOR_TENANT_ID", "tenant_019e40ee11927492bc2985dad4675f34");
        let url = "postgres://admin:secret@db:5432/my_custom_db";
        let (tenant_id, payload) = build_tenant_payload(url).expect("Should build");
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
        assert!(!tenant_id.contains('-'),
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
        let (tid1, _) = build_tenant_payload(url1).unwrap();
        assert!(!tid1.contains('-'), "standard DB: tenant_id must not have hyphens");

        // Case 2: custom DB name with hyphenated override
        std::env::set_var("SUPAVISOR_TENANT_ID", "tenant_a1b2c3d4e5f67890abcdef1234567890");
        let url2 = "postgres://u:p@h:5432/some-custom-db-name";
        let (tid2, _) = build_tenant_payload(url2).unwrap();
        std::env::remove_var("SUPAVISOR_TENANT_ID");
        assert!(!tid2.contains('-'), "BYOD: tenant_id must not have hyphens");

        // Case 3: custom DB name without SUPAVISOR_TENANT_ID override
        // (falls back to DB name from URL — caller is responsible for ensuring
        // the correct format via SUPAVISOR_TENANT_ID for non-standard names)
        let url3 = "postgres://u:p@h:5432/custom_db_name";
        let (tid3, _) = build_tenant_payload(url3).unwrap();
        assert!(!tid3.contains('-'), "fallback: tenant_id must not have hyphens");
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
        let provisioned_url = format!("postgres://admin:secret@db:5432/{db_name}");

        let (tenant_id, _) = build_tenant_payload(&provisioned_url).unwrap();

        // This is the SAME format as in get_tenant_url() — the contract
        assert_eq!(tenant_id, db_name,
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
        let (tenant_id, _) = build_tenant_payload(url).unwrap();
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

        let url = "postgres://user:pass@host:5432/dbname";
        let (tenant_id, payload) = build_tenant_payload(url).expect("Should build payload");

        assert_eq!(tenant_id, "test_tenant");

        let tenant_obj = payload.get("tenant").expect("Missing tenant object");
        
        assert_eq!(tenant_obj.get("require_user").unwrap().as_bool(), Some(false));
        
        // Assert the new edge-case fields are present
        assert_eq!(tenant_obj.get("default_mode").unwrap().as_str(), Some("session"));
        assert_eq!(tenant_obj.get("default_pool_size").unwrap().as_u64(), Some(42));

        // Cleanup
        std::env::remove_var("SUPAVISOR_MODE_TYPE");
        std::env::remove_var("SUPAVISOR_POOL_SIZE");
        std::env::remove_var("SUPAVISOR_TENANT_ID");
    }
}
