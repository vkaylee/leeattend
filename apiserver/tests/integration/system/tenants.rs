use crate::common::app::spawn_app_with_factory;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_api_lifecycle() {
    let (address, _workspace_id, user_id, factory, test_db) = spawn_app_with_factory().await;
    let test_db_url = test_db.url();

    // Assign PlatformAdmin role
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid,
        user_id
    );

    let client = reqwest::Client::new();

    // 1. List tenants (initially empty or has test workspace)
    let list_res = client
        .get(&format!("{address}/api/v1/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    // 2. Create a BYOD tenant (with test database URL)
    let create_res = client
        .post(&format!("{address}/api/v1/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({
            "name": "Test Tenant API",
            "database_url": test_db_url
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);
    let created: serde_json::Value = create_res.json().await.unwrap();
    assert_eq!(created["name"], "Test Tenant API");
    assert_eq!(created["is_byod"], true);

    let tenant_id: Uuid = created["id"].as_str().unwrap().parse().unwrap();

    // 3. List again — should include new tenant
    let list_res2 = client
        .get(&format!("{address}/api/v1/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(list_res2.status(), StatusCode::OK);
    let list2: Vec<serde_json::Value> = list_res2.json().await.unwrap();
    assert!(list2.iter().any(|t| t["id"] == created["id"]));

    // 4. Get tenant stats
    let stats_res = client
        .get(&format!(
            "{address}/api/v1/system/tenants/{tenant_id}/stats"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(stats_res.status(), StatusCode::OK);
    let stats: serde_json::Value = stats_res.json().await.unwrap();
    assert_eq!(stats["name"], "Test Tenant API");
    assert_eq!(stats["is_byod"], serde_json::json!(true));
    assert!(stats["connection"]["host"].as_str().is_some());
    assert!(stats["connection"]["port"].as_str().is_some());

    // 5. Update tenant database URL
    let update_res = client
        .put(&format!(
            "{address}/api/v1/system/tenants/{tenant_id}/database"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"database_url": test_db_url}))
        .send()
        .await
        .unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);

    // 6. Delete tenant
    let delete_res = client
        .delete(&format!("{address}/api/v1/system/tenants/{tenant_id}"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    // 7. Verify it's gone from list
    let list_res3 = client
        .get(&format!("{address}/api/v1/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let list3: Vec<serde_json::Value> = list_res3.json().await.unwrap();
    assert!(!list3.iter().any(|t| t["id"] == created["id"]));
}

#[tokio::test]
async fn test_tenant_api_authorization() {
    let (address, _workspace_id, user_id, _factory, _test_db) = spawn_app_with_factory().await;

    // user_id is a normal user, NOT a PlatformAdmin
    let client = reqwest::Client::new();

    // 1. List should be forbidden
    let list_res = client
        .get(&format!("{address}/api/v1/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(list_res.status(), StatusCode::FORBIDDEN);

    // 2. Create should be forbidden
    let create_res = client
        .post(&format!("{address}/api/v1/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "name": "Unauthorized" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::FORBIDDEN);

    // 3. Stats should be forbidden
    let stats_res = client
        .get(&format!(
            "{address}/api/v1/system/tenants/{}/stats",
            Uuid::new_v4()
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(stats_res.status(), StatusCode::FORBIDDEN);

    // 4. Delete should be forbidden
    let delete_res = client
        .delete(&format!(
            "{address}/api/v1/system/tenants/{}",
            Uuid::new_v4()
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), StatusCode::FORBIDDEN);

    // 5. Update DB URL should be forbidden
    let update_res = client
        .put(&format!(
            "{address}/api/v1/system/tenants/{}/database",
            Uuid::new_v4()
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "database_url": "postgres://user:pass@host/db" }))
        .send()
        .await
        .unwrap();
    assert_eq!(update_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_tenant_api_edge_cases() {
    let (address, _workspace_id, user_id, factory, test_db) = spawn_app_with_factory().await;
    let test_db_url = test_db.url();

    // Assign PlatformAdmin role
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid,
        user_id
    );

    let client = reqwest::Client::new();
    let nonexistent_id = Uuid::new_v4();

    // 1. Stats for nonexistent tenant → 404
    let stats_res = client
        .get(&format!(
            "{address}/api/v1/system/tenants/{nonexistent_id}/stats"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(stats_res.status(), StatusCode::NOT_FOUND);

    // 2. Delete nonexistent tenant is idempotent (204)
    let delete_res = client
        .delete(&format!("{address}/api/v1/system/tenants/{nonexistent_id}"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), StatusCode::NO_CONTENT);

    // 3. Update database URL for nonexistent tenant is idempotent (200)
    let update_res = client
        .put(&format!(
            "{address}/api/v1/system/tenants/{nonexistent_id}/database"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({"database_url": test_db_url}))
        .send()
        .await
        .unwrap();
    assert_eq!(update_res.status(), StatusCode::OK);
}
