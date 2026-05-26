#[cfg(test)]
mod tests {
    use crate::supavisor::*;
    use serde_json::json;
    #[test]
    fn test_build_tenant_payload_valid() {
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");

        assert_eq!(tenant_id, "my_company_db");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["db_host"], "my_db_host");
        assert_eq!(tenant["db_port"], 5433);
        assert_eq!(tenant["db_database"], "my_company_db");

        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["db_user"], "my_user");
        assert_eq!(users[0]["db_password"], "my_secret");
        assert!(users[0].get("pool_password").is_some());
        assert_eq!(users[0]["is_manager"], false);
    }

    #[test]
    fn test_build_tenant_payload_enforce_ssl_from_env() {
        std::env::set_var("DATABASE_REQUIRE_SSL", "true");
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");
        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["enforce_ssl"], true);
        std::env::remove_var("DATABASE_REQUIRE_SSL");

        // Test false fallback
        let (_, payload2) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");
        let tenant2 = payload2.get("tenant").expect("missing tenant key");
        assert_eq!(tenant2["enforce_ssl"], false);
    }

    #[test]
    fn test_build_tenant_payload_sets_pool_password_from_env() {
        let physical_db_url = "postgres://tenant_user:physical_pass@my_db_host:5433/tenant_db";

        // 1. Without DATABASE_URL or DATABASE_POOLER_URL set, pool_password should default to db_password
        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("DATABASE_POOLER_TX_URL");
        std::env::remove_var("DATABASE_POOLER_SESS_URL");
        let (_, payload) = build_tenant_payload(
            physical_db_url,
            TenantPayloadOptions {
                is_master: false,
                ..Default::default()
            },
        )
        .unwrap();
        let users = payload.get("tenant").unwrap()["users"].as_array().unwrap();
        assert_eq!(users[0]["db_password"], "physical_pass");
        assert_eq!(users[0]["pool_password"], "physical_pass");

        // 2. With DATABASE_URL set, pool_password is STILL the physical_pass (BYOD alignment rule)
        std::env::set_var(
            "DATABASE_URL",
            "postgres://postgres.leeattend:master_pass@supavisor:6543/leeattend",
        );
        let (_, payload2) = build_tenant_payload(
            physical_db_url,
            TenantPayloadOptions {
                is_master: false,
                ..Default::default()
            },
        )
        .unwrap();
        let users2 = payload2.get("tenant").unwrap()["users"].as_array().unwrap();
        assert_eq!(users2[0]["db_password"], "physical_pass");
        assert_eq!(users2[0]["pool_password"], "physical_pass");

        // 3. With DATABASE_POOLER_TX_URL set, pool_password is STILL the physical_pass
        std::env::set_var(
            "DATABASE_POOLER_TX_URL",
            "postgres://postgres.leeattend:pooler_pass@supavisor:6543/leeattend",
        );
        let (_, payload3) = build_tenant_payload(
            physical_db_url,
            TenantPayloadOptions {
                is_master: false,
                ..Default::default()
            },
        )
        .unwrap();
        let users3 = payload3.get("tenant").unwrap()["users"].as_array().unwrap();
        assert_eq!(users3[0]["db_password"], "physical_pass");
        assert_eq!(users3[0]["pool_password"], "physical_pass");

        std::env::remove_var("DATABASE_URL");
        std::env::remove_var("DATABASE_POOLER_URL");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_tenant_id() {
        std::env::set_var("SUPAVISOR_TENANT_ID", "custom_tenant_name");
        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");

        assert_eq!(tenant_id, "custom_tenant_name");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "my_company_db",
            "DB name should still be the original"
        );

        std::env::remove_var("SUPAVISOR_TENANT_ID");
    }

    #[test]
    fn test_build_tenant_payload_dynamic_ignores_global_env() {
        std::env::set_var("SUPAVISOR_TENANT_ID", "global_tenant_id");

        let url = "postgres://dynamic_user:dynamic_secret@dynamic_db_host:5433/dynamic_db";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                tenant_id: None,
                pool_size: None,
                is_master: false,
            },
        )
        .expect("Should build payload");

        assert_eq!(tenant_id, "dynamic_db");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["db_user"], "dynamic_user");
        assert_eq!(users[0]["db_password"], "dynamic_secret");

        std::env::remove_var("SUPAVISOR_TENANT_ID");
    }

    #[test]
    fn test_build_tenant_payload_with_explicit_options() {
        std::env::set_var("SUPAVISOR_TENANT_ID", "global_tenant_id");
        std::env::set_var("SUPAVISOR_POOL_SIZE", "50");

        let url = "postgres://dynamic_user:dynamic_secret@dynamic_db_host:5433/dynamic_db";

        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                tenant_id: Some("explicit_tenant_id".to_string()),
                pool_size: Some(99),
                is_master: false, // The options should override ENV regardless of is_master anyway
            },
        )
        .expect("Should build payload");

        assert_eq!(
            tenant_id, "explicit_tenant_id",
            "Explicit tenant_id option should override env and db_name"
        );

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(
            users[0]["pool_size"], 99,
            "Explicit pool_size option should override env"
        );
        assert_eq!(
            tenant["db_database"], "dynamic_db",
            "db_database must be extracted from URL, not set to tenant_id"
        );

        std::env::remove_var("SUPAVISOR_TENANT_ID");
        std::env::remove_var("SUPAVISOR_POOL_SIZE");
    }

    #[test]
    fn test_build_tenant_payload_byod_preserves_db_name() {
        let url = "postgres://byod_user:byod_pass@remote_host:5432/custom_byod_db";

        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                tenant_id: Some("tenant_019e615b51dc7163893682f42318cf84".to_string()),
                pool_size: None,
                is_master: false,
            },
        )
        .expect("Should build payload");

        assert_eq!(
            tenant_id, "tenant_019e615b51dc7163893682f42318cf84",
            "Tenant ID must be the explicit one provided"
        );

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "custom_byod_db",
            "CRITICAL BYOD REQUIREMENT: db_database must be the actual database in the URL, not the tenant_id"
        );
    }

    #[test]
    fn test_build_tenant_payload_decodes_percent_encoding() {
        // "user@name":"pass#word" encoded in URL
        let url = "postgres://user%40name:pass%23word@dynamic_db_host:5433/dynamic_db";
        let (_, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                tenant_id: None,
                pool_size: None,
                is_master: false,
            },
        )
        .expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);

        // Assert they are properly decoded!
        assert_eq!(users[0]["db_user"], "user@name");
        assert_eq!(users[0]["db_password"], "pass#word");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_pool_settings() {
        std::env::set_var("SUPAVISOR_POOL_SIZE", "50");
        std::env::set_var("SUPAVISOR_MODE_TYPE", "session");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        let users = tenant["users"].as_array().expect("missing users array");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0]["pool_size"], 50);
        assert_eq!(users[0]["mode_type"], "session");
        assert!(users[0].get("pool_password").is_some());
        assert_eq!(users[0]["is_manager"], false);

        std::env::remove_var("SUPAVISOR_POOL_SIZE");
        std::env::remove_var("SUPAVISOR_MODE_TYPE");
    }

    #[test]
    fn test_build_tenant_payload_with_custom_ip_version() {
        std::env::set_var("SUPAVISOR_IP_VERSION", "v4");

        let url = "postgres://my_user:my_secret@my_db_host:5433/my_company_db";
        let (_, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["ip_version"], "v4");

        std::env::remove_var("SUPAVISOR_IP_VERSION");
    }

    #[test]
    fn test_build_tenant_payload_missing_password() {
        let url = "postgres://my_user@my_db_host:5432/my_company_db";
        let result = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        );
        assert!(
            result.is_none(),
            "Should return None if password is missing"
        );
    }

    #[test]
    fn test_build_tenant_payload_missing_user() {
        let url = "postgres://:my_secret@my_db_host:5432/my_company_db";
        let result = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        );
        assert!(result.is_none(), "Should return None if user is missing");
    }

    #[test]
    fn test_build_tenant_payload_missing_db_name() {
        let url = "postgres://my_user:my_secret@my_db_host:5432/";
        let result = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        );
        assert!(result.is_none(), "Should return None if db name is missing");
    }

    #[test]
    fn test_build_tenant_payload_invalid_url() {
        let url = "not_a_url";
        let result = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        );
        assert!(result.is_none(), "Should return None for invalid URL");
    }

    #[test]
    fn test_build_tenant_payload_with_query_params() {
        // Query parameters like sslmode or application_name should not break the path extraction
        let url = "postgres://my_user:my_secret@my_db_host:5432/my_company_db?sslmode=disable&application_name=leeattend";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload correctly even with query params");

        assert_eq!(
            tenant_id, "my_company_db",
            "DB name should not include query params"
        );

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(tenant["db_database"], "my_company_db");
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_matches_connection_url_format() {
        // The tenant_id must match the format produced by diesel_pool::get_tenant_url():
        // tenant_<uuid-without-hyphens>. This contract prevents "Tenant or user not found"
        // errors when the API server connects through Supavisor.
        let url = "postgres://admin:secret@db_system:5432/tenant_019e40ee11927492bc2985dad4675f34";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");

        assert_eq!(
            tenant_id, "tenant_019e40ee11927492bc2985dad4675f34",
            "tenant_id must be tenant_<uuid-no-hyphens> to match get_tenant_url() format"
        );

        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "tenant_019e40ee11927492bc2985dad4675f34",
            "db_database should match the database name from URL"
        );

        // Verify that the tenant_id is NOT a raw UUID with hyphens
        assert!(
            !tenant_id.contains('-'),
            "tenant_id must NOT contain hyphens — raw UUID format is incompatible with get_tenant_url()"
        );
        assert!(
            tenant_id.starts_with("tenant_"),
            "tenant_id must start with 'tenant_' prefix"
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Contract: build_tenant_payload(, TenantPayloadOptions { is_master: true, ..Default::default() }) MUST return a tenant_id that matches
    // the tenant identifier embedded in the connection URL produced by
    // diesel_pool::get_tenant_url(). The format is:
    //
    //   tenant_<uuid-without-hyphens>
    //
    // These tests lock down the registration side of the contract so it
    // cannot drift from the connection side. See diesel_pool.rs for the
    // connection-side contract tests.
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_tenant_payload_tenant_id_standard_db_name() {
        // When the DB name follows tenant_<uuid-no-hyphens>, tenant_id
        // defaults to the database name from the URL.
        let url = "postgres://admin:secret@db_system:5432/tenant_a1b2c3d4e5f67890abcdef1234567890";
        let (tenant_id, _) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build");
        assert_eq!(tenant_id, "tenant_a1b2c3d4e5f67890abcdef1234567890");
        assert_eq!(tenant_id.len(), 39); // "tenant_" + 32 hex
        assert!(!tenant_id.contains('-'));
    }

    #[test]
    #[serial_test::serial]
    fn test_build_tenant_payload_tenant_id_nil_uuid_db_name() {
        let url = "postgres://admin:secret@db_system:5432/tenant_00000000000000000000000000000000";
        let (tenant_id, _) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build");
        assert_eq!(tenant_id, "tenant_00000000000000000000000000000000");
    }

    #[test]
    #[serial_test::serial]
    fn test_build_tenant_payload_tenant_id_max_uuid_db_name() {
        let url = "postgres://admin:secret@db_system:5432/tenant_ffffffffffffffffffffffffffffffff";
        let (tenant_id, _) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build");
        assert_eq!(tenant_id, "tenant_ffffffffffffffffffffffffffffffff");
    }

    #[test]
    fn test_build_tenant_payload_supavisor_tenant_id_overrides_db_name() {
        // When SUPAVISOR_TENANT_ID is set, tenant_id takes that value,
        // but db_database in the payload stays as the actual PG database name.
        // This is the BYOD scenario — custom DB, but Supavisor routing
        // still uses tenant_<uuid-no-hyphens>.
        std::env::set_var(
            "SUPAVISOR_TENANT_ID",
            "tenant_019e40ee11927492bc2985dad4675f34",
        );
        let url = "postgres://admin:secret@db_system:5432/my_custom_db";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build");
        std::env::remove_var("SUPAVISOR_TENANT_ID");

        assert_eq!(
            tenant_id, "tenant_019e40ee11927492bc2985dad4675f34",
            "tenant_id must come from SUPAVISOR_TENANT_ID for BYOD"
        );
        // The payload must still point to the actual PG database
        let tenant = payload.get("tenant").expect("missing tenant key");
        assert_eq!(
            tenant["db_database"], "my_custom_db",
            "db_database must be the actual PG database, not the tenant_id"
        );
        assert!(
            !tenant_id.contains('-'),
            "BYOD tenant_id must NOT contain hyphens"
        );
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_never_contains_hyphens() {
        // Regardless of URL format or env vars, the tenant_id must NEVER
        // contain hyphens — the connection URL strips them and Supavisor
        // would not find the tenant.

        // Case 1: standard tenant_ DB name
        let url1 = "postgres://u:p@h:5432/tenant_019e40ee11927492bc2985dad4675f34";
        let (tid1, _) = build_tenant_payload(
            url1,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            !tid1.contains('-'),
            "standard DB: tenant_id must not have hyphens"
        );

        // Case 2: custom DB name with hyphenated override
        std::env::set_var(
            "SUPAVISOR_TENANT_ID",
            "tenant_a1b2c3d4e5f67890abcdef1234567890",
        );
        let url2 = "postgres://u:p@h:5432/some-custom-db-name";
        let (tid2, _) = build_tenant_payload(
            url2,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .unwrap();
        std::env::remove_var("SUPAVISOR_TENANT_ID");
        assert!(!tid2.contains('-'), "BYOD: tenant_id must not have hyphens");

        // Case 3: custom DB name without SUPAVISOR_TENANT_ID override
        // (falls back to DB name from URL — caller is responsible for ensuring
        // the correct format via SUPAVISOR_TENANT_ID for non-standard names)
        let url3 = "postgres://u:p@h:5432/custom_db_name";
        let (tid3, _) = build_tenant_payload(
            url3,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(
            !tid3.contains('-'),
            "fallback: tenant_id must not have hyphens"
        );
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_matches_connection_format_exactly() {
        // Roundtrip verification: given a standard DB URL, the tenant_id
        // must match EXACTLY what diesel_pool::get_tenant_url() would embed
        // in the Supavisor connection URL for the same workspace.
        //
        // This test duplicates the format from diesel_pool.rs to ensure
        // both sides agree. If either test fails, the contract is broken.

        // Simulate what provision_tenant_db + get_tenant_url would produce
        let ws_id = uuid::Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
        let db_name = format!("tenant_{}", ws_id.to_string().replace('-', ""));
        let provisioned_url = format!("postgres://admin:secret@db_system:5432/{db_name}");

        let (tenant_id, _) = build_tenant_payload(
            &provisioned_url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .unwrap();

        // This is the SAME format as in get_tenant_url() — the contract
        assert_eq!(
            tenant_id, db_name,
            "tenant_id must equal the db_name from provision_tenant_db, \
             which matches the tenant identifier in get_tenant_url()"
        );
        assert_eq!(tenant_id, "tenant_a1b2c3d4e5f67890abcdef1234567890");
        assert_eq!(tenant_id.len(), 39);
        assert!(!tenant_id.contains('-'));
    }

    #[test]
    fn test_build_tenant_payload_tenant_id_format_length_invariant() {
        // The tenant_id for standard workspace DBs is always exactly
        // 39 characters: "tenant_" (7) + 32 hex digits.
        let url = "postgres://u:p@h:5432/tenant_0123456789abcdef0123456789abcdef";
        let (tenant_id, _) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            tenant_id.len(),
            39,
            "tenant_id must be 39 chars, got {}: '{tenant_id}'",
            tenant_id.len()
        );
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

    #[test]
    fn test_build_tenant_payload_includes_default_mode_and_pool_size() {
        // Test that Supavisor payload correctly injects default_mode and default_pool_size
        std::env::set_var("SUPAVISOR_MODE_TYPE", "session");
        std::env::set_var("SUPAVISOR_POOL_SIZE", "42");
        std::env::set_var("SUPAVISOR_TENANT_ID", "test_tenant");

        let url = "postgres://user:pass@host:5432/dbname";
        let (tenant_id, payload) = build_tenant_payload(
            url,
            TenantPayloadOptions {
                is_master: true,
                ..Default::default()
            },
        )
        .expect("Should build payload");

        assert_eq!(tenant_id, "test_tenant");

        let tenant_obj = payload.get("tenant").expect("Missing tenant object");

        assert_eq!(
            tenant_obj.get("require_user").unwrap().as_bool(),
            Some(true)
        );

        // Assert the new edge-case fields are present
        assert_eq!(
            tenant_obj.get("default_pool_size").unwrap().as_u64(),
            Some(42)
        );

        // Cleanup
        std::env::remove_var("SUPAVISOR_MODE_TYPE");
        std::env::remove_var("SUPAVISOR_POOL_SIZE");
        std::env::remove_var("SUPAVISOR_TENANT_ID");
    }

    #[test]
    fn test_parse_postgres_url() {
        // Basic case
        let url = "postgres://user:password@localhost:5432/mydb";
        let parsed = parse_postgres_url(url).unwrap();
        assert_eq!(parsed.0, "user");
        assert_eq!(parsed.1, "password");
        assert_eq!(parsed.2, "localhost");
        assert_eq!(parsed.3, 5432);
        assert_eq!(parsed.4, "mydb");

        // With query params
        let url = "postgres://user:password@localhost:5432/mydb?sslmode=disable";
        let parsed = parse_postgres_url(url).unwrap();
        assert_eq!(parsed.4, "mydb");

        // Unencoded special characters in password (libpq style)
        let url = "postgres://user:k!@#XYZ@viaduct.proxy.net:6543/railway";
        let parsed = parse_postgres_url(url).unwrap();
        assert_eq!(parsed.0, "user");
        assert_eq!(parsed.1, "k!@#XYZ");
        assert_eq!(parsed.2, "viaduct.proxy.net");
        assert_eq!(parsed.3, 6543);
        assert_eq!(parsed.4, "railway");

        // Multiple unencoded @ in password
        let url = "postgres://user:k!@@#@XYZ@viaduct:5432/db";
        let parsed = parse_postgres_url(url).unwrap();
        assert_eq!(parsed.1, "k!@@#@XYZ");

        // Username with dot
        let url = "postgres://postgres.leeattend:master_pass@supavisor:6543/leeattend";
        let parsed = parse_postgres_url(url).unwrap();
        assert_eq!(parsed.0, "postgres.leeattend");
        assert_eq!(parsed.1, "master_pass");

        // Percent-encoded password
        let url = "postgres://user:pass%40word@host:5432/db";
        let parsed = parse_postgres_url(url).unwrap();
        assert_eq!(parsed.1, "pass@word");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_init_tenant_always_updates_even_if_exists() {
        use axum::http::StatusCode;
        use axum::{extract::Path as AxumPath, routing::get, Json, Router};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let get_calls = Arc::new(AtomicUsize::new(0));
        let put_calls = Arc::new(AtomicUsize::new(0));

        let get_calls_clone = get_calls.clone();
        let put_calls_clone = put_calls.clone();

        let app = Router::new().route(
            "/api/tenants/:id",
            get(move |AxumPath(id): AxumPath<String>| {
                get_calls_clone.fetch_add(1, Ordering::SeqCst);
                // Return a fake existing tenant to simulate already exists
                let response = json!({
                    "data": {
                        "db_database": id,
                        "default_mode": "session",
                        "users": []
                    }
                });
                async move { Json(response) }
            })
            .put(
                move |AxumPath(_id): AxumPath<String>, Json(_payload): Json<serde_json::Value>| {
                    put_calls_clone.fetch_add(1, Ordering::SeqCst);
                    async move { StatusCode::OK }
                },
            ),
        );

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // Set environment variables
        let api_url = format!("http://{addr}");
        std::env::set_var("SUPAVISOR_API_URL", &api_url);
        std::env::set_var(
            "API_JWT_SECRET",
            "super_secret_test_key_with_at_least_32_bytes_length",
        );
        std::env::set_var(
            "DATABASE_URL",
            "postgres://postgres.leeattend:master_pass@postgres-host:5432/leeattend",
        );

        let tenant_manager = SupavisorTenantManager::new().expect("Should build manager");
        tenant_manager.init_tenant().await;

        // Verify that BOTH GET and PUT requests were made!
        assert_eq!(
            get_calls.load(Ordering::SeqCst),
            1,
            "Should check if tenant exists"
        );
        assert_eq!(
            put_calls.load(Ordering::SeqCst),
            1,
            "Should always update/PUT the tenant"
        );

        // Clean up env vars
        std::env::remove_var("SUPAVISOR_API_URL");
        std::env::remove_var("API_JWT_SECRET");
        std::env::remove_var("DATABASE_URL");
    }
}
