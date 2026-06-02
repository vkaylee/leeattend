use crate::common::db::TestDatabase;
use apiserver::services::tenant_provisioner::{TenantMigrator, TenantProvisionerTrait};
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_provisioner_success() {
    let test_db = TestDatabase::new().await;
    let workspace_id = Uuid::new_v4();
    let db_url = test_db.url();

    let migrator = TenantMigrator::new();
    let result = migrator.provision_tenant_db(&workspace_id, &db_url).await;

    assert!(
        result.is_ok(),
        "Failed to provision tenant DB: {:?}",
        result.err()
    );
    let returned_url = result.unwrap();
    assert_eq!(returned_url, db_url);
}
