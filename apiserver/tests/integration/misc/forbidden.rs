//! Forbidden Access integration tests

use crate::common::app::spawn_app;
use reqwest::StatusCode;

#[tokio::test]
async fn test_access_without_token_is_forbidden() {
    let (address, _test_db) = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/v1/workspaces", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Should be Unauthorized or Forbidden depending on endpoint config
    // Assuming 401 Unauthorized for missing token
    assert!(
        response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::FORBIDDEN
    );
}
