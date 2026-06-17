use crate::services::tenant_provisioner::{TenantMigrator, TenantProvisionerTrait};
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_migrator_invalid_url() {
    let migrator = TenantMigrator::new();
    let workspace_id = Uuid::new_v4();
    let bad_url = "postgres://username:password@localhost:5432/non_existent_db";
    let res = migrator.provision_tenant_db(&workspace_id, bad_url).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_tenant_migrator_with_empty_url() {
    let migrator = TenantMigrator::new();
    let workspace_id = Uuid::new_v4();
    let empty_url = "";
    let res = migrator.provision_tenant_db(&workspace_id, empty_url).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_tenant_migrator_integration_success() {
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        use diesel::Connection;
        if diesel::PgConnection::establish(&db_url).is_ok() {
            let migrator = TenantMigrator::new();
            let workspace_id = Uuid::new_v4();
            let res = migrator.provision_tenant_db(&workspace_id, &db_url).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), db_url);
        }
    }
}
