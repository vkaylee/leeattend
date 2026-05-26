use serde_json::json;

#[derive(Default)]
pub struct TenantPayloadOptions {
    pub tenant_id: Option<String>,
    pub pool_size: Option<u32>,
    pub is_master: bool,
}

pub fn parse_postgres_url(db_url: &str) -> Option<(String, String, String, u16, String)> {
    let prefix = if db_url.starts_with("postgres://") {
        "postgres://"
    } else if db_url.starts_with("postgresql://") {
        "postgresql://"
    } else {
        return None;
    };

    let without_scheme = &db_url[prefix.len()..];
    let (rest, _query) = match without_scheme.split_once('?') {
        Some((r, q)) => (r, Some(q)),
        None => (without_scheme, None),
    };

    let (auth_host_port, dbname) = match rest.rsplit_once('/') {
        Some((ahp, db)) => (ahp, db.to_string()),
        None => (rest, String::new()),
    };

    let (auth, host_port) = match auth_host_port.rsplit_once('@') {
        Some((a, hp)) => (Some(a), hp),
        None => (None, auth_host_port),
    };

    let (host, port) = match host_port.rsplit_once(':') {
        Some((h, p)) => {
            if let Ok(port_num) = p.parse::<u16>() {
                (h.to_string(), port_num)
            } else {
                (host_port.to_string(), 5432)
            }
        }
        None => (host_port.to_string(), 5432),
    };

    let (user, password) = match auth {
        Some(a) => match a.split_once(':') {
            Some((u, p)) => (u.to_string(), p.to_string()),
            None => (a.to_string(), String::new()),
        },
        None => (String::new(), String::new()),
    };

    let user = percent_encoding::percent_decode_str(&user)
        .decode_utf8_lossy()
        .to_string();
    let password = percent_encoding::percent_decode_str(&password)
        .decode_utf8_lossy()
        .to_string();

    Some((user, password, host, port, dbname))
}

pub fn build_tenant_payload(
    db_url: &str,
    opts: TenantPayloadOptions,
) -> Option<(String, serde_json::Value)> {
    let Some((mut url_username, url_password, db_host, db_port, db_name)) =
        parse_postgres_url(db_url)
    else {
        tracing::warn!("Failed to parse DATABASE_URL (details omitted for security)");
        return None;
    };

    // The actual upstream DB user shouldn't include the tenant dot prefix
    if url_username.contains('.') {
        if let Some((prefix, _)) = url_username.split_once('.') {
            url_username = prefix.to_string();
        }
    }

    let db_user = url_username;
    let db_password = url_password;

    if db_name.is_empty() || db_user.is_empty() || db_password.is_empty() {
        tracing::warn!("DATABASE_URL is missing required components (user/password/name). Cannot initialize Supavisor.");
        return None;
    }

    let tenant_id = opts.tenant_id.unwrap_or_else(|| {
        if opts.is_master {
            std::env::var("SUPAVISOR_TENANT_ID")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| db_name.clone())
        } else {
            db_name.clone()
        }
    });

    let enforce_ssl = std::env::var("DATABASE_REQUIRE_SSL")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    let pool_size: u32 = opts.pool_size.unwrap_or_else(|| {
        std::env::var("SUPAVISOR_POOL_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20)
    });

    let mode_type = std::env::var("SUPAVISOR_MODE_TYPE").unwrap_or_else(|_| "session".to_string());

    let ip_version = std::env::var("SUPAVISOR_IP_VERSION").unwrap_or_else(|_| "auto".to_string());

    let mut tenant_map = serde_json::Map::new();
    tenant_map.insert("db_host".to_string(), json!(db_host));
    tenant_map.insert("db_port".to_string(), json!(db_port));
    tenant_map.insert("db_database".to_string(), json!(db_name));
    tenant_map.insert("ip_version".to_string(), json!(ip_version));
    tenant_map.insert("enforce_ssl".to_string(), json!(enforce_ssl));
    tenant_map.insert("default_pool_size".to_string(), json!(pool_size));

    // BYOD architecture: We explicitely push all user credentials to Supavisor via the API.
    // We strictly disable `auth_query` and force `require_user = true`. This prevents Supavisor
    // from querying the upstream database's `pg_authid` table, which external BYOD users
    // usually do not have permission to read.
    tenant_map.insert("require_user".to_string(), json!(true));

    // The client pool password should be the same as the downstream database password,
    // otherwise the API server will fail to authenticate when trying to connect to the pool.
    let client_password = db_password.clone();

    tenant_map.insert(
        "users".to_string(),
        json!([
            {
                "db_user": &db_user,
                "db_password": &db_password,
                "pool_password": &client_password,
                "pool_size": pool_size,
                "mode_type": &mode_type,
                "is_manager": false
            }
        ]),
    );

    let payload = json!({
        "tenant": tenant_map
    });

    Some((tenant_id, payload))
}
