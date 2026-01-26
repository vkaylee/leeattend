mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::services::auth_service::AuthService;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_context() -> (TestDatabase, TestFactory, Arc<AppState>) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret_session".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
    });

    (test_db, factory, app_state)
}

#[tokio::test]
async fn test_auth_session_lifecycle() {
    let (_test_db, factory, state) = setup_context().await;
    let user_id = factory.user().create().await;

    // 1. Create session
    let refresh_token = AuthService::create_session(&state.db, user_id)
        .await
        .unwrap();
    assert!(!refresh_token.is_empty());

    // 2. Refresh session
    let auth_res = AuthService::refresh_session(&state.db, &refresh_token, &state.jwt_secret)
        .await
        .unwrap();
    assert_eq!(auth_res.user.id, user_id);
    assert!(!auth_res.access_token.is_empty());
    assert!(!auth_res.refresh_token.is_empty());
    assert_ne!(auth_res.refresh_token, refresh_token); // Token rotation

    let new_refresh_token = auth_res.refresh_token;

    // 3. Verify access token
    let verified_id = AuthService::verify_token(&auth_res.access_token, &state.jwt_secret).unwrap();
    assert_eq!(verified_id, user_id);

    // 4. Revoke session
    AuthService::revoke_session(&state.db, &new_refresh_token)
        .await
        .unwrap();

    // 5. Try to refresh again (should fail)
    let result =
        AuthService::refresh_session(&state.db, &new_refresh_token, &state.jwt_secret).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid or expired refresh token");
}

#[tokio::test]
async fn test_revoke_all_sessions() {
    let (_test_db, factory, state) = setup_context().await;
    let user_id = factory.user().create().await;

    let token1 = AuthService::create_session(&state.db, user_id)
        .await
        .unwrap();
    let token2 = AuthService::create_session(&state.db, user_id)
        .await
        .unwrap();

    // Revoke all
    AuthService::revoke_all_sessions_for_user(&state.db, user_id)
        .await
        .unwrap();

    // Verify both are invalid
    assert!(
        AuthService::refresh_session(&state.db, &token1, &state.jwt_secret)
            .await
            .is_err()
    );
    assert!(
        AuthService::refresh_session(&state.db, &token2, &state.jwt_secret)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn test_get_me_and_logout() {
    let (_test_db, factory, state) = setup_context().await;
    let app = apiserver::create_app(state.clone());

    let user_id = factory.user().create().await;
    let refresh_token = AuthService::create_session(&state.db, user_id)
        .await
        .unwrap();

    // 1. Get Me
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/me")
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Logout
    let logout_req = serde_json::json!({
        "refresh_token": refresh_token
    });
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(logout_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Verify session revoked
    let result = AuthService::refresh_session(&state.db, &refresh_token, &state.jwt_secret).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_me_not_found() {
    let (_test_db, _factory, state) = setup_context().await;
    let app = apiserver::create_app(state.clone());

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/me")
                .header("X-User-Id", Uuid::new_v4().to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_refresh_token_api() {
    let (_test_db, factory, state) = setup_context().await;
    let app = apiserver::create_app(state.clone());

    let user_id = factory.user().create().await;
    let refresh_token = AuthService::create_session(&state.db, user_id)
        .await
        .unwrap();

    let req = serde_json::json!({
        "refresh_token": refresh_token
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/auth/refresh")
                .header("Content-Type", "application/json")
                .body(axum::body::Body::from(req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_invalid_refresh_token() {
    let (_test_db, _factory, state) = setup_context().await;
    let result =
        AuthService::refresh_session(&state.db, "non-existent-token", &state.jwt_secret).await;
    assert!(result.is_err());
}
