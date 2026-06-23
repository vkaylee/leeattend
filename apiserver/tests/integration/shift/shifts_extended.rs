//! Additional tests for shift and roster handlers to improve coverage

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use diesel::sql_types;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => axum::body::Body::from(b),
            None => axum::body::Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_get_workspace_schedules_empty() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/schedules?start_date=2026-01-01&end_date=2026-01-31"
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let schedules = body.get("data").unwrap().as_array().unwrap();
    assert!(schedules.is_empty());
}

#[tokio::test]
async fn test_get_workspace_schedules_missing_dates() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/schedules"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert!(resp.status().is_client_error());
}

#[tokio::test]
async fn test_get_assignment_history_empty() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/assignment-history"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let history = body.get("data").unwrap().as_array().unwrap();
    assert!(history.is_empty());
}

#[tokio::test]
async fn test_get_employee_rosters_empty() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/rosters?start_date=2026-01-01&end_date=2026-01-31"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = body_json(resp).await;
    let rosters = body.get("data").unwrap().as_array().unwrap();
    assert!(rosters.is_empty());
}

#[tokio::test]
async fn test_delete_roster_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/rosters/2026-01-15"),
            ws_id,
            member_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_create_shift_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let body = serde_json::json!({
        "name": "Night Shift",
        "start_time": "22:00:00",
        "end_time": "06:00:00",
        "break_duration_minutes": 30
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/shifts"),
            ws_id,
            member_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_get_workspace_schedules_filtered_by_employment_ids() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create 2 employees
    let emp_id_1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice")
        .create()
        .await;
    let emp_id_2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob")
        .create()
        .await;

    // Get their employment IDs
    let employment_id_1: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        sql_types::Uuid,
        emp_id_1
    );
    let employment_id_2: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        sql_types::Uuid,
        emp_id_2
    );

    // Assign shifts to both employees
    let _shift_id_1 = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Alice Shift")
        .assign_to_employee(
            emp_id_1,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        )
        .await;
    let _shift_id_2 = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Bob Shift")
        .assign_to_employee(
            emp_id_2,
            chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        )
        .await;

    // 1. Query workspace schedules without any filter
    let resp_all = app
        .clone()
        .oneshot(req(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/schedules?start_date=2026-01-01&end_date=2026-01-07"
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp_all.status(), StatusCode::OK);
    let body_all = body_json(resp_all).await;
    let schedules_all = body_all.get("data").unwrap().as_array().unwrap();
    // It should return schedules for both employees
    assert!(schedules_all
        .iter()
        .any(|s| s.get("employment_id").and_then(|id| id.as_str())
            == Some(&employment_id_1.to_string())));
    assert!(schedules_all
        .iter()
        .any(|s| s.get("employment_id").and_then(|id| id.as_str())
            == Some(&employment_id_2.to_string())));

    // 2. Query workspace schedules filtered by employment_id_1
    let resp_filtered = app
        .clone()
        .oneshot(req(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/schedules?start_date=2026-01-01&end_date=2026-01-07&employment_ids={}",
                employment_id_1
            ),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp_filtered.status(), StatusCode::OK);
    let body_filtered = body_json(resp_filtered).await;
    let schedules_filtered = body_filtered.get("data").unwrap().as_array().unwrap();

    // It should contain ONLY Alice's schedule
    assert_eq!(schedules_filtered.len(), 1);
    assert_eq!(
        schedules_filtered[0]
            .get("employment_id")
            .and_then(|id| id.as_str()),
        Some(employment_id_1.to_string().as_str())
    );
}
