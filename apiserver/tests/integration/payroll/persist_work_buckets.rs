//! Integration test: persist-workbuckets — verify 5 new summary fields
//! are persisted correctly and payroll reads them as single source of truth.

use crate::common::factory::TestFactory;
use apiserver::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository;
use apiserver::services::attendance_summary_service::{
    AttendanceSummaryService, AttendanceSummaryServiceArgs,
};
use apiserver::services::payroll_service::PayrollService;
use chrono::{NaiveDate, NaiveTime, TimeZone};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

fn asia_tz() -> chrono_tz::Tz {
    chrono_tz::Asia::Ho_Chi_Minh
}

/// End-to-end: create time entries → recalculate → verify persisted fields → payroll
#[tokio::test]
async fn test_persist_work_buckets_and_payroll_consistency() {
    let test_db = crate::common::db::TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone());

    // ---- Setup ----
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Monday 2026-07-27
    let work_date = NaiveDate::from_ymd_opt(2026, 7, 27).unwrap();
    let first_of_month = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

    // Default policy: work_unit_minutes=60 (1 unit = 1 hour), rounding=floor
    crate::test_execute!(&pool.api,
        "UPDATE attendance_policy_versions SET rules = '{\"work_unit_minutes\": 60, \"rounding_policy\": \"floor\"}'::jsonb WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true)",
        diesel::sql_types::Uuid, workspace_id
    );

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .with_hire_date(first_of_month)
        .create()
        .await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp
    );

    // Create shift: 8:00-17:00, 1h break 12:00-13:00
    let shift = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_break(
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
        )
        .create()
        .await;

    // Assign shift to employee via roster for the work date
    factory
        .roster()
        .with_employee(emp)
        .with_shift(&shift)
        .with_date(work_date)
        .create()
        .await;

    // Punch 7:30-17:15 → 30min early OT + 15min late OT = 45min OT
    let tz = asia_tz();
    let punch_in = work_date
        .and_hms_opt(7, 30, 0)
        .and_then(|dt| tz.from_local_datetime(&dt).single())
        .unwrap();
    let punch_out = work_date
        .and_hms_opt(17, 15, 0)
        .and_then(|dt| tz.from_local_datetime(&dt).single())
        .unwrap();

    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    // Salary: 5,200,000 VND/month
    let comp_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "INSERT INTO salary_component_definitions (workspace_id, name, code)
         VALUES ($1, 'Base', 'BASE') RETURNING id",
        diesel::sql_types::Uuid, workspace_id
    );
    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date)
         VALUES ($1, $2, 5200000, $3)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, comp_id,
        diesel::sql_types::Date, first_of_month
    );

    // ---- Phase 1: recalculate daily summary ----
    let auth_token =
        apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id);

    let summary_svc = {
        let tr = Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        ));
        let rr = Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        ));
        let pol = Arc::new(apiserver::repositories::DieselAttendancePolicyRepository::new(
            diesel_pools.clone(),
        ));
        let ct = Arc::new(apiserver::repositories::DieselConditionTypeRepository::new(
            diesel_pools.clone(),
        ));
        let wr = Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        ));
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let emp_r = Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
            diesel_pools.clone(),
        ));
        let hr = Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        ));
        let lr = Arc::new(apiserver::repositories::DieselLeaveRequestRepository::new(
            diesel_pools.clone(),
        ));
        let dsr: Arc<dyn AttendanceDailySummaryRepository> = Arc::new(
            apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                diesel_pools.clone(),
            ),
        );

        AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
            time_entry_repo: tr,
            roster_repo: rr,
            policy_repo: pol,
            condition_type_repo: ct,
            workspace_repo: wr,
            employee_repo: er,
            employment_repo: emp_r,
            holiday_repo: hr,
            leave_request_repo: lr,
            daily_summary_repo: dsr,
        })
    };

    summary_svc
        .recalculate_daily_summary(&auth_token, employment_id, work_date)
        .await
        .expect("recalculate_daily_summary should succeed");

    // ---- Phase 2: verify persisted fields in DB ----
    let summary = {
        let mut conn = diesel_pools
            .tenants_tx
            .get_connection(&workspace_id)
            .await
            .unwrap();
        use apiserver::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;
        use apiserver::schema::attendance_daily_summaries::dsl as ads;
        use diesel::prelude::*;

        ads::attendance_daily_summaries
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.eq(work_date))
            .first::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .unwrap()
    };

    // Monday → is_weekend = false
    assert!(!summary.is_weekend, "Monday should not be weekend");

    // 7:30-17:15 with 8:00-17:00 shift + 1h break:
    // Standard: 8:00-12:00 + 13:00-17:00 = 480min
    // Early OT: 7:30-8:00 = 30min
    // Late OT: 17:00-17:15 = 15min
    // Break OT: 0 (break inside shift)
    assert_eq!(summary.standard_minutes, 480, "standard_minutes");
    assert_eq!(summary.early_ot_minutes, 30, "early_ot_minutes");
    assert_eq!(summary.late_ot_minutes, 15, "late_ot_minutes");
    assert_eq!(summary.break_ot_minutes, 0, "break_ot_minutes");
    assert_eq!(
        summary.actual_work_minutes,
        480 + 30 + 15,
        "actual_work_minutes"
    );
    assert_eq!(
        summary.system_detected_ot_minutes,
        30 + 15,
        "system_detected_ot_minutes"
    );

    // ---- Phase 3: Payroll from summaries ----
    let payslip = {
        let pr = Arc::new(apiserver::repositories::DieselPayrollRepository::new(
            diesel_pools.clone(),
        ));
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
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );
        let dsr: Arc<dyn AttendanceDailySummaryRepository> = Arc::new(
            apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                diesel_pools.clone(),
            ),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
            department_repo: dr,
            time_entry_repo: tr,
            workspace_repo: wr,
            holiday_repo: hr,
            roster_repo: rr,
            leave_repo: lr,
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
            employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
                diesel_pools.clone(),
            )),
            employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
                diesel_pools.clone(),
            )),
            daily_summary_repo: dsr,
        })
    }
    .calculate_monthly_payslip(emp, &auth_token, 2026, 7)
    .await
    .expect("Payroll calculation should succeed");

    // work_unit_minutes=60, rounding=floor:
    // standard_units = floor(480/60) = 8
    // ot_units = floor((30+15)/60) = floor(0.75) = 0
    assert_eq!(payslip.standard_units(), 8, "standard_units");
    assert_eq!(payslip.overtime_units(), 0, "overtime_units");

    // daily_rate = 5,200,000 / 26 = 200,000
    // worked_days = 8/8 = 1.0
    // base_pay = 200,000
    assert_eq!(
        payslip.base_pay_amount().round_dp(0),
        Decimal::from(200_000),
        "base_pay_amount"
    );
}

/// Verify is_weekend is true when recalculating on a Saturday.
#[tokio::test]
async fn test_is_weekend_persisted_for_saturday() {
    let test_db = crate::common::db::TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Saturday 2026-07-25
    let saturday = NaiveDate::from_ymd_opt(2026, 7, 25).unwrap();
    let first_of_month = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .with_hire_date(first_of_month)
        .create()
        .await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp
    );

    // Shift on Saturday
    let shift = factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        )
        .create()
        .await;

    factory
        .roster()
        .with_employee(emp)
        .with_shift(&shift)
        .with_date(saturday)
        .create()
        .await;

    // Minimal time entry
    let tz = asia_tz();
    let punch_in = saturday
        .and_hms_opt(8, 0, 0)
        .and_then(|dt| tz.from_local_datetime(&dt).single())
        .unwrap();
    let punch_out = saturday
        .and_hms_opt(12, 0, 0)
        .and_then(|dt| tz.from_local_datetime(&dt).single())
        .unwrap();

    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    let auth_token =
        apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id);

    // Build summary service inline
    let dsr: Arc<dyn AttendanceDailySummaryRepository> = Arc::new(
        apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
            diesel_pools.clone(),
        ),
    );
    let summary_svc = AttendanceSummaryService::new(AttendanceSummaryServiceArgs {
        time_entry_repo: Arc::new(apiserver::repositories::DieselTimeEntryRepository::new(
            diesel_pools.clone(),
        )),
        roster_repo: Arc::new(apiserver::repositories::DieselRosterRepository::new(
            diesel_pools.clone(),
        )),
        policy_repo: Arc::new(apiserver::repositories::DieselAttendancePolicyRepository::new(
            diesel_pools.clone(),
        )),
        condition_type_repo: Arc::new(
            apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
        ),
        workspace_repo: Arc::new(apiserver::repositories::DieselWorkspaceRepository::new(
            diesel_pools.clone(),
        )),
        employee_repo: Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        )),
        employment_repo: Arc::new(apiserver::repositories::DieselEmploymentRepository::new(
            diesel_pools.clone(),
        )),
        holiday_repo: Arc::new(apiserver::repositories::DieselHolidayRepository::new(
            diesel_pools.clone(),
        )),
        leave_request_repo: Arc::new(
            apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
        ),
        daily_summary_repo: dsr.clone(),
    });

    summary_svc
        .recalculate_daily_summary(&auth_token, employment_id, saturday)
        .await
        .expect("recalculate_daily_summary should succeed");

    // Verify is_weekend in DB
    let summary = {
        let mut conn = diesel_pools
            .tenants_tx
            .get_connection(&workspace_id)
            .await
            .unwrap();
        use apiserver::models::diesel::attendance_daily_summary::AttendanceDailySummaryEntity;
        use apiserver::schema::attendance_daily_summaries::dsl as ads;
        use diesel::prelude::*;

        ads::attendance_daily_summaries
            .filter(ads::employment_id.eq(employment_id))
            .filter(ads::date.eq(saturday))
            .first::<AttendanceDailySummaryEntity>(&mut *conn)
            .await
            .unwrap()
    };

    assert!(summary.is_weekend, "Saturday must have is_weekend=true");
    // 8:00-12:00 with no break = 240 standard minutes on Saturday
    assert_eq!(summary.standard_minutes, 240, "standard_minutes for half day");
    assert_eq!(summary.early_ot_minutes, 0, "no early OT");
    assert_eq!(summary.late_ot_minutes, 0, "no late OT");
    assert_eq!(summary.break_ot_minutes, 0, "no break OT");
}
