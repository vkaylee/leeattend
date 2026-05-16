//! Schema Isolation integration tests

use crate::common::db::TestDatabase;

#[tokio::test]
async fn test_schema_isolation_between_tests() {
    // This verifies that each test gets a fresh database
    let db1 = TestDatabase::new().await;
    let db2 = TestDatabase::new().await;

    let name1: String =
        crate::test_query_scalar_string!(&db1.diesel_pools.api, "SELECT current_database() as val");
    let name2: String =
        crate::test_query_scalar_string!(&db2.diesel_pools.api, "SELECT current_database() as val");

    assert_ne!(name1, name2);
}
