use reqwest::Client;
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

    let payload = json!({
        "tenant": {
            "db_host": db_host,
            "db_port": db_port,
            "db_database": db_name,
            "ip_version": "auto",
            "enforce_ssl": enforce_ssl,
            "require_user": false,
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

/// Initializes the Supavisor connection pooler by registering the database tenant via its HTTP API.
/// This runs on startup if the `SUPAVISOR_API_URL` environment variable is set.
pub async fn init_supavisor_tenant() {
    let Ok(api_url) = std::env::var("SUPAVISOR_API_URL") else {
        tracing::debug!("SUPAVISOR_API_URL not set, skipping Supavisor initialization");
        return;
    };

    let Ok(jwt_secret) = std::env::var("API_JWT_SECRET") else {
        tracing::warn!(
            "SUPAVISOR_API_URL is set but API_JWT_SECRET is missing. Cannot initialize Supavisor."
        );
        return;
    };

    let Ok(db_url) = std::env::var("DATABASE_URL") else {
        tracing::warn!("DATABASE_URL is not set. Cannot initialize Supavisor.");
        return;
    };

    let Some((tenant_id, payload)) = build_tenant_payload(&db_url) else {
        return;
    };

    let Ok(token) = generate_supavisor_token(&jwt_secret) else {
        tracing::error!("Failed to generate Supavisor JWT token");
        return;
    };

    tracing::info!("Initializing Supavisor tenant '{tenant_id}' at {api_url}");

    let client = Client::new();
    let url = format!("{}/api/tenants/{tenant_id}", api_url.trim_end_matches('/'));

    match client
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
                // Sometimes it returns 400 if it already exists or if fields conflict.
                // Log it but don't panic.
                tracing::warn!("Supavisor initialization returned 400: {text}");
            } else {
                let text = response.text().await.unwrap_or_default();
                tracing::warn!("Supavisor initialization returned status {status}: {text}");
            }
        }
        Err(e) => {
            tracing::warn!("Failed to reach Supavisor API: {e}");
        }
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
pub async fn delete_tenant(tenant_id: &str) -> crate::errors::AppResult<()> {
    let api_url = std::env::var("SUPAVISOR_API_URL").map_err(|_err| {
        crate::errors::AppError::Internal("SUPAVISOR_API_URL not set".to_string())
    })?;
    let jwt_secret = std::env::var("API_JWT_SECRET")
        .map_err(|_err| crate::errors::AppError::Internal("API_JWT_SECRET not set".to_string()))?;
    let token = generate_supavisor_token(&jwt_secret)
        .map_err(|e| crate::errors::AppError::Internal(format!("Failed to generate token: {e}")))?;

    let client = Client::new();
    let url = format!(
        "{}/api/tenants/{}",
        api_url.trim_end_matches('/'),
        tenant_id
    );

    let response = client
        .delete(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .map_err(|e| {
            crate::errors::AppError::Internal(format!("Failed to reach Supavisor API: {e}"))
        })?;

    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(crate::errors::AppError::Internal(format!(
            "Supavisor returned {status}: {text}"
        )))
    }
}

/// Dynamically registers a tenant in Supavisor using the provided payload.
pub async fn register_tenant_dynamic(
    tenant_id: &str,
    payload: &serde_json::Value,
) -> crate::errors::AppResult<()> {
    let api_url = std::env::var("SUPAVISOR_API_URL").map_err(|_err| {
        crate::errors::AppError::Internal("SUPAVISOR_API_URL not set".to_string())
    })?;
    let jwt_secret = std::env::var("API_JWT_SECRET")
        .map_err(|_err| crate::errors::AppError::Internal("API_JWT_SECRET not set".to_string()))?;
    let token = generate_supavisor_token(&jwt_secret)
        .map_err(|e| crate::errors::AppError::Internal(format!("Failed to generate token: {e}")))?;

    let client = Client::new();
    let url = format!(
        "{}/api/tenants/{}",
        api_url.trim_end_matches('/'),
        tenant_id
    );

    let response = client
        .put(&url)
        .header("Authorization", format!("Bearer {token}"))
        .json(payload)
        .send()
        .await
        .map_err(|e| {
            crate::errors::AppError::Internal(format!("Failed to reach Supavisor API: {e}"))
        })?;

    let status = response.status();
    if status.is_success() {
        Ok(())
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(crate::errors::AppError::Internal(format!(
            "Supavisor returned {status}: {text}"
        )))
    }
}

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
}
