mod common;

use apiserver::repositories::{roster_repository, working_pattern_repository, shift_repository};
use chrono::{NaiveDate, NaiveTime, Datelike};
use common::factory::TestFactory;
use common::db::TestDatabase;
use uuid::Uuid;

async fn setup_test_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.pool());
    (test_db, factory)
}

#[tokio::test]
async fn test_roster_pattern_resolution() {
    let (_test_db, factory) = setup_test_db().await;
    let pool = factory.pool.clone();

    // 1. Setup
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let shift_morning_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    // 2. Create Pattern: Mon-Fri Morning, Sat-Sun Off
    let pattern = working_pattern_repository::create_pattern(
        &pool,
        workspace_id,
        "Standard Office",
        Some("Monday to Friday 8-5"),
    )
    .await
    .unwrap();

    // Details: 1=Mon to 5=Fri
    for dow in 1..=5 {
        working_pattern_repository::add_pattern_detail(&pool, pattern.id, dow, Some(shift_morning_id))
            .await
            .unwrap();
    }
    // Sat and Sun (6 and 0) as NULL
    working_pattern_repository::add_pattern_detail(&pool, pattern.id, 6, None).await.unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pattern.id, 0, None).await.unwrap();

    // 3. Assign Pattern to Employee
    working_pattern_repository::assign_pattern(
        &pool,
        workspace_id,
        employee_id,
        pattern.id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // 4. Verify Resolution
    
    // 2026-01-05 is a Monday
    let monday = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    assert_eq!(monday.weekday().number_from_monday(), 1); // Extra check
    
    let shift_mon = roster_repository::get_effective_shift(&pool, employee_id, monday).await.unwrap();
    assert_eq!(shift_mon, Some(shift_morning_id), "Monday should be Morning shift");

    // 2026-01-11 is a Sunday
    let sunday = NaiveDate::from_ymd_opt(2026, 1, 11).unwrap();
    let shift_sun = roster_repository::get_effective_shift(&pool, employee_id, sunday).await.unwrap();
    assert_eq!(shift_sun, None, "Sunday should be Day Off (None)");

    // 5. Verify Override (Roster > Pattern)
    // Manually assign a different shift for that Monday
    let shift_special_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
        )
        .create()
        .await;

    roster_repository::upsert_roster(
        &pool,
        &apiserver::models::ShiftRoster {
            id: Uuid::new_v4(),
            employee_id,
            shift_id: Some(shift_special_id),
            date: monday,
            status: "confirmed".to_string(),
            notes: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    )
    .await
    .unwrap();

    let shift_overridden = roster_repository::get_effective_shift(&pool, employee_id, monday).await.unwrap();
    assert_eq!(shift_overridden, Some(shift_special_id), "Roster should override Pattern");
}

#[tokio::test]
async fn test_pattern_overrides_fixed() {
    let (_test_db, factory) = setup_test_db().await;
    let pool = factory.pool.clone();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let shift_fixed_id = factory.shift().with_workspace(workspace_id).create().await;
    let shift_pattern_id = factory.shift().with_workspace(workspace_id).create().await;

    // 1. Assign Fixed Shift
    shift_repository::assign_employee_shift(
        &pool,
        employee_id,
        shift_fixed_id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // 2. Assign Pattern
    let pattern = working_pattern_repository::create_pattern(&pool, workspace_id, "Pattern", None).await.unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pattern.id, 1, Some(shift_pattern_id)).await.unwrap(); // Monday
    
    working_pattern_repository::assign_pattern(
        &pool,
        workspace_id,
        employee_id,
        pattern.id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // 3. Verify Pattern wins
    let monday = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
    let effective = roster_repository::get_effective_shift(&pool, employee_id, monday).await.unwrap();
    assert_eq!(effective, Some(shift_pattern_id), "Pattern should override Legacy Fixed Assignment");
}
