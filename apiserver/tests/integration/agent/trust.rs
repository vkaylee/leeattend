//! Agent trust lifecycle integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_authorized_agent_lifecycle() {
    let (address, workspace_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    // 1. Khai báo Agent mới
    let agent_id = "AGENT_TEST_001";
    let body = serde_json::json!({
        "id": agent_id,
        "name": "Test Branch",
        "token": "secret123"
    });

    let response = client
        .post(format!("{address}/api/v1/authorized-agents"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 201);

    // 2. Kiểm tra danh sách (phải có agent vừa tạo và is_trusted = false)
    let response = client
        .get(format!("{address}/api/v1/authorized-agents"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    let test_agent = agents
        .iter()
        .find(|a| a["id"] == agent_id)
        .expect("Agent not found");
    assert_eq!(test_agent["is_trusted"], false);

    // 3. Phê duyệt Agent (Trust)
    let response = client
        .put(format!("{address}/api/v1/authorized-agents/{agent_id}/trust"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "is_trusted": true }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    // 4. Kiểm tra lại trạng thái
    let response = client
        .get(format!("{address}/api/v1/authorized-agents"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    let test_agent = agents.iter().find(|a| a["id"] == agent_id).unwrap();
    assert_eq!(test_agent["is_trusted"], true);
}
