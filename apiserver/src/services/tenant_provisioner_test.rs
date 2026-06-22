use crate::diesel_pool::DieselPools;
use crate::services::tenant_provisioner::{TenantMigrator, TenantProvisionerTrait};
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_migrator_invalid_url() {
    let bad_url = "postgres://username:password@localhost:5432/non_existent_db";
    let pools = DieselPools::for_test(bad_url);
    let migrator = TenantMigrator::new(pools);
    let workspace_id = Uuid::new_v4();
    let res = migrator.provision_tenant_db(&workspace_id, bad_url).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_tenant_migrator_with_empty_url() {
    let empty_url = "";
    let pools = DieselPools::for_test(empty_url);
    let migrator = TenantMigrator::new(pools);
    let workspace_id = Uuid::new_v4();
    let res = migrator.provision_tenant_db(&workspace_id, empty_url).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_tenant_migrator_database_integration_scenarios() {
    if let Ok(db_url) = std::env::var("DATABASE_URL") {
        use diesel::Connection;
        if let Ok(mut conn) = diesel::PgConnection::establish(&db_url) {
            let pools = DieselPools::for_test(&db_url);
            let migrator = TenantMigrator::new(pools);

            // Scenario 1: Basic provision success
            let workspace_id_1 = Uuid::new_v4();
            // Delete conflicting headquarters row if exists
            use diesel::RunQueryDsl;
            diesel::sql_query("DELETE FROM work_locations WHERE id = '11111111-1111-1111-1111-111111111111'::uuid")
                .execute(&mut conn)
                .ok();

            let res = migrator.provision_tenant_db(&workspace_id_1, &db_url).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), db_url);

            // Scenario 2: Provision success with timezone query from system database
            let workspace_id_2 = Uuid::new_v4();
            // Delete conflicting headquarters row before the second provision
            diesel::sql_query("DELETE FROM work_locations WHERE id = '11111111-1111-1111-1111-111111111111'::uuid")
                .execute(&mut conn)
                .ok();

            // Insert workspace into workspaces table
            diesel::sql_query(
                "INSERT INTO workspaces (id, name, date_format, time_format, timezone, default_language) \
                 VALUES ($1, $2, $3, $4, $5, $6)"
            )
            .bind::<diesel::sql_types::Uuid, _>(workspace_id_2)
            .bind::<diesel::sql_types::Text, _>("Test Timezone Workspace")
            .bind::<diesel::sql_types::Text, _>("YYYY-MM-DD")
            .bind::<diesel::sql_types::Text, _>("HH:mm")
            .bind::<diesel::sql_types::Text, _>("Asia/Ho_Chi_Minh")
            .bind::<diesel::sql_types::Text, _>("en")
            .execute(&mut conn)
            .expect("Failed to insert workspace");

            let res = migrator.provision_tenant_db(&workspace_id_2, &db_url).await;
            assert!(res.is_ok());

            // Verify timezone of seeded work_location
            #[derive(diesel::QueryableByName, Debug)]
            struct LocationRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                timezone: String,
            }
            let rows = diesel::sql_query(
                "SELECT timezone FROM work_locations WHERE workspace_id = $1 AND is_headquarters = true"
            )
            .bind::<diesel::sql_types::Uuid, _>(workspace_id_2)
            .load::<LocationRow>(&mut conn)
            .expect("Failed to query work_locations");
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].timezone, "Asia/Ho_Chi_Minh");
        }
    }
}
