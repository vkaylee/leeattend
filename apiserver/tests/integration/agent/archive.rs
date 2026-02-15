//! Agent archive and restore integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn test_agent_archive_and_restore() {
    let (address, workspace_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let agent_id = "AGENT_ARCHIVE_01";
    factory
        .agent()
        .with_id(agent_id)
        .with_workspace(workspace_id)
        .create()
        .await;

    // 1. Soft Delete (Archive)
    let response = client
        .delete(format!("{}/api/authorized-agents/{}", address, agent_id))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 2. List agents (should be empty by default)
    let response = client
        .get(format!("{}/api/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(agents.len(), 0);

    // 3. List agents with include_inactive=true
    let response = client
        .get(format!(
            "{}/api/authorized-agents?include_inactive=true",
            address
        ))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["id"], agent_id);
    assert_eq!(agents[0]["is_active"], false);

    // 4. Restore Agent
    let response = client
        .patch(format!("{}/api/authorized-agents/{}", address, agent_id))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "is_active": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. List agents again (should now show up)
    let response = client
        .get(format!("{}/api/authorized-agents", address))
        .header("X-Workspace-Id", workspace_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();
    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0]["is_active"], true);
}
