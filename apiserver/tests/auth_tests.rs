use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::AppState;
use apiserver::models::{RegisterRequest, LoginRequest, ChangePasswordRequest, AuthResponse};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

async fn spawn_app() -> (String, sqlx::PgPool) {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // Cleanup before test
    sqlx::query("DELETE FROM sessions").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM workspace_members").execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM users WHERE email = $1").bind("test_change@example.com").execute(&pool).await.unwrap();

    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_123".to_string(),
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), pool)
}

#[tokio::test]
async fn test_change_password_flow() {
    let (address, _pool) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Register
    let reg_req = RegisterRequest {
        email: "test_change@example.com".to_string(),
        password: "old_password_123".to_string(),
        full_name: "Test User".to_string(),
    };
    let res = client.post(&format!("{}/auth/register", address))
        .json(&reg_req)
        .send().await.unwrap();
    assert_eq!(res.status(), 200);

    // 2. Login
    let login_req = LoginRequest {
        email: "test_change@example.com".to_string(),
        password: "old_password_123".to_string(),
    };
    let res = client.post(&format!("{}/auth/login", address))
        .json(&login_req)
        .send().await.unwrap();
    assert_eq!(res.status(), 200);
    let auth_res: AuthResponse = res.json().await.unwrap();
    let token = auth_res.access_token;

    // 3. Change Password
    let change_req = ChangePasswordRequest {
        old_password: "old_password_123".to_string(),
        new_password: "new_password_456".to_string(),
    };
    let res = client.post(&format!("{}/auth/change-password", address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&change_req)
        .send().await.unwrap();
    assert_eq!(res.status(), 200);

    // 4. Try login with OLD password (should fail)
    let login_req_old = LoginRequest {
        email: "test_change@example.com".to_string(),
        password: "old_password_123".to_string(),
    };
    let res = client.post(&format!("{}/auth/login", address))
        .json(&login_req_old)
        .send().await.unwrap();
    assert_eq!(res.status(), 401);

    // 5. Try login with NEW password (should succeed)
    let login_req_new = LoginRequest {
        email: "test_change@example.com".to_string(),
        password: "new_password_456".to_string(),
    };
    let res = client.post(&format!("{}/auth/login", address))
        .json(&login_req_new)
        .send().await.unwrap();
    assert_eq!(res.status(), 200);
}
