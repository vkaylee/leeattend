mod common;
use apiserver::services::AttendanceLogService;
use apiserver::models::MemberRole;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use common::db::TestDatabase;
use common::factory::TestFactory;
use reqwest::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid, TestFactory, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let db_pool = test_db.pool();

    let factory = TestFactory::new(db_pool.clone());

    let user_id = factory
        .user()
        .with_email("dept_test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("Dept Test Workspace")
        .create_with_owner(user_id)
        .await;

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository {
        pool: db_pool.clone(),
    });
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository {
        pool: db_pool.clone(),
    });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: db_pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(db_pool.clone(), 100, 10, 1000).0),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (
        format!("http://127.0.0.1:{}", port),
        workspace_id,
        user_id,
        factory,
        test_db,
    )
}

#[tokio::test]
async fn test_department_crud() {
    let (address, workspace_id, user_id, _factory, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Create a department
    let response = client
        .post(format!(
            "{}/api/workspaces/{}/departments",
            address, workspace_id
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
            "{}/api/workspaces/{}/departments",
            address, workspace_id
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
            "{}/api/workspaces/{}/departments/{}",
            address, workspace_id, dept_id
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
            "{}/api/workspaces/{}/departments/{}",
            address, workspace_id, dept_id
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
            "{}/api/workspaces/{}/departments",
            address, workspace_id
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
    let (address, workspace_id, _owner_id, factory, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    // Create a regular member
    let member_id = factory.user().create().await;
    factory
        .add_member(workspace_id, member_id, MemberRole::Member)
        .await;

    // Member tries to create a department - should fail
    let response = client
        .post(format!(
            "{}/api/workspaces/{}/departments",
            address, workspace_id
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
    let dept_id = factory
        .department()
        .with_workspace(workspace_id)
        .with_name("Owner Dept")
        .create()
        .await;

    // Member tries to update - should fail
    let response = client
        .patch(format!(
            "{}/api/workspaces/{}/departments/{}",
            address, workspace_id, dept_id
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
            "{}/api/workspaces/{}/departments/{}",
            address, workspace_id, dept_id
        ))
        .header("X-User-Id", member_id.to_string())
        .header("X-Workspace-Id", workspace_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}
