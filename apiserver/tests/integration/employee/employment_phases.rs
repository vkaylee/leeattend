//! Integration tests for Employment Phases API

use axum::http::StatusCode;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use std::sync::Arc;

static EMPLOYEE_ROLE_ID: &str = "a0000000-0000-0000-0000-000000000006";
static HR_MANAGER_ROLE_ID: &str = "a0000000-0000-0000-0000-000000000003";

#[tokio::test]
async fn test_employment_phases_crud_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Create a Phase Definition (Probation)
    let def_payload = json!({
        "name": "Probation",
        "description": "2 months probation",
        "phase_type": "probation",
        "rule_config": { "duration_days": 60 },
        "display_order": 1
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{ws_id}/employment-phases"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&def_payload).unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_def: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let phase_def_id = created_def["id"].as_str().unwrap().to_string();
    assert_eq!(created_def["name"], "Probation");

    // 2. Fetch all Phase Definitions
    let req = axum::http::Request::builder()
        .method("GET")
        .uri(format!("/api/workspaces/{ws_id}/employment-phases"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let defs: Vec<serde_json::Value> = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(defs.len(), 1);

    // 3. Assign Phase 1 to Employee
    let assign_payload_1 = json!({
        "phase_definition_id": phase_def_id,
        "start_date": "2026-01-01",
        "end_date": "2026-03-31",
        "override_config": null
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!(
            "/api/workspaces/{ws_id}/employees/{employee_id}/phases"
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&assign_payload_1).unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_assignment_1: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let assignment_id_1 = created_assignment_1["id"].as_str().unwrap().to_string();
    assert_eq!(created_assignment_1["status"], "active");

    // 4. Assign Phase 2 (Overlapping) to Employee -> Should Auto-Close Phase 1
    let assign_payload_2 = json!({
        "phase_definition_id": phase_def_id,
        "start_date": "2026-02-15",
        "end_date": "2026-05-31",
        "override_config": null
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!(
            "/api/workspaces/{ws_id}/employees/{employee_id}/phases"
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&assign_payload_2).unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 5. Verify Phase 1 is auto-closed
    let pool = _test_db.diesel_pools.clone();
    let phase_count: i64 = crate::test_query_scalar_i64!(
        &pool.api,
        "SELECT COUNT(*) AS count FROM employee_phases WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id
    );

    assert_eq!(phase_count, 2);

    let p1_status: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT status::text AS val FROM employee_phases WHERE id = $1",
        diesel::sql_types::Uuid,
        Uuid::parse_str(&assignment_id_1).unwrap()
    );
    let p1_end_date: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT end_date::text AS val FROM employee_phases WHERE id = $1",
        diesel::sql_types::Uuid,
        Uuid::parse_str(&assignment_id_1).unwrap()
    );

    assert_eq!(p1_status, "completed", "Phase 1 should be auto-closed");
    assert_eq!(
        p1_end_date, "2026-02-14",
        "Phase 1 end_date should be truncated to day before Phase 2"
    );

    let p2_status: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT status::text AS val FROM employee_phases WHERE employee_id = $1 AND id != $2",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        Uuid::parse_str(&assignment_id_1).unwrap()
    );
    let p2_end_date: String = crate::test_query_scalar_string!(
        &pool.api,
        "SELECT end_date::text AS val FROM employee_phases WHERE employee_id = $1 AND id != $2",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        Uuid::parse_str(&assignment_id_1).unwrap()
    );

    assert_eq!(p2_status, "active", "Phase 2 should be active");
    assert_eq!(
        p2_end_date, "2026-05-31",
        "Phase 2 end_date should be untouched"
    );
}

#[tokio::test]
async fn test_employment_phases_permission_enforcement() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().with_email("owner@test.com").create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // ── Create a definition as owner (so we can test assign/update) ──
    let def_payload = json!({
        "name": "Probation",
        "description": "Test phase",
        "phase_type": "probation",
        "rule_config": { "duration_days": 60 },
        "display_order": 1
    });

    let req = axum::http::Request::builder()
        .method("POST")
        .uri(format!("/api/workspaces/{ws_id}/employment-phases"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", owner_id.to_string())
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(
            serde_json::to_string(&def_payload).unwrap(),
        ))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created_def: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let phase_def_id = created_def["id"].as_str().unwrap().to_string();

    // ── Add member with "employee" role (no EmploymentPhasesManage) ──
    let member_id = factory.user().with_email("regular@test.com").create().await;
    factory
        .add_member(ws_id, member_id, Uuid::parse_str(EMPLOYEE_ROLE_ID).unwrap())
        .await;

    let unauthorized_auth = |uri: String, method: &str, body: Option<axum::body::Body>| {
        let builder = axum::http::Request::builder()
            .method(axum::http::Method::from_bytes(method.as_bytes()).unwrap())
            .uri(uri)
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", member_id.to_string());
        let req = if let Some(b) = body {
            builder.header("Content-Type", "application/json").body(b)
        } else {
            builder.body(axum::body::Body::empty())
        };
        req.unwrap()
    };

    // ── A: Employee-role user cannot create definition ──
    let req = unauthorized_auth(
        format!("/api/workspaces/{ws_id}/employment-phases"),
        "POST",
        Some(axum::body::Body::from(
            serde_json::to_string(&json!({
                "name": "Illegal",
                "phase_type": "probation",
                "rule_config": {}
            }))
            .unwrap(),
        )),
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee-role user should be forbidden from creating phase definitions"
    );

    // ── B: Employee-role user cannot assign phase ──
    let assign_payload = json!({
        "phase_definition_id": phase_def_id,
        "start_date": "2026-03-01",
        "end_date": "2026-04-30",
        "override_config": null
    });
    let req = unauthorized_auth(
        format!("/api/workspaces/{ws_id}/employees/{employee_id}/phases"),
        "POST",
        Some(axum::body::Body::from(
            serde_json::to_string(&assign_payload).unwrap(),
        )),
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee-role user should be forbidden from assigning phases"
    );

    // ── C: Employee-role user cannot update phase status ──
    let status_payload = json!({
        "status": "completed",
        "evaluation_notes": "Should fail"
    });
    let req = unauthorized_auth(
        format!("/api/workspaces/{ws_id}/employees/{employee_id}/phases/{phase_def_id}/status"),
        "PUT",
        Some(axum::body::Body::from(
            serde_json::to_string(&status_payload).unwrap(),
        )),
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Employee-role user should be forbidden from updating phase status"
    );

    // ── D: But employee-role user CAN read definitions (no permission required) ──
    let req = unauthorized_auth(
        format!("/api/workspaces/{ws_id}/employment-phases"),
        "GET",
        None,
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Employee-role user should be able to read phase definitions"
    );

    // ── Add member with "hr_manager" role (has EmploymentPhasesManage) ──
    let hr_user_id = factory.user().with_email("hr@test.com").create().await;
    factory
        .add_member(
            ws_id,
            hr_user_id,
            Uuid::parse_str(HR_MANAGER_ROLE_ID).unwrap(),
        )
        .await;

    let hr_auth = |uri: String, method: &str, body: Option<axum::body::Body>| {
        let builder = axum::http::Request::builder()
            .method(axum::http::Method::from_bytes(method.as_bytes()).unwrap())
            .uri(uri)
            .header("X-Workspace-Id", ws_id.to_string())
            .header("X-User-Id", hr_user_id.to_string());
        let req = if let Some(b) = body {
            builder.header("Content-Type", "application/json").body(b)
        } else {
            builder.body(axum::body::Body::empty())
        };
        req.unwrap()
    };

    // ── E: HR Manager can create definition ──
    let req = hr_auth(
        format!("/api/workspaces/{ws_id}/employment-phases"),
        "POST",
        Some(axum::body::Body::from(
            serde_json::to_string(&json!({
                "name": "Performance Review",
                "phase_type": "review",
                "rule_config": { "duration_days": 90 }
            }))
            .unwrap(),
        )),
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "HR Manager should be allowed to create phase definitions"
    );

    // ── F: HR Manager can assign phase ──
    let req = hr_auth(
        format!("/api/workspaces/{ws_id}/employees/{employee_id}/phases"),
        "POST",
        Some(axum::body::Body::from(
            serde_json::to_string(&json!({
                "phase_definition_id": phase_def_id,
                "start_date": "2026-05-01",
                "end_date": "2026-06-30",
                "override_config": null
            }))
            .unwrap(),
        )),
    );
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "HR Manager should be allowed to assign phases"
    );
}
