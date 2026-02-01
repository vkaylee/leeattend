mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::MemberRole;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::{Duration, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("apiserver=debug,sqlx=info")
        .with_test_writer()
        .try_init();

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

    let storage_provider = apiserver::services::create_storage_provider(&storage)


    // Setup test storage



    let storage = apiserver::StorageBackend::Local {



        base_path: std::path::PathBuf::from("./test_storage"),



    };



    let storage_provider = apiserver::services::create_storage_provider(&storage)



        .await



        .expect("Failed to create test storage");




    // Setup test storage
    let storage = apiserver::StorageBackend::Local {
        base_path: std::path::PathBuf::from("./test_storage"),
    };
    let storage_provider = apiserver::services::create_storage_provider(&storage)
        .await
        .expect("Failed to create test storage");

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        storage,
        storage_provider,
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_member_roles_and_permissions() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup: Admin User and Member User
    let admin_id = factory.user().create().await;
    let member_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    factory
        .add_member(ws_id, member_id, MemberRole::Member)
        .await;

    let headers = [
        ("X-Workspace-Id", ws_id.to_string()),
        ("X-User-Id", member_id.to_string()),
    ];

    let shift_req = serde_json::json!({
        "name": "Member Created Shift",
        "start_time": "09:00:00",
        "end_time": "17:00:00",
        "break_duration_minutes": 60,
        "break_is_overtime": false,
        "early_punch_in_grace_period_minutes": 0,
        "allow_early_ot": false
    });

    // 2. Member tries to create a shift (should fail)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from(
                    serde_json::to_string(&shift_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    // 3. Add temporary Admin role
    factory
        .add_extra_role(
            ws_id,
            member_id,
            MemberRole::Admin,
            Utc::now() + Duration::hours(1),
        )
        .await;

    // 4. Member tries to create a shift again (should now be OK)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/shifts", ws_id))
                .header("Content-Type", "application/json")
                .header(headers[0].0, &headers[0].1)
                .header(headers[1].0, &headers[1].1)
                .body(axum::body::Body::from(
                    serde_json::to_string(&shift_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}
