//! Agent token update integration tests

use crate::common::app::spawn_app_with_factory;
use reqwest::StatusCode;

use uuid::Uuid;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;

#[tokio::test]
async fn test_agent_token_update() {
    let (address, workspace_id, user_id, _, _tdb): (String, Uuid, Uuid, TestFactory, TestDatabase) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let agent_id = "TOKEN_UPDATE_AGENT";
    let initial_token = "initial_secret";
    let new_token = "updated_secret_2026";

    // 1. Khai báo agent
    client
        .post(format!("{address}/api/authorized-agents"))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({
            "id": agent_id,
            "name": "Test Agent",
            "token": initial_token
        }))
        .send()
        .await
        .unwrap();

    // 2. Cập nhật token
    let response = client
        .patch(format!("{address}/api/authorized-agents/{agent_id}"))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({
            "token": new_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated_agent["token"], new_token);

    // 3. Kiểm tra lại bằng GET
    let response = client
        .get(format!("{address}/api/authorized-agents/{agent_id}"))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    let fetched_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent["token"], new_token);
}
