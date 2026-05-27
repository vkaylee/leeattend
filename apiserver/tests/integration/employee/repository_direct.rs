//! Repository-level integration tests for split employee sub-modules.
//!
//! These tests exercise free functions in `employee::core_queries`,
//! `employee::recruitment_queries`, and `employee::analytics_queries`
//! that are NOT exposed through REST endpoints.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::WorkingCondition;
use apiserver::repositories::{
    diesel_employee_repository::DieselEmployeeRepository, employee, EmployeeRepository,
};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

// Helper: set up a clean DB + factory (no axum router needed)
async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    (test_db, factory)
}

// ── Working Conditions ──────────────────────────────────────────────

#[tokio::test]
async fn test_working_condition_crud() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("WC Employee")
        .create()
        .await;

    // Create a working condition
    let wc = WorkingCondition {
        id: Uuid::new_v4(),
        employee_id: emp_id,
        type_: "late_arrival".to_string(),
        minute_allowance: 15,
        is_paid: true,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: None,
    };
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    repo.create_working_condition(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        &wc,
    )
    .await
    .unwrap();

    // Read back
    let conditions = repo
        .get_working_conditions(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
        )
        .await
        .unwrap();
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].type_, "late_arrival");
    assert_eq!(conditions[0].minute_allowance, 15);
    assert!(conditions[0].is_paid);

    // Delete
    let deleted = repo
        .delete_working_condition(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            wc.id,
            emp_id,
        )
        .await
        .unwrap();
    assert!(deleted);

    // Confirm empty
    let conditions = repo
        .get_working_conditions(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
        )
        .await
        .unwrap();
    assert!(conditions.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_working_condition_returns_false() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let deleted = repo
        .delete_working_condition(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            Uuid::new_v4(),
            emp_id,
        )
        .await
        .unwrap();
    assert!(!deleted, "Should return false for non-existent condition");
}

#[tokio::test]
async fn test_payroll_config_upsert_and_lookup() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let effective = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());

    // Insert
    let config = repo
        .upsert_employee_payroll_config(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            Decimal::from(22),
            Decimal::from(8),
            effective,
        )
        .await
        .unwrap();
    assert_eq!(config.std_working_days, Decimal::from(22));
    assert_eq!(config.hours_per_day, Decimal::from(8));
    assert_eq!(config.effective_date, effective);

    // Upsert (same effective_date → update)
    let updated = repo
        .upsert_employee_payroll_config(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            Decimal::from(20),
            Decimal::from(7),
            effective,
        )
        .await
        .unwrap();
    assert_eq!(updated.std_working_days, Decimal::from(20));
    assert_eq!(updated.hours_per_day, Decimal::from(7));

    // Lookup at exactly the effective date
    let found = repo
        .get_employee_payroll_config_at(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            effective,
        )
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().std_working_days, Decimal::from(20));

    // Lookup before effective date → should return None
    let before = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();
    let not_found = repo
        .get_employee_payroll_config_at(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            before,
        )
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_payroll_config_returns_closest_effective() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Insert two configs with different effective dates
    let jan = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let jul = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());

    repo.upsert_employee_payroll_config(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        emp_id,
        Decimal::from(22),
        Decimal::from(8),
        jan,
    )
    .await
    .unwrap();
    repo.upsert_employee_payroll_config(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        emp_id,
        Decimal::from(20),
        Decimal::from(7),
        jul,
    )
    .await
    .unwrap();

    // Lookup in March → should get Jan config
    let march = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
    let config = repo
        .get_employee_payroll_config_at(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            march,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config.std_working_days, Decimal::from(22));

    // Lookup in August → should get Jul config
    let august = NaiveDate::from_ymd_opt(2026, 8, 15).unwrap();
    let config = repo
        .get_employee_payroll_config_at(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            august,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(config.std_working_days, Decimal::from(20));
}

// ── Active Employees & Search ───────────────────────────────────────

#[tokio::test]
async fn test_get_active_employees_excludes_inactive() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create 2 active employees
    let _e1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("ACTIVE01")
        .with_full_name("Active One")
        .create()
        .await;
    let e2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("INACTIVE02")
        .with_full_name("Inactive Two")
        .create()
        .await;

    // Deactivate one
    crate::test_execute!(
        &pool.api,
        "UPDATE employees SET is_active = false WHERE id = $1",
        diesel::sql_types::Uuid,
        e2
    );

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let active = repo
        .get_active_employees(&apiserver::diesel_pool::TenantAuthToken::system_override(
            ws_id,
        ))
        .await
        .unwrap();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].full_name, "Active One");
}

#[tokio::test]
async fn test_get_active_employee_ids_with_filter() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let e1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("FILT1")
        .create()
        .await;
    let _e2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("FILT2")
        .create()
        .await;
    let e3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("FILT3")
        .create()
        .await;

    // Request only e1 and e3
    let filter = vec![e1, e3];
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let ids = repo
        .get_active_employee_ids(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            Some(&filter),
        )
        .await
        .unwrap();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&e1));
    assert!(ids.contains(&e3));

    // Without filter → all 3
    let all_ids = repo
        .get_active_employee_ids(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            None,
        )
        .await
        .unwrap();
    assert_eq!(all_ids.len(), 3);
}

// ── Find by Number ──────────────────────────────────────────────────

#[tokio::test]
async fn test_find_by_number_normalizes_input() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let _emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("42")
        .with_full_name("Number Test")
        .create()
        .await;

    // Search with leading zeros → should still find due to normalization
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let found = repo
        .find_by_number(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            "0042",
        )
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().full_name, "Number Test");

    // Search non-existent
    let not_found = repo
        .find_by_number(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            "999999",
        )
        .await
        .unwrap();
    assert!(not_found.is_none());
}

// ── Existing Employee ID Check ──────────────────────────────────────

#[tokio::test]
async fn test_get_existing_employee_ids_by_numbers() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("100")
        .create()
        .await;
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("200")
        .create()
        .await;

    let to_check = vec!["100".to_string(), "200".to_string(), "300".to_string()];
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let existing = repo
        .get_existing_employee_ids_by_numbers(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &to_check,
        )
        .await
        .unwrap();
    assert_eq!(existing.len(), 2);
    assert!(existing.contains(&"100".to_string()));
    assert!(existing.contains(&"200".to_string()));

    // Empty input → empty result
    let empty_result = repo
        .get_existing_employee_ids_by_numbers(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &[],
        )
        .await
        .unwrap();
    assert!(empty_result.is_empty());
}

// ── Analytics Queries ───────────────────────────────────────────────

#[tokio::test]
async fn test_get_turnover_analytics_direct() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create a terminated employee
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Terminated Emp")
        .create()
        .await;

    // Manually set hire_date and termination_date
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "UPDATE employees SET is_active = false, hire_date = $1, termination_date = $2 WHERE id = $3",
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2025, 6, 1).unwrap(),
        diesel::sql_types::Uuid, emp_id
    );

    let start_date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let analytics = repo
        .get_turnover_analytics(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            start_date,
            end_date,
        )
        .await
        .unwrap();

    assert_eq!(analytics.total_terminated, 1);

    // Check tenure distribution: 5 months should be in "3-6 months"
    let bucket = analytics
        .tenure_distribution
        .iter()
        .find(|b| b.bucket == "3-6 months");
    assert!(bucket.is_some());
    assert_eq!(bucket.unwrap().count, 1);
}

// ── Recruitment Queries ─────────────────────────────────────────────

#[tokio::test]
async fn test_recruitment_source_breakdown_direct() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create a recruitment source
    let source_id = Uuid::new_v4();
    let source = apiserver::models::RecruitmentSource {
        id: source_id,
        workspace_id: ws_id,
        source_type: apiserver::models::enums::RecruitmentSourceType::Agency,
        name: "Top Recruiting".to_string(),
        referrer_employee_id: None,
        referral_bonus_amount: None,
        referral_bonus_paid: false,
        agency_contact_name: None,
        agency_contact_email: None,
        agency_fee_amount: None,
        agency_fee_paid: false,
        notes: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    repo.create_recruitment_source(&source).await.unwrap();

    // Create an employee and set recruitment source
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Sourced Emp")
        .create()
        .await;

    repo.set_employee_recruitment_source(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        emp_id,
        Some(source_id),
    )
    .await
    .expect("Failed to link employee to source");

    let breakdown = repo
        .get_recruitment_source_breakdown(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        )
        .await
        .unwrap();

    assert_eq!(breakdown.len(), 1);
    assert_eq!(breakdown[0].source_type, "agency");
    assert_eq!(breakdown[0].source_name, "Top Recruiting");
    assert_eq!(breakdown[0].count, 1);
}

// ── Timezone Fallback ───────────────────────────────────────────────

#[tokio::test]
async fn test_get_employee_timezone_fallback() {
    let (test_db, factory) = setup_db().await;

    let user_id = factory.user().create().await;
    // Create workspace (creates HQ location by default)
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());

    // 1. Employee with custom location and explicit timezone
    use apiserver::schema::work_locations;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    #[derive(Insertable)]
    #[diesel(table_name = work_locations)]
    struct NewLoc {
        id: Uuid,
        workspace_id: Uuid,
        name: String,
        timezone: Option<String>,
        is_headquarters: bool,
    }

    let mut tenant_conn = test_db.diesel_pools.tenants_tx.get_connection(&ws_id).await.unwrap();
    let custom_loc_id = Uuid::new_v4();
    
    diesel::insert_into(work_locations::table)
        .values(&NewLoc {
            id: custom_loc_id,
            workspace_id: ws_id,
            name: "Tokyo Branch".to_string(),
            timezone: Some("Asia/Tokyo".to_string()),
            is_headquarters: false,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .create()
        .await;

    // Update employee to use custom location
    use apiserver::schema::employees;
    diesel::update(employees::table.find(emp_id))
        .set(employees::primary_location_id.eq(custom_loc_id))
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    // Test: returns Asia/Tokyo
    let tz = repo
        .get_employee_timezone_fallback(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
        )
        .await
        .unwrap();
    assert_eq!(tz.name(), "Asia/Tokyo");

    // 2. Employee with location that relies on default timezone
    let default_tz_loc_id = Uuid::new_v4();
    diesel::insert_into(work_locations::table)
        .values(&NewLoc {
            id: default_tz_loc_id,
            workspace_id: ws_id,
            name: "Unknown Branch".to_string(),
            timezone: None, // Triggers PostgreSQL DEFAULT 'UTC'
            is_headquarters: false,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    diesel::update(employees::table.find(emp_id))
        .set(employees::primary_location_id.eq(default_tz_loc_id))
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    // Test: returns 'UTC' because the location's timezone defaults to 'UTC'
    let tz2 = repo
        .get_employee_timezone_fallback(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
        )
        .await
        .unwrap();
    assert_eq!(tz2.name(), "UTC");
}
