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
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

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
    let repo = apiserver::repositories::DieselWorkingPatternRepository::new(diesel_pools.clone());
    use apiserver::repositories::working_pattern_repository::WorkingPatternRepositoryTrait;

    let pattern_id = repo
        .create_pattern_with_details(
            workspace_id,
            "Test Pattern",
            Some("Description"),
            vec![(1, Some(shift_mon_id)), (2, Some(shift_tuj_id))],
        )
        .await
        .unwrap()
        .id;

    // 4. Assign Pattern to Employee
    repo.assign_pattern(
        workspace_id,
        employee_id,
        pattern_id,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    )
    .await
    .unwrap();

    // 5. Build Roster Cache Data
    let _rosters = apiserver::cache::build_roster_data(&diesel_pools, workspace_id)
        .await
        .unwrap();

    // Check assignment
    let assignments = repo
        .get_employee_pattern_history(workspace_id, employee_id)
        .await
        .unwrap();
    assert_eq!(assignments.len(), 1);
}
