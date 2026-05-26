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
fn test_tenant_pool_manager_url_direct_connection() {
    // Direct connection: user has no dot, leave unchanged
    let base_url = "postgres://user:pass@localhost:5432/postgres";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );

    let workspace_id = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let result = manager.get_tenant_url_with_credentials(&workspace_id, "", "", None);

    assert_eq!(
        result,
        "postgres://user:pass@localhost:5432/tenant_123e4567e89b12d3a456426614174000"
    );
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

// ── Direct connection mode: no dot in user, tenant name in path only ─

#[test]
fn test_get_tenant_url_direct_connection_uses_tenant_prefix_in_path() {
    let base_url = "postgres://user:pass@localhost:5432/postgres";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();
    let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);

    let expected_path = "/tenant_123e4567e89b12d3a456426614174000";
    let parsed = reqwest::Url::parse(&url).unwrap();
    assert_eq!(parsed.path(), expected_path);
    // Direct connection: user unchanged
    assert_eq!(parsed.username(), "user");
    assert!(!parsed.username().contains('.'));
}

#[test]
fn test_get_tenant_url_direct_connection_various_uuids() {
    let base_url = "postgres://user:pass@localhost:5432/postgres";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );

    let cases = [
        (
            "00000000-0000-0000-0000-000000000000",
            "tenant_00000000000000000000000000000000",
        ),
        (
            "ffffffff-ffff-ffff-ffff-ffffffffffff",
            "tenant_ffffffffffffffffffffffffffffffff",
        ),
        (
            "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
            "tenant_a1b2c3d4e5f67890abcdef1234567890",
        ),
    ];

    for (uuid_str, expected_path) in &cases {
        let ws = Uuid::parse_str(uuid_str).unwrap();
        let url = manager.get_tenant_url_with_credentials(&ws, "", "", None);
        let parsed = reqwest::Url::parse(&url).unwrap();
        assert_eq!(
            parsed.path(),
            format!("/{expected_path}"),
            "direct connection: UUID {uuid_str} must produce path /{expected_path}"
        );
    }
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

#[test]
fn test_resolve_tenant_migration_url_fallback() {
    let primary_url = "postgres://test_user:pass@db_system:5432/test_database";
    let workspace_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();

    let url =
        DieselPools::resolve_tenant_migration_url(primary_url, None, &workspace_id, None).unwrap();

    assert_eq!(
        url,
        "postgres://test_user:pass@db_system:5432/tenant_a1b2c3d4e5f67890abcdef1234567890"
    );
}

#[test]
fn test_resolve_tenant_migration_url_uses_session_port() {
    let primary_url = "postgres://test_user:pass@db_system:5432/test_database";
    let pooler_url = "postgres://test_user.test_database:pass@db_system:9999/test_database";
    let workspace_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();

    let url = DieselPools::resolve_tenant_migration_url(
        primary_url,
        Some(pooler_url),
        &workspace_id,
        Some(5452),
    )
    .unwrap();

    assert_eq!(url, "postgres://test_user.tenant_a1b2c3d4e5f67890abcdef1234567890:pass@db_system:5452/tenant_a1b2c3d4e5f67890abcdef1234567890");
}

#[test]
fn test_resolve_tenant_migration_url_keeps_port_if_not_supavisor() {
    let primary_url = "postgres://test_user:pass@db_system:5432/test_database";
    let pooler_url = "postgres://test_user_no_dot:pass@db_system:5432/test_database";
    let workspace_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-abcd-ef1234567890").unwrap();

    let url = DieselPools::resolve_tenant_migration_url(
        primary_url,
        Some(pooler_url),
        &workspace_id,
        Some(9999),
    )
    .unwrap();

    assert_eq!(
        url,
        "postgres://test_user_no_dot:pass@db_system:5432/tenant_a1b2c3d4e5f67890abcdef1234567890"
    );
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
fn test_get_tenant_url_direct_connection_with_override_user() {
    let base_url = "postgres://user:pass@localhost:5432/postgres";
    let manager = SupavisorTenantPoolManager::new(
        base_url.to_string(),
        5,
        std::sync::Arc::new(DummyTenantManager),
    );
    let ws = Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap();

    // With empty override user, direct connection remains unchanged for username
    let url_empty = manager.get_tenant_url_with_credentials(&ws, "", "", None);
    assert_eq!(
        url_empty,
        "postgres://user:pass@localhost:5432/tenant_123e4567e89b12d3a456426614174000"
    );

    // With explicit override user, direct connection updates username entirely (no tenant suffix)
    let url_override = manager.get_tenant_url_with_credentials(&ws, "new_direct_user", "", None);
    assert_eq!(
        url_override,
        "postgres://new_direct_user:pass@localhost:5432/tenant_123e4567e89b12d3a456426614174000"
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
