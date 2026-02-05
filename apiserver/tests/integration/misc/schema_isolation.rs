//! Schema Isolation integration tests

use crate::integration::common::db::TestDatabase;

#[tokio::test]
async fn test_schema_isolation_between_tests() {
    // This verifies that each test gets a fresh schema
    let db1 = TestDatabase::new().await;
    let db2 = TestDatabase::new().await;

    let schema1: String = sqlx::query_scalar("SELECT current_schema()").fetch_one(&db1.pool).await.unwrap();
    let schema2: String = sqlx::query_scalar("SELECT current_schema()").fetch_one(&db2.pool).await.unwrap();
    
    assert_ne!(schema1, schema2);
}
