//! Integration tests for workspace configuration cache

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::cache::{RedisCache, WorkspaceCache};

use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[tokio::test]
async fn test_workspace_cache_get_and_invalidate() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());
    let redis_pool = apiserver::queue::RedisPool::new(&redis_url)
        .await
        .expect("Failed to create Redis pool");
    let factory = TestFactory::new(pool.clone());

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let redis_cache = RedisCache::new(redis_pool.clone());
    let workspace_cache = WorkspaceCache::new(redis_cache, pool.clone());

    // 1. First `get` should fetch from DB and populate cache
    let config = workspace_cache
        .get(ws_id)
        .await
        .expect("Failed to get workspace config");
    assert_eq!(config.id, ws_id);
    assert_eq!(config.timezone, "Asia/Ho_Chi_Minh");

    // 2. Modify DB directly (bypassing service/cache)
    sqlx::query("UPDATE workspaces SET timezone = $1 WHERE id = $2")
        .bind("UTC")
        .bind(ws_id)
        .execute(&pool)
        .await
        .expect("Failed to update workspace in DB");

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
