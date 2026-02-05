//! Health Check integration tests

use crate::integration::common::app::spawn_app;
use reqwest::StatusCode;

#[tokio::test]
async fn health_check_works() {
    let (address, _test_db) = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(Some(0), response.content_length());
}
