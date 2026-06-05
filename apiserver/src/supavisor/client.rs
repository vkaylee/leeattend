use super::*;
use crate::errors::{AppError, AppResult};
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
    pub fn new() -> Result<Self, AppError> {
        let api_url = std::env::var("SUPAVISOR_API_URL")
            .map_err(|_err| AppError::Validation("error.supavisor_api_url_required".to_string()))?;

        let jwt_secret = std::env::var("API_JWT_SECRET")
            .or_else(|_err| std::env::var("SUPAVISOR_API_JWT"))
            .map_err(|_err| AppError::Validation("error.api_jwt_secret_required".to_string()))?;

        let master_db_url = std::env::var("DATABASE_URL").unwrap_or_default();
        Ok(Self {
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

        let Some((tenant_id, payload)) = build_tenant_payload(
            &self.master_db_url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        ) else {
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

        // Check if tenant already exists. We used to skip initialization to prevent dropping connections,
        // but this causes the master tenant to be stuck with stale/buggy configurations.
        // Now, we always PUT the tenant to ensure its configuration is up to date.
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
                        "Supavisor tenant '{tenant_id}' already exists. Proceeding to update it to apply the latest config..."
                    );
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
                tracing::info!(
                    "Supavisor tenant '{tenant_id}' already exists. Proceeding to update it to apply the latest config..."
                );
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
                tracing::warn!("Failed to reach Supavisor API: {e:?}");
                AppError::Internal(format!("Failed to reach Supavisor API: {e:?}"))
            })?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let text = response.text().await.unwrap_or_default();
            tracing::warn!("Supavisor returned {status}: {text}");
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

pub(crate) fn generate_supavisor_token(
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
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
