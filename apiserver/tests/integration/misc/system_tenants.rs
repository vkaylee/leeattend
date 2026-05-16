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
    // Only run if pooler is enabled
    if std::env::var("SUPAVISOR_API_URL").is_err() {
        println!("Skipping system_tenants test because SUPAVISOR_API_URL is not set.");
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

    // 3. Admin creates a tenant without creating physical DB (using test_database)
    let payload = json!({
        "db_name": "test_database",
        "create_db": false,
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
            "/api/system/tenants/test_database",
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
