use apiserver::models::employee_assignment::{
    AssignmentStatusEnum, EmployeeAssignment, NewEmployeeAssignment,
};
use apiserver::schema::employee_assignments;
use apiserver::schema::work_locations;
use axum::http::StatusCode;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;

async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    (test_db, factory)
}

#[tokio::test]
async fn test_employee_assignment_crud_and_status_enum() {
    let (test_db, factory) = setup_db().await;
    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let mut tenant_conn = test_db
        .diesel_pools
        .tenants_tx
        .get_connection(&ws_id)
        .await
        .unwrap();

    let hq_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT primary_location_id AS id FROM employees WHERE id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );

    #[derive(Insertable)]
    #[diesel(table_name = work_locations)]
    struct NewLoc {
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        timezone: Option<String>,
        is_headquarters: bool,
    }

    let tokyo_loc_id = Uuid::new_v4();
    diesel::insert_into(work_locations::table)
        .values(&NewLoc {
            id: tokyo_loc_id,
            workspace_id: ws_id,
            name: "Tokyo Branch".to_string(),
            timezone: Some("Asia/Tokyo".to_string()),
            is_headquarters: false,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    // Test inserting with Active status
    let new_assignment = NewEmployeeAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        home_location_id: hq_id,
        host_location_id: tokyo_loc_id,
        start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        end_date: Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
        reason: Some("Secondment to Tokyo".to_string()),
        status: AssignmentStatusEnum::Active,
    };

    let inserted: EmployeeAssignment = diesel::insert_into(employee_assignments::table)
        .values(&new_assignment)
        .get_result(&mut *tenant_conn)
        .await
        .unwrap();

    assert_eq!(inserted.status, AssignmentStatusEnum::Active);
    assert_eq!(inserted.reason.unwrap(), "Secondment to Tokyo");

    // Test updating to Completed status
    let updated: EmployeeAssignment = diesel::update(
        employee_assignments::table.filter(employee_assignments::id.eq(new_assignment.id)),
    )
    .set(employee_assignments::status.eq(AssignmentStatusEnum::Completed))
    .get_result(&mut *tenant_conn)
    .await
    .unwrap();

    assert_eq!(updated.status, AssignmentStatusEnum::Completed);

    // Test inserting with Cancelled status
    let cancelled_assignment = NewEmployeeAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id: emp_id,
        home_location_id: hq_id,
        host_location_id: tokyo_loc_id,
        start_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        end_date: None,
        reason: None,
        status: AssignmentStatusEnum::Cancelled,
    };

    let inserted_cancelled: EmployeeAssignment = diesel::insert_into(employee_assignments::table)
        .values(&cancelled_assignment)
        .get_result(&mut *tenant_conn)
        .await
        .unwrap();

    assert_eq!(inserted_cancelled.status, AssignmentStatusEnum::Cancelled);
}

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
async fn test_employee_assignments_api_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Get hq_id which is the default primary_location_id for the employee
    let hq_id: Uuid = crate::test_query_scalar_uuid!(
        &_test_db.diesel_pools.api,
        "SELECT primary_location_id AS id FROM employees WHERE id = $1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let tokyo_loc_id = Uuid::new_v4();
    let mut tenant_conn = _test_db
        .diesel_pools
        .tenants_tx
        .get_connection(&ws_id)
        .await
        .unwrap();

    #[derive(Insertable)]
    #[diesel(table_name = work_locations)]
    struct NewLoc {
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        timezone: Option<String>,
        is_headquarters: bool,
    }

    diesel::insert_into(work_locations::table)
        .values(&NewLoc {
            id: tokyo_loc_id,
            workspace_id: ws_id,
            name: "Tokyo Branch".to_string(),
            timezone: Some("Asia/Tokyo".to_string()),
            is_headquarters: false,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    let create_payload = json!({
        "host_location_id": tokyo_loc_id,
        "start_date": "2026-06-01",
        "end_date": "2026-06-30",
        "reason": "API secondment"
    });

    // 1. Create Assignment
    let response = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/assignments"),
            ws_id,
            user_id,
            Some(create_payload),
        ))
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Should create employee assignment"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let assignment_id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["reason"], "API secondment");
    assert_eq!(created["home_location_id"], hq_id.to_string());
    assert_eq!(created["host_location_id"], tokyo_loc_id.to_string());
    assert_eq!(created["status"], "active");

    // 2. List Assignments
    let response = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/assignments"),
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
    let assignments: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0]["id"], assignment_id);

    // 3. Update Assignment
    let update_payload = json!({
        "status": "completed",
        "reason": "Finished"
    });

    let response = app
        .clone()
        .oneshot(make_req(
            "PATCH",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/assignments/{assignment_id}"),
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
    assert_eq!(updated["status"], "completed");
    assert_eq!(updated["reason"], "Finished");

    // 4. Delete Assignment
    let response = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/employees/{emp_id}/assignments/{assignment_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
