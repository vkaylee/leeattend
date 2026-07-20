//! Integration tests for Employee Employments API and Multi-Employer conflict checks.

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
async fn test_same_employer_overlap_fails_across_employment_types() {
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

    // 1. Create first primary employment (2026-01-01 to 2026-06-30)
    let payload_1 = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2026-06-30",
        "employment_type": "primary",
        "job_title": "Software Engineer"
    });

    let res1 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload_1),
        ))
        .await
        .unwrap();

    assert_eq!(
        res1.status(),
        StatusCode::CREATED,
        "Should allow creating first primary employment"
    );

    // 2. Try creating overlapping primary employment (2026-06-01 to 2026-12-31)
    let payload_2 = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-06-01",
        "end_date": "2026-12-31",
        "employment_type": "contractor",
        "job_title": "Tech Lead"
    });

    let res2 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload_2),
        ))
        .await
        .unwrap();

    assert_eq!(
        res2.status(),
        StatusCode::CONFLICT,
        "Should return conflict (409) for overlapping employment with the same employer"
    );

    // 3. The same dates for a different employer remain valid.
    let second_employer_id = Uuid::new_v4();
    crate::test_execute!(
        &_test_db.diesel_pools.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Second Test Employer', 'MST-2', 'Addr', NOW(), NOW())",
        diesel::sql_types::Uuid,
        second_employer_id,
        diesel::sql_types::Uuid,
        ws_id
    );
    let different_employer_payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": second_employer_id.to_string(),
        "status": "active",
        "start_date": "2026-06-01",
        "end_date": "2026-12-31",
        "employment_type": "contractor"
    });
    let different_employer_response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(different_employer_payload),
        ))
        .await
        .unwrap();
    assert_eq!(different_employer_response.status(), StatusCode::CREATED);

    // 4. Create non-overlapping primary employment (2026-07-01 to open-ended)
    let payload_3 = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-07-01",
        "end_date": null,
        "employment_type": "primary",
        "job_title": "Staff Engineer"
    });

    let res3 = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload_3),
        ))
        .await
        .unwrap();

    assert_eq!(
        res3.status(),
        StatusCode::CREATED,
        "Should allow creating non-overlapping primary employment"
    );

    let body3 = axum::body::to_bytes(res3.into_body(), usize::MAX)
        .await
        .unwrap();
    let created3: serde_json::Value = serde_json::from_slice(&body3).unwrap();
    let emp3_id = created3["id"].as_str().unwrap().to_string();

    // 5. Try updating the non-overlapping employment start_date to overlap (2026-06-15)
    let update_payload = json!({
        "start_date": "2026-06-15"
    });

    let res4 = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{emp3_id}"),
            ws_id,
            user_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        res4.status(),
        StatusCode::CONFLICT,
        "Should return conflict when patching start_date to cause overlap"
    );
}

#[tokio::test]
async fn test_employment_crud_lifecycle() {
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

    // 1. POST - Create Employment
    let payload = json!({
        "employee_id": emp_id.to_string(),
        "employer_id": employer_id.to_string(),
        "status": "active",
        "start_date": "2026-01-01",
        "end_date": "2026-12-31",
        "employment_type": "primary",
        "job_title": "Software Engineer"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let employment_id = created["id"].as_str().unwrap().to_string();

    // 2. GET - List Employments
    let res_list = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_list.status(), StatusCode::OK);

    let body_list = axum::body::to_bytes(res_list.into_body(), usize::MAX)
        .await
        .unwrap();
    let list: serde_json::Value = serde_json::from_slice(&body_list).unwrap();
    assert!(list.is_array());
    assert!(!list.as_array().unwrap().is_empty());

    // 3. GET - Detail Employment
    let res_detail = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_detail.status(), StatusCode::OK);

    let body_detail = axum::body::to_bytes(res_detail.into_body(), usize::MAX)
        .await
        .unwrap();
    let detail: serde_json::Value = serde_json::from_slice(&body_detail).unwrap();
    assert_eq!(detail["job_title"].as_str().unwrap(), "Software Engineer");

    // 4. GET - Detail NotFound
    let res_not_found = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!(
                "/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{}",
                Uuid::new_v4()
            ),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_not_found.status(), StatusCode::NOT_FOUND);

    // 5. PATCH - Update Employment
    let update_payload = json!({
        "job_title": "Senior Software Engineer"
    });

    let res_update = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            Some(update_payload),
        ))
        .await
        .unwrap();

    assert_eq!(res_update.status(), StatusCode::OK);

    let body_update = axum::body::to_bytes(res_update.into_body(), usize::MAX)
        .await
        .unwrap();
    let updated: serde_json::Value = serde_json::from_slice(&body_update).unwrap();
    assert_eq!(
        updated["job_title"].as_str().unwrap(),
        "Senior Software Engineer"
    );

    // 6. DELETE - Soft Delete Employment
    let res_delete = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/employments/{employment_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res_delete.status(), StatusCode::NO_CONTENT);
}
