use crate::errors::{AppError, AppResult};
use async_trait::async_trait;
use tracing::info;
use uuid::Uuid;

#[async_trait]
pub trait TenantProvisionerTrait: Send + Sync {
    /// Initializes an existing database for tenant use:
    /// 1. Runs tenant migrations on the provided database
    /// 2. Returns the database URL for Supavisor registration
    ///
    /// The database must already exist. This trait does NOT create databases.
    /// The caller (user or external provider) is responsible for provisioning
    /// the database and providing a valid connection string.
    async fn provision_tenant_db(&self, workspace_id: &Uuid, db_url: &str) -> AppResult<String>;
}

/// Runs tenant migrations on an already-existing database.
///
/// This is the default (and currently only) implementation of
/// `TenantProvisionerTrait`. It connects to the provided `db_url`,
/// runs all pending tenant-level migrations, and returns the URL
/// unchanged for downstream Supavisor registration.
#[derive(Default)]
pub struct TenantMigrator;

impl TenantMigrator {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TenantProvisionerTrait for TenantMigrator {
    async fn provision_tenant_db(&self, workspace_id: &Uuid, db_url: &str) -> AppResult<String> {
        let db_url_owned = db_url.to_string();
        let ws_id = *workspace_id;

        info!(
            workspace_id = %ws_id,
            "Running tenant migrations on provided database"
        );

        let db_url_clone = db_url_owned.clone();
        tokio::task::spawn_blocking(move || {
            use diesel::Connection;
            use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

            const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

            let mut sync_conn = diesel::PgConnection::establish(&db_url_clone)
                .map_err(|e| AppError::Internal(format!("Failed to connect to tenant DB: {e}")))?;

            sync_conn
                .run_pending_migrations(TENANT_MIGRATIONS)
                .map_err(|e| AppError::Internal(format!("Failed to run tenant migrations: {e}")))?;

            // Ensure Headquarters location exists
            let insert_hq_sql = format!(
                "INSERT INTO work_locations (id, workspace_id, name, timezone, is_headquarters)
                 SELECT '11111111-1111-1111-1111-111111111111'::uuid, '{ws_id}', 'Headquarters', COALESCE(timezone, 'UTC'), true
                 FROM workspaces WHERE id = '{ws_id}'
                 ON CONFLICT (id) DO NOTHING"
            );

            use diesel::RunQueryDsl;
            diesel::sql_query(insert_hq_sql)
                .execute(&mut sync_conn)
                .map_err(|e| AppError::Internal(format!("Failed to seed Headquarters location: {e}")))?;

            Ok::<(), AppError>(())
        })
        .await
        .map_err(|e| AppError::Internal(format!("Migration blocking task failed: {e}")))??;

        info!(
            workspace_id = %ws_id,
            "Successfully migrated tenant database"
        );

        Ok(db_url_owned)
    }
}
