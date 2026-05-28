//! Health Check integration tests

use crate::common::app::spawn_app;
use reqwest::StatusCode;

#[tokio::test]
async fn health_check_works() {
    let (address, _test_db) = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/api/v1/health", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::OK);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["message"], "API Server is running!");
}
