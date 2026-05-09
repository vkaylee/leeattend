//! Integration tests for Employee Contracts API

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

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
async fn test_employee_contracts_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let create_payload = json!({
        "employee_id": emp_id,
        "contract_type": "fixed_term",
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2027-01-01",
        "contract_number": "HD-2026-001",
        "notes": "Test contract"
    });

    // 1. Create Contract
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts"),
            ws_id,
            user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "Should create employee contract"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let contract_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["contract_type"], "fixed_term");
    assert_eq!(created["contract_number"], "HD-2026-001");

    // 2. List Contracts
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let contracts: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(contracts.len(), 1);
    assert_eq!(contracts[0]["id"], contract_id);

    // 3. Update Contract
    let update_payload = json!({
        "status": "expired",
        "notes": "Updated note"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts/{contract_id}"),
            ws_id,
            user_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(updated["status"], "expired");
    assert_eq!(updated["notes"], "Updated note");

    // 4. Delete Contract
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts/{contract_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_auto_create_employment_phase_for_probation_contract() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let create_payload = json!({
        "employee_id": emp_id,
        "contract_type": "probation",
        "start_date": "2026-05-01",
        "end_date": "2026-07-01",
    });

    // Create Probation Contract
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts"),
            ws_id,
            user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Verify that the Employee was assigned to a Probation phase automatically
    let pool = _test_db.pool();
    let phases =
        sqlx::query("SELECT start_date, end_date FROM employee_phases WHERE employee_id = $1")
            .bind(emp_id)
            .fetch_all(&pool)
            .await
            .unwrap();

    // There should be 1 phase assigned to the employee
    assert_eq!(
        phases.len(),
        1,
        "Employee should have 1 auto-assigned phase"
    );

    use sqlx::Row;
    let phase = &phases[0];
    let start_date: chrono::NaiveDate = phase.get("start_date");
    let end_date: Option<chrono::NaiveDate> = phase.get("end_date");

    assert_eq!(start_date.to_string(), "2026-05-01");
    assert_eq!(end_date.unwrap().to_string(), "2026-07-01");
}
