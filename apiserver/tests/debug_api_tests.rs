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
        .with_email("debug_test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("Debug Workspace")
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
async fn test_clear_attendance_data() {
    let (address, ws_id, user_id, factory, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Setup some data
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Add time entries
    sqlx::query("INSERT INTO time_entries (employee_id, workspace_id, punch_in_time) VALUES ($1, $2, NOW())")
        .bind(emp_id)
        .bind(ws_id)
        .execute(&_db.pool())
        .await
        .unwrap();

    // Verify data exists
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM time_entries WHERE employee_id = $1")
        .bind(emp_id)
        .bind(ws_id)
        .fetch_one(&_db.pool())
        .await
        .unwrap();
    assert_eq!(count, 1);

    // 2. Clear attendance data
    if cfg!(debug_assertions) {
        let response = client
            .post(format!(
                "{}/api/dev/workspaces/{}/clear?group=attendance",
                address, ws_id
            ))
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", user_id.to_string())
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // 3. Verify data is gone
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM time_entries WHERE employee_id = $1")
                .bind(emp_id)
        .bind(ws_id)
                .fetch_one(&_db.pool())
                .await
                .unwrap();
        assert_eq!(count, 0);
    }
}

#[tokio::test]
async fn test_full_reset() {
    let (address, ws_id, user_id, factory, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Setup data
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 2. Full reset
    if cfg!(debug_assertions) {
        let response = client
            .post(format!(
                "{}/api/dev/workspaces/{}/clear?group=full_reset",
                address, ws_id
            ))
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", user_id.to_string())
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // 3. Verify employee is gone
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM employees WHERE id = $1)")
                .bind(emp_id)
        .bind(ws_id)
                .fetch_one(&_db.pool())
                .await
                .unwrap();
        assert!(!exists);
    }
}

#[tokio::test]
async fn test_clear_invalid_group() {
    let (address, ws_id, user_id, _, _db) = spawn_app().await;
    let client = reqwest::Client::new();

    if cfg!(debug_assertions) {
        let response = client
            .post(format!(
                "{}/api/dev/workspaces/{}/clear?group=invalid",
                address, ws_id
            ))
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", user_id.to_string())
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
