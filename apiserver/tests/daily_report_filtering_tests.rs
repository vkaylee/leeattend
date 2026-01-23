use chrono::NaiveDate;
use uuid::Uuid;
use apiserver::services::ReportService;

mod common;

use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_daily_report_employee_filtering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    // 1. Setup: Workspace
    let workspace_id = factory.workspace().create().await;

    // 2. Setup: 3 Employees
    let emp1 = factory.employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee One")
        .with_employee_id_number("EMP001")
        .create()
        .await;

    let emp2 = factory.employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee Two")
        .with_employee_id_number("EMP002")
        .create()
        .await;

    let emp3 = factory.employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee Three")
        .with_employee_id_number("EMP003")
        .create()
        .await;

    // 3. Define Range
    let start_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // 4. Test: Filter by emp1
    let result_1 = ReportService::get_daily_attendance_report(
        &pool,
        workspace_id,
        start_date,
        end_date,
        None,
        None,
        Some(vec![emp1]),
    ).await.expect("Failed to get report");

    // Assert: Only emp1 should be in the report
    assert_eq!(result_1.data.len(), 1);
    assert_eq!(result_1.data[0].employee_id, emp1);

    // 5. Test: Filter by emp2 and emp3
    let result_23 = ReportService::get_daily_attendance_report(
        &pool,
        workspace_id,
        start_date,
        end_date,
        None,
        None,
        Some(vec![emp2, emp3]),
    ).await.expect("Failed to get report");

    // Assert: emp2 and emp3 should be in the report
    assert_eq!(result_23.data.len(), 2);
    let ids: Vec<Uuid> = result_23.data.iter().map(|r| r.employee_id).collect();
    assert!(ids.contains(&emp2));
    assert!(ids.contains(&emp3));
    assert!(!ids.contains(&emp1));

    // 6. Test: No Filter (All)
    let result_all = ReportService::get_daily_attendance_report(
        &pool,
        workspace_id,
        start_date,
        end_date,
        None,
        None,
        None,
    ).await.expect("Failed to get report");

    assert_eq!(result_all.data.len(), 3);
}
