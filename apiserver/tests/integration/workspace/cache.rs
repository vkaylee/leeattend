//! Integration tests for workspace configuration cache

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::cache::{RedisCache, WorkspaceCache};
use std::sync::Arc;

use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[tokio::test]
async fn test_workspace_cache_get_and_invalidate() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let redis_pool = crate::common::app::create_test_redis_pool()
        .await
        .expect("Failed to create Redis pool");
    let factory = TestFactory::new(pool.clone()).with_redis(Some(redis_pool.clone()));
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let redis_cache = RedisCache::new(redis_pool.clone());
    let workspace_cache = WorkspaceCache::new(Arc::new(redis_cache), diesel_pools.clone());

    // 1. First `get` should fetch from DB and populate cache
    let config = workspace_cache
        .get(ws_id)
        .await
        .expect("Failed to get workspace config");
    assert_eq!(config.id, ws_id);
    assert_eq!(config.timezone, "Asia/Ho_Chi_Minh");

    // 2. Modify DB directly (bypassing service/cache)
    crate::test_execute!(
        &pool.api,
        "UPDATE workspaces SET timezone = $1 WHERE id = $2",
        diesel::sql_types::Text,
        "UTC",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 3. Second `get` should fetch from Cache (returning stale data)
    let stale_config = workspace_cache
        .get(ws_id)
        .await
        .expect("Failed to get workspace config");
    assert_eq!(stale_config.timezone, "Asia/Ho_Chi_Minh"); // Should be old data

    // 4. Invalidate cache
    workspace_cache
        .invalidate(ws_id)
        .await
        .expect("Failed to invalidate cache");

    // Give Redis a tiny bit of time to process deletion
    sleep(Duration::from_millis(50)).await;

    // 5. Third `get` should fetch from DB again (returning fresh data)
    let fresh_config = workspace_cache
        .get(ws_id)
        .await
        .expect("Failed to get workspace config");
    assert_eq!(fresh_config.timezone, "UTC");
}
