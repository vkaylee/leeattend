use apiserver::create_app;
use std::sync::Arc;
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
use sqlx::postgres::PgPoolOptions;

async fn spawn_app() -> String {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    let state = Arc::new(AppState {
        agent_manager: AgentManager::new(),
        db: pool,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
}
