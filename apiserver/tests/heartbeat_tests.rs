mod common;
use apiserver::{agent_manager::AgentManager, create_app, AppState};
use common::factory::TestFactory;
use reqwest::StatusCode;
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};

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

    let user_id = factory.user().with_email("hb_test@example.com").create().await;
    let workspace_id = factory
        .workspace()
        .with_name("HB Workspace")
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
        jwt_secret: "hb_secret".to_string(),
    });

    let app = create_app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}", addr);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (url, user_id, workspace_id, factory)
}

#[tokio::test]
#[serial]
async fn test_agent_heartbeat_success() {
    let (url, _, workspace_id, factory) = spawn_app().await;
    let client = reqwest::Client::new();

    let agent_id = "HB_AGENT_01".to_string();
    let token = "hb_token_123".to_string();

    // 1. Khai báo agent
    factory.agent()
        .with_id(&agent_id)
        .with_token(&token)
        .with_workspace(workspace_id)
        .with_trust(true)
        .create()
        .await;

    // 2. Gửi heartbeat
    let response = client
        .post(format!("{}/agents/heartbeat", url))
        .header("X-Agent-Id", &agent_id)
        .header("X-Agent-Token", &token)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::OK);

    // 3. Kiểm tra DB xem last_seen_at có được cập nhật không
    let last_seen: chrono::DateTime<Utc> = sqlx::query_scalar("SELECT last_seen_at FROM authorized_agents WHERE id = $1")
        .bind(&agent_id)
        .fetch_one(&factory.pool)
        .await
        .unwrap();

    let now = Utc::now();
    let diff = if now > last_seen { now - last_seen } else { last_seen - now };
    
    println!("Now: {}, Last Seen: {}, Diff: {}ms", now, last_seen, diff.num_milliseconds());
    
    // Nới lỏng kiểm tra vì có thể có lệch múi giờ giữa client test và DB
    assert!(diff < Duration::seconds(60));
}

#[tokio::test]
#[serial]
async fn test_agent_heartbeat_unauthorized() {
    let (url, _, _, _) = spawn_app().await;
    let client = reqwest::Client::new();

    // Gửi với token sai
    let response = client
        .post(format!("{}/agents/heartbeat", url))
        .header("X-Agent-Id", "UNKNOWN")
        .header("X-Agent-Token", "WRONG")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
