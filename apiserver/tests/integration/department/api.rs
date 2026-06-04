//! Department API integration tests (Tower-based)

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{Department, EmployeeDepartmentHistory};
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Utc;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_department_management() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    // 1. Setup: User, Workspace, Employee
    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // 2. Create Department
    let dept_req = serde_json::json!({
        "name": "Engineering"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/departments"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&dept_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dept: Department = serde_json::from_slice(&body).unwrap();
    assert_eq!(dept.name, "Engineering");

    // 3. List Departments
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/departments"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let paginated: apiserver::models::PaginatedResponse<Department> =
        serde_json::from_slice(&body).unwrap();
    let depts = paginated.data;
    assert!(!depts.is_empty());

    // 4. Transfer Employee to Department
    let transfer_req = serde_json::json!({
        "department_id": dept.id,
        "effective_date": "2026-01-10"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&transfer_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 5. Get Employee Department History
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let history: Vec<EmployeeDepartmentHistory> = serde_json::from_slice(&body).unwrap();
    assert!(!history.is_empty());
    assert_eq!(history[0].department_id, dept.id);
}

#[tokio::test]
async fn test_department_working_pattern_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Create a pattern first
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&_test_db.url());
    let repo = apiserver::repositories::DieselWorkingPatternRepository::new(diesel_pools);
    use apiserver::repositories::working_pattern_repository::WorkingPatternRepositoryTrait;

    let pattern = repo
        .create_pattern_with_details(ws_id, "Dept Pattern", None, "weekly", None, vec![])
        .await
        .unwrap();

    // 1. Create Department with Pattern
    let dept_req = serde_json::json!({
        "name": "HR",
        "default_working_pattern_id": pattern.id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/departments"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&dept_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let dept: Department = serde_json::from_slice(&body).unwrap();
    assert_eq!(dept.default_working_pattern_id, Some(pattern.id));

    // 2. Update Department Pattern
    let update_req = serde_json::json!({
        "default_working_pattern_id": null
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!(
                    "/api/v1/workspaces/{}/departments/{}",
                    ws_id, dept.id
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_string(&update_req).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_dept: Department = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_dept.default_working_pattern_id, None);
}

#[tokio::test]
async fn test_department_api_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create Department
    let dept_req = serde_json::json!({
        "name": "Engineering",
        "description": "Tech team",
        "manager_id": null
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/departments"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(dept_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let dept: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), 10000)
            .await
            .unwrap(),
    )
    .unwrap();
    let dept_id = dept["id"].as_str().unwrap();

    // 2. List Departments
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/departments"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Transfer Employee to Department
    let emp_id: Uuid = factory.employee().with_workspace(ws_id).create().await;
    let transfer_req = serde_json::json!({
        "department_id": dept_id,
        "effective_date": Utc::now().date_naive().to_string(),
        "notes": "Joining the team"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::from(transfer_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Get Employee Department History
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", admin_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
