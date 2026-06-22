use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::tenant_provisioner::{
    DatabaseMigratorTrait, TenantMigrator, TenantProvisionerTrait,
};
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_provisioner_timezone_lookup() {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());

    // 1. Create a workspace in the system database with a specific timezone
    let target_tz = "Asia/Tokyo";
    let workspace_id = factory.workspace().with_timezone(target_tz).create().await;

    // Delete default work location seeded by workspace factory,
    // so that tenant provisioner's HQ seeding does not conflict on the same DB.
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "DELETE FROM work_locations WHERE workspace_id = $1",
        diesel::sql_types::Uuid,
        workspace_id
    );
    let db_url = test_db.url();

    // 2. Provision the tenant DB (which is our test_db)
    let migrator = TenantMigrator::new(test_db.diesel_pools.clone());
    let result = migrator.provision_tenant_db(&workspace_id, &db_url).await;

    assert!(
        result.is_ok(),
        "Failed to provision tenant DB: {:?}",
        result.err()
    );
    let returned_url = result.unwrap();
    assert_eq!(returned_url, db_url);

    // 3. Verify that the headquarters location was seeded with the workspace's timezone
    let timezone_in_db = crate::test_query_scalar_string!(
        &test_db.diesel_pools.api,
        "SELECT timezone AS val FROM work_locations WHERE id = '11111111-1111-1111-1111-111111111111'::uuid"
    );
    assert_eq!(timezone_in_db, target_tz);
}

#[tokio::test]
async fn test_tenant_provisioner_invalid_db_url() {
    let test_db = TestDatabase::new().await;
    let migrator = TenantMigrator::new(test_db.diesel_pools.clone());
    let workspace_id = Uuid::new_v4();
    let invalid_url = "postgres://invalid:invalid@localhost:12345/invalid_db";

    let result = migrator
        .provision_tenant_db(&workspace_id, invalid_url)
        .await;

    assert!(
        result.is_err(),
        "Expected provision to fail with invalid DB URL"
    );
}

#[tokio::test]
async fn test_database_migrator_create_and_delete() {
    let test_db = TestDatabase::new().await;
    let migrator = TenantMigrator::new(test_db.diesel_pools.clone());

    // Construct the admin/system URL to create/drop database on the same PG instance
    let base_url = test_db.url();
    let mut url_parts: Vec<&str> = base_url.split('/').collect();
    url_parts.pop();
    let base_connection_string = url_parts.join("/");
    let admin_url = format!("{base_connection_string}/test_database");

    let unique_db_name = format!("temp_migrator_db_{}", Uuid::new_v4().simple());

    // Create DB
    let create_res = migrator
        .create_tenant_database(&admin_url, &unique_db_name)
        .await;
    assert!(
        create_res.is_ok(),
        "Failed to create tenant DB: {:?}",
        create_res.err()
    );

    // Delete DB
    let delete_res = migrator
        .delete_tenant_database(&admin_url, &unique_db_name)
        .await;
    assert!(
        delete_res.is_ok(),
        "Failed to delete tenant DB: {:?}",
        delete_res.err()
    );
}
