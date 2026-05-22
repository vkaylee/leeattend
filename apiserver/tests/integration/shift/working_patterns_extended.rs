//! Integration tests for Working Patterns API

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

static EMPLOYEE_ROLE_ID: &str = "a0000000-0000-0000-0000-000000000006";

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if let Some(b) = body {
        builder
            .header("Content-Type", "application/json")
            .body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(axum::body::Body::empty()).unwrap()
    }
}

#[tokio::test]
async fn test_working_pattern_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@pattern.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Morning Shift")
        .create()
        .await;

    // 1. Create a working pattern (Mon-Fri)
    let create_payload = json!({
        "name": "Standard 5-Day",
        "description": "Monday to Friday standard schedule",
        "details": [
            { "schedule_day": 1, "shift_id": shift_id },
            { "schedule_day": 2, "shift_id": shift_id },
            { "schedule_day": 3, "shift_id": shift_id },
            { "schedule_day": 4, "shift_id": shift_id },
            { "schedule_day": 5, "shift_id": shift_id }
        ]
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            owner_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Should create working pattern"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let pattern: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let pattern_id = pattern["id"].as_str().unwrap().to_string();
    assert_eq!(pattern["name"], "Standard 5-Day");

    // 2. List patterns — should contain our new pattern
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let patterns: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        patterns.iter().any(|p| p["id"] == pattern_id),
        "List should contain created pattern"
    );

    // 3. Get pattern details (day-of-week details)
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/working-patterns/{pattern_id}"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let details: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(details.len(), 5, "Should have 5 day details (Mon-Fri)");

    // 4. Assign pattern to employee
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let assign_payload = json!({
        "pattern_id": pattern_id,
        "start_date": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/employees/{emp_id}/working-patterns"),
            ws_id,
            owner_id,
            Some(assign_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should assign pattern to employee"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let assignment: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(assignment["pattern_id"], pattern_id);
    assert_eq!(assignment["employee_id"], emp_id.to_string());

    // 5. Get employee pattern history
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/workspaces/{ws_id}/employees/{emp_id}/working-patterns"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let history: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(
        !history.is_empty(),
        "Should have pattern assignment history"
    );

    // 6. Delete pattern
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/workspaces/{ws_id}/working-patterns/{pattern_id}"),
            ws_id,
            owner_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::NO_CONTENT,
        "Should delete working pattern"
    );
}

#[tokio::test]
async fn test_working_pattern_permission_enforcement() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@pattern2.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Unprivileged user
    let regular_user_id = factory
        .user()
        .with_email("regular@pattern2.test")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            regular_user_id,
            Uuid::parse_str(EMPLOYEE_ROLE_ID).unwrap(),
        )
        .await;

    let create_payload = json!({
        "name": "Forbidden Pattern",
        "description": "Should not be creatable",
        "details": []
    });

    // Employee role cannot create patterns
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            regular_user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee role must not create working patterns"
    );
}

#[tokio::test]
async fn test_bulk_assign_pattern() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@bulk.pattern")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Create a pattern
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Day Shift")
        .create()
        .await;

    let create_payload = json!({
        "name": "Full Week",
        "description": "7 days",
        "details": [
            { "schedule_day": 0, "shift_id": shift_id },
            { "schedule_day": 1, "shift_id": shift_id }
        ]
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/working-patterns"),
            ws_id,
            owner_id,
            Some(create_payload),
        ))
        .await
        .unwrap();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let pattern: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let pattern_id = pattern["id"].as_str().unwrap().to_string();

    // Create 3 employees
    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let emp2 = factory.employee().with_workspace(ws_id).create().await;
    let emp3 = factory.employee().with_workspace(ws_id).create().await;

    // Bulk assign
    let bulk_payload = json!({
        "pattern_id": pattern_id,
        "employee_ids": [emp1, emp2, emp3],
        "start_date": "2026-01-01"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/workspaces/{ws_id}/working-patterns/bulk-assign"),
            ws_id,
            owner_id,
            Some(bulk_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Bulk assign should succeed"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let result: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        result["success_count"], 3,
        "All 3 employees should be assigned"
    );
}
