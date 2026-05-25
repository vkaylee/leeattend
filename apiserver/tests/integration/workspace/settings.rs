#![allow(warnings)]
#![allow(clippy::all)]
//! Workspace theme and settings tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::tenant_provisioner::TenantProvisionerTrait;
use apiserver::services::workspace_service::WorkspaceService;
use apiserver::supavisor::TenantManager;
use reqwest::StatusCode;
use uuid::Uuid;

struct DummyTenantProvisioner;
#[async_trait::async_trait]
impl TenantProvisionerTrait for DummyTenantProvisioner {
    async fn provision_tenant_db(
        &self,
        _workspace_id: &Uuid,
        db_url: &str,
    ) -> Result<String, apiserver::errors::AppError> {
        Ok(db_url.to_string())
    }
}

struct DummySystemDbClusterRepository;
#[async_trait::async_trait]
impl apiserver::repositories::system_db_cluster_repository::SystemDbClusterRepository
    for DummySystemDbClusterRepository
{
    async fn get_least_loaded_cluster(
        &self,
    ) -> Result<
        Option<apiserver::models::system_db_cluster::SystemDbCluster>,
        apiserver::errors::AppError,
    > {
        Ok(None)
    }
    async fn increment_active_tenants(
        &self,
        _cluster_id: uuid::Uuid,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn create_cluster(
        &self,
        _cluster: &apiserver::models::system_db_cluster::NewSystemDbCluster,
    ) -> Result<apiserver::models::system_db_cluster::SystemDbCluster, apiserver::errors::AppError>
    {
        unimplemented!()
    }
    async fn list_clusters(
        &self,
    ) -> Result<
        Vec<apiserver::models::system_db_cluster::SystemDbCluster>,
        apiserver::errors::AppError,
    > {
        Ok(vec![])
    }
    async fn delete_cluster(
        &self,
        _cluster_id: uuid::Uuid,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn update_cluster_status(
        &self,
        _cluster_id: uuid::Uuid,
        _is_active: bool,
    ) -> Result<apiserver::models::system_db_cluster::SystemDbCluster, apiserver::errors::AppError>
    {
        unimplemented!()
    }
}

struct DummyTenantManager;
#[async_trait::async_trait]
impl TenantManager for DummyTenantManager {
    async fn init_tenant(&self) {}
    async fn delete_tenant(&self, _workspace_id: &str) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn register_tenant_dynamic(
        &self,
        _workspace_id: &str,
        _db_config: &serde_json::Value,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }
    async fn get_tenant(
        &self,
        _tenant_id: &str,
    ) -> Result<serde_json::Value, apiserver::errors::AppError> {
        Ok(serde_json::json!({}))
    }
}

#[tokio::test]
async fn test_workspace_theme_color_update() {
    let (address, workspace_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Check default color (Violet #7c3aed)
    let response = client
        .get(format!("{address}/api/workspaces/{workspace_id}"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    assert_eq!(workspace["theme_primary_color"], "#7c3aed");

    // 2. Update to new color (Emerald #059669)
    let new_color = "#059669";
    let response = client
        .patch(format!("{address}/api/workspaces/{workspace_id}"))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "theme_primary_color": new_color }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    assert_eq!(workspace["theme_primary_color"], new_color);

    // 3. Verify persistence
    let response = client
        .get(format!("{address}/api/workspaces/{workspace_id}"))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    assert_eq!(workspace["theme_primary_color"], new_color);
}

#[tokio::test]
async fn test_update_workspace_time_format() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let workspace_repo = std::sync::Arc::new(
        apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
    );
    let user_repo = std::sync::Arc::new(apiserver::repositories::DieselUserRepository::new(
        diesel_pools.clone(),
    ));

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // 1. Verify default time format is 24h via repository
    use apiserver::repositories::workspace_repository::WorkspaceRepository;
    let ws = workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ws.time_format, "24h");

    // 2. Update to 12h
    let updated_ws = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        std::sync::Arc::new(DummyTenantProvisioner),
        std::sync::Arc::new(DummyTenantManager),
        std::sync::Arc::new(DummySystemDbClusterRepository),
    )
    .update_workspace(
        apiserver::services::workspace_service::UpdateWorkspaceArgs {
            ws_id,
            name: None,
            date_format: None,
            time_format: Some("12h".to_string()),
            timezone: None,
            theme_primary_color: None,
            default_language: None,
            holiday_overnight_strategy: None,
            default_holiday_ot_multiplier: None,
            logo_url: None,
            tagline: None,
        },
    )
    .await
    .expect("Failed to update workspace");

    assert_eq!(updated_ws.time_format, "12h");

    // 3. Verify persistence via repository
    let persisted_ws = workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted_ws.time_format, "12h");
}

#[tokio::test]
async fn test_update_workspace_all_fields() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let workspace_repo = std::sync::Arc::new(
        apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
    );
    let user_repo = std::sync::Arc::new(apiserver::repositories::DieselUserRepository::new(
        diesel_pools.clone(),
    ));

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let updated_ws = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        std::sync::Arc::new(DummyTenantProvisioner),
        std::sync::Arc::new(DummyTenantManager),
        std::sync::Arc::new(DummySystemDbClusterRepository),
    )
    .update_workspace(
        apiserver::services::workspace_service::UpdateWorkspaceArgs {
            ws_id,
            name: Some("New Name".to_string()),
            date_format: Some("YYYY-MM-DD".to_string()),
            time_format: Some("12h".to_string()),
            timezone: Some("Asia/Ho_Chi_Minh".to_string()),
            theme_primary_color: Some("#ffffff".to_string()),
            default_language: Some("vi".to_string()),
            holiday_overnight_strategy: Some("split".to_string()),
            default_holiday_ot_multiplier: Some(rust_decimal::Decimal::new(20, 1)),
            logo_url: Some("https://example.com/logo.png".to_string()),
            tagline: Some("My Tagline".to_string()),
        },
    )
    .await
    .expect("Failed to update workspace");

    assert_eq!(updated_ws.name, "New Name");
    assert_eq!(updated_ws.date_format, "YYYY-MM-DD");
    assert_eq!(updated_ws.timezone, "Asia/Ho_Chi_Minh");
    assert_eq!(updated_ws.default_language, "vi");
}

#[tokio::test]
async fn test_update_workspace_no_fields() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let workspace_repo = std::sync::Arc::new(
        apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
    );
    let user_repo = std::sync::Arc::new(apiserver::repositories::DieselUserRepository::new(
        diesel_pools.clone(),
    ));

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let result = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        std::sync::Arc::new(DummyTenantProvisioner),
        std::sync::Arc::new(DummyTenantManager),
        std::sync::Arc::new(DummySystemDbClusterRepository),
    )
    .update_workspace(
        apiserver::services::workspace_service::UpdateWorkspaceArgs {
            ws_id,
            name: None,
            date_format: None,
            time_format: None,
            timezone: None,
            theme_primary_color: None,
            default_language: None,
            holiday_overnight_strategy: None,
            default_holiday_ot_multiplier: None,
            logo_url: None,
            tagline: None,
        },
    )
    .await;

    let err = result.unwrap_err();
    assert!(err.to_string().contains("No fields to update"));
}
