//! Department CRUD and RBAC integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn test_department_crud_smoke() {
    let (address, workspace_id, user_id, _factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Create a department
    let response = client
        .post(format!(
            "{address}/api/workspaces/{workspace_id}/departments"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&serde_json::json!({
            "name": "Engineering",
            "description": "Building stuff"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let dept: serde_json::Value = response.json().await.unwrap();
    let dept_id = dept["id"].as_str().unwrap();
    assert_eq!(dept["name"], "Engineering");

    // 2. List departments
    let response = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/departments"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let depts: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(depts.iter().any(|d| d["id"] == dept_id));

    // 3. Update department
    let response = client
        .patch(format!(
            "{address}/api/workspaces/{workspace_id}/departments/{dept_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&serde_json::json!({
            "name": "Software Engineering"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated_dept: serde_json::Value = response.json().await.unwrap();
    assert_eq!(updated_dept["name"], "Software Engineering");

    // 4. Delete department
    let response = client
        .delete(format!(
            "{address}/api/workspaces/{workspace_id}/departments/{dept_id}"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // 5. Verify deletion
    let response = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/departments"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .unwrap();

    let depts: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(!depts.iter().any(|d| d["id"] == dept_id));
}

#[tokio::test]
async fn test_department_rbac() {
    let (address, workspace_id, _owner_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Create a regular member
    let member_id: Uuid = factory.user().create().await;
    factory
        .add_member(
            workspace_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    // Member tries to create a department - should fail
    let response = client
        .post(format!(
            "{address}/api/workspaces/{workspace_id}/departments"
        ))
        .header("X-User-Id", member_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&serde_json::json!({
            "name": "Illegal Dept"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Owner creates a department
    let dept_id: Uuid = factory
        .department()
        .with_workspace(workspace_id)
        .with_name("Owner Dept")
        .create()
        .await;

    // Member tries to update - should fail
    let response = client
        .patch(format!(
            "{address}/api/workspaces/{workspace_id}/departments/{dept_id}"
        ))
        .header("X-User-Id", member_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .json(&serde_json::json!({
            "name": "Member Hijacked"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // Member tries to delete - should fail
    let response = client
        .delete(format!(
            "{address}/api/workspaces/{workspace_id}/departments/{dept_id}"
        ))
        .header("X-User-Id", member_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
