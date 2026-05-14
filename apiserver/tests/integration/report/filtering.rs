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
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

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

    let report_service =
        ReportService::new(apiserver::services::report_service::ReportServiceArgs {
            time_entry_repo: std::sync::Arc::new(
                apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
            ),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            attendance_daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            leave_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            diesel: diesel_pools.clone(),
        });

    // Filter by emp1
    let result_1 = report_service
        .get_daily_attendance_report(apiserver::services::report_service::AttendanceReportArgs {
            workspace_id,
            start_date: date,
            end_date: date,
            page: None,
            limit: None,
            employee_ids: Some(vec![emp1]),
            department_id: None,
            sort: None,
        })
        .await
        .expect("Failed to get report");
    assert_eq!(result_1.data.len(), 1);
    assert_eq!(result_1.data[0].employee_id, emp1);

    // Filter by emp2, emp3
    let result_23 = report_service
        .get_daily_attendance_report(apiserver::services::report_service::AttendanceReportArgs {
            workspace_id,
            start_date: date,
            end_date: date,
            page: None,
            limit: None,
            employee_ids: Some(vec![emp2, emp3]),
            department_id: None,
            sort: None,
        })
        .await
        .expect("Failed to get report");
    assert_eq!(result_23.data.len(), 2);
}
