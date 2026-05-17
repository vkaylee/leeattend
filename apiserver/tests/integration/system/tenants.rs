use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::json;

use apiserver::handlers::system_tenants::CreateTenantRequest;
use apiserver::models::enums::SystemRoleType;

use crate::common::app::spawn_app_with_factory_extended;

#[derive(Default)]
pub struct TestMockTenantManager {
    pub registered_tenants: Arc<Mutex<Vec<(String, serde_json::Value)>>>,
    pub deleted_tenants: Arc<Mutex<Vec<String>>>,
}

#[async_trait::async_trait]
impl apiserver::supavisor::TenantManager for TestMockTenantManager {
    async fn init_tenant(&self) {}

    async fn register_tenant_dynamic(
        &self,
        tenant_id: &str,
        payload: &serde_json::Value,
    ) -> apiserver::errors::AppResult<()> {
        self.registered_tenants.lock().await.push((tenant_id.to_string(), payload.clone()));
        Ok(())
    }

    async fn delete_tenant(&self, tenant_id: &str) -> apiserver::errors::AppResult<()> {
        self.deleted_tenants.lock().await.push(tenant_id.to_string());
        Ok(())
    }
}

#[tokio::test]
async fn test_create_and_delete_tenant_with_mock() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, _workspace_id, user_id, factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Make the user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();

    // 1. Create Tenant
    let new_ws_id = uuid::Uuid::new_v4();
    let req = serde_json::json!({
        "workspace_id": new_ws_id,
        "custom_db_url": null,
        "pool_size": 15
    });

    let response = client
        .post(&format!("{address}/api/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert_eq!(
        status,
        reqwest::StatusCode::CREATED,
        "Failed to create tenant: {}", body
    );

    // Verify the mock recorded the registration
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert_eq!(registered.len(), 1, "Should have registered exactly 1 tenant");
        assert_eq!(registered[0].0, new_ws_id.to_string(), "Should register the correct tenant id");
    }

    // 2. Delete Tenant
    let delete_res = client
        .delete(&format!("{address}/api/system/tenants/{}", new_ws_id))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute delete request");

    assert_eq!(
        delete_res.status(),
        reqwest::StatusCode::NO_CONTENT,
        "Failed to delete tenant"
    );

    // Verify the mock recorded the deletion
    {
        let deleted = mock_manager.deleted_tenants.lock().await;
        assert_eq!(deleted.len(), 1, "Should have deleted exactly 1 tenant");
        assert_eq!(deleted[0], new_ws_id.to_string(), "Should delete the correct tenant id");
    }
}

#[tokio::test]
async fn test_create_tenant_with_byod() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, _workspace_id, user_id, factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Make the user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();

    let new_ws_id = uuid::Uuid::new_v4();
    let custom_url = "postgres://custom_user:super_secret@custom_host:5432/custom_db";
    
    let req = serde_json::json!({
        "workspace_id": new_ws_id,
        "custom_db_url": custom_url,
        "pool_size": 25
    });

    let response = client
        .post(&format!("{address}/api/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert_eq!(
        status,
        reqwest::StatusCode::CREATED,
        "Failed to create BYOD tenant: {}", body
    );

    // Verify the mock recorded the registration and used the custom DB settings
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert_eq!(registered.len(), 1, "Should have registered exactly 1 tenant");
        
        let (tenant_id, payload) = &registered[0];
        assert_eq!(tenant_id, &new_ws_id.to_string(), "Should register the correct tenant id");
        
        let tenant = payload.get("tenant").expect("Should have tenant object");
        assert_eq!(tenant.get("db_database").unwrap().as_str().unwrap(), "custom_db");
        assert_eq!(tenant.get("db_host").unwrap().as_str().unwrap(), "custom_host");
        assert_eq!(tenant.get("db_port").unwrap().as_u64().unwrap(), 5432);
        
        let users = tenant.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 1);
        let user_config = &users[0];
        assert_eq!(user_config.get("db_user").unwrap().as_str().unwrap(), "custom_user");
        assert_eq!(user_config.get("db_password").unwrap().as_str().unwrap(), "super_secret");
        assert_eq!(user_config.get("pool_size").unwrap().as_u64().unwrap(), 25);
    }
}

#[tokio::test]
async fn test_create_tenant_requires_platform_admin() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, _workspace_id, user_id, _factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Login as a normal user (no platform admin role)
    let client = reqwest::Client::new();

    let new_ws_id = uuid::Uuid::new_v4();
    let req = serde_json::json!({
        "workspace_id": new_ws_id,
        "custom_db_url": "postgres://fake:fake@host/hacked_db",
        "pool_size": 15
    });

    let response = client
        .post(&format!("{address}/api/system/tenants"))
        .header("X-User-Id", user_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to execute request");

    // Should be Forbidden because they lack PlatformAdmin
    assert_eq!(
        response.status(),
        reqwest::StatusCode::FORBIDDEN,
        "Normal user should not be able to create tenants"
    );

    // Verify the mock recorded NO registration
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert!(registered.is_empty(), "No tenant should be registered");
    }
}
