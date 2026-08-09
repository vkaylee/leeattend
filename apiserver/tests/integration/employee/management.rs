//! Employee management API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::diesel_pool::{DieselPoolsTenantTxExt, TenantAuthToken};
use apiserver::models::Employee;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_employee_crud_lifecycle_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    let pool = factory.diesel_pools.clone();
    let hq_loc_id = pool
        .with_tenant_tx(&TenantAuthToken::system_override(ws_id), |conn| {
            Box::pin(async move {
                use apiserver::schema::work_locations::dsl::*;
                use diesel::prelude::*;
                use diesel_async::RunQueryDsl;
                work_locations
                    .filter(workspace_id.eq(ws_id))
                    .filter(is_headquarters.eq(true))
                    .select(id)
                    .first::<uuid::Uuid>(conn)
                    .await
                    .map_err(apiserver::errors::AppError::from)
            })
        })
        .await
        .unwrap();

    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Create Employee
    let create_req = serde_json::json!({
        "full_name": "Test Employee",
        "employee_id_number": "EMP001",
        "hire_date": "2026-01-01",
        "primary_location_id": hq_loc_id,
        "employer_id": employer_id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(create_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let emp: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(emp.full_name(), "Test Employee");

    // 2. Get Employee
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{}/employees/{}",
                    ws_id,
                    emp.id()
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 3. Update Employee
    let update_req = serde_json::json!({
        "full_name": "Updated Name"
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{}/employees/{}",
                    ws_id,
                    emp.id()
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(update_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated_emp: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated_emp.full_name(), "Updated Name");

    // 4. Delete Employee
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/workspaces/{}/employees/{}",
                    ws_id,
                    emp.id()
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_get_employees_list_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Create some employees
    let _emp1: Uuid = factory.employee().with_workspace(ws_id).create().await;
    let _emp2: Uuid = factory.employee().with_workspace(ws_id).create().await;

    // List employees
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_delete_non_existent_employee() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let fake_emp_id = Uuid::new_v4();

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/{fake_emp_id}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_employee_user_id_uniqueness() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_user_id).await;

    // Create a target user profile to be linked
    let target_user_id = factory.user().create().await;

    let pool = factory.diesel_pools.clone();
    let hq_loc_id = pool
        .with_tenant_tx(&TenantAuthToken::system_override(ws_id), |conn| {
            Box::pin(async move {
                use apiserver::schema::work_locations::dsl::*;
                use diesel::prelude::*;
                use diesel_async::RunQueryDsl;
                work_locations
                    .filter(workspace_id.eq(ws_id))
                    .filter(is_headquarters.eq(true))
                    .select(id)
                    .first::<uuid::Uuid>(conn)
                    .await
                    .map_err(apiserver::errors::AppError::from)
            })
        })
        .await
        .unwrap();

    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );

    // 1. Create first employee linked to target_user_id -> Should SUCCEED
    let create_req1 = serde_json::json!({
        "full_name": "First Employee",
        "employee_id_number": "EMP001",
        "hire_date": "2026-01-01",
        "primary_location_id": hq_loc_id,
        "user_id": target_user_id,
        "employer_id": employer_id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_user_id.to_string())
                .body(axum::body::Body::from(create_req1.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let emp1: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(emp1.user_id(), Some(target_user_id));

    // 2. Create second employee linked to target_user_id -> Should FAIL with BAD_REQUEST
    let create_req2 = serde_json::json!({
        "full_name": "Second Employee",
        "employee_id_number": "EMP002",
        "hire_date": "2026-01-01",
        "primary_location_id": hq_loc_id,
        "user_id": target_user_id,
        "employer_id": employer_id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_user_id.to_string())
                .body(axum::body::Body::from(create_req2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // 3. Create a third employee NOT linked to any user -> Should SUCCEED
    let create_req3 = serde_json::json!({
        "full_name": "Third Employee",
        "employee_id_number": "EMP003",
        "hire_date": "2026-01-01",
        "primary_location_id": hq_loc_id,
        "user_id": null,
        "employer_id": employer_id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/employees"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_user_id.to_string())
                .body(axum::body::Body::from(create_req3.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let emp3: Employee = serde_json::from_slice(&body).unwrap();
    assert_eq!(emp3.user_id(), None);

    // 4. Update the third employee to link to target_user_id -> Should FAIL with BAD_REQUEST
    let update_req = serde_json::json!({
        "user_id": target_user_id
    });

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{}/employees/{}",
                    ws_id,
                    emp3.id()
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_user_id.to_string())
                .body(axum::body::Body::from(update_req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
