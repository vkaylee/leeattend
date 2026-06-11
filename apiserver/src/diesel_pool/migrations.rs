impl crate::diesel_pool::core::DieselPools {
    pub fn run_system_migrations(
        config: &crate::diesel_pool::config::DbPoolConfig,
    ) -> Result<(), crate::errors::AppError> {
        use diesel::Connection;
        use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

        // Force compile to detect new migrations (last touched: 2026-06-09 v4)
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

        // force recompile of system migrations
        pub const SYSTEM_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations/system");
        // force recompile of tenant migrations
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    fn strip_sql_comments(sql: &str) -> String {
        let mut result = String::new();
        let mut chars = sql.chars().peekable();
        let mut in_single_line_comment = false;
        let mut in_multi_line_comment = false;

        while let Some(c) = chars.next() {
            if in_single_line_comment {
                if c == '\n' {
                    in_single_line_comment = false;
                    result.push('\n');
                }
            } else if in_multi_line_comment {
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    in_multi_line_comment = false;
                }
            } else if c == '-' && chars.peek() == Some(&'-') {
                chars.next();
                in_single_line_comment = true;
            } else if c == '/' && chars.peek() == Some(&'*') {
                chars.next();
                in_multi_line_comment = true;
            } else {
                result.push(c);
            }
        }
        result
    }

    #[test]
    fn test_strip_sql_comments() {
        let sql = "
            -- This is a comment containing concurrently
            CREATE INDEX idx ON tab (col); /* another concurrently comment */
            VACUUM full;
        ";
        let stripped = strip_sql_comments(sql).to_lowercase();
        assert!(!stripped.contains("concurrently"));
        assert!(stripped.contains("vacuum"));
    }

    #[test]
    fn test_tenant_migrations_safety_for_transaction_pooling() {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
        let migrations_path = Path::new(&manifest_dir).join("migrations").join("tenant");

        if !migrations_path.exists() {
            println!(
                "Migrations directory not found at {:?}, skipping static check",
                migrations_path
            );
            return;
        }

        let mut errors = Vec::new();

        fn visit_dirs(dir: &Path, errors: &mut Vec<String>) -> std::io::Result<()> {
            if dir.is_dir() {
                for entry in fs::read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dirs(&path, errors)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                        let content = fs::read_to_string(&path)?;
                        let stripped = strip_sql_comments(&content);
                        let content_lower = stripped.to_lowercase();

                        // List of checks for non-transactional SQL commands in Postgres
                        if content_lower.contains("concurrently") {
                            errors.push(format!(
                                "File {:?} contains 'CONCURRENTLY' which cannot be run over a Transaction Pooler (e.g. PgBouncer/Supavisor transaction mode). Please use regular CREATE/DROP INDEX or run it manually.",
                                path
                            ));
                        }
                        if content_lower.contains("vacuum") {
                            errors.push(format!(
                                "File {:?} contains 'VACUUM' which cannot be run inside a transaction block (required for Transaction Poolers).",
                                path
                            ));
                        }
                        if content_lower.contains("create database")
                            || content_lower.contains("drop database")
                            || content_lower.contains("alter database")
                        {
                            errors.push(format!(
                                "File {:?} contains DATABASE DDL (CREATE/DROP/ALTER DATABASE) which is not allowed in tenant migrations.",
                                path
                            ));
                        }
                        if content_lower.contains("create tablespace")
                            || content_lower.contains("drop tablespace")
                            || content_lower.contains("alter tablespace")
                        {
                            errors.push(format!(
                                "File {:?} contains TABLESPACE DDL which is not allowed in tenant migrations.",
                                path
                            ));
                        }
                        if content_lower.contains("reindex system") {
                            errors.push(format!(
                                "File {:?} contains REINDEX SYSTEM which is not allowed in tenant migrations.",
                                path
                            ));
                        }
                    }
                }
            }
            Ok(())
        }

        visit_dirs(&migrations_path, &mut errors).unwrap();

        if !errors.is_empty() {
            let error_msg = errors.join("\n");
            panic!("Tenant migration safety check failed:\n{}", error_msg);
        }
    }
}
