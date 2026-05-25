use crate::diesel_pool::config::DbPoolConfig;

impl crate::diesel_pool::core::DieselPools {
    #[allow(clippy::expect_used)]
    pub fn run_system_migrations(config: &DbPoolConfig) {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");

        let mut conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to primary database for system migrations");

        conn.run_pending_migrations(SYSTEM_MIGRATIONS)
            .expect("Failed to run system migrations");

        tracing::info!("Database system migrations completed successfully");
    }

    /// Resolve the tenant database connection URL for running migrations.
    pub fn resolve_tenant_migration_url(
        primary_url: &str,
        pooler_url: Option<&str>,
        workspace_id: &uuid::Uuid,
        session_port_override: Option<u16>,
    ) -> Result<String, String> {
        let mut base_url_parsed = reqwest::Url::parse(pooler_url.unwrap_or(primary_url))
            .map_err(|e| format!("Failed to parse URL: {e}"))?;

        if pooler_url.is_some() && base_url_parsed.username().contains('.') {
            if let Some(port) = session_port_override {
                let _ = base_url_parsed.set_port(Some(port));
            }
        }

        let db_name = format!("tenant_{}", workspace_id.to_string().replace('-', ""));
        base_url_parsed.set_path(&db_name);

        let current_user = base_url_parsed.username().to_string();
        if current_user.contains('.') {
            if let Some((prefix, _)) = current_user.split_once('.') {
                base_url_parsed
                    .set_username(&format!("{prefix}.{db_name}"))
                    .map_err(|_e| "Failed to set username".to_string())?;
            }
        }

        Ok(base_url_parsed.to_string())
    }

    /// Run pending tenant migrations on ALL existing tenant databases.
    ///
    /// Iterates over every workspace in the system DB, connects to each tenant DB,
    /// and runs any pending tenant-level migrations. Called once at startup.
    #[allow(clippy::expect_used)]
    pub fn run_tenant_migrations(config: &DbPoolConfig) {
        use diesel::sql_types::Uuid as UuidSql;
        use diesel::{Connection, QueryableByName, RunQueryDsl};
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

        #[derive(QueryableByName)]
        struct WorkspaceRow {
            #[diesel(sql_type = UuidSql)]
            id: uuid::Uuid,
        }

        // Connect to system DB and list all workspaces
        let mut sys_conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to system DB for tenant migrations");

        #[allow(clippy::disallowed_methods)]
        let workspace_rows: Vec<WorkspaceRow> = diesel::sql_query("SELECT id FROM workspaces")
            .load(&mut sys_conn)
            .expect("Failed to query workspaces for tenant migrations");

        if workspace_rows.is_empty() {
            tracing::info!("No workspaces found — skipping tenant migrations");
            return;
        }

        tracing::info!(
            workspace_count = workspace_rows.len(),
            "Running tenant migrations on existing workspaces"
        );

        let session_port: Option<u16> = if config.pooler_sess_url.is_some() {
            let port: u16 = std::env::var("SUPAVISOR_PORT_SESSION")
                .unwrap_or_else(|_| "5452".to_string())
                .parse()
                .unwrap_or(5452);
            Some(port)
        } else {
            None
        };

        for row in &workspace_rows {
            let tenant_url = match Self::resolve_tenant_migration_url(
                &config.primary_url,
                config.pooler_sess_url.as_deref(),
                &row.id,
                session_port,
            ) {
                Ok(url) => url,
                Err(e) => {
                    tracing::error!(
                        workspace_id = %row.id,
                        error = %e,
                        "Failed to resolve tenant migration URL — skipping workspace"
                    );
                    continue;
                }
            };

            let mut retries = 10;

            loop {
                match diesel::PgConnection::establish(&tenant_url) {
                    Ok(mut tenant_conn) => {
                        match tenant_conn.run_pending_migrations(TENANT_MIGRATIONS) {
                            Ok(_) => {
                                tracing::info!(
                                    workspace_id = %row.id,
                                    "Tenant migrations completed successfully"
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    workspace_id = %row.id,
                                    error = %e,
                                    "Failed to run tenant migrations — skipping workspace"
                                );
                            }
                        }
                        break;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if retries > 0 && crate::diesel_pool::tenant_manager::is_transient_supavisor_error(&err_msg)
                        {
                            tracing::warn!(
                                workspace_id = %row.id,
                                error = %err_msg,
                                "Transient connection error (Supavisor sync delay), retrying in 1s..."
                            );
                            std::thread::sleep(std::time::Duration::from_secs(1));
                            retries -= 1;
                            continue;
                        }

                        tracing::warn!(
                            workspace_id = %row.id,
                            error = %e,
                            "Failed to connect to tenant DB after retries — may not exist yet, skipping"
                        );
                        break;
                    }
                }
            }
        }

        tracing::info!("Tenant migrations on existing workspaces completed");
    }

    #[allow(clippy::expect_used)]
    pub fn run_all_migrations_for_test(config: &DbPoolConfig) {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");
        // force recompile to pick up new tenant migrations
        pub const TENANT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/tenant");

        let mut conn = diesel::PgConnection::establish(&config.primary_url)
            .expect("Failed to connect to primary database for all migrations");

        conn.run_pending_migrations(SYSTEM_MIGRATIONS)
            .expect("Failed to run system migrations");

        conn.run_pending_migrations(TENANT_MIGRATIONS)
            .expect("Failed to run tenant migrations");

        tracing::info!("Database test migrations (system + tenant) completed successfully");
    }
}
