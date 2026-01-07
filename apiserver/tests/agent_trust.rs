use sqlx::postgres::PgPoolOptions;
use apiserver::create_app;
use std::sync::Arc;
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use serial_test::serial;

async fn spawn_app() -> String {
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
    sqlx::query("DELETE FROM authorized_agents").execute(&db_pool).await.unwrap();

    let state = Arc::new(AppState {
        agent_manager: AgentManager::new(),
        db: db_pool,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
#[serial]
async fn test_authorized_agent_lifecycle() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Khai báo Agent mới
    let agent_id = "AGENT_TEST_001";
    let body = serde_json::json!({
        "id": agent_id,
        "name": "Test Branch",
        "token": "secret123"
    });

    let response = client
        .post(&format!("{}/authorized-agents", address))
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 201);

    // 2. Kiểm tra danh sách (phải có agent vừa tạo và is_trusted = false)
    let response = client
        .get(&format!("{}/authorized-agents", address))
        .send()
        .await
        .unwrap();

    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    let test_agent = agents.iter().find(|a| a["id"] == agent_id).expect("Agent not found");
    assert_eq!(test_agent["is_trusted"], false);

    // 3. Phê duyệt Agent (Trust)
    let response = client
        .put(&format!("{}/authorized-agents/{}/trust", address, agent_id))
        .json(&serde_json::json!({ "is_trusted": true }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    // 4. Kiểm tra lại trạng thái
    let response = client
        .get(&format!("{}/authorized-agents", address))
        .send()
        .await
        .unwrap();

    let agents: Vec<serde_json::Value> = response.json().await.unwrap();
    let test_agent = agents.iter().find(|a| a["id"] == agent_id).unwrap();
    assert_eq!(test_agent["is_trusted"], true);
}
