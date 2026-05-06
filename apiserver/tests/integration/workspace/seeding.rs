use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{SqlxWorkspaceRepository, WorkspaceRepository};
use uuid::Uuid;

#[tokio::test]
async fn test_workspace_creation_no_default_leave_types() {
    let (_address, _, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    
    // 1. Grant platform_admin to create workspace
    sqlx::query(
        "INSERT INTO user_system_roles (user_id, role_id, assigned_by)
         SELECT $1, id, $1 FROM system_roles WHERE name = 'platform_admin'
         ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .execute(&factory.pool)
    .await
    .expect("Failed to seed platform_admin role");

    let repo = SqlxWorkspaceRepository { pool: factory.pool.clone() };

    // 2. Create workspace using repository directly (which is used by Service)
    let workspace = repo.create_workspace("Fresh Start WS").await.expect("Failed to create workspace");

    // 3. Verify leave_types table is empty for this workspace
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM leave_types WHERE workspace_id = $1")
        .bind(workspace.id)
        .fetch_one(&factory.pool)
        .await
        .expect("Failed to count leave types");

    assert_eq!(count, 0, "New workspace should not have any default leave types seeded");
}
