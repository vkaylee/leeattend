use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LeaveRequest, LeaveStatus, SubmitLeaveRequest};
use apiserver::AppState;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_service_transaction_successful_commit() {
    let (_test_db, factory, state, _app) = setup_app_tower().await;
    let pools = factory.diesel_pools.clone();

    // Setup: owner, workspace
    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Create target employee
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let leave_service = state.leaves().service();

    // 1. Submit leave request via service (this calls our new transaction code)
    let submit_req = SubmitLeaveRequest {
        employment_id,
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2027, 9, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2027, 9, 2).unwrap(),
        reason: Some("Summer holiday".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        attachment_file_ids: None,
    };

    let result = leave_service
        .submit_leave_request(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            owner_id,
            submit_req,
            true,
        )
        .await;

    assert!(
        result.is_ok(),
        "submit_leave_request should commit successfully in transaction"
    );
    let request = result.unwrap();
    assert_eq!(request.status(), LeaveStatus::PreApproved);

    // Verify it was successfully written to the database
    let db_status: String = crate::test_query_scalar_string!(
        &pools.api,
        "SELECT status::text AS val FROM leave_requests WHERE id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        request.id()
    );
    assert_eq!(db_status, "pre_approved");
}

#[tokio::test]
async fn test_leave_service_transaction_rollback_invalid_employment() {
    let (_test_db, factory, state, _app) = setup_app_tower().await;
    let pools = factory.diesel_pools.clone();

    // Setup: owner, workspace
    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let vacation_id: Uuid = crate::test_query_scalar_uuid!(
        &pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let leave_service = state.leaves().service();

    // Get the initial count of leave requests
    let initial_count: i64 =
        crate::test_query_scalar_i64!(&pools.api, "SELECT COUNT(*) FROM leave_requests");

    // Submit leave request with non-existent employment ID
    let submit_req = SubmitLeaveRequest {
        employment_id: Uuid::new_v4(), // Random ID that does not exist
        leave_type_id: vacation_id,
        start_date: chrono::NaiveDate::from_ymd_opt(2027, 9, 1).unwrap(),
        end_date: chrono::NaiveDate::from_ymd_opt(2027, 9, 2).unwrap(),
        reason: Some("Summer holiday".to_string()),
        duration_type: apiserver::models::enums::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        attachment_file_ids: None,
    };

    let result = leave_service
        .submit_leave_request(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            owner_id,
            submit_req,
            true,
        )
        .await;

    assert!(
        result.is_err(),
        "submit_leave_request should fail with invalid employment ID"
    );

    // Verify count did not change (rolled back / never inserted)
    let final_count: i64 =
        crate::test_query_scalar_i64!(&pools.api, "SELECT COUNT(*) FROM leave_requests");
    assert_eq!(
        initial_count, final_count,
        "No requests should be inserted on transaction rollback"
    );
}
