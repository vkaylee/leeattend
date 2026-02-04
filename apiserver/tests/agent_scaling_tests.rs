use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use apiserver::models::MemberRole;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt; 
use uuid::Uuid;
use serde_json::json;


async fn spawn_app() -> (Arc<AppState>, Uuid, String) {
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

    // Clean up
    sqlx::query("DELETE FROM authorized_agents").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM workspace_members").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM workspaces").execute(&db_pool).await.unwrap();
    sqlx::query("DELETE FROM users").execute(&db_pool).await.unwrap();

    let workspace_id = Uuid::new_v4();
    sqlx::query("INSERT INTO workspaces (id, name) VALUES ($1, 'Test Workspace')")
        .bind(workspace_id)
        .execute(&db_pool)
        .await
        .unwrap();

    // Create User
    let user_id = Uuid::new_v4();
    let email = "admin@example.com";
    sqlx::query("INSERT INTO users (id, email, password_hash) VALUES ($1, $2, 'hash')")
        .bind(user_id)
        .bind(email)
        .execute(&db_pool)
        .await
        .unwrap();

    // Add as Owner
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
        roster_cache: None,
    });

    // Generate token
    let token = apiserver::services::auth_service::AuthService::generate_token(user_id, &state.jwt_secret, 3600).unwrap();

    (state, workspace_id, token)
}

#[tokio::test]
async fn test_agent_scaling_config() {
    let (state, workspace_id, token) = spawn_app().await;
    let app = create_app(state.clone());

    // 1. Create Agent with default workers (2)
    let agent_id = "AGENT_SCALE_TEST";
    let create_req = json!({
        "id": agent_id,
        "name": "Scaling Agent",
        "token": "secret_token",
        // workspace_id is inferred from context if not provided in body, but handler expects it?
        // Actually handler create_authorized_agent takes CreateAuthorizedAgentRequest which has optional workspace_id
        // But the context provides workspace_id.
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/authorized-agents")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .body(Body::from(create_req.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify DB default
    let agent = sqlx::query!("SELECT max_concurrent_workers FROM authorized_agents WHERE id = $1", agent_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(agent.max_concurrent_workers, 2);

    // 2. Update to 5 workers
    let update_req = json!({
        "max_concurrent_workers": 5
    });

    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/authorized-agents/{}", agent_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .body(Body::from(update_req.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify DB update
    let agent = sqlx::query!("SELECT max_concurrent_workers FROM authorized_agents WHERE id = $1", agent_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(agent.max_concurrent_workers, 5);

    // 3. Update to 1 worker
    let update_req = json!({
        "max_concurrent_workers": 1
    });

    let req = Request::builder()
        .method("PATCH")
        .uri(format!("/api/authorized-agents/{}", agent_id))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", token))
        .header("X-Workspace-Id", workspace_id.to_string())
        .body(Body::from(update_req.to_string()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify DB update
    let agent = sqlx::query!("SELECT max_concurrent_workers FROM authorized_agents WHERE id = $1", agent_id)
        .fetch_one(&state.db)
        .await
        .unwrap();
    assert_eq!(agent.max_concurrent_workers, 1);
}
