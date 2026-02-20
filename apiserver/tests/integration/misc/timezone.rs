//! Timezone Consistency integration tests

use chrono::Utc;

#[tokio::test]
async fn test_timezone_consistency() {
    // Verify that the server (and tests) run in UTC or handle shifts correctly
    let now = Utc::now();
    let now_local = now.with_timezone(&chrono::Local);

    // Just a sanity check that chrono is working as expected
    assert_eq!(now.timestamp(), now_local.timestamp());
}
