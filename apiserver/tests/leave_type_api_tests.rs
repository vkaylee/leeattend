use apiserver::services::AttendanceLogService;
use crate::common::factory::TestFactory;
use apiserver::{agent_manager::AgentManager, create_app, AppState, models::MemberRole};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

mod common;

async fn setup_test_db() -> PgPool {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres")
}

async fn setup_test_app(pool: PgPool) -> axum::Router {
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(agent_repo.clone(), agent_manager.clone()));

    // Setup test storage

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
    });

    create_app(state)
}

#[tokio::test]
async fn test_create_leave_type() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create().await;
    factory.add_member(ws_id, user_id, MemberRole::Admin).await;

    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/types", ws_id))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(json!({
                    "code": "COMP_LEAVE",
                    "name": "Compensatory Leave",
                    "is_paid": true
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_update_leave_type_config_success() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create().await;
    factory.add_member(ws_id, user_id, MemberRole::Admin).await;

    let lt_id: Uuid = sqlx::query_scalar("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = $2")
        .bind(ws_id).bind("vacation").fetch_one(&pool).await.unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/leave/types/{}/config", ws_id, lt_id))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(json!({
                    "policy_config": {
                        "approval_required": true,
                        "requires_document": true,
                        "allow_negative_balance": false,
                        "negative_balance_limit": 0.0
                    }
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    
    // Verify changes
    let doc_required: bool = sqlx::query_scalar("SELECT (policy_config->>'requires_document')::boolean FROM leave_types WHERE id = $1")
        .bind(lt_id).fetch_one(&pool).await.unwrap();
    assert!(doc_required);
}

#[tokio::test]
async fn test_update_leave_type_config_forbidden() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create().await;
    factory.add_member(ws_id, user_id, MemberRole::Member).await; // Normal member

    let lt_id: Uuid = sqlx::query_scalar("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = $2")
        .bind(ws_id).bind("vacation").fetch_one(&pool).await.unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{}/leave/types/{}/config", ws_id, lt_id))
                .header("Content-Type", "application/json")
                .header("X-User-Id", user_id.to_string())
                .body(Body::from(json!({
                    "policy_config": { "approval_required": false }
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_leave_type_fail_when_in_use() {
    let pool = setup_test_db().await;
    let factory = TestFactory::new(pool.clone());
    let app = setup_test_app(pool.clone()).await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create().await;
    factory.add_member(ws_id, user_id, MemberRole::Admin).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let lt_id: Uuid = sqlx::query_scalar("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = $2")
        .bind(ws_id).bind("vacation").fetch_one(&pool).await.unwrap();

    sqlx::query("INSERT INTO leave_balances (employee_id, leave_type_id, year, entitlement, carried_over) VALUES ($1, $2, 2026, 12, 0)")
        .bind(emp_id).bind(lt_id).execute(&pool).await.unwrap();

    let res = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{}/leave/types/{}", ws_id, lt_id))
                .header("X-User-Id", user_id.to_string())
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(res.status(), StatusCode::CONFLICT);
}
