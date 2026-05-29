use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

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
        self.registered_tenants
            .lock()
            .await
            .push((tenant_id.to_string(), payload.clone()));
        Ok(())
    }

    async fn delete_tenant(&self, tenant_id: &str) -> apiserver::errors::AppResult<()> {
        self.deleted_tenants
            .lock()
            .await
            .push(tenant_id.to_string());
        Ok(())
    }

    async fn get_tenant(&self, tenant_id: &str) -> apiserver::errors::AppResult<serde_json::Value> {
        let registered = self.registered_tenants.lock().await;
        for (id, payload) in registered.iter() {
            if id == tenant_id {
                return Ok(payload.clone());
            }
        }
        Ok(serde_json::json!({ "error": "Not Found" }))
    }
}

#[tokio::test]
async fn test_create_and_delete_tenant_with_mock() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, workspace_id, user_id, factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Make the user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();

    // 1. Create Tenant
    let new_ws_id = workspace_id;
    let req = serde_json::json!({
        "workspace_id": new_ws_id,
        "database_url": "postgres://user:pass@localhost:5432/test_db",
        "pool_size": 15
    });

    let response = client
        .post(&format!("{address}/api/v1/system/tenants"))
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
        "Failed to create tenant: {}",
        body
    );

    // Verify the mock recorded the registration
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert_eq!(
            registered.len(),
            1,
            "Should have registered exactly 1 tenant"
        );
        assert_eq!(
            registered[0].0,
            format!("tenant_{}", new_ws_id.to_string().replace('-', "")),
            "Should register the correct tenant id"
        );
    }

    // 2. Delete Tenant
    let delete_res = client
        .delete(&format!("{address}/api/v1/system/tenants/{}", new_ws_id))
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
        assert_eq!(
            deleted[0],
            new_ws_id.to_string(),
            "Should delete the correct tenant id"
        );
    }
}

#[tokio::test]
async fn test_create_tenant_with_byod() {
    let keys_json = serde_json::json!({
        "keys": { "v1": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" },
        "active_key": "v1"
    })
    .to_string();
    std::env::set_var("ENCRYPTION_KEYS_JSON", keys_json.clone());

    let mock_manager = std::sync::Arc::new(TestMockTenantManager::default());
    let (address, workspace_id, user_id, factory, _db) =
        crate::common::app::spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Use the workspace_id that was already created
    let new_ws_id = workspace_id;

    // Make the user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();

    let custom_url = "postgres://custom_user:super_secret@custom_host:5432/custom_db";

    let req = serde_json::json!({
        "workspace_id": new_ws_id,
        "database_url": custom_url,
        "pool_size": 25
    });

    let response = client
        .post(&format!("{address}/api/v1/system/tenants"))
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
        "Failed to create BYOD tenant: {}",
        body
    );

    // Verify the mock recorded the registration and used the custom DB settings
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert_eq!(
            registered.len(),
            1,
            "Should have registered exactly 1 tenant"
        );

        let (tenant_id, payload) = &registered[0];
        assert_eq!(
            tenant_id,
            &format!("tenant_{}", new_ws_id.to_string().replace('-', "")),
            "Should register the correct tenant id"
        );

        let tenant = payload.get("tenant").expect("Should have tenant object");
        assert_eq!(
            tenant.get("db_database").unwrap().as_str().unwrap(),
            "custom_db"
        );
        assert_eq!(
            tenant.get("db_host").unwrap().as_str().unwrap(),
            "custom_host"
        );
        assert_eq!(tenant.get("db_port").unwrap().as_u64().unwrap(), 5432);

        // Assert new Edge Case fields
        assert_eq!(
            tenant.get("default_pool_size").unwrap().as_u64().unwrap(),
            25
        );

        let users = tenant.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 1);
        let user_config = &users[0];
        assert_eq!(
            user_config.get("db_user").unwrap().as_str().unwrap(),
            "custom_user"
        );
        assert_eq!(
            user_config.get("db_password").unwrap().as_str().unwrap(),
            "super_secret"
        );
        assert_eq!(user_config.get("pool_size").unwrap().as_u64().unwrap(), 25);
    }

    // Verify database encryption
    let workspace_repo =
        apiserver::repositories::diesel_workspace_repository::DieselWorkspaceRepository::new(
            factory.diesel_pools.clone(),
        );
    let ws =
        apiserver::repositories::workspace_repository::WorkspaceRepository::get_workspace_by_id(
            &workspace_repo,
            new_ws_id,
        )
        .await
        .unwrap()
        .unwrap();
    let encrypted_url = ws
        .custom_db_url_encrypted
        .expect("Encrypted URL should be saved");

    // Decrypt and verify
    let crypto = apiserver::crypto::CryptoService::from_json(&keys_json).unwrap();
    let decrypted = crypto.decrypt(&encrypted_url).unwrap();

    assert_eq!(decrypted, custom_url);
    std::env::remove_var("ENCRYPTION_KEYS_JSON");
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
        "database_url": "postgres://fake:fake@host/hacked_db",
        "pool_size": 15
    });

    let response = client
        .post(&format!("{address}/api/v1/system/tenants"))
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

#[tokio::test]
async fn test_get_tenant_stats() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, workspace_id, user_id, factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Make the user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();

    // Setup: mock manager has a tenant
    let mock_tenant_config = serde_json::json!({
        "tenant": {
            "db_database": "test_db",
            "db_host": "test_host",
            "db_port": 5432,
            "users": [
                {
                    "db_user": "test_user",
                    "pool_size": 15,
                    "mode_type": "transaction"
                }
            ]
        }
    });
    mock_manager
        .registered_tenants
        .lock()
        .await
        .push((workspace_id.to_string(), mock_tenant_config.clone()));

    // Execute GET
    let response = client
        .get(&format!(
            "{address}/api/v1/system/tenants/{workspace_id}/stats"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert_eq!(
        status,
        reqwest::StatusCode::OK,
        "Failed to get stats: {}",
        body
    );

    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["config"]["tenant"]["db_database"], "test_db");
    assert!(parsed["connection"]["host"].is_string());
    assert!(parsed["connection"]["port"].is_string());
}

#[tokio::test]
async fn test_update_tenant_database() {
    let keys_json = serde_json::json!({
        "keys": { "v1": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=" },
        "active_key": "v1"
    })
    .to_string();
    std::env::set_var("ENCRYPTION_KEYS_JSON", keys_json.clone());

    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, workspace_id, user_id, factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Make user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();
    let tenant_id = format!("tenant_{}", workspace_id.to_string().replace('-', ""));

    let custom_url = "postgres://user:pass@new-host:5432/new_db";
    let req = serde_json::json!({
        "database_url": custom_url
    });

    // 1. PUT update database
    let response = client
        .put(&format!(
            "{address}/api/v1/system/tenants/{tenant_id}/database"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to execute request");

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    assert_eq!(
        status,
        reqwest::StatusCode::OK,
        "Failed to update tenant DB: {}",
        body
    );

    // 2. Verify mock manager recorded the registration
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert!(
            registered.iter().any(|(id, _)| id == &tenant_id),
            "Supavisor should have been called with tenant_id: {}",
            tenant_id
        );
    }

    // 3. Verify workspace DB URL was encrypted & saved using workspace repository
    let workspace_repo =
        apiserver::repositories::diesel_workspace_repository::DieselWorkspaceRepository::new(
            factory.diesel_pools.clone(),
        );
    let ws =
        apiserver::repositories::workspace_repository::WorkspaceRepository::get_workspace_by_id(
            &workspace_repo,
            workspace_id,
        )
        .await
        .unwrap()
        .unwrap();
    let encrypted_url = ws
        .custom_db_url_encrypted
        .expect("Encrypted URL must be saved after update");

    // Decrypt and verify
    let crypto = apiserver::crypto::CryptoService::from_json(&keys_json).unwrap();
    let decrypted = crypto.decrypt(&encrypted_url).unwrap();
    assert_eq!(decrypted, custom_url);

    // 4. Verify GET stats returns masked_db_url with password redacted
    let response = client
        .get(&format!(
            "{address}/api/v1/system/tenants/{tenant_id}/stats"
        ))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    let masked_db_url = body["connection"]["masked_db_url"]
        .as_str()
        .expect("masked_db_url should be present");
    // Password should be masked as ***
    assert_eq!(masked_db_url, "postgres://user:***@new-host:5432/new_db");
    // Verify password is NOT leaked
    assert!(
        !masked_db_url.contains("pass"),
        "Password must NOT appear in masked URL"
    );

    std::env::remove_var("ENCRYPTION_KEYS_JSON");
}

#[tokio::test]
async fn test_update_tenant_database_requires_admin() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, workspace_id, user_id, _factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // NO PlatformAdmin role assigned

    let client = reqwest::Client::new();
    let tenant_id = format!("tenant_{}", workspace_id.to_string().replace('-', ""));

    let req = serde_json::json!({
        "database_url": "postgres://hacker:evil@evil-host:5432/stolen_db"
    });

    let response = client
        .put(&format!(
            "{address}/api/v1/system/tenants/{tenant_id}/database"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::FORBIDDEN,
        "Normal user should NOT be able to update tenant database"
    );

    // Verify mock recorded NO registration
    {
        let registered = mock_manager.registered_tenants.lock().await;
        assert!(
            registered.is_empty(),
            "No tenant should be registered by unauthorized user"
        );
    }
}

#[tokio::test]
async fn test_update_tenant_database_invalid_tenant_id() {
    let mock_manager = Arc::new(TestMockTenantManager::default());
    let (address, _workspace_id, user_id, factory, _db) =
        spawn_app_with_factory_extended(mock_manager.clone()).await;

    // Make user a PlatformAdmin
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id) VALUES ($1, (SELECT id FROM system_roles WHERE name = 'platform_admin'))",
        diesel::sql_types::Uuid, user_id
    );

    let client = reqwest::Client::new();

    let req = serde_json::json!({
        "database_url": "postgres://user:pass@host:5432/db"
    });

    // Use an invalid tenant_id (not a valid UUID)
    let response = client
        .put(&format!(
            "{address}/api/v1/system/tenants/tenant_not-a-uuid/database"
        ))
        .header("X-User-Id", user_id.to_string())
        .json(&req)
        .send()
        .await
        .expect("Failed to execute request");

    // Should return a 4xx error (Validation)
    assert!(
        response.status().is_client_error(),
        "Invalid tenant_id should return client error, got: {}",
        response.status()
    );
}
