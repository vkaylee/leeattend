use crate::errors::{AppError, AppResult};
use async_trait::async_trait;
use diesel::sql_query;
use diesel_async::RunQueryDsl;
use tracing::info;
use uuid::Uuid;
use std::env;

#[async_trait]
pub trait TenantProvisionerTrait: Send + Sync {
    /// Provisions a tenant database. If custom_db_url is provided, it uses that (BYOD),
    /// otherwise it provisions a new logical database on the primary cluster.
    /// Returns the full connection URL of the provisioned database.
    async fn provision_tenant_db(&self, workspace_id: &Uuid, custom_db_url: Option<String>) -> AppResult<String>;
}

pub struct DefaultTenantProvisioner {
    system_pool: crate::diesel_pool::DieselPool,
}

impl DefaultTenantProvisioner {
    pub fn new(system_pool: crate::diesel_pool::DieselPool) -> Self {
        Self { system_pool }
    }
}

#[async_trait]
impl TenantProvisionerTrait for DefaultTenantProvisioner {
    async fn provision_tenant_db(&self, workspace_id: &Uuid, custom_db_url: Option<String>) -> AppResult<String> {
        let final_db_url = match custom_db_url {
            Some(url) => {
                info!("Using custom BYOD database URL for workspace {}", workspace_id);
                url
            }
            None => {
                // Remove hyphens for safe Postgres DB name
                let db_name = format!("tenant_{}", workspace_id.to_string().replace("-", ""));
                let query = format!("CREATE DATABASE {}", db_name);
                
                // 1. Get connection from system pool
                let mut conn = self.system_pool.get().await.map_err(|e| {
                    AppError::Internal(format!("Failed to get connection from system pool: {}", e))
                })?;
                
                // 2. Execute CREATE DATABASE
                sql_query(query).execute(&mut conn).await.map_err(|e| {
                    AppError::Internal(format!("Failed to create physical database {}: {}", db_name, e))
                })?;
                
                info!("Successfully created physical database: {}", db_name);
                
                // 3. Construct new database URL based on existing master DATABASE_URL
                let primary_url = env::var("DATABASE_URL").unwrap_or_default();
                if primary_url.is_empty() {
                    return Err(AppError::Internal("DATABASE_URL is not set".to_string()));
                }

                let mut parsed_url = reqwest::Url::parse(&primary_url).map_err(|e| {
                    AppError::Internal(format!("Invalid DATABASE_URL format: {}", e))
                })?;
                
                parsed_url.set_path(&db_name);
                parsed_url.to_string()
            }
        };

        // 4. Run tenant migrations on the new database (either BYOD or newly created)
        let final_db_url_clone = final_db_url.clone();
        tokio::task::spawn_blocking(move || {
            use diesel::Connection;
            use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

            const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

            let mut sync_conn = diesel::PgConnection::establish(&final_db_url_clone).map_err(|e| {
                AppError::Internal(format!("Failed to connect to tenant DB: {}", e))
            })?;
            
            sync_conn.run_pending_migrations(TENANT_MIGRATIONS).map_err(|e| {
                AppError::Internal(format!("Failed to run tenant migrations: {}", e))
            })?;
                
            Ok::<(), AppError>(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Migration blocking task failed: {}", e)))??;

        info!("Successfully migrated database at: {}", final_db_url);
        
        Ok(final_db_url)
    }
}
