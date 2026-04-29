//! Roster pattern resolution tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::working_pattern_repository;
use chrono::NaiveDate;

#[tokio::test]
async fn test_roster_pattern_resolution() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // 1. Create Shifts
    let shift_mon_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Mon Shift")
        .create()
        .await;
    let shift_tuj_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Tue Shift")
        .create()
        .await;

    // 2. Create Working Pattern
    let pattern_id = working_pattern_repository::create_pattern(
        &pool,
        workspace_id,
        "Test Pattern",
        Some("Description"),
    )
    .await
    .unwrap()
    .id;

    // 3. Add Pattern Shifts (Mon=1, Tue=2)
    working_pattern_repository::add_pattern_detail(&pool, pattern_id, 1, Some(shift_mon_id))
        .await
        .unwrap();
    working_pattern_repository::add_pattern_detail(&pool, pattern_id, 2, Some(shift_tuj_id))
        .await
        .unwrap();

    // 4. Assign Pattern to Employee
    working_pattern_repository::assign_pattern(
        &pool,
        workspace_id,
        employee_id,
        pattern_id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // 5. Build Roster Cache Data
    let _rosters = apiserver::cache::build_roster_data(&pool, workspace_id)
        .await
        .unwrap();

    // Check assignment
    let assignments =
        working_pattern_repository::get_employee_pattern_history(&pool, workspace_id, employee_id)
            .await
            .unwrap();
    assert_eq!(assignments.len(), 1);
}
