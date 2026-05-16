//! Employee ID normalization tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{
    CreateEmployeeRequest, Employee, SanitizedEmployeeId, UpdateEmployeeRequest,
};
use apiserver::AppState;
use axum::body::Body;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_create_employee_id_normalization() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    // Create employee with leading zeros "00123"
    let req_body = CreateEmployeeRequest {
        user_id: None,
        full_name: "Test Employee".to_string(),
        employee_id_number: SanitizedEmployeeId::new("00123"),
        hire_date: chrono::Utc::now().date_naive(),
        tags: None,
        recruitment_source_id: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/employees"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let created_emp: Employee = serde_json::from_slice(&body).unwrap();

    // Verify response has normalized ID
    assert_eq!(created_emp.employee_id_number, "123");

    // Verify in DB
    let db_emp: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT employee_id_number AS val FROM employees WHERE id = $1",
        diesel::sql_types::Uuid,
        created_emp.id
    );

    assert_eq!(db_emp, "123");
}

#[tokio::test]
async fn test_update_employee_id_normalization() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    // Create initial employee "100"
    let emp_id: Uuid = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("100")
        .with_full_name("Original Name")
        .create()
        .await;

    // Update employee ID to "00200"
    let req_body = UpdateEmployeeRequest {
        user_id: None,
        full_name: Some("Updated Name".to_string()),
        employee_id_number: Some(SanitizedEmployeeId::new("00200")),
        hire_date: None,
        termination_date: None,
        tags: None,
        is_active: None,
        max_concurrent_workers: None,
        recruitment_source_id: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!("/api/workspaces/{ws_id}/employees/{emp_id}"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let updated_emp: Employee = serde_json::from_slice(&body).unwrap();

    // Verify response has normalized ID
    assert_eq!(updated_emp.employee_id_number, "200");

    // Verify in DB
    let db_emp: String = crate::test_query_scalar_string!(
        &factory.diesel_pools.api,
        "SELECT employee_id_number AS val FROM employees WHERE id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );

    assert_eq!(db_emp, "200");
}

#[tokio::test]
async fn test_employee_id_uniqueness_on_normalized_values() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let token = factory.auth().create_token(user_id);

    // Create first employee "555"
    let _: Uuid = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("555")
        .with_full_name("First Employee")
        .create()
        .await;

    // Try to create second employee with "00555" (should conflict after normalization)
    let req_body = CreateEmployeeRequest {
        user_id: None,
        full_name: "Duplicate Employee".to_string(),
        employee_id_number: SanitizedEmployeeId::new("00555"),
        hire_date: chrono::Utc::now().date_naive(),
        tags: None,
        recruitment_source_id: None,
    };

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/employees"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&req_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Expect failure due to uniqueness constraint
    assert_ne!(response.status(), StatusCode::CREATED);
}
