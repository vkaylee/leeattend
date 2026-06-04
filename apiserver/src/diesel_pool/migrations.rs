impl crate::diesel_pool::core::DieselPools {
    pub fn run_system_migrations(
        config: &crate::diesel_pool::config::DbPoolConfig,
    ) -> Result<(), crate::errors::AppError> {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        // Force compile to detect new migrations
        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");

        let mut conn = diesel::PgConnection::establish(&config.primary_url).map_err(|e| {
            crate::errors::AppError::Internal(format!(
                "Failed to connect to primary database for system migrations: {e}"
            ))
        })?;

        conn.run_pending_migrations(SYSTEM_MIGRATIONS)
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Failed to run system migrations: {e}"))
            })?;

        tracing::info!("Database system migrations completed successfully");
        Ok(())
    }

    /// Run pending tenant migrations on a single tenant database.
    ///
    /// Accepts any valid PostgreSQL connection URL — either a direct connection
    /// or a Supavisor-routed URL resolved by
    /// [`TenantConnectionProvider::resolve_migration_url`].
    pub fn run_single_tenant_migration(
        url: &str,
        workspace_id: &uuid::Uuid,
    ) -> Result<(), crate::errors::AppError> {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

        let mut conn = diesel::PgConnection::establish(url).map_err(|e| {
            crate::errors::AppError::Internal(format!(
                "Failed to connect to tenant DB {workspace_id} for migration: {e}"
            ))
        })?;

        conn.run_pending_migrations(TENANT_MIGRATIONS)
            .map_err(|e| {
                crate::errors::AppError::Internal(format!(
                    "Failed to run tenant migrations for {workspace_id}: {e}"
                ))
            })?;

        tracing::info!(
            workspace_id = %workspace_id,
            "Tenant migrations completed successfully"
        );
        Ok(())
    }

    pub fn run_all_migrations_for_test(
        config: &crate::diesel_pool::config::DbPoolConfig,
    ) -> Result<(), crate::errors::AppError> {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");
        // force recompile to pick up new tenant migrations
        pub const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

        let mut conn = diesel::PgConnection::establish(&config.primary_url).map_err(|e| {
            crate::errors::AppError::Internal(format!(
                "Failed to connect to primary database for all migrations: {e}"
            ))
        })?;

        conn.run_pending_migrations(SYSTEM_MIGRATIONS)
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Failed to run system migrations: {e}"))
            })?;

        conn.run_pending_migrations(TENANT_MIGRATIONS)
            .map_err(|e| {
                crate::errors::AppError::Internal(format!("Failed to run tenant migrations: {e}"))
            })?;

        tracing::info!("Database test migrations (system + tenant) completed successfully");
        Ok(())
    }
}
