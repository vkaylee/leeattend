//! Integration tests for system tenant endpoints.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn assign_system_role(
    pool: &apiserver::diesel_pool::DieselPools,
    user_id: Uuid,
    role_name: &str,
) {
    crate::test_execute!(
        &pool.api,
        "INSERT INTO user_system_roles (user_id, role_id) \
         SELECT $1, id FROM system_roles WHERE name = $2",
        diesel::sql_types::Uuid,
        user_id,
        diesel::sql_types::Text,
        role_name
    );
}

fn make_req(
    method: &str,
    uri: &str,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let req = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json");

    if let Some(b) = body {
        req.body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        req.body(axum::body::Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_system_tenants_lifecycle() {
    let supavisor_url = match std::env::var("SUPAVISOR_API_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping system_tenants test because SUPAVISOR_API_URL is not set.");
            return;
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();

    // Just try to hit the host to see if it's reachable.
    // If it's not reachable, it'll timeout/fail fast and we skip.
    if client.get(&supavisor_url).send().await.is_err() {
        println!(
            "Skipping system_tenants test because Supavisor is not reachable at {}.",
            supavisor_url
        );
        return;
    }

    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let regular_id = factory.user().create().await;

    assign_system_role(&factory.diesel_pools, admin_id, "platform_admin").await;

    // 1. Regular user cannot access GET /api/v1/system/tenants
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/tenants", regular_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // 2. Admin CAN access GET
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/tenants", admin_id, None))
        .await
        .unwrap();
    let status = res.status();
    if status != StatusCode::OK {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        println!(
            "GET /api/system/tenants failed: {:?}",
            String::from_utf8(bytes.to_vec())
        );
    }
    assert_eq!(status, StatusCode::OK);

    let workspace_id = Uuid::new_v4();
    let payload = json!({
        "workspace_id": workspace_id,
        "database_url": "postgres://tenant_test_user:tenant_test_password@db_workspace:5432/tenant_test_db",
        "pool_size": 15
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            "/api/system/tenants",
            admin_id,
            Some(payload),
        ))
        .await
        .unwrap();
    let status = res.status();
    if status != StatusCode::CREATED {
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        std::fs::write(
            "system_tenants_error.log",
            String::from_utf8(bytes.to_vec()).unwrap_or_default(),
        )
        .ok();
        println!(
            "POST /api/system/tenants failed: {:?}",
            String::from_utf8(bytes.to_vec())
        );
    }
    assert_eq!(status, StatusCode::CREATED);

    // 4. Verify tenant is in GET list
    let res = app
        .clone()
        .oneshot(make_req("GET", "/api/system/tenants", admin_id, None))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // (Optional: parse and assert body includes "test_database")

    // 5. Verify system_audit_logs recorded the creation
    let count: i64 = crate::test_query_scalar_i64!(
        &factory.diesel_pools.api,
        "SELECT count(*) AS count FROM system_audit_logs WHERE action = 'tenant.create'"
    );
    assert_eq!(count, 1);

    // 6. Delete the tenant
    let res = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/system/tenants/{}", workspace_id),
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 7. Verify system_audit_logs recorded the deletion
    let count: i64 = crate::test_query_scalar_i64!(
        &factory.diesel_pools.api,
        "SELECT count(*) AS count FROM system_audit_logs WHERE action = 'tenant.delete'"
    );
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_system_tenant_startup_registration() {
    use apiserver::supavisor::TenantManager;

    let supavisor_url = match std::env::var("SUPAVISOR_API_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping system_tenant_startup_registration test because SUPAVISOR_API_URL is not set.");
            return;
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();

    if client.get(&supavisor_url).send().await.is_err() {
        println!(
            "Skipping system_tenant_startup_registration test because Supavisor is not reachable at {}.",
            supavisor_url
        );
        return;
    }

    // 1. Create a dynamic test database
    let test_db = TestDatabase::new().await;
    let db_name = test_db.db_name().to_string();
    let test_db_url = test_db.url();

    // 2. Temporarily back up and override environment variables
    let orig_db_url = std::env::var("DATABASE_URL").ok();
    let orig_tenant_id = std::env::var("SUPAVISOR_TENANT_ID").ok();

    std::env::set_var("DATABASE_URL", &test_db_url);
    std::env::remove_var("SUPAVISOR_TENANT_ID");

    // 3. Construct a real SupavisorTenantManager
    let tenant_manager = apiserver::supavisor::SupavisorTenantManager::new()
        .expect("Failed to build SupavisorTenantManager");

    // 4. Run the startup initialization flow (First time)
    tenant_manager.init_tenant().await;

    // 5. Verify the tenant was registered successfully in Supavisor
    let get_res = tenant_manager.get_tenant(&db_name).await;
    assert!(
        get_res.is_ok(),
        "Failed to retrieve registered tenant from Supavisor: {:?}",
        get_res.err()
    );
    let payload = get_res.unwrap();
    let tenant_obj = if let Some(data) = payload.get("data") {
        data
    } else if let Some(tenant) = payload.get("tenant") {
        tenant
    } else {
        &payload
    };
    assert_eq!(
        tenant_obj["db_database"].as_str().unwrap(),
        db_name,
        "Registered db_database name does not match expected database name"
    );

    // 6. Run the startup initialization flow AGAIN (Second time)
    // This should hit the GET check and gracefully skip the PUT without error.
    tenant_manager.init_tenant().await;

    // Verify it still exists and is accessible
    let get_res_second = tenant_manager.get_tenant(&db_name).await;
    assert!(
        get_res_second.is_ok(),
        "Tenant should still be accessible after second init_tenant call"
    );

    // 7. Clean up: Delete the registered tenant from Supavisor
    let del_res = tenant_manager.delete_tenant(&db_name).await;
    assert!(
        del_res.is_ok(),
        "Failed to clean up registered tenant from Supavisor: {:?}",
        del_res.err()
    );

    // 8. Restore original environment variables
    if let Some(url) = orig_db_url {
        std::env::set_var("DATABASE_URL", url);
    } else {
        std::env::remove_var("DATABASE_URL");
    }

    if let Some(tid) = orig_tenant_id {
        std::env::set_var("SUPAVISOR_TENANT_ID", tid);
    }
}

#[tokio::test]
async fn test_dynamic_tenant_registration_idempotency() {
    use apiserver::supavisor::TenantManager;

    let supavisor_url = match std::env::var("SUPAVISOR_API_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Skipping test_dynamic_tenant_registration_idempotency test because SUPAVISOR_API_URL is not set.");
            return;
        }
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();

    if client.get(&supavisor_url).send().await.is_err() {
        println!(
            "Skipping test_dynamic_tenant_registration_idempotency test because Supavisor is not reachable at {}.",
            supavisor_url
        );
        return;
    }

    // 1. Create a dynamic test database so Supavisor's connection test passes
    let test_db = crate::common::db::TestDatabase::new().await;
    let tenant_id = format!("tenant_{}", Uuid::new_v4().simple());
    let db_name = test_db.db_name().to_string();

    // 2. Construct a real SupavisorTenantManager
    let tenant_manager = apiserver::supavisor::SupavisorTenantManager::new()
        .expect("Failed to build SupavisorTenantManager");

    let db_url = test_db.url();
    let without_scheme = db_url.strip_prefix("postgres://").unwrap_or(&db_url);
    let credentials = without_scheme.split('@').next().unwrap_or("");
    let mut creds_parts = credentials.split(':');
    let db_user = creds_parts.next().unwrap_or("test_user").to_string();
    let db_password = creds_parts.next().unwrap_or("test_password").to_string();

    // 3. Build a dummy payload using the real database name and real user credentials
    let payload = json!({
        "tenant": {
            "db_host": "db_system",
            "db_port": 5432,
            "db_database": db_name,
            "ip_version": "auto",
            "enforce_ssl": false,
            "default_pool_size": 20,
            "default_mode": "session",
            "require_user": true,
            "users": [
                {
                    "db_user": db_user,
                    "db_password": db_password,
                    "pool_size": 20,
                    "mode_type": "session",
                    "is_manager": true
                }
            ]
        }
    });

    // 4. Run the dynamic registration (First time)
    let res1 = tenant_manager.register_tenant_dynamic(&tenant_id, &payload).await;
    assert!(
        res1.is_ok(),
        "First dynamic registration failed: {:?}",
        res1.err()
    );

    // 5. Run the dynamic registration AGAIN (Second time)
    // This should hit the GET check and gracefully skip the PUT without error.
    let res2 = tenant_manager.register_tenant_dynamic(&tenant_id, &payload).await;
    assert!(
        res2.is_ok(),
        "Second dynamic registration failed: {:?}",
        res2.err()
    );

    // 6. Verify it still exists and is accessible
    let get_res = tenant_manager.get_tenant(&tenant_id).await;
    assert!(
        get_res.is_ok(),
        "Tenant should be accessible after dynamic registration"
    );

    let fetched_payload = get_res.unwrap();
    let tenant_obj = if let Some(data) = fetched_payload.get("data") {
        data
    } else if let Some(tenant) = fetched_payload.get("tenant") {
        tenant
    } else {
        &fetched_payload
    };
    assert_eq!(
        tenant_obj["db_database"].as_str().unwrap(),
        db_name,
        "Registered db_database name does not match expected database name"
    );

    // 7. Clean up: Delete the registered tenant from Supavisor
    let del_res = tenant_manager.delete_tenant(&tenant_id).await;
    assert!(
        del_res.is_ok(),
        "Failed to clean up registered dynamic tenant from Supavisor: {:?}",
        del_res.err()
    );
}

