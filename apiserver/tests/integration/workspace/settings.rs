#![allow(warnings)]
#![allow(clippy::all)]
//! Workspace theme and settings tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::tenant_provisioner::TenantProvisionerTrait;
use apiserver::services::workspace_service::WorkspaceService;
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

#[async_trait::async_trait]
impl apiserver::services::tenant_provisioner::DatabaseMigratorTrait for DummyTenantProvisioner {
    async fn create_tenant_database(
        &self,
        _admin_url: &str,
        _db_name: &str,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
    }

    async fn delete_tenant_database(
        &self,
        _admin_url: &str,
        _db_name: &str,
    ) -> Result<(), apiserver::errors::AppError> {
        Ok(())
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
    async fn decrement_active_tenants(
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

#[tokio::test]
async fn test_workspace_theme_color_update() {
    let (address, workspace_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Check default color (Violet #7c3aed)
    let response = client
        .get(format!("{address}/api/v1/workspaces/{workspace_id}"))
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
        .patch(format!("{address}/api/v1/workspaces/{workspace_id}"))
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
        .get(format!("{address}/api/v1/workspaces/{workspace_id}"))
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
    assert_eq!(ws.time_format(), "24h");

    // 2. Update to 12h
    let workspace_config_repo = std::sync::Arc::new(
        apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()),
    );
    let dummy_migrator = std::sync::Arc::new(DummyTenantProvisioner);
    let updated_ws = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        dummy_migrator.clone(),
        std::sync::Arc::new(DummySystemDbClusterRepository),
        workspace_config_repo.clone(),
        dummy_migrator,
    )
    .update_workspace(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
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

    assert_eq!(updated_ws.time_format(), "12h");

    // 3. Verify persistence via repository
    let persisted_ws = workspace_repo
        .get_workspace_by_id(ws_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(persisted_ws.time_format(), "12h");
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

    let workspace_config_repo = std::sync::Arc::new(
        apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()),
    );
    let dummy_migrator = std::sync::Arc::new(DummyTenantProvisioner);
    let updated_ws = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        dummy_migrator.clone(),
        std::sync::Arc::new(DummySystemDbClusterRepository),
        workspace_config_repo.clone(),
        dummy_migrator,
    )
    .update_workspace(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
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

    assert_eq!(updated_ws.name(), "New Name");
    assert_eq!(updated_ws.date_format(), "YYYY-MM-DD");
    assert_eq!(updated_ws.timezone(), "Asia/Ho_Chi_Minh");
    assert_eq!(updated_ws.default_language(), "vi");
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

    let workspace_config_repo = std::sync::Arc::new(
        apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()),
    );
    let dummy_migrator = std::sync::Arc::new(DummyTenantProvisioner);
    let result = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        dummy_migrator.clone(),
        std::sync::Arc::new(DummySystemDbClusterRepository),
        workspace_config_repo.clone(),
        dummy_migrator,
    )
    .update_workspace(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
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
    assert!(err.to_string().contains("error.no_fields_to_update"));
}

#[tokio::test]
async fn test_update_workspace_syncs_headquarters_timezone() {
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

    // Get the existing Headquarters work location seeded by the factory
    use apiserver::schema::work_locations;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;
    let mut conn = diesel_pools
        .tenants_tx
        .get_connection(&ws_id)
        .await
        .unwrap();
    let mut conn = apiserver::diesel_pool::TransactionConn(conn);

    let hq_id: Uuid = work_locations::table
        .filter(work_locations::workspace_id.eq(ws_id))
        .filter(work_locations::is_headquarters.eq(true))
        .select(work_locations::id)
        .first(&mut *conn)
        .await
        .unwrap();

    // Now update workspace timezone
    let workspace_config_repo = std::sync::Arc::new(
        apiserver::repositories::diesel_workspace_config_repository::DieselWorkspaceConfigRepository::new(diesel_pools.clone()),
    );
    let dummy_migrator = std::sync::Arc::new(DummyTenantProvisioner);
    let _updated_ws = WorkspaceService::new(
        workspace_repo.clone(),
        user_repo.clone(),
        dummy_migrator.clone(),
        std::sync::Arc::new(DummySystemDbClusterRepository),
        workspace_config_repo.clone(),
        dummy_migrator,
    )
    .update_workspace(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        apiserver::services::workspace_service::UpdateWorkspaceArgs {
            ws_id,
            name: None,
            date_format: None,
            time_format: None,
            timezone: Some("Asia/Ho_Chi_Minh".to_string()),
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

    // Verify headquarters location timezone
    let hq_tz: String = work_locations::table
        .filter(work_locations::id.eq(hq_id))
        .select(work_locations::timezone)
        .first(&mut *conn)
        .await
        .unwrap();

    assert_eq!(hq_tz, "Asia/Ho_Chi_Minh");
}
