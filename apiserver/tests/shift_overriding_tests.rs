mod common;

use apiserver::repositories::shift_repository;
use chrono::{NaiveDate, NaiveTime};
use common::factory::TestFactory;
use common::db::TestDatabase;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

#[tokio::test]
async fn test_temporal_shift_overriding_logic() {
    let (_test_db, factory) = setup_test_db().await;
    let pool = factory.pool.clone();

    // 1. Setup Cơ bản: User, Workspace, Employee
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    // 2. Tạo 2 ca làm việc mẫu
    let shift_morning_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_night_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    // 3. Gán ca MORNING vĩnh viễn từ ngày 2026-01-01
    shift_repository::assign_employee_shift(
        &pool,
        employee_id,
        shift_morning_id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // Kiểm tra ngày 2026-01-15 -> Phải là ca MORNING
    let active_shift = shift_repository::get_employee_active_shift(
        &pool,
        employee_id,
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(active_shift.unwrap().id, shift_morning_id);

    // 4. Gán ca NIGHT tạm thời từ 2026-01-10 đến 2026-01-20
    // Giả lập tạo sau 1 chút để test created_at
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    shift_repository::assign_employee_shift(
        &pool,
        employee_id,
        shift_night_id,
        NaiveDate::from_ymd_opt(2026, 1, 10).unwrap(),
        Some(NaiveDate::from_ymd_opt(2026, 1, 20).unwrap()),
    )
    .await
    .unwrap();

    // Kiểm tra ngày 2026-01-05 (Trước đợt tạm thời) -> Phải là ca MORNING
    let shift_before = shift_repository::get_employee_active_shift(
        &pool,
        employee_id,
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(shift_before.unwrap().id, shift_morning_id);

    // Kiểm tra ngày 2026-01-15 (Trong đợt tạm thời) -> Phải là ca NIGHT (Vì created_at mới hơn)
    let shift_during = shift_repository::get_employee_active_shift(
        &pool,
        employee_id,
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(shift_during.unwrap().id, shift_night_id);

    // Kiểm tra ngày 2026-01-21 (Sau đợt tạm thời) -> Phải tự quay về ca MORNING
    let shift_after = shift_repository::get_employee_active_shift(
        &pool,
        employee_id,
        NaiveDate::from_ymd_opt(2026, 1, 21).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(shift_after.unwrap().id, shift_morning_id);

    // 5. Test gán đè một ca MORNING mới (mãi mãi) lên trên
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let shift_morning_v2_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        )
        .create()
        .await;

    shift_repository::assign_employee_shift(
        &pool,
        employee_id,
        shift_morning_v2_id,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // Kiểm tra ngày 2026-02-25 -> Phải là MORNING V2
    let shift_v2 = shift_repository::get_employee_active_shift(
        &pool,
        employee_id,
        NaiveDate::from_ymd_opt(2026, 2, 25).unwrap(),
    )
    .await
    .unwrap();
    assert_eq!(shift_v2.unwrap().id, shift_morning_v2_id);
}
