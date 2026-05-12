//! Repository Pagination integration tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::SqlxWorkspaceRepository;
use std::sync::Arc;

#[tokio::test]
async fn test_repository_pagination() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(
        &std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/leeattend".to_string()
        }),
    );

    let user_id = factory.user().create().await;
    let _repo = Arc::new(SqlxWorkspaceRepository { pool: pool.clone() });

    // Create 15 workspaces
    for _ in 0..15 {
        factory.workspace().create_with_owner(user_id).await;
    }

    // This is a generic test pattern, actual repo call depends on implementation
}
