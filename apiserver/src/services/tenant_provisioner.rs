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

#[async_trait]
pub trait DatabaseMigratorTrait: Send + Sync {
    /// Encapsulates direct connection to establish/create tenant database,
    /// decoupling the service layer from direct connection management.
    async fn create_tenant_database(&self, admin_url: &str, db_name: &str) -> AppResult<()>;
}

use crate::diesel_pool::DieselPools;

/// Runs tenant migrations on an already-existing database.
///
/// This is the default (and currently only) implementation of
/// `TenantProvisionerTrait`. It connects to the provided `db_url`,
/// runs all pending tenant-level migrations, and returns the URL
/// unchanged for downstream Supavisor registration.
pub struct TenantMigrator {
    diesel_pools: DieselPools,
}

impl TenantMigrator {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl DatabaseMigratorTrait for TenantMigrator {
    async fn create_tenant_database(&self, admin_url: &str, db_name: &str) -> AppResult<()> {
        let create_db_sql = format!("CREATE DATABASE {db_name}");
        use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};

        let mut conn = AsyncPgConnection::establish(admin_url)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to connect to admin DB: {e}")))?;

        diesel::sql_query(create_db_sql)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create tenant database: {e}")))?;

        Ok(())
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

        // Fetch timezone from workspaces table in system database using the connection pool
        let mut timezone = "UTC".to_string();
        {
            use crate::schema::workspaces;
            use diesel::prelude::*;
            use diesel_async::RunQueryDsl;

            if let Ok(mut conn) = self.diesel_pools.api.get().await {
                if let Ok(tz) = workspaces::table
                    .filter(workspaces::id.eq(ws_id))
                    .select(workspaces::timezone)
                    .first::<String>(&mut *conn)
                    .await
                {
                    timezone = tz;
                }
            }
        }

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

            use diesel::RunQueryDsl;
            diesel::sql_query(
                "INSERT INTO work_locations (id, workspace_id, name, timezone, is_headquarters) \
                 VALUES ('11111111-1111-1111-1111-111111111111'::uuid, $1, 'Headquarters', $2, true) \
                 ON CONFLICT (id) DO NOTHING"
            )
            .bind::<diesel::sql_types::Uuid, _>(ws_id)
            .bind::<diesel::sql_types::Text, _>(&timezone)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_migrator_new() {
        let pools = DieselPools::for_test("postgres://invalid:invalid@localhost:12345/invalid_db");
        let _migrator = TenantMigrator::new(pools);
    }

    #[tokio::test]
    async fn test_provision_tenant_db_connection_failure() {
        let invalid_url = "postgres://invalid:invalid@localhost:12345/invalid_db";
        let pools = DieselPools::for_test(invalid_url);
        let migrator = TenantMigrator::new(pools);
        let workspace_id = Uuid::new_v4();

        let result = migrator
            .provision_tenant_db(&workspace_id, invalid_url)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        if let AppError::Internal(msg) = err {
            assert!(
                msg.contains("Failed to connect to tenant DB")
                    || msg.contains("could not translate host name")
            );
        } else {
            panic!("Expected AppError::Internal");
        }
    }

    #[tokio::test]
    async fn test_provision_tenant_db_success() {
        if let Ok(db_url) = std::env::var("DATABASE_URL") {
            use diesel::Connection;
            if diesel::PgConnection::establish(&db_url).is_ok() {
                let pools = DieselPools::for_test(&db_url);
                let migrator = TenantMigrator::new(pools);
                let workspace_id = Uuid::new_v4();
                let result = migrator.provision_tenant_db(&workspace_id, &db_url).await;
                assert!(
                    result.is_ok(),
                    "provision_tenant_db failed: {:?}",
                    result.err()
                );
                assert_eq!(result.unwrap(), db_url);
            } else {
                println!("Skipping test_provision_tenant_db_success: DB connection not available");
            }
        }
    }
}
