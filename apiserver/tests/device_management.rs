mod common;
use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use common::factory::TestFactory;
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

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let factory = TestFactory::new(db_pool.clone());
    factory.cleanup().await;

    let user_id = factory
        .user()
        .with_email("device-test@example.com")
        .create()
        .await;
    let workspace_id = factory
        .workspace()
        .with_name("Device Test Workspace")
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
    )
}

#[tokio::test]
#[serial]
async fn test_create_and_get_device() {
    let (address, workspace_id, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    let body = serde_json::json!({
        "agent_id": "TEST_AGENT",
        "name": "Test Device",
        "ip": "192.168.1.50",
        "port": 4370
    });

    let response = client
        .post(format!("{}/api/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 201);
    let json: serde_json::Value = response.json().await.unwrap();
    let device_id = json["id"].as_str().unwrap();
    assert!(Uuid::parse_str(device_id).is_ok());

    let response = client
        .get(format!("{}/api/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
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
    let (address, workspace_id, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    let response = client
        .post(format!("{}/api/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
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
        .delete(format!("{}/api/devices/{}", address, id))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 204);

    let response = client
        .get(format!("{}/api/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let devices: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(devices.len(), 0);
}

#[tokio::test]
#[serial]
async fn test_update_device() {
    let (address, workspace_id, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    let res = client
        .post(format!("{}/api/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&serde_json::json!({
            "agent_id": "TEST_AGENT",
            "name": "Old Name",
            "ip": "1.1.1.1"
        }))
        .send()
        .await
        .unwrap();
    let id = res.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let update_body = serde_json::json!({
        "name": "New Name",
        "is_enabled": false
    });

    let response = client
        .put(format!("{}/api/devices/{}", address, id))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .json(&update_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);

    // Verify updated
    let response = client
        .get(format!("{}/api/devices", address))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();
    let devices: Vec<serde_json::Value> = response.json().await.unwrap();
    assert_eq!(devices[0]["name"], "New Name");
    assert_eq!(devices[0]["is_enabled"], false);
}

#[tokio::test]
#[serial]
async fn test_delete_device_not_found() {
    let (address, workspace_id, user_id, _) = spawn_app().await;
    let client = reqwest::Client::new();
    let ws_header = ("X-Workspace-Id", workspace_id.to_string());
    let user_header = ("X-User-Id", user_id.to_string());

    let response = client
        .delete(format!("{}/api/devices/{}", address, Uuid::new_v4()))
        .header(ws_header.0, ws_header.1.clone())
        .header(user_header.0, user_header.1.clone())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 404);
}
