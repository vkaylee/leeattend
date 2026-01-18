mod common;

use apiserver::services::workspace_service::WorkspaceService;
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_update_workspace_time_format() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Verify default time format is 24h
    let ws =
        sqlx::query_as::<_, apiserver::models::Workspace>("SELECT * FROM workspaces WHERE id = $1")
            .bind(ws_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(ws.time_format, "24h");

    // 2. Update to 12h
    let updated_ws = WorkspaceService::update_workspace(
        &pool,
        ws_id,
        None,
        None,
        None,
        None,
        Some("12h".to_string()),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("Failed to update workspace");

    assert_eq!(updated_ws.time_format, "12h");

    // 3. Verify persistence
    let persisted_ws =
        sqlx::query_as::<_, apiserver::models::Workspace>("SELECT * FROM workspaces WHERE id = $1")
            .bind(ws_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(persisted_ws.time_format, "12h");

    // 4. Update back to 24h
    let updated_ws_24 = WorkspaceService::update_workspace(
        &pool,
        ws_id,
        None,
        None,
        None,
        None,
        Some("24h".to_string()),
        None,
        None,
        None,
        None,
    )
    .await
    .expect("Failed to update workspace");

    assert_eq!(updated_ws_24.time_format, "24h");
}
