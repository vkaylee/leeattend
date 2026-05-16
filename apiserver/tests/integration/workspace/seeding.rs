use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{DieselWorkspaceRepository, WorkspaceRepository};
use uuid::Uuid;

#[tokio::test]
async fn test_workspace_creation_no_default_leave_types() {
    let (_address, _, user_id, factory, test_db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;

    // 1. Grant platform_admin to create workspace
    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO user_system_roles (user_id, role_id, assigned_by)
         SELECT $1, id, $1 FROM system_roles WHERE name = 'platform_admin'
         ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid,
        user_id
    ); // expect("Failed to seed platform_admin role");

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let repo = DieselWorkspaceRepository::new(diesel_pools.clone());

    // 2. Create workspace using repository directly (which is used by Service)
    let workspace = repo
        .create_workspace_with_owner("Fresh Start WS", user_id)
        .await
        .expect("Failed to create workspace");

    // 3. Verify leave_types table is empty for this workspace
    let count: i64 = crate::test_query_scalar_i64!(
        &factory.diesel_pools.api,
        "SELECT count(*) AS count FROM leave_types WHERE workspace_id = $1",
        diesel::sql_types::Uuid,
        workspace.id
    ); // expect("Failed to count leave types");

    assert_eq!(
        count, 0,
        "New workspace should not have any default leave types seeded"
    );
}
