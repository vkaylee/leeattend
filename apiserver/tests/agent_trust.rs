use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::MemberRole;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, Uuid, Uuid) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Run migrations before testing
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    // Dọn dẹp dữ liệu cũ trước khi test
    sqlx::query("DELETE FROM workspace_members")
        .execute(&db_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM devices")
        .execute(&db_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM authorized_agents")
        .execute(&db_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users")
        .execute(&db_pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM workspaces")
        .execute(&db_pool)
        .await
        .unwrap();

    // Create a test user
    let user_id =
        sqlx::query_scalar("INSERT INTO users (email, password_hash) VALUES ($1, $2) RETURNING id")
            .bind("trust-test@example.com")
            .bind("hash")
            .fetch_one(&db_pool)
            .await
            .unwrap();

    // Create a test workspace
    let workspace_id = sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ($1) RETURNING id")
        .bind("Trust Test Workspace")
        .fetch_one(&db_pool)
        .await
        .unwrap();

    // Add user to workspace as owner
    sqlx::query("INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, $3)")
        .bind(workspace_id)
        .bind(user_id)
        .bind(MemberRole::Owner)
        .execute(&db_pool)
        .await
        .unwrap();

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

    // Setup test storage

    let state = Arc::new(AppState {
        agent_manager,
        db: db_pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(db_pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), workspace_id, user_id)
}

#[tokio::test]
#[serial]
async fn test_authorized_agent_lifecycle() {
    let (address, workspace_id, user_id) = spawn_app().await;
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
        .post(format!("{}/api/authorized-agents", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 201);

    // 2. Kiểm tra danh sách (phải có agent vừa tạo và is_trusted = false)
    let response = client
        .get(format!("{}/api/authorized-agents", address))
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
        .put(format!(
            "{}/api/authorized-agents/{}/trust",
            address, agent_id
        ))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({ "is_trusted": true }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    // 4. Kiểm tra lại trạng thái
    let response = client
        .get(format!("{}/api/authorized-agents", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    let test_agent = agents.iter().find(|a| a["id"] == agent_id).unwrap();
    assert_eq!(test_agent["is_trusted"], true);
}
