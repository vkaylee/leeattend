mod common;

use axum::http::StatusCode;
use chrono::Utc;
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::models::{LeaveRequest, SubmitLeaveRequest, ApproveLeaveRequest, ShiftRoster, EmployeeLeaveBalance};
use apiserver::AppState;
use apiserver::agent_manager::AgentManager;
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
    
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(pool.clone()));
    let workspace_repo = Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(agent_repo.clone(), agent_manager.clone()));
    
    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
    });
    
    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

#[tokio::test]
async fn test_leave_request_lifecycle() {
    let (_test_db, factory, app) = setup_app().await;

    // 1. Setup: User, Workspace, Employee
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 2. Submit Leave Request
    let start_date = Utc::now().date_naive();
    let end_date = Utc::now().date_naive();
    
    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type: "annual".to_string(),
        start_date,
        end_date,
        reason: Some("Family vacation".to_string()),
    };

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/submit", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&submit_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    println!("Submitted leave request successfully");
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&body).unwrap();
    assert_eq!(leave_req.status, "pending");
    assert_eq!(leave_req.employee_id, emp_id);

    // 3. Get My Leave Requests
    println!("Fetching my leave requests...");
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/employees/{}/leave/requests", emp_id))
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    println!("Fetched my leave requests successfully");
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let requests: Vec<LeaveRequest> = serde_json::from_slice(&body).unwrap();
    assert!(!requests.is_empty());
    assert_eq!(requests[0].id, leave_req.id);

    // 4. Approve Leave Request
    println!("Approving leave request...");
    let approve_req = ApproveLeaveRequest {
        status: "approved".to_string(),
    };

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/requests/{}/approve", ws_id, leave_req.id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&approve_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify Roster Update
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees/{}/rosters", ws_id, emp_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let rosters: Vec<ShiftRoster> = serde_json::from_slice(&body).unwrap();
    assert!(!rosters.is_empty());
    assert_eq!(rosters[0].status, "on_leave");
}

#[tokio::test]
async fn test_reject_leave_request() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let submit_req = SubmitLeaveRequest {
        employee_id: emp_id,
        leave_type: "annual".to_string(),
        start_date: Utc::now().date_naive(),
        end_date: Utc::now().date_naive(),
        reason: Some("Reject me".to_string()),
    };

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/submit", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&submit_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    let leave_req: LeaveRequest = serde_json::from_slice(&axum::body::to_bytes(response.into_body(), 10000).await.unwrap()).unwrap();

    // Reject
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/requests/{}/approve", ws_id, leave_req.id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&serde_json::json!({ "status": "rejected" })).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Verify rejected in DB
    let fetched = apiserver::repositories::leave_request_repository::get_leave_request_by_id(&factory.pool, leave_req.id).await.unwrap().unwrap();
    assert_eq!(fetched.status, "rejected");
}

#[tokio::test]
async fn test_submit_leave_request_non_existent_employee() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let submit_req = serde_json::json!({
        "employee_id": uuid::Uuid::new_v4(),
        "leave_type": "annual",
        "start_date": "2026-01-01",
        "end_date": "2026-01-01"
    });

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/submit", ws_id))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(submit_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_approve_non_existent_leave_request() {
    let (_test_db, factory, app) = setup_app().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/leave/requests/{}/approve", ws_id, uuid::Uuid::new_v4()))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from("{\"status\":\"approved\"}"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}



#[tokio::test]
async fn test_leave_balances() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    factory.leave_balance()
        .with_employee(emp_id)
        .with_entitlement(15)
        .create()
        .await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/employees/{}/leave/balances", ws_id, emp_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let balances: Vec<EmployeeLeaveBalance> = serde_json::from_slice(&body).unwrap();
    assert!(!balances.is_empty());
    assert_eq!(balances[0].entitlement, 15);
    assert_eq!(balances[0].remaining, 15.0);
}

#[tokio::test]
async fn test_workspace_leave_requests_filter() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    factory.leave_request().with_employee(emp_id).with_status("pending").create().await;
    factory.leave_request().with_employee(emp_id).with_status("approved").create().await;

    // Filter by pending
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/requests?status=pending", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let requests: Vec<LeaveRequest> = serde_json::from_slice(&body).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].status, "pending");

    // Filter by approved
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/workspaces/{}/leave/requests?status=approved", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let requests: Vec<LeaveRequest> = serde_json::from_slice(&body).unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].status, "approved");
}
