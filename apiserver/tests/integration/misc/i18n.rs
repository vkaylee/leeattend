//! Integration tests for backend i18n header and query parsing and error response translation.

use crate::common::app::spawn_app;
use reqwest::StatusCode;

#[tokio::test]
async fn test_api_i18n_missing_workspace() {
    let (address, _test_db) = spawn_app().await;
    let client = reqwest::Client::new();
    let user_id = uuid::Uuid::new_v4().to_string();

    // 1. Test default language (should fallback to Vietnamese "vi")
    let response = client
        .get(format!("{}/api/v1/ess/profile", &address))
        .header("X-User-Id", &user_id)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["error"]["code"], "VALIDATION_ERROR");
    let msg = json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("Thiếu thông tin Workspace") || msg.contains("missing_workspace"),
        "Expected Vietnamese missing workspace message, got: {msg}"
    );

    // 2. Test Accept-Language: en
    let response_en = client
        .get(format!("{}/api/v1/ess/profile", &address))
        .header("X-User-Id", &user_id)
        .header("Accept-Language", "en")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response_en.status(), StatusCode::BAD_REQUEST);
    let json_en: serde_json::Value = response_en.json().await.unwrap();
    assert_eq!(
        json_en["error"]["message"],
        "Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header."
    );

    // 3. Test Accept-Language: zh
    let response_zh = client
        .get(format!("{}/api/v1/ess/profile", &address))
        .header("X-User-Id", &user_id)
        .header("Accept-Language", "zh-CN")
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response_zh.status(), StatusCode::BAD_REQUEST);
    let json_zh: serde_json::Value = response_zh.json().await.unwrap();
    let msg_zh = json_zh["error"]["message"].as_str().unwrap();
    assert!(
        msg_zh.contains("缺少工作空间上下文"),
        "Expected Chinese missing workspace message, got: {msg_zh}"
    );

    // 4. Test query parameter ?lang=en (takes precedence over Accept-Language)
    let response_query = client
        .get(format!("{}/api/v1/ess/profile?lang=en", &address))
        .header("X-User-Id", &user_id)
        .header("Accept-Language", "vi") // Accept-Language is vi, but query param is en
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response_query.status(), StatusCode::BAD_REQUEST);
    let json_query: serde_json::Value = response_query.json().await.unwrap();
    assert_eq!(
        json_query["error"]["message"],
        "Missing Workspace Context. Provide 'ws_id' in path or 'X-Workspace-Id' header."
    );
}
