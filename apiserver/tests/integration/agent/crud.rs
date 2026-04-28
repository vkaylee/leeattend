//! Agent CRUD integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn test_agent_crud_operations_smoke() {
    let (address, workspace_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    // 1. Khai báo Agent mới
    let agent_id = "AGENT_CRUD_001";
    let create_body = serde_json::json!({
        "id": agent_id,
        "name": "Initial Name",
        "token": "initial_token"
    });

    let response = client
        .post(format!("{address}/api/authorized-agents"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&create_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // 2. Check initial retention_days (default 365)
    let response = client
        .get(format!("{address}/api/authorized-agents/{agent_id}"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent["retention_days"], 365);

    // 3. Update Agent (including retention_days)
    let update_body = serde_json::json!({
        "name": "Updated Name",
        "retention_days": 180
    });

    let response = client
        .patch(format!("{address}/api/authorized-agents/{agent_id}"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&update_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 4. Verify updated retention_days
    let response = client
        .get(format!("{address}/api/authorized-agents/{agent_id}"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    let fetched_agent: serde_json::Value = response.json().await.unwrap();
    assert_eq!(fetched_agent["name"], "Updated Name");
    assert_eq!(fetched_agent["retention_days"], 180);

    // 5. Soft Delete
    let response = client
        .delete(format!("{address}/api/authorized-agents/{agent_id}"))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_workspace_isolation() {
    let (address, workspace_id_1, user_id, _, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let user_header = ("X-User-Id", user_id.to_string());

    // Create second workspace
    let ws_2_resp = client
        .post(format!("{address}/api/workspaces"))
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "name": "Second Workspace" }))
        .send()
        .await
        .unwrap();
    let ws_2: serde_json::Value = ws_2_resp.json().await.unwrap();
    let workspace_id_2 = Uuid::parse_str(ws_2["id"].as_str().unwrap()).unwrap();

    // 1. Create agent in WS 1
    client
        .post(format!("{address}/api/authorized-agents"))
        .header("X-Workspace-Id", workspace_id_1.to_string())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "id": "A1", "name": "Agent 1", "token": "T1" }))
        .send()
        .await
        .unwrap();

    // 2. Create agent in WS 2
    client
        .post(format!("{address}/api/authorized-agents"))
        .header("X-Workspace-Id", workspace_id_2.to_string())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "id": "A2", "name": "Agent 2", "token": "T2" }))
        .send()
        .await
        .unwrap();

    // 3. List agents from WS 1
    let resp_1 = client
        .get(format!("{address}/api/authorized-agents"))
        .header("X-Workspace-Id", workspace_id_1.to_string())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let agents_1: Vec<serde_json::Value> = resp_1.json().await.unwrap();
    assert_eq!(agents_1.len(), 1);
    assert_eq!(agents_1[0]["id"], "A1");

    // 4. List agents from WS 2
    let resp_2 = client
        .get(format!("{address}/api/authorized-agents"))
        .header("X-Workspace-Id", workspace_id_2.to_string())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let agents_2: Vec<serde_json::Value> = resp_2.json().await.unwrap();
    assert_eq!(agents_2.len(), 1);
    assert_eq!(agents_2[0]["id"], "A2");
}

#[tokio::test]
async fn test_rbac_member_cannot_invite() {
    let (address, workspace_id, _owner_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Create a "Member" user using factory
    let member_user_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(workspace_id, member_user_id, uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap())
        .await;

    // 3. Try to invite someone as this member
    let response = client
        .post(format!("{address}/api/workspaces/{workspace_id}/members"))
        .header("X-User-Id", member_user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&serde_json::json!({ "email": "other@test.com", "role_id": "a0000000-0000-0000-0000-000000000002" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
