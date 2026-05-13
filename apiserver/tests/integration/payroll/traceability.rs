//! Payroll traceability tests

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::repositories::{department_repository, employee, payroll_repository};
use apiserver::services::payroll_service::PayrollService;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::sync::Arc;

#[tokio::test]
async fn test_payroll_std_days_source_traceability() {
    let db = TestDatabase::new().await;
    let pool = db.pool();
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
    sqlx::query("INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, $3)")
        .bind(employee_id)
        .bind(dept_id)
        .bind(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap())
        .execute(&pool)
        .await
        .unwrap();

    // 2. Setup Base Salary
    let base_comp = payroll_repository::create_salary_component_definition(
        &pool,
        workspace_id,
        "Base Salary".to_string(),
        "BASE".to_string(),
    )
    .await
    .unwrap();

    payroll_repository::assign_employee_salary_component(
        &pool,
        employee_id,
        base_comp.id,
        Decimal::from(10_000_000),
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
    )
    .await
    .unwrap();

    // 3. Scenario A: System Default (No overrides)
    let payslip_a = {
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<
            dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait,
        > = std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    pool.clone(),
                    diesel_pools.clone(),
                ),
            ),

            pool: pool.clone(),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2024, 1)
    .await
    .unwrap();
    let segments_a = payroll_repository::get_payslip_segments(&pool, payslip_a.id)
        .await
        .unwrap();
    assert_eq!(segments_a.len(), 1);
    assert_eq!(segments_a[0].std_days_source.as_deref(), Some("system"));
    assert_eq!(segments_a[0].applied_std_days, Decimal::from(26));

    // 4. Scenario B: Workspace Override
    payroll_repository::update_payroll_settings(
        &pool,
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
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<
            dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait,
        > = std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    pool.clone(),
                    diesel_pools.clone(),
                ),
            ),

            pool: pool.clone(),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2024, 2)
    .await
    .unwrap();
    let segments_b = payroll_repository::get_payslip_segments(&pool, payslip_b.id)
        .await
        .unwrap();
    assert_eq!(segments_b.len(), 1);
    assert_eq!(segments_b[0].std_days_source.as_deref(), Some("workspace"));
    assert_eq!(segments_b[0].applied_std_days, Decimal::from(24));

    // 5. Scenario C: Department Override
    department_repository::upsert_department_payroll_config(
        &pool,
        dept_id,
        Decimal::from(22),
        Decimal::from(8),
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(),
    )
    .await
    .unwrap();

    let payslip_c = {
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<
            dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait,
        > = std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    pool.clone(),
                    diesel_pools.clone(),
                ),
            ),

            pool: pool.clone(),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2024, 3)
    .await
    .unwrap();
    let segments_c = payroll_repository::get_payslip_segments(&pool, payslip_c.id)
        .await
        .unwrap();
    assert_eq!(segments_c.len(), 1);
    assert_eq!(segments_c[0].std_days_source.as_deref(), Some("department"));
    assert_eq!(segments_c[0].applied_std_days, Decimal::from(22));

    // 6. Scenario D: Individual Override
    employee::upsert_employee_payroll_config(
        &pool,
        employee_id,
        Decimal::from(20),
        Decimal::from(8),
        NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(),
    )
    .await
    .unwrap();

    let payslip_d = {
        let pr: std::sync::Arc<dyn apiserver::repositories::payroll_repository::PayrollRepository> =
            std::sync::Arc::new(apiserver::repositories::DieselPayrollRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            ));
        let dr: std::sync::Arc<
            dyn apiserver::repositories::department_repository::DepartmentRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselDepartmentRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let tr: std::sync::Arc<
            dyn apiserver::repositories::time_entry_repository::TimeEntryRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let wr: std::sync::Arc<
            dyn apiserver::repositories::workspace_repository::WorkspaceRepository,
        > = std::sync::Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        let hr: std::sync::Arc<
            dyn apiserver::repositories::holiday_repository::HolidayRepositoryTrait,
        > = std::sync::Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            pool.clone(),
            diesel_pools.clone(),
        ));
        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
                pool.clone(),
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    pool.clone(),
                    diesel_pools.clone(),
                ),
            ),

            pool: pool.clone(),
        })
    }
    .calculate_monthly_payslip(employee_id, workspace_id, 2024, 4)
    .await
    .unwrap();
    let segments_d = payroll_repository::get_payslip_segments(&pool, payslip_d.id)
        .await
        .unwrap();
    assert_eq!(segments_d.len(), 1);
    assert_eq!(segments_d[0].std_days_source.as_deref(), Some("individual"));
    assert_eq!(segments_d[0].applied_std_days, Decimal::from(20));
}
