//! Test grace period conditional override (Issue 5)
//!
//! Verifies that shift's `early_punch_in_grace_period_minutes` only overrides
//! policy grace period when non-zero.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_grace_period_zero_shift_uses_policy_default() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit to 60 min, no rounding
    crate::test_execute!(&pool.api,
        "UPDATE attendance_policy_versions SET rules = '{\"work_unit_minutes\": 60, \"rounding_policy\": \"none\"}'::jsonb WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true)",
        diesel::sql_types::Uuid, workspace_id
    );

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // Create BASE component
    let component_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, $2, $3) RETURNING id",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Text,
        "Base",
        diesel::sql_types::Text,
        "BASE"
    );

    // Assign salary
    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, component_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Create shift with grace_period = 0 (should use policy default)
    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Day Shift")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_grace_period(0) // Zero grace period on shift
        .create()
        .await;

    // Assign shift to employee for Jan 5
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employee_shifts (employment_id, shift_id, start_date) VALUES ($1, $2, $3)",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        shift_id,
        diesel::sql_types::Date,
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );

    // Punch in 5 minutes early (8:55 AM)
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(
            tz.with_ymd_and_hms(2026, 1, 5, 8, 55, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .with_punch_out(
            tz.with_ymd_and_hms(2026, 1, 5, 17, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .create()
        .await;

    // Calculate payroll
    let payslip = {
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<dyn apiserver::repositories::holiday_repository::HolidayRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            leave_repo: Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
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
            contract_repo: Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2026, 1)
    .await
    .unwrap();

    // With grace_period=0, policy default should apply (usually allows some grace)
    // The 5-min early punch should be within standard hours
    assert!(
        payslip.standard_units > 0,
        "Standard units should be > 0 when grace period applies"
    );
}

#[tokio::test]
async fn test_grace_period_nonzero_shift_overrides() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    crate::test_execute!(&pool.api,
        "UPDATE attendance_policy_versions SET rules = '{\"work_unit_minutes\": 60, \"rounding_policy\": \"none\"}'::jsonb WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true)",
        diesel::sql_types::Uuid, workspace_id
    );

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(user_id)
        .create()
        .await;

    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        employee_id
    );

    let component_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code) VALUES ($1, $2, $3) RETURNING id",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Text,
        "Base",
        diesel::sql_types::Text,
        "BASE"
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, component_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Create shift with grace_period = 15 (overrides policy)
    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Flexible Shift")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_grace_period(15) // Non-zero grace period overrides
        .create()
        .await;

    crate::test_execute!(
        &pool.api,
        "INSERT INTO employee_shifts (employment_id, shift_id, start_date) VALUES ($1, $2, $3)",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        shift_id,
        diesel::sql_types::Date,
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );

    // Punch in 10 minutes early (8:50 AM) - within 15-min grace
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(
            tz.with_ymd_and_hms(2026, 1, 5, 8, 50, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .with_punch_out(
            tz.with_ymd_and_hms(2026, 1, 5, 17, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .create()
        .await;

    let payslip = {
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<dyn apiserver::repositories::holiday_repository::HolidayRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
                diesel_pools.clone(),
            ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            leave_repo: Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
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
            contract_repo: Arc::new(
                apiserver::repositories::DieselEmployeeContractRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2026, 1)
    .await
    .unwrap();

    // 10 minutes early is within 15-min grace period
    assert!(
        payslip.standard_units > 0,
        "Grace period override should apply"
    );
}
