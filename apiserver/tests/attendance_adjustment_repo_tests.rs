mod common;

use apiserver::models::{AdjustmentStatus, AttendanceAdjustmentRequest};
use apiserver::repositories::attendance_adjustment_repository;
use chrono::Utc;
use common::db::TestDatabase;
use common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_attendance_adjustment_repo_lifecycle() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    let request = AttendanceAdjustmentRequest {
        id: Uuid::new_v4(),
        employee_id,
        time_entry_id: None,
        requested_punch_in_time: Utc::now(),
        requested_punch_out_time: Some(Utc::now()),
        is_break_overtime: false,
        reason: Some("Forgot to punch in".to_string()),
        status: AdjustmentStatus::Pending,
        requested_by_user_id: Some(user_id),
        approved_by_user_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // 1. Create
    attendance_adjustment_repository::create_request(&pool, &request)
        .await
        .unwrap();

    // 2. Get by ID
    let fetched = attendance_adjustment_repository::get_request_by_id(&pool, request.id)
        .await
        .unwrap()
        .expect("Request not found");
    assert_eq!(fetched.reason.unwrap(), "Forgot to punch in");

    // 3. Update status
    attendance_adjustment_repository::update_adjustment_status(
        &pool,
        request.id,
        AdjustmentStatus::Approved,
        Some(user_id),
    )
    .await
    .unwrap();
    let updated = attendance_adjustment_repository::get_request_by_id(&pool, request.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated.status, AdjustmentStatus::Approved);
    assert_eq!(updated.approved_by_user_id, Some(user_id));

    // 4. Get workspace requests
    let ws_requests =
        attendance_adjustment_repository::get_workspace_adjustment_requests(&pool, ws_id, None)
            .await
            .unwrap();
    assert_eq!(ws_requests.len(), 1);

    // 5. Get employee requests
    let emp_requests =
        attendance_adjustment_repository::get_employee_adjustment_requests(&pool, employee_id)
            .await
            .unwrap();
    assert_eq!(emp_requests.len(), 1);
}
