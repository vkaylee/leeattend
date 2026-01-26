mod common;
use apiserver::services::AttendanceLogService;
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
        .with_email("theme_test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("Theme Test Workspace")
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
        db: db_pool,
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
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
async fn test_workspace_theme_color_update() {
    let (address, workspace_id, user_id, _, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Kiểm tra màu mặc định (Violet #7c3aed)
    let response = client
        .get(format!("{}/api/workspaces/{}", address, workspace_id))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    assert_eq!(workspace["theme_primary_color"], "#7c3aed");

    // 2. Cập nhật màu mới (ví dụ: Emerald #059669)
    let new_color = "#059669";
    let response = client
        .patch(format!("{}/api/workspaces/{}", address, workspace_id))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "theme_primary_color": new_color }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    assert_eq!(workspace["theme_primary_color"], new_color);

    // 3. Kiểm tra lại bằng GET
    let response = client
        .get(format!("{}/api/workspaces/{}", address, workspace_id))
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let workspace: serde_json::Value = response.json().await.unwrap();
    assert_eq!(workspace["theme_primary_color"], new_color);
}
