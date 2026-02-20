//! Report filtering tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::ReportService;
use chrono::NaiveDate;

#[tokio::test]
async fn test_daily_report_employee_filtering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let workspace_id = factory.workspace().create().await;

    let emp1 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let emp2 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let emp3 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // Filter by emp1
    let result_1 = ReportService::get_daily_attendance_report(
        &pool,
        workspace_id,
        date,
        date,
        None,
        None,
        Some(vec![emp1]),
    )
    .await
    .expect("Failed to get report");
    assert_eq!(result_1.data.len(), 1);
    assert_eq!(result_1.data[0].employee_id, emp1);

    // Filter by emp2, emp3
    let result_23 = ReportService::get_daily_attendance_report(
        &pool,
        workspace_id,
        date,
        date,
        None,
        None,
        Some(vec![emp2, emp3]),
    )
    .await
    .expect("Failed to get report");
    assert_eq!(result_23.data.len(), 2);
}
