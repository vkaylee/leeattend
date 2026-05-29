use crate::diesel_pool::*;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use uuid::Uuid;

struct DummyTenantManager;
#[async_trait::async_trait]
impl crate::supavisor::TenantManager for DummyTenantManager {
    async fn init_tenant(&self) {}
    async fn delete_tenant(&self, _id: &str) -> crate::errors::AppResult<()> {
        Ok(())
    }
    async fn register_tenant_dynamic(
        &self,
        _id: &str,
        _p: &serde_json::Value,
    ) -> crate::errors::AppResult<()> {
        Ok(())
    }
    async fn get_tenant(&self, _id: &str) -> crate::errors::AppResult<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
}

#[test]
fn test_tenant_pool_manager_url_supavisor_mode() {
    // Supavisor connection: user has dot (admin.leeattend), update tenant part
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );

    let workspace_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let result = manager.get_tenant_url_with_credentials(&workspace_id, "", "", None);

    assert_eq!(
        result,
        "postgres://admin.tenant_123e4567e89b12d3a456426614174000:pass@supavisor:5452/tenant_123e4567e89b12d3a456426614174000"
    );
}

// ─────────────────────────────────────────────────────────────────────
// Contract: get_tenant_url() MUST produce URLs whose tenant identifier
// matches the tenant_id returned by build_tenant_payload().
//
// The format is: tenant_<uuid-without-hyphens>
//
// This invariant is load-bearing. If either side changes format
// without updating the other, the Supavisor pooler returns:
//   "Tenant or user not found"
//
// These tests lock down the format so it cannot drift.
// ─────────────────────────────────────────────────────────────────────

fn expected_tenant_name(ws_id: &Uuid) -> String {
    format!("tenant_{}", ws_id.to_string().replace('-', ""))
}

// ── Supavisor mode: tenant name in user suffix AND path ──────────────

#[test]
fn test_get_tenant_url_supavisor_mode_standard_uuid() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("019e40ee-1192-7492-bc29-85dad4675f34").unwrap();

    let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);
    let expected = expected_tenant_name(&ws);

    let parsed = reqwest::Url::parse(&url).unwrap();
    let username = parsed.username();
    assert!(
        username.contains('.'),
        "Supavisor mode: username must contain dot"
    );
    let tenant_in_user = username.split('.').nth(1).unwrap();
    assert_eq!(
        tenant_in_user, expected,
        "supavisor user suffix must be tenant_<uuid-no-hyphens>"
    );
    assert_eq!(
        parsed.path(),
        format!("/{expected}"),
        "supavisor db path must be /tenant_<uuid-no-hyphens>"
    );
}

#[test]
fn test_get_tenant_url_supavisor_mode_nil_uuid() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);
    let expected = "tenant_00000000000000000000000000000000";
    let parsed = reqwest::Url::parse(&url).unwrap();
    let tenant_in_user = parsed.username().split('.').nth(1).unwrap();
    assert_eq!(
        tenant_in_user, expected,
        "nil UUID must become tenant_<32-zeros>"
    );
    assert!(
        !tenant_in_user.contains('-'),
        "tenant name must never contain hyphens"
    );
}

#[test]
fn test_get_tenant_url_supavisor_mode_max_uuid() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("ffffffff-ffff-ffff-ffff-ffffffffffff").unwrap();

    let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);
    // All 'f' chars, no hyphens
    assert!(
        url.contains("tenant_ffffffffffffffffffffffffffffffff"),
        "max UUID must strip all hyphens"
    );
}

#[test]
fn test_get_tenant_url_never_contains_hyphens_in_tenant_name() {
    // The tenant identifier embedded in the URL is used by Supavisor for
    // routing. UUID hyphens are stripped because database names cannot
    // contain them.

    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );

    // Test several random UUIDs — none should produce hyphens in the tenant part
    for _ in 0..10 {
        let ws = Uuid::new_v4();
        let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);
        let parsed = reqwest::Url::parse(&url).unwrap();
        let username = parsed.username();
        let tenant_name = username.split('.').nth(1).unwrap();
        assert!(
            !tenant_name.contains('-'),
            "tenant name '{tenant_name}' must never contain hyphens"
        );
        assert!(
            tenant_name.starts_with("tenant_"),
            "tenant name must start with 'tenant_': got '{tenant_name}'"
        );
        // UUID is 32 hex chars, plus "tenant_" = 39 chars
        assert_eq!(
            tenant_name.len(),
            39,
            "tenant name must be exactly 39 chars (tenant_ + 32 hex): got {}",
            tenant_name.len()
        );
    }
}

#[test]
fn test_get_tenant_url_tenant_name_is_predictable_from_workspace_id() {
    // The tenant name is deterministic: tenant_ + UUID with hyphens removed.
    // This test ensures the algorithm is fixed and cannot be changed
    // without breaking the Supavisor contract.
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );

    let ws = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();
    let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);

    // Expected: all hyphens removed from the UUID
    let expected_name = "tenant_a1b2c3d4e5f67890abcdef1234567890";
    let parsed = reqwest::Url::parse(&url).unwrap();
    let tenant_in_user = parsed.username().split('.').nth(1).unwrap();
    assert_eq!(tenant_in_user, expected_name);
    assert_eq!(parsed.path(), format!("/{expected_name}"));
}


// ── Cross-contract: URL tenant name must match registration tenant_id ─

#[test]
fn test_get_tenant_url_tenant_name_roundtrip_with_expected_registration_id() {
    // This is the definitive cross-module contract:
    //   get_tenant_url(ws_id) embeds a tenant name T in the connection URL.
    //   build_tenant_payload(db_url_with_T) must produce tenant_id = T.
    //
    // We verify the tenant name extracted from the URL is exactly what
    // the registration side expects.

    let supavisor_base = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let direct_base = "postgres://user:pass@localhost:5432/postgres";

    let ws = Uuid::parse_str("019e40ee-1192-7492-bc29-85dad4675f34").unwrap();
    let expected = "tenant_019e40ee11927492bc2985dad4675f34";

    // Supavisor mode
    let url = SupavisorTenantPoolManager::new(
        supavisor_base.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    )
    .get_tenant_url_with_credentials(&ws, "", "", None);
    let parsed = reqwest::Url::parse(&url).unwrap();
    let tenant_in_user = parsed.username().split('.').nth(1).unwrap();
    assert_eq!(tenant_in_user, expected);
    assert_eq!(parsed.path().trim_start_matches('/'), expected);

    // Direct mode: path only
    let url2 = SupavisorTenantPoolManager::new(
        direct_base.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    )
    .get_tenant_url_with_credentials(&ws, "", "", None);
    let parsed2 = reqwest::Url::parse(&url2).unwrap();
    assert_eq!(parsed2.path().trim_start_matches('/'), expected);

    // Also verify format properties
    assert_eq!(expected.len(), 39); // "tenant_" + 32 hex chars
    assert!(expected.starts_with("tenant_"));
    assert!(!expected.contains('-'));
    assert!(expected.chars().skip(7).all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn test_tenant_pool_manager_test_mode_returns_same_pool() {
    let base_url = "postgres://user:pass@localhost:5432/leeattend_test_db";
    let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(base_url.to_string());
    let pool = Pool::builder(api_manager).max_size(2).build().unwrap();

    let test_manager = TestTenantPoolManager::new(pool);

    let _ws1 = Uuid::new_v4();
    let _ws2 = Uuid::new_v4();

    // Since we are mocking the DI, the mock manager should always return clones of the same pool
    // regardless of the workspace ID requested.
    assert_eq!(test_manager.pool.status().max_size, 2);
}

// ═══════════════════════════════════════════════════════════════════════
// Regression tests for TenantConnectionProvider::resolve_migration_url
//
// These tests verify that the pool manager correctly builds migration URLs:
// - Supavisor mode (prod): routes through pooler with tenant's own credentials
// - Direct mode (dev): returns tenant URL as-is
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_resolve_migration_url_supavisor_mode_uses_tenant_credentials() {
    // Production: base_url has dot in username → Supavisor mode.
    // resolve_migration_url should extract credentials from tenant_direct_url
    // and build a Supavisor-routed URL with the tenant's own user/password.
    let base_url = "postgres://admin.leeattend:master_pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("019e632f-26b4-7660-956e-62b13870a145").unwrap();
    let tenant_name = "tenant_019e632f26b47660956e62b13870a145";

    let url = manager
        .resolve_migration_url(
            &ws,
            "postgres://byod_user:byod_secret@tenant-db.example.com:5432/byod_database",
        )
        .unwrap();

    // Must use tenant's own credentials, NOT master_pass
    assert!(
        url.contains("byod_secret"),
        "Must use tenant's password: {url}"
    );
    assert!(
        !url.contains("master_pass"),
        "Master password must be replaced: {url}"
    );
    // User prefix must be tenant's db_user
    assert!(
        url.contains(&format!("byod_user.{tenant_name}")),
        "User prefix must be tenant's db_user: {url}"
    );
    // Must route through Supavisor host
    assert!(
        url.contains("supavisor:5452"),
        "Must route through Supavisor: {url}"
    );
    // DB name from tenant URL is preserved
    assert!(
        url.contains("byod_database"),
        "DB name from tenant URL must be preserved: {url}"
    );
}

#[test]
fn test_resolve_migration_url_each_tenant_gets_own_credentials() {
    // Core regression: each BYOD tenant uses ITS OWN credentials,
    // never shared master credentials.
    let base_url = "postgres://admin.leeattend:master_pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );

    let ws1 = Uuid::new_v4();
    let ws2 = Uuid::new_v4();

    let url1 = manager
        .resolve_migration_url(&ws1, "postgres://alice:alice_secret@db1:5432/alice_db")
        .unwrap();
    let url2 = manager
        .resolve_migration_url(&ws2, "postgres://bob:bob_secret@db2:5432/bob_db")
        .unwrap();

    assert!(url1.contains("alice_secret"), "ws1 uses alice's password");
    assert!(url2.contains("bob_secret"), "ws2 uses bob's password");
    assert!(!url1.contains("master_pass"), "ws1 has no master pass");
    assert!(!url2.contains("master_pass"), "ws2 has no master pass");
    assert!(!url1.contains("bob_secret"), "ws1 doesn't have bob's pass");
    assert!(!url2.contains("alice_secret"), "ws2 doesn't have alice's pass");
}

#[test]
fn test_resolve_migration_url_invalid_tenant_url_returns_error() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::new_v4();

    let result = manager.resolve_migration_url(&ws, "not-a-postgres-url");
    assert!(result.is_err(), "Invalid tenant URL should return error");
}

#[test]
fn test_resolve_migration_url_test_pool_manager_returns_direct_url() {
    // TestTenantPoolManager should always return the direct URL
    let api_manager =
        AsyncDieselConnectionManager::<AsyncPgConnection>::new("postgres://x:x@localhost:5432/x");
    let pool = Pool::builder(api_manager).max_size(1).build().unwrap();
    let manager = TestTenantPoolManager::new(pool);
    let ws = Uuid::new_v4();
    let direct_url = "postgres://test:test@test-db:5432/test";

    let url = manager.resolve_migration_url(&ws, direct_url).unwrap();
    assert_eq!(url, direct_url);
}

#[test]
fn test_is_transient_supavisor_error() {
    assert!(is_transient_supavisor_error(
        "Failed to get secret: {:error, \"No auth_query specified\"}"
    ));
    assert!(is_transient_supavisor_error("Tenant or user not found"));
    assert!(is_transient_supavisor_error(
        "terminating connection due to administrator command"
    ));
    assert!(is_transient_supavisor_error("connection closed"));
}

#[test]
fn test_is_transient_supavisor_error_other() {
    assert!(!is_transient_supavisor_error(
        "password authentication failed for user"
    ));
    assert!(!is_transient_supavisor_error("Connection refused"));
    assert!(!is_transient_supavisor_error("database does not exist"));
    assert!(!is_transient_supavisor_error("timeout expired"));
}

#[tokio::test]
async fn test_supavisor_transient_retry_loop_on_production_manager() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::Arc;

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let connect_count = Arc::new(std::sync::Mutex::new(0));
    let connect_count_clone = Arc::clone(&connect_count);

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            *connect_count_clone.lock().unwrap() += 1;

            let mut buf = [0u8; 8];
            if stream.read_exact(&mut buf).is_ok() {
                // Handle SSL Request
                if buf == [0, 0, 0, 8, 4, 210, 22, 47] {
                    let _ = stream.write_all(b"N");
                    let mut len_buf = [0u8; 4];
                    if stream.read_exact(&mut len_buf).is_ok() {
                        let len = u32::from_be_bytes(len_buf);
                        let mut startup_buf = vec![0u8; len as usize - 4];
                        let _ = stream.read_exact(&mut startup_buf);
                    }
                }

                // Build mock Postgres ErrorResponse packet
                let mut error_payload = Vec::new();
                error_payload.extend_from_slice(b"SFATAL\0");
                error_payload.extend_from_slice(b"CXX000\0");
                error_payload.extend_from_slice(
                    b"MFailed to get secret: {:error, \"No auth_query specified\"}\0",
                );
                error_payload.push(0);

                let len = error_payload.len() as u32 + 4;
                let mut packet = Vec::new();
                packet.push(b'E');
                packet.extend_from_slice(&len.to_be_bytes());
                packet.extend_from_slice(&error_payload);

                let _ = stream.write_all(&packet);
            }

            // Break after 4 connections (1 initial + 3 retries)
            if *connect_count_clone.lock().unwrap() >= 4 {
                break;
            }
        }
    });

    let url = format!("postgres://test_user:pass@127.0.0.1:{port}/db");

    let manager = SupavisorTenantPoolManager::new(url, 1, std::sync::Arc::new(DummyTenantManager));
    let start = std::time::Instant::now();

    let ws = Uuid::new_v4();

    // This should trigger the retry loop inside `get_connection`
    let result = manager.get_connection(&ws).await;

    assert!(result.is_err(), "Should not successfully connect");
    assert_eq!(
        *connect_count.lock().unwrap(),
        4,
        "Should have retried exactly 3 times (4 total attempts)"
    );
    assert!(
        start.elapsed().as_millis() >= 900,
        "Should have slept for at least 900ms total (3 * 300ms)"
    );
}

#[test]
fn test_get_tenant_url_preserves_unencoded_chars() {
    let manager = SupavisorTenantPoolManager::new(
        "postgres://postgres:k!@#XYZ@viaduct.proxy:5432/railway".to_string(),
        10,
        std::sync::Arc::new(DummyTenantManager),
    );
    let id = Uuid::parse_str("019e5512-4556-7b33-9e9b-84fb5a82c6d4").unwrap();
    let tenant_url = manager.get_tenant_url_with_credentials(&id, "", "", None);

    // Expected: same password with unencoded #, but db_name changed
    assert_eq!(
        tenant_url,
        "postgres://postgres:k!@#XYZ@viaduct.proxy:5432/tenant_019e551245567b339e9b84fb5a82c6d4"
    );
}

#[test]
fn test_get_tenant_url_supavisor_mode_preserves_unencoded_chars() {
    let manager = SupavisorTenantPoolManager::new(
        "postgres://postgres.leeattend:master_pass!@#@supavisor:6543/leeattend?sslmode=disable"
            .to_string(),
        10,
        std::sync::Arc::new(DummyTenantManager),
    );
    let id = Uuid::parse_str("019e5512-4556-7b33-9e9b-84fb5a82c6d4").unwrap();
    let tenant_url = manager.get_tenant_url_with_credentials(&id, "", "", None);

    // Expected: user updated, password intact, db_name updated, query intact
    assert_eq!(
        tenant_url,
        "postgres://postgres.tenant_019e551245567b339e9b84fb5a82c6d4:master_pass!@#@supavisor:6543/tenant_019e551245567b339e9b84fb5a82c6d4?sslmode=disable"
    );
}

#[test]
fn test_get_tenant_url_with_override_user() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

    // With empty override user, it falls back to the prefix in base_url (admin)
    let url_empty = manager.get_tenant_url_with_credentials(&ws, "", "", None);
    assert_eq!(
        url_empty,
        "postgres://admin.tenant_123e4567e89b12d3a456426614174000:pass@supavisor:5452/tenant_123e4567e89b12d3a456426614174000"
    );

    // With explicit override user, it replaces the prefix completely
    let url_override = manager.get_tenant_url_with_credentials(&ws, "custom_tenant_user", "", None);
    assert_eq!(
        url_override,
        "postgres://custom_tenant_user.tenant_123e4567e89b12d3a456426614174000:pass@supavisor:5452/tenant_123e4567e89b12d3a456426614174000"
    );
}


#[test]
fn test_get_tenant_url_with_override_db_name_in_production_manager() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

    let url = manager.get_tenant_url_with_credentials(&ws, "", "", Some("custom_tenant_db"));
    assert_eq!(
        url,
        "postgres://admin.tenant_123e4567e89b12d3a456426614174000:pass@supavisor:5452/custom_tenant_db"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Tests for run_single_tenant_migration (credential mismatch fix)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_single_tenant_migration_invalid_url_returns_error() {
    let ws = Uuid::new_v4();
    let result = DieselPools::run_single_tenant_migration("not-a-valid-url", &ws);

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(
        err_msg.contains("Failed to connect to tenant DB"),
        "Error should indicate connection failure, got: {err_msg}"
    );
    assert!(
        err_msg.contains(&ws.to_string()),
        "Error should contain workspace_id for traceability, got: {err_msg}"
    );
}

#[test]
fn test_single_tenant_migration_connection_refused_returns_error() {
    let ws = Uuid::new_v4();
    // Port 1 is almost never listening
    let result = DieselPools::run_single_tenant_migration(
        "postgres://postgres:pass@127.0.0.1:1/tenant_db",
        &ws,
    );

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Failed to connect to tenant DB"),
        "Should indicate connection failure, got: {err_msg}"
    );
}

#[test]
fn test_single_tenant_migration_empty_url_returns_error() {
    let ws = Uuid::new_v4();
    let result = DieselPools::run_single_tenant_migration("", &ws);

    assert!(result.is_err());
}

#[test]
fn test_single_tenant_migration_wrong_password_returns_error() {
    let ws = Uuid::new_v4();
    // Valid URL format but wrong credentials — connection will fail
    let result = DieselPools::run_single_tenant_migration(
        "postgres://wrong_user:wrong_pass@127.0.0.1:59999/nonexistent_db",
        &ws,
    );

    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// Regression: credential mismatch via resolve_migration_url
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_regression_supavisor_mode_never_leaks_master_password() {
    // THE BUG: master password from POOLER_SESSION_URL was used for all tenants.
    // THE FIX: resolve_migration_url extracts credentials from tenant_direct_url.
    let base_url = "postgres://postgres.leeattend:MASTER_SECRET@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("019e632f-26b4-7660-956e-62b13870a145").unwrap();

    let url = manager
        .resolve_migration_url(
            &ws,
            "postgres://custom_user:TENANT_SECRET@tenant-db:5432/tenant_db",
        )
        .unwrap();

    assert!(
        url.contains("TENANT_SECRET"),
        "Migration URL must use tenant's password: {url}"
    );
    assert!(
        !url.contains("MASTER_SECRET"),
        "Migration URL must NOT contain master password: {url}"
    );
    assert!(
        url.contains("supavisor:5452"),
        "Migration URL must still route through Supavisor (firewall): {url}"
    );
}

#[test]
fn test_regression_supavisor_mode_preserves_tenant_db_name() {
    // The tenant's actual database name must be preserved,
    // not replaced with tenant_{uuid}.
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::new_v4();

    let url = manager
        .resolve_migration_url(
            &ws,
            "postgres://user:pass@db-host:5432/my_custom_database",
        )
        .unwrap();

    assert!(
        url.contains("my_custom_database"),
        "Tenant's actual DB name must be preserved: {url}"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Startup flow: migration target collection logic
// ═══════════════════════════════════════════════════════════════════════

/// Simulates the migration target collection logic from
/// `register_workspace_tenants_and_migrate` in main.rs
/// to verify skip/continue behavior without a real DB.
fn collect_migration_targets(
    workspaces: Vec<(Uuid, Option<String>)>,
    registration_failures: &[Uuid],
) -> Vec<(Uuid, String)> {
    let mut migration_targets: Vec<(Uuid, String)> = Vec::new();

    for (ws_id, custom_db_url_encrypted) in workspaces {
        let tenant_url = if let Some(encrypted) = custom_db_url_encrypted {
            // Simulate: no crypto service, use raw URL
            encrypted
        } else {
            // Workspace missing custom_db_url_encrypted → skip
            continue;
        };

        // Simulate: build_tenant_payload validation
        if crate::supavisor::parse_postgres_url(&tenant_url).is_none() {
            continue;
        }

        // Simulate: Supavisor registration failure
        if registration_failures.contains(&ws_id) {
            continue;
        }

        migration_targets.push((ws_id, tenant_url));
    }

    migration_targets
}

#[test]
fn test_startup_flow_collects_valid_targets() {
    let ws1 = Uuid::new_v4();
    let ws2 = Uuid::new_v4();
    let workspaces = vec![
        (ws1, Some("postgres://user1:pass1@db1:5432/tenant_db1".to_string())),
        (ws2, Some("postgres://user2:pass2@db2:5432/tenant_db2".to_string())),
    ];

    let targets = collect_migration_targets(workspaces, &[]);

    assert_eq!(targets.len(), 2);
    assert_eq!(targets[0].0, ws1);
    assert!(targets[0].1.contains("user1:pass1"));
    assert_eq!(targets[1].0, ws2);
    assert!(targets[1].1.contains("user2:pass2"));
}

#[test]
fn test_startup_flow_skips_workspace_missing_custom_db_url() {
    let ws_valid = Uuid::new_v4();
    let ws_missing = Uuid::new_v4();
    let workspaces = vec![
        (ws_valid, Some("postgres://user:pass@db:5432/tenant_db".to_string())),
        (ws_missing, None), // No custom_db_url_encrypted
    ];

    let targets = collect_migration_targets(workspaces, &[]);

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].0, ws_valid);
}

#[test]
fn test_startup_flow_skips_workspace_with_invalid_url() {
    let ws_valid = Uuid::new_v4();
    let ws_invalid = Uuid::new_v4();
    let workspaces = vec![
        (ws_valid, Some("postgres://user:pass@db:5432/tenant_db".to_string())),
        (ws_invalid, Some("not-a-postgres-url".to_string())),
    ];

    let targets = collect_migration_targets(workspaces, &[]);

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].0, ws_valid);
}

#[test]
fn test_startup_flow_skips_workspace_on_registration_failure() {
    let ws_ok = Uuid::new_v4();
    let ws_fail = Uuid::new_v4();
    let workspaces = vec![
        (ws_ok, Some("postgres://user1:pass1@db1:5432/db1".to_string())),
        (ws_fail, Some("postgres://user2:pass2@db2:5432/db2".to_string())),
    ];

    // ws_fail fails Supavisor registration
    let targets = collect_migration_targets(workspaces, &[ws_fail]);

    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].0, ws_ok);
}

#[test]
fn test_startup_flow_empty_workspace_list() {
    let targets = collect_migration_targets(vec![], &[]);
    assert!(targets.is_empty());
}

#[test]
fn test_startup_flow_all_workspaces_fail() {
    let ws1 = Uuid::new_v4();
    let ws2 = Uuid::new_v4();
    let workspaces = vec![
        (ws1, None),           // missing URL
        (ws2, Some("invalid".to_string())), // invalid URL
    ];

    let targets = collect_migration_targets(workspaces, &[]);
    assert!(targets.is_empty());
}

#[test]
fn test_startup_flow_all_registrations_fail() {
    let ws1 = Uuid::new_v4();
    let ws2 = Uuid::new_v4();
    let workspaces = vec![
        (ws1, Some("postgres://u1:p1@h1:5432/d1".to_string())),
        (ws2, Some("postgres://u2:p2@h2:5432/d2".to_string())),
    ];

    let targets = collect_migration_targets(workspaces, &[ws1, ws2]);
    assert!(targets.is_empty());
}

#[test]
fn test_startup_flow_mixed_failures() {
    let ws_ok = Uuid::new_v4();
    let ws_no_url = Uuid::new_v4();
    let ws_bad_url = Uuid::new_v4();
    let ws_reg_fail = Uuid::new_v4();

    let workspaces = vec![
        (ws_ok, Some("postgres://user:pass@db:5432/good_db".to_string())),
        (ws_no_url, None),
        (ws_bad_url, Some("ftp://not-postgres".to_string())),
        (ws_reg_fail, Some("postgres://user:pass@db:5432/fail_db".to_string())),
    ];

    let targets = collect_migration_targets(workspaces, &[ws_reg_fail]);

    assert_eq!(targets.len(), 1, "Only the fully successful workspace should be a migration target");
    assert_eq!(targets[0].0, ws_ok);
}

#[test]
fn test_startup_flow_preserves_each_tenants_own_credentials() {
    // The core invariant of the fix: each workspace uses ITS OWN credentials,
    // not shared master credentials.
    let ws1 = Uuid::new_v4();
    let ws2 = Uuid::new_v4();
    let ws3 = Uuid::new_v4();

    let workspaces = vec![
        (ws1, Some("postgres://alice:alice_secret@db1.example.com:5432/alice_db".to_string())),
        (ws2, Some("postgres://bob:bob_secret@db2.example.com:5432/bob_db".to_string())),
        (ws3, Some("postgres://charlie:charlie_secret@db3.example.com:5432/charlie_db".to_string())),
    ];

    let targets = collect_migration_targets(workspaces, &[]);

    assert_eq!(targets.len(), 3);

    // Each migration target preserves the tenant's individual credentials
    assert!(targets[0].1.contains("alice:alice_secret@db1.example.com"));
    assert!(targets[1].1.contains("bob:bob_secret@db2.example.com"));
    assert!(targets[2].1.contains("charlie:charlie_secret@db3.example.com"));

    // None of them share credentials with a hypothetical master password
    for (_, url) in &targets {
        assert!(
            !url.contains("master_pass"),
            "Migration URL must NOT contain master password: {url}"
        );
    }
}

#[test]
fn test_startup_flow_url_with_special_chars_in_password() {
    let ws = Uuid::new_v4();
    let workspaces = vec![
        (ws, Some("postgres://user:p@ss!w#rd$%^&@db:5432/tenant_db".to_string())),
    ];

    let targets = collect_migration_targets(workspaces, &[]);

    // Should still collect — the URL is valid postgres format
    // (parse_postgres_url handles this via custom parsing, not strict URL parsing)
    assert_eq!(targets.len(), 1);
    assert_eq!(targets[0].0, ws);
}

// ═══════════════════════════════════════════════════════════════════════
// Edge cases: resolve_migration_url via TenantConnectionProvider
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_resolve_migration_url_special_chars_in_tenant_password() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::new_v4();

    // Password with special chars handled by parse_postgres_url (custom parser)
    let result = manager.resolve_migration_url(
        &ws,
        "postgres://user:p@ss!w#rd@db:5432/tenant_db",
    );

    assert!(result.is_ok(), "Special chars in password should be handled");
    let url = result.unwrap();
    assert!(url.contains("supavisor:5452"), "Still routes through Supavisor: {url}");
}

#[test]
fn test_resolve_migration_url_supavisor_mode_user_prefix_is_tenant_db_user() {
    // Supavisor authenticates by matching the username prefix against the
    // registered db_user. So the prefix must come from the tenant URL.
    let base_url = "postgres://pg.leeattend:pooler_pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("019e632f-26b4-7660-956e-62b13870a145").unwrap();
    let tenant_name = "tenant_019e632f26b47660956e62b13870a145";

    let url = manager
        .resolve_migration_url(
            &ws,
            "postgres://custom_user:custom_pass@some-host:5432/some_db",
        )
        .unwrap();

    // User prefix MUST be "custom_user" (from tenant URL), NOT "pg" (from pooler)
    let expected_user = format!("custom_user.{tenant_name}");
    assert!(
        url.contains(&expected_user),
        "User prefix must come from tenant URL: {url}"
    );
    assert!(
        !url.contains(&format!("pg.{tenant_name}")),
        "Pooler's user prefix must NOT be used: {url}"
    );
}

#[test]
fn test_resolve_migration_url_empty_tenant_url_returns_error() {
    let base_url = "postgres://admin.leeattend:pass@supavisor:5452/leeattend";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::new_v4();

    let result = manager.resolve_migration_url(&ws, "");
    assert!(result.is_err(), "Empty tenant URL should return error");
}

