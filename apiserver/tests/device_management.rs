use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;
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

    // Clean up devices before each test
    sqlx::query("DELETE FROM devices").execute(&db_pool).await.unwrap();

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
async fn test_create_and_get_device() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "agent_id": "TEST_AGENT",
        "name": "Test Device",
        "ip": "192.168.1.50",
        "port": 4370
    });

    let response = client
        .post(&format!("{}/devices", address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201);
    let json: serde_json::Value = response.json().await.unwrap();
    let device_id = json["id"].as_str().unwrap();
    assert!(Uuid::parse_str(device_id).is_ok());

    let response = client
        .get(&format!("{}/devices", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);
    let devices: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(devices.len(), 1);
}

#[tokio::test]
#[serial]
async fn test_delete_device() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/devices", address))
        .json(&serde_json::json!({
            "agent_id": "TEST_AGENT",
            "name": "To Delete",
            "ip": "1.1.1.1"
        }))
        .send()
        .await
        .unwrap();
    
    let json: serde_json::Value = response.json().await.unwrap();
    let id = json["id"].as_str().unwrap();

    let response = client
        .delete(&format!("{}/devices/{}", address, id))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 204);

    let response = client.get(&format!("{}/devices", address)).send().await.unwrap();
    let devices: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(devices.len(), 0);
}
