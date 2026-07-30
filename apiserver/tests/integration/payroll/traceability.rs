//! Payroll traceability tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{
    department_repository::DepartmentRepository,
    employee::{EmployeePayrollConfigRepository, EmployeeRepository},
    holiday_repository::HolidayRepository,
    leave_request_repository::LeaveRequestRepository,
    payroll_repository::PayrollRepository,
    roster::RosterRepository,
    time_entry_repository::TimeEntryRepository,
    workspace_repository::WorkspaceRepository,
};
use apiserver::services::payroll_service::PayrollService;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_payroll_std_days_source_traceability() {
    let db = TestDatabase::new().await;
    let pool = db.diesel_pools.clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&db.url());
    let factory = TestFactory::new(pool.clone());

    // 1. Setup Workspace & Employee
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let dept_id = factory
        .department()
        .with_workspace(workspace_id)
        .with_name("Engineering")
        .create()
        .await;
    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // Assign employee to department
    crate::test_execute!(
        &diesel_pools.api,
        "UPDATE employments SET department_id = $2, start_date = $3 WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        dept_id,
        diesel::sql_types::Date,
        NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()
    );

    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // 2. Setup Base Salary
    let pr = apiserver::repositories::DieselPayrollRepository::new(diesel_pools.clone());
    let base_comp = pr
        .create_salary_component_definition(
            workspace_id,
            "Base Salary".to_string(),
            "BASE".to_string(),
            None,
        )
        .await
        .unwrap();

    pr.assign_employment_salary_component(
        workspace_id,
        employment_id,
        base_comp.id,
        Decimal::from(10_000_000),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    )
    .await
    .unwrap();

    // 3. Scenario A: System Default (No overrides)
    let payslip_a = {
        let dr = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr = Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr = Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));
        let rr = Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        ));
        let lr = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: Arc::new(pr),
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: rr,
            leave_repo: lr,
            employee_repo: er,
            policy_repo: atr,
            condition_type_repo: Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(
                    std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                        diesel_pools.clone(),
                    )),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            daily_summary_repo: std::sync::Arc::new(apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone())),
        })
    }
    .calculate_monthly_payslip(
        employee_id,
        &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        2024,
        1,
    )
    .await
    .unwrap();
    let pr_instance = apiserver::repositories::DieselPayrollRepository::new(diesel_pools.clone());
    let segments_a = pr_instance
        .get_payslip_segments(workspace_id, payslip_a.id())
        .await
        .unwrap();
    assert_eq!(segments_a.len(), 1);
    assert_eq!(segments_a[0].std_days_source(), Some("system"));
    assert_eq!(segments_a[0].applied_std_days(), Decimal::from(26));

    // 4. Scenario B: Workspace Override
    pr_instance
        .update_payroll_settings(
            workspace_id,
            &apiserver::models::SavePayrollSettings {
                std_working_day_mode: "fixed".to_string(),
                fixed_std_days: Decimal::from(24),
                working_days_per_week: 5,
                hours_per_day: Decimal::from(8),
                max_ot_hours_per_day: None,
                max_ot_hours_per_month: None,
                effective_date: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
            },
        )
        .await
        .unwrap();

    let payslip_b = {
        let dr = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr = Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr = Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));
        let rr = Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        ));
        let lr = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            )),
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: rr,
            leave_repo: lr,
            employee_repo: er,
            policy_repo: atr,
            condition_type_repo: Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(
                    std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                        diesel_pools.clone(),
                    )),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            daily_summary_repo: std::sync::Arc::new(apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone())),
        })
    }
    .calculate_monthly_payslip(
        employee_id,
        &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        2024,
        2,
    )
    .await
    .unwrap();
    let segments_b = pr_instance
        .get_payslip_segments(workspace_id, payslip_b.id())
        .await
        .unwrap();
    assert_eq!(segments_b.len(), 1);
    assert_eq!(segments_b[0].std_days_source(), Some("workspace"));
    assert_eq!(segments_b[0].applied_std_days(), Decimal::from(24));

    // 5. Scenario C: Department Override
    let dept_repo = apiserver::repositories::DieselDepartmentRepository::new(diesel_pools.clone());
    dept_repo
        .upsert_department_payroll_config(
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
            dept_id,
            Decimal::from(22),
            Decimal::from(8),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
        )
        .await
        .unwrap();

    let payslip_c = {
        let dr = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr = Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr = Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));
        let rr = Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        ));
        let lr = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            )),
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: rr,
            leave_repo: lr,
            employee_repo: er,
            policy_repo: atr,
            condition_type_repo: Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(
                    std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                        diesel_pools.clone(),
                    )),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            daily_summary_repo: std::sync::Arc::new(apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone())),
        })
    }
    .calculate_monthly_payslip(
        employee_id,
        &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        2024,
        3,
    )
    .await
    .unwrap();
    let segments_c = pr_instance
        .get_payslip_segments(workspace_id, payslip_c.id())
        .await
        .unwrap();
    assert_eq!(segments_c.len(), 1);
    assert_eq!(segments_c[0].std_days_source(), Some("department"));
    assert_eq!(segments_c[0].applied_std_days(), Decimal::from(22));

    // 6. Scenario D: Individual Override
    let employee_repo =
        apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone());
    employee_repo
        .upsert_employee_payroll_config(
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
            employment_id,
            Decimal::from(20),
            Decimal::from(8),
            NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
        )
        .await
        .unwrap();

    let payslip_d = {
        let dr = Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr = Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr = Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));
        let rr = Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        ));
        let lr = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            )),
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: rr,
            leave_repo: lr,
            employee_repo: er,
            policy_repo: atr,
            condition_type_repo: Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            overtime_policy_repo: Arc::new(
                apiserver::repositories::DieselOvertimePolicyRepository::new(diesel_pools.clone()),
            ),
            salary_base_repo: std::sync::Arc::new(
                apiserver::repositories::DieselSalaryBaseRepository::new(diesel_pools.clone()),
            ),

            roster_provider: std::sync::Arc::new(
                apiserver::services::payroll_service::DirectRosterDataProvider(
                    std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                        diesel_pools.clone(),
                    )),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            daily_summary_repo: std::sync::Arc::new(apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone())),
        })
    }
    .calculate_monthly_payslip(
        employee_id,
        &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        2024,
        4,
    )
    .await
    .unwrap();
    let segments_d = pr_instance
        .get_payslip_segments(workspace_id, payslip_d.id())
        .await
        .unwrap();
    assert_eq!(segments_d.len(), 1);
    assert_eq!(segments_d[0].std_days_source(), Some("individual"));
    assert_eq!(segments_d[0].applied_std_days(), Decimal::from(20));
}
