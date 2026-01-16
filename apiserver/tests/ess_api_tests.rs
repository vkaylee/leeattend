//! Employee Self-Service (ESS) API Tests
//!
//! Tests for ESS endpoints that allow employees to manage their own data.

mod common;

use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::models::{EmployeeEmergencyContact, EmployeeProfile, MyProfileResponse};
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

/// Helper to setup user, workspace, and employee linked to user
async fn setup_employee_with_user(factory: &TestFactory) -> (uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;
    (user_id, ws_id, emp_id)
}

// =============================================================================
// Profile Tests
// =============================================================================

#[tokio::test]
async fn test_get_my_profile_success() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/profile")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let profile_response: MyProfileResponse = serde_json::from_slice(&body).unwrap();

    // Employee should be present
    assert_eq!(profile_response.employee.first_name, "Test");
    assert_eq!(profile_response.employee.last_name, "Employee");

    // Profile may be None if not created yet
    assert!(profile_response.profile.is_none());

    // Emergency contacts should be empty
    assert!(profile_response.emergency_contacts.is_empty());
}

#[tokio::test]
async fn test_get_my_profile_user_not_linked_to_employee() {
    let (_test_db, factory, app) = setup_app().await;

    // Create user without employee record
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/profile")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return FORBIDDEN when user has no employee record
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_my_profile_creates_new_profile() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let update_req = serde_json::json!({
        "phone": "+84901234567",
        "address": "123 Main St, Ho Chi Minh City",
        "gender": "male",
        "bank_name": "Vietcombank",
        "bank_account_number": "1234567890"
    });

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/api/ess/profile")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&update_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let profile: EmployeeProfile = serde_json::from_slice(&body).unwrap();

    assert_eq!(profile.phone, Some("+84901234567".to_string()));
    assert_eq!(profile.address, Some("123 Main St, Ho Chi Minh City".to_string()));
    assert_eq!(profile.gender, Some("male".to_string()));
    assert_eq!(profile.bank_name, Some("Vietcombank".to_string()));
}

#[tokio::test]
async fn test_update_my_profile_updates_existing_profile() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    // First update - create profile
    let initial_req = serde_json::json!({
        "phone": "+84901234567"
    });

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/api/ess/profile")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&initial_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second update - update existing profile
    let update_req = serde_json::json!({
        "address": "456 New Address"
    });

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri("/api/ess/profile")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&update_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let profile: EmployeeProfile = serde_json::from_slice(&body).unwrap();

    // Phone should be preserved, address should be updated
    assert_eq!(profile.phone, Some("+84901234567".to_string()));
    assert_eq!(profile.address, Some("456 New Address".to_string()));
}

// =============================================================================
// Emergency Contacts Tests
// =============================================================================

#[tokio::test]
async fn test_add_emergency_contact_success() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let contact_req = serde_json::json!({
        "contact_name": "John Doe",
        "relationship": "Spouse",
        "phone": "+84901111111",
        "is_primary": true
    });

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/ess/emergency-contacts")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&contact_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let contact: EmployeeEmergencyContact = serde_json::from_slice(&body).unwrap();

    assert_eq!(contact.contact_name, "John Doe");
    assert_eq!(contact.relationship, "Spouse");
    assert_eq!(contact.phone, "+84901111111");
    assert!(contact.is_primary);
}

#[tokio::test]
async fn test_get_emergency_contacts_returns_sorted_list() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    // Add first contact (not primary)
    let contact1 = serde_json::json!({
        "contact_name": "Parent",
        "relationship": "Parent",
        "phone": "+84901111111",
        "is_primary": false
    });

    app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/ess/emergency-contacts")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&contact1).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Add second contact (primary)
    let contact2 = serde_json::json!({
        "contact_name": "Spouse",
        "relationship": "Spouse",
        "phone": "+84902222222",
        "is_primary": true
    });

    app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/ess/emergency-contacts")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&contact2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Get contacts
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/emergency-contacts")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let contacts: Vec<EmployeeEmergencyContact> = serde_json::from_slice(&body).unwrap();

    assert_eq!(contacts.len(), 2);
    // Primary contact should be first
    assert!(contacts[0].is_primary);
    assert_eq!(contacts[0].contact_name, "Spouse");
}

#[tokio::test]
async fn test_delete_emergency_contact_success() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    // Add a contact
    let contact_req = serde_json::json!({
        "contact_name": "Test Contact",
        "relationship": "Friend",
        "phone": "+84909999999",
        "is_primary": false
    });

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/api/ess/emergency-contacts")
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(serde_json::to_string(&contact_req).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let contact: EmployeeEmergencyContact = serde_json::from_slice(&body).unwrap();

    // Delete the contact
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/ess/emergency-contacts/{}", contact.id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify contact is deleted
    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/emergency-contacts")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let contacts: Vec<EmployeeEmergencyContact> = serde_json::from_slice(&body).unwrap();
    assert!(contacts.is_empty());
}

// =============================================================================
// Attendance Tests
// =============================================================================

#[tokio::test]
async fn test_get_my_attendance_empty() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/attendance")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn test_get_my_attendance_with_date_filter() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/attendance?start_date=2026-01-01&end_date=2026-01-31")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

// =============================================================================
// Leave Balance Tests
// =============================================================================

#[tokio::test]
async fn test_get_my_leave_balances_empty() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/leave/balances")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let balances: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(balances.is_empty());
}

// =============================================================================
// Leave Requests Tests
// =============================================================================

#[tokio::test]
async fn test_get_my_leave_requests_empty() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/leave/requests")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let requests: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(requests.is_empty());
}

// =============================================================================
// Documents Tests
// =============================================================================

#[tokio::test]
async fn test_get_my_documents_empty() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/documents")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let docs: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(docs.is_empty());
}

// =============================================================================
// Payslips Tests
// =============================================================================

#[tokio::test]
async fn test_get_my_payslips_empty() {
    let (_test_db, factory, app) = setup_app().await;
    let (user_id, ws_id, _emp_id) = setup_employee_with_user(&factory).await;

    let response = app.clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/api/ess/payslips")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let payslips: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(payslips.is_empty());
}
