//! Report filtering tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::ReportService;
use chrono::NaiveDate;

#[tokio::test]
async fn test_daily_report_employee_filtering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
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
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            attendance_daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            leave_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employer_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployerRepository::new(diesel_pools.clone()),
            ),
        });

    let emp1_employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp1
    );

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
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        })
        .await
        .expect("Failed to get report");
    assert_eq!(result_1.data.len(), 1);
    assert_eq!(result_1.data[0].employment_id, emp1_employment_id);

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
            employee_status: None,
            employer_id: None,
            employment_id: None,
            shift_type: None,
        })
        .await
        .expect("Failed to get report");
    assert_eq!(result_23.data.len(), 2);
}

#[tokio::test]
async fn test_attendance_report_resigned_employee_filtering() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let workspace_id = factory.workspace().create().await;

    // Create emp1: will remain active
    let emp1 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Create emp2: will be terminated/resigned
    let emp2 = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Modify emp2's employment to be terminated on 2026-01-15
    crate::test_execute!(
        &pool.api,
        "UPDATE employments SET status = 'terminated', end_date = '2026-01-15'::date WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp2
    );

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
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            attendance_daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            leave_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employer_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployerRepository::new(diesel_pools.clone()),
            ),
        });

    // Case 1: Report date range is [2026-01-01, 2026-01-31], status filter is "active"
    // emp2 resigned on 2026-01-15 (within the range), so emp2 must be included!
    let range_jan = report_service
        .get_daily_attendance_report(apiserver::services::report_service::AttendanceReportArgs {
            workspace_id,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
            page: None,
            limit: None,
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: Some("active".to_string()),
            employer_id: None,
            employment_id: None,
            shift_type: None,
        })
        .await
        .expect("Failed to get report");
    // Both employees must be active in January
    assert_eq!(range_jan.data.len(), 62);

    // Case 2: Report date range is [2026-02-01, 2026-02-28], status filter is "active"
    // emp2 resigned on 2026-01-15 (before the start of range), so emp2 must be excluded!
    let range_feb_active = report_service
        .get_daily_attendance_report(apiserver::services::report_service::AttendanceReportArgs {
            workspace_id,
            start_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
            page: None,
            limit: None,
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: Some("active".to_string()),
            employer_id: None,
            employment_id: None,
            shift_type: None,
        })
        .await
        .expect("Failed to get report");
    // Only emp1 should show up (28 days * 1 employee = 28)
    assert_eq!(range_feb_active.data.len(), 28);

    // Case 3: Report date range is [2026-02-01, 2026-02-28], status filter is "all"
    // Both employees must be included regardless of resignation (28 days * 2 employees = 56)
    let range_feb_all = report_service
        .get_daily_attendance_report(apiserver::services::report_service::AttendanceReportArgs {
            workspace_id,
            start_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
            page: None,
            limit: None,
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: Some("all".to_string()),
            employer_id: None,
            employment_id: None,
            shift_type: None,
        })
        .await
        .expect("Failed to get report");
    assert_eq!(range_feb_all.data.len(), 56);

    // Case 4: Report date range is [2026-02-01, 2026-02-28], status filter is "terminated"
    // Only emp2 (terminated) should show up (28 days * 1 employee = 28)
    let range_feb_terminated = report_service
        .get_daily_attendance_report(apiserver::services::report_service::AttendanceReportArgs {
            workspace_id,
            start_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 2, 28).unwrap(),
            page: None,
            limit: None,
            employee_ids: None,
            department_id: None,
            sort: None,
            employee_status: Some("terminated".to_string()),
            employer_id: None,
            employment_id: None,
            shift_type: None,
        })
        .await
        .expect("Failed to get report");
    assert_eq!(range_feb_terminated.data.len(), 28);
}

#[tokio::test]
async fn test_report_multi_employment_and_employer() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let workspace_id = factory.workspace().create().await;

    // Create Employee
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // The factory created 1 employment for this employee.
    // Let's retrieve that first employment and its employer.
    let first_employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp
    );

    let first_employer_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT employer_id AS id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp
    );

    // Let's create a second Employer.
    let second_employer_id = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Second Test Employer', 'TAX-2', 'Address 2', NOW(), NOW())",
        diesel::sql_types::Uuid, second_employer_id,
        diesel::sql_types::Uuid, workspace_id
    );

    // Let's create a second active Employment for the same employee under the second employer.
    let second_employment_id = uuid::Uuid::new_v4();
    let hire_date = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, employment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, NULL, 'active', $5, 'part_time', NOW(), NOW())",
        diesel::sql_types::Uuid, second_employment_id,
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Uuid, emp,
        diesel::sql_types::Uuid, second_employer_id,
        diesel::sql_types::Date, hire_date
    );

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
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            attendance_daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            leave_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employer_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployerRepository::new(diesel_pools.clone()),
            ),
        });

    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    // 1. Get attendance summary. It should return 2 entries for the single employee (one for each employment).
    let summary = report_service
        .get_workspace_attendance_summary(
            apiserver::services::report_service::AttendanceReportArgs {
                workspace_id,
                start_date: date,
                end_date: date,
                page: None,
                limit: None,
                employee_ids: Some(vec![emp]),
                department_id: None,
                sort: None,
                employee_status: None,
                employer_id: None,
                employment_id: None,
                shift_type: None,
            },
        )
        .await
        .expect("Failed to get summary");

    assert_eq!(summary.data.len(), 2);
    assert!(summary.data.iter().any(
        |s| s.employment_id == first_employment_id && s.employer_id == Some(first_employer_id)
    ));
    assert!(summary
        .data
        .iter()
        .any(|s| s.employment_id == second_employment_id
            && s.employer_id == Some(second_employer_id)));

    // 2. Filter by employer_id (first employer)
    let summary_first_employer = report_service
        .get_workspace_attendance_summary(
            apiserver::services::report_service::AttendanceReportArgs {
                workspace_id,
                start_date: date,
                end_date: date,
                page: None,
                limit: None,
                employee_ids: Some(vec![emp]),
                department_id: None,
                sort: None,
                employee_status: None,
                employer_id: Some(first_employer_id),
                employment_id: None,
                shift_type: None,
            },
        )
        .await
        .expect("Failed to get summary");

    assert_eq!(summary_first_employer.data.len(), 1);
    assert_eq!(
        summary_first_employer.data[0].employment_id,
        first_employment_id
    );

    // 3. Filter by employment_id (second employment)
    let summary_second_employment = report_service
        .get_workspace_attendance_summary(
            apiserver::services::report_service::AttendanceReportArgs {
                workspace_id,
                start_date: date,
                end_date: date,
                page: None,
                limit: None,
                employee_ids: Some(vec![emp]),
                department_id: None,
                sort: None,
                employee_status: None,
                employer_id: None,
                employment_id: Some(second_employment_id),
                shift_type: None,
            },
        )
        .await
        .expect("Failed to get summary");

    assert_eq!(summary_second_employment.data.len(), 1);
    assert_eq!(
        summary_second_employment.data[0].employment_id,
        second_employment_id
    );
}
