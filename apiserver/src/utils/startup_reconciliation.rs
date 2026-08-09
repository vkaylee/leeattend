use crate::diesel_pool::{DbPoolConfig, DieselPools, TenantAuthToken};
use std::collections::HashMap;
use uuid::Uuid;

/// di trú cho các tenant.
#[allow(clippy::too_many_lines, clippy::implicit_hasher)]
pub async fn register_workspace_tenants_and_migrate(
    db_config: &DbPoolConfig,
    diesel_pools: &DieselPools,
    backup_cache: HashMap<Uuid, (String, String, rust_decimal::Decimal)>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Query workspaces from system DB via direct connection
    let primary_url = db_config.primary_url.clone();
    let workspaces: Vec<(Uuid, Option<String>)> = match tokio::task::spawn_blocking(move || {
        use diesel::sql_types::{Nullable, Text, Uuid as UuidSql};
        use diesel::{Connection, QueryableByName, RunQueryDsl};

        #[derive(QueryableByName)]
        struct WsRow {
            #[diesel(sql_type = UuidSql)]
            id: Uuid,
            #[diesel(sql_type = Nullable<Text>)]
            custom_db_url_encrypted: Option<String>,
        }

        let mut conn = diesel::PgConnection::establish(&primary_url)
            .map_err(|e| format!("Failed to connect: {e}"))?;

        let workspaces = diesel::sql_query("SELECT id, custom_db_url_encrypted FROM workspaces")
            .load::<WsRow>(&mut conn)
            .map(|r| {
                r.into_iter()
                    .map(|w| (w.id, w.custom_db_url_encrypted))
                    .collect()
            })
            .map_err(|e| format!("Failed to query workspaces: {e}"))?;

        Ok::<_, String>(workspaces)
    })
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            tracing::warn!("{e} — skipping tenant migrations");
            return Ok(());
        }
        Err(e) => {
            tracing::error!("Failed to spawn blocking task for workspace query: {e}");
            return Ok(());
        }
    };

    if workspaces.is_empty() {
        tracing::info!("No workspaces — skipping tenant migrations");
        return Ok(());
    }

    let config = crate::utils::config::get_config();
    let app_env = &config.env;
    let crypto_service: Option<crate::crypto::CryptoService> =
        if let Some(keys) = &config.encryption_keys_json {
            let crypto = crate::crypto::CryptoService::from_json(keys)
                .map_err(|e| format!("Invalid ENCRYPTION_KEYS_JSON: {e}"))?;
            Some(crypto)
        } else if app_env == "production" {
            return Err(Box::from("ENCRYPTION_KEYS_JSON must be set in production"));
        } else {
            None
        };

    let mut migration_targets: Vec<(Uuid, String)> = Vec::new();

    for (ws_id, custom_db_url_encrypted) in workspaces {
        let tenant_url = if let Some(encrypted) = custom_db_url_encrypted {
            if let Some(crypto) = &crypto_service {
                crypto.decrypt(&encrypted).unwrap_or(encrypted)
            } else {
                encrypted
            }
        } else {
            tracing::warn!(
                "Workspace {} missing custom_db_url_encrypted! Skipping migration.",
                ws_id
            );
            continue;
        };

        // Acquire through the session-pool helper, then release before the blocking migration.
        // TenantPoolManager resolves migration URLs identically to the decrypted tenant URL.
        let tenant_token = TenantAuthToken::system_override(ws_id);
        if let Err(e) = diesel_pools.tenant_session_conn(&tenant_token).await {
            tracing::warn!(
                workspace_id = %ws_id,
                error = %e,
                "Failed to acquire tenant session connection — skipping"
            );
            continue;
        }

        migration_targets.push((ws_id, tenant_url));
    }

    if !migration_targets.is_empty() {
        tracing::info!(
            workspace_count = migration_targets.len(),
            "Running tenant migrations with per-tenant credentials"
        );

        let cache_clone = backup_cache.clone();
        match tokio::task::spawn_blocking(move || {
            for (ws_id, migration_url) in &migration_targets {
                if let Err(e) = DieselPools::run_single_tenant_migration(migration_url, ws_id) {
                    tracing::warn!(
                        workspace_id = %ws_id,
                        error = %e,
                        "Tenant migration failed — skipping workspace"
                    );
                    continue;
                }

                // Run EAV reconciliation
                let backup_info = cache_clone.get(ws_id);
                if let Err(e) = reconcile_tenant_database(migration_url, ws_id, backup_info) {
                    tracing::error!(
                        workspace_id = %ws_id,
                        error = %e,
                        "Failed to reconcile tenant database: {e}"
                    );
                }
            }
        })
        .await
        {
            Ok(()) => {}
            Err(e) => tracing::error!("Tenant migration task panicked: {e}"),
        }
    }
    Ok(())
}

fn reconcile_tenant_database(
    migration_url: &str,
    ws_id: &Uuid,
    backup_info: Option<&(String, String, rust_decimal::Decimal)>,
) -> Result<(), String> {
    use diesel::Connection;
    use diesel::RunQueryDsl;

    #[derive(diesel::QueryableByName)]
    struct ExistsRow {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        exists: bool,
    }

    let mut conn = diesel::PgConnection::establish(migration_url)
        .map_err(|e| format!("Failed to connect to tenant database: {e}"))?;

    // 1. Seed overtime.holiday_overnight_strategy
    let strategy_exists: bool = diesel::sql_query(
        "SELECT EXISTS(SELECT 1 FROM workspace_config_entries WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND config_key = 'overtime.holiday_overnight_strategy')"
    )
    .bind::<diesel::sql_types::Uuid, _>(*ws_id)
    .get_result::<ExistsRow>(&mut conn)
    .map(|r| r.exists)
    .map_err(|e| format!("Failed to check strategy existence: {e}"))?;

    if !strategy_exists {
        let strategy_val = if let Some((_, strategy, _)) = backup_info {
            strategy.clone()
        } else {
            "split_at_midnight".to_string()
        };

        let _ = diesel::sql_query(
            "INSERT INTO workspace_config_entries (id, workspace_id, scope_type, scope_id, config_key, value_text, effective_from) \
             VALUES (gen_random_uuid(), $1, 'workspace', NULL, 'overtime.holiday_overnight_strategy', $2, '1970-01-01')"
        )
        .bind::<diesel::sql_types::Uuid, _>(*ws_id)
        .bind::<diesel::sql_types::Text, _>(strategy_val)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to insert default strategy: {e}"))?;
    }

    // 2. Seed overtime.default_holiday_ot_multiplier
    let multiplier_exists: bool = diesel::sql_query(
        "SELECT EXISTS(SELECT 1 FROM workspace_config_entries WHERE workspace_id = $1 AND scope_type = 'workspace' AND scope_id IS NULL AND config_key = 'overtime.default_holiday_ot_multiplier')"
    )
    .bind::<diesel::sql_types::Uuid, _>(*ws_id)
    .get_result::<ExistsRow>(&mut conn)
    .map(|r| r.exists)
    .map_err(|e| format!("Failed to check multiplier existence: {e}"))?;

    if !multiplier_exists {
        let multiplier_val = if let Some((_, _, multiplier)) = backup_info {
            *multiplier
        } else {
            rust_decimal::Decimal::new(30, 1) // 3.0
        };

        let _ = diesel::sql_query(
            "INSERT INTO workspace_config_entries (id, workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from) \
             VALUES (gen_random_uuid(), $1, 'workspace', NULL, 'overtime.default_holiday_ot_multiplier', $2, '1970-01-01')"
        )
        .bind::<diesel::sql_types::Uuid, _>(*ws_id)
        .bind::<diesel::sql_types::Numeric, _>(multiplier_val)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to insert default multiplier: {e}"))?;
    }

    Ok(())
}
