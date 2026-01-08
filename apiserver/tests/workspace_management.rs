mod common;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use common::factory::TestFactory;
use reqwest::StatusCode;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid, TestFactory) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let factory = TestFactory::new(db_pool.clone());
    factory.cleanup().await;

    let user_id = factory
        .user()
        .with_email("ws_test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("Base Workspace")
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
    )
}

#[tokio::test]
#[serial]
async fn test_workspace_creation_and_membership() {
    let (address, _, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Tạo workspace mới
    let response = client
        .post(&format!("{}/workspaces", address))
        .header("X-User-Id", user_id.to_string())
        .json(&serde_json::json!({ "name": "New Ventures" }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let workspace: serde_json::Value = response.json().await.unwrap();
    let ws_id = workspace["id"].as_str().unwrap();

    // 2. Kiểm tra danh sách members của workspace vừa tạo
    let response = client
        .get(&format!("{}/workspaces/{}/members", address, ws_id))
        .header("X-Workspace-Id", ws_id)
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let members: Vec<serde_json::Value> = response.json().await.unwrap();
    assert!(members
        .iter()
        .any(|m| m["user_id"] == user_id.to_string() && m["role"] == "owner"));
}

#[tokio::test]
#[serial]
async fn test_create_workspace_with_non_existent_user() {
    let (address, _, _, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let fake_user_id = Uuid::new_v4();

    let response = client
        .post(&format!("{}/workspaces", address))
        .header("X-User-Id", fake_user_id.to_string())
        .json(&serde_json::json!({ "name": "Ghost Workspace" }))
        .send()
        .await
        .unwrap();

    // Nên trả về 500 kèm lỗi chi tiết (theo logic mới đã sửa ở lượt trước)
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let error_text = response.text().await.unwrap();
    assert!(error_text.contains("does not exist"));
}

#[tokio::test]
#[serial]
async fn test_get_dev_super_user() {
    let (address, _, _, _) = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/dev/super-user", address))
        .send()
        .await
        .unwrap();

    // Chỉ hoạt động trong debug build
    if cfg!(debug_assertions) {
        assert_eq!(response.status(), StatusCode::OK);
        let body: serde_json::Value = response.json().await.unwrap();
        assert!(body.get("user_id").is_some());
        assert!(body.get("workspace_id").is_some());
    } else {
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
