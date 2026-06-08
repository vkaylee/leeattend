//! Integration tests for Employee Department Transfer and History API

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
async fn test_employee_transfer_and_department_history() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@transfer.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let dept_eng = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;
    let dept_sales = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Get initial department history (empty)
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"),
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
    assert_eq!(
        history.len(),
        0,
        "Should have no department history initially"
    );

    // 2. Transfer employee to Engineering
    let transfer_payload = json!({
        "department_id": dept_eng,
        "effective_date": "2026-01-01",
        "notes": "Initial placement"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"),
            ws_id,
            owner_id,
            Some(transfer_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Transfer to Engineering should succeed"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let history_entry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(history_entry["department_id"], dept_eng.to_string());
    assert_eq!(history_entry["effective_date"], "2026-01-01");
    assert_eq!(history_entry["notes"], "Initial placement");

    // 3. Transfer employee to Sales
    let transfer_payload2 = json!({
        "department_id": dept_sales,
        "effective_date": "2026-04-01",
        "notes": "Moved to Sales team"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"),
            ws_id,
            owner_id,
            Some(transfer_payload2),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let second_entry: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(second_entry["department_id"], dept_sales.to_string());

    // 4. Verify department history shows 2 entries (ordered newest first)
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"),
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
    assert_eq!(history.len(), 2, "Should have 2 department history entries");
}

#[tokio::test]
async fn test_employee_transfer_permission_enforcement() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@transfer2.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let dept = factory
        .department()
        .with_workspace(ws_id)
        .with_name("HR Dept")
        .create()
        .await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Unprivileged user (Employee role — no DepartmentsTransfer)
    let regular_user_id = factory
        .user()
        .with_email("regular@transfer2.test")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            regular_user_id,
            Uuid::parse_str(EMPLOYEE_ROLE_ID).unwrap(),
        )
        .await;

    let transfer_payload = json!({
        "department_id": dept,
        "effective_date": "2026-06-01",
        "notes": "Should be forbidden"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/departments"),
            ws_id,
            regular_user_id,
            Some(transfer_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee role must not be able to transfer employees"
    );
}

#[tokio::test]
async fn test_employee_rehire_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory
        .user()
        .with_email("owner@rehire.test")
        .create()
        .await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Step 1: Terminate employee (via updating active employment)
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' AND deleted_at IS NULL LIMIT 1",
        diesel::sql_types::Uuid, emp_id,
        diesel::sql_types::Uuid, ws_id
    );
    let terminate_payload = json!({
        "status": "terminated",
        "end_date": "2026-02-28"
    });
    let response = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            owner_id,
            Some(terminate_payload),
        ))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should terminate employee employment"
    );

    // Step 2: Rehire employee
    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );

    let rehire_payload = json!({
        "hire_date": "2026-06-01",
        "employer_id": employer_id
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/rehire"),
            ws_id,
            owner_id,
            Some(rehire_payload),
        ))
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert_eq!(
        status,
        StatusCode::OK,
        "Rehire should succeed for a terminated employee. Response body: {}",
        body_str
    );
    let emp: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(
        emp["active_employment_id"].is_string(),
        "Rehired employee should be active with a new employment"
    );

    // Step 3: Fetch employee details to verify rehire date
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
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
    let employments: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(employments.len(), 2, "Should have two employments now");

    // One should be active with start_date 2026-06-01
    let active_emp = employments
        .iter()
        .find(|e| e["status"] == "active")
        .unwrap();
    assert_eq!(active_emp["start_date"], "2026-06-01");

    // The other should be terminated with end_date 2026-02-28
    let terminated_emp = employments
        .iter()
        .find(|e| e["status"] == "terminated")
        .unwrap();
    assert_eq!(terminated_emp["end_date"], "2026-02-28");
}
