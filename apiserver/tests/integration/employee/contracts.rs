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

    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let create_payload = json!({
        "employment_id": employment_id,
        "contract_type": "fixed_term",
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2027-01-01",
        "contract_number": "HD-2026-001",
        "notes": "Test contract",
        "employer_id": employer_id.to_string()
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
async fn test_explicitly_assign_employment_phase_during_contract_creation() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Create a phase definition first
    let phase_def_id = Uuid::new_v4();
    let pool = _test_db.diesel_pools.clone();
    crate::test_execute!(
        &pool.api,
        r#"
        INSERT INTO employment_phase_definitions (id, workspace_id, name, phase_type, rule_config)
        VALUES ($1, $2, 'Custom Phase', 'probation', '{"duration_days": 60}')
        "#,
        diesel::sql_types::Uuid,
        phase_def_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let create_payload = json!({
        "employment_id": employment_id,
        "contract_type": "probation",
        "start_date": "2026-05-01",
        "end_date": "2026-07-01",
        "assign_phase_id": phase_def_id,
        "employer_id": employer_id.to_string()
    });

    // Create Contract with assign_phase_id
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

    // Verify that the Employee was assigned to the explicit phase
    let phase_count: i64 = crate::test_query_scalar_i64!(
        &pool.api,
        "SELECT COUNT(*) AS count FROM employee_phases WHERE employment_id = $1",
        diesel::sql_types::Uuid,
        employment_id
    );

    // There should be 1 phase assigned to the employee
    assert_eq!(
        phase_count, 1,
        "Employee should have 1 explicitly assigned phase"
    );

    let assigned_def_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT phase_definition_id AS id FROM employee_phases WHERE employment_id = $1",
        diesel::sql_types::Uuid,
        employment_id
    );

    assert_eq!(assigned_def_id, phase_def_id);
}

#[tokio::test]
async fn test_employee_contracts_overlap_auto_close() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Test Employer', 'MST-1', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // 1. Create first contract
    let create_payload_1 = json!({
        "employment_id": employment_id,
        "contract_type": "fixed_term",
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "contract_number": "HD-001",
        "notes": "Original note",
        "employer_id": employer_id.to_string()
    });

    let res1 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts"),
            ws_id,
            user_id,
            Some(create_payload_1),
        ))
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);

    let body1 = axum::body::to_bytes(res1.into_body(), usize::MAX)
        .await
        .unwrap();
    let created1: serde_json::Value = serde_json::from_slice(&body1).unwrap();
    let contract1_id = created1["id"].as_str().unwrap().to_string();

    // 2. Create second contract that overlaps
    let create_payload_2 = json!({
        "employment_id": employment_id,
        "contract_type": "fixed_term",
        "status": "active",
        "start_date": "2026-06-01",
        "end_date": "2027-05-31",
        "contract_number": "HD-002",
        "auto_close_overlapping": true,
        "employer_id": employer_id.to_string()
    });

    let res2 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/contracts"),
            ws_id,
            user_id,
            Some(create_payload_2),
        ))
        .await
        .unwrap();

    assert_eq!(
        res2.status(),
        StatusCode::CREATED,
        "Should succeed with auto_close_overlapping"
    );

    // 3. Verify the old contract has been updated
    let get_res = app
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

    assert_eq!(get_res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(get_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let contracts: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    let old_contract = contracts.iter().find(|c| c["id"] == contract1_id).unwrap();

    // Verify status is expired, end_date is updated
    assert_eq!(old_contract["status"], "expired");
    assert_eq!(old_contract["end_date"], "2026-05-31");

    // Verify notes contain the auto-generated string
    let notes = old_contract["notes"].as_str().unwrap();
    assert!(notes.contains("Hệ thống tự động chấm dứt sớm vào ngày 31/05/2026"));
    assert!(notes.contains("để ký hợp đồng mới HD-002"));
    assert!(notes.contains("Original note"));
}
