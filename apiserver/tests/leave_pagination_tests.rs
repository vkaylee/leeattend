mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::{PaginatedResponse, WorkspaceLeaveBalance};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
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
async fn test_workspace_leave_balances_pagination() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create 15 employees
    for i in 1..=15 {
        let emp_id = factory.employee()
            .with_workspace(ws_id)
            .with_full_name(&format!("Employee {}", i))
            .create().await;
            
        // Create balance for each
        factory.leave_balance()
            .with_employee(emp_id)
            .create().await;
    }

    // 1. Test Default Pagination (Page 1, Limit 20) - Should get all 15
    // Note: API default limit is 20 if not specified
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/balances", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let resp: PaginatedResponse<WorkspaceLeaveBalance> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(resp.total, 60);
    assert_eq!(resp.data.len(), 20);
    assert_eq!(resp.page, 1);

    // 2. Test Limit 5, Page 1
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/balances?limit=5&page=1", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let resp: PaginatedResponse<WorkspaceLeaveBalance> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(resp.total, 60);
    assert_eq!(resp.data.len(), 5);
    assert_eq!(resp.page, 1);
    
    // 3. Test Limit 5, Page 2
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/balances?limit=5&page=2", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
        
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let resp: PaginatedResponse<WorkspaceLeaveBalance> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(resp.total, 60);
    assert_eq!(resp.data.len(), 5);
    assert_eq!(resp.page, 2);

    // 4. Test Search
    // Create specific employee
    let target_emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_full_name("Target Person")
        .create().await;
    factory.leave_balance().with_employee(target_emp_id).create().await;

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/balances?q=target", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let resp: PaginatedResponse<WorkspaceLeaveBalance> = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(resp.total, 4);
    assert_eq!(resp.data.len(), 4);
    assert_eq!(resp.data[0].employee_name, "Target Person");
}
