mod common;

use common::db::TestDatabase;
use apiserver::dev_initialization::ensure_dev_data;

#[tokio::test]
async fn test_ensure_dev_data() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();

    ensure_dev_data(&pool).await.unwrap();

    // Verify admin user exists
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE email = 'admin@leeattend.com')")
        .fetch_one(&pool).await.unwrap();
    assert!(exists);

    // Run again to test idempotency (ON CONFLICT)
    ensure_dev_data(&pool).await.unwrap();
}
