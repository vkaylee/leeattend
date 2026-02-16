//! Schema Isolation integration tests

use crate::common::db::TestDatabase;

#[tokio::test]
async fn test_schema_isolation_between_tests() {
    // This verifies that each test gets a fresh database
    let db1 = TestDatabase::new().await;
    let db2 = TestDatabase::new().await;

    let name1: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&db1.pool).await.unwrap();
    let name2: String = sqlx::query_scalar("SELECT current_database()").fetch_one(&db2.pool).await.unwrap();
    
    assert_ne!(name1, name2);
}
