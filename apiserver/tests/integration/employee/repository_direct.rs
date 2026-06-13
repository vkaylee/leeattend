//! Repository-level integration tests for split employee sub-modules.
//!
//! These tests exercise free functions in `employee::core_queries`,
//! `employee::recruitment_queries`, and `employee::analytics_queries`
//! that are NOT exposed through REST endpoints.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::WorkingCondition;
use apiserver::repositories::{
    diesel_employee_repository::DieselEmployeeRepository,
    employee::{
        EmployeeCoreRepository, EmployeePayrollConfigRepository, EmployeeRecruitmentRepository,
        EmployeeWorkingConditionRepository,
    },
    EmployeeRepository,
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

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // Create a working condition
    let wc = WorkingCondition::new(
        Uuid::new_v4(),
        employment_id,
        "late_arrival".to_string(),
        15,
        true,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        None,
    );
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
            employment_id,
            1,
            50,
        )
        .await
        .unwrap();
    assert_eq!(conditions.0.len(), 1);
    assert_eq!(conditions.0[0].type_(), "late_arrival");
    assert_eq!(conditions.0[0].minute_allowance(), 15);
    assert!(conditions.0[0].is_paid());

    // Delete
    let deleted = repo
        .delete_working_condition(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            wc.id(),
            employment_id,
        )
        .await
        .unwrap();
    assert!(deleted);

    // Confirm empty
    let conditions = repo
        .get_working_conditions(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
            1,
            50,
        )
        .await
        .unwrap();
    assert!(conditions.0.is_empty());
}

#[tokio::test]
async fn test_delete_nonexistent_working_condition_returns_false() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let deleted = repo
        .delete_working_condition(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            Uuid::new_v4(),
            employment_id,
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

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let effective = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());

    // Insert
    let config = repo
        .upsert_employee_payroll_config(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            employment_id,
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
            employment_id,
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
            employment_id,
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
            employment_id,
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

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // Insert two configs with different effective dates
    let jan = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let jul = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());

    repo.upsert_employee_payroll_config(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        employment_id,
        Decimal::from(22),
        Decimal::from(8),
        jan,
    )
    .await
    .unwrap();
    repo.upsert_employee_payroll_config(
        &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
        employment_id,
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
            employment_id,
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
            employment_id,
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
        "UPDATE employments SET status = 'terminated' WHERE employee_id = $1",
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
    assert_eq!(active[0].full_name(), "Active One");
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
    assert_eq!(found.unwrap().full_name(), "Number Test");

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

    // Manually set start_date, end_date and status
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "UPDATE employments SET status = 'terminated', start_date = $1, end_date = $2 WHERE employee_id = $3",
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
    let source = apiserver::models::RecruitmentSource::new(
        source_id,
        ws_id,
        apiserver::models::enums::RecruitmentSourceType::Agency,
        "Top Recruiting".to_string(),
        None,
        None,
        false,
        None,
        None,
        None,
        false,
        None,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    repo.create_recruitment_source(&source).await.unwrap();

    // 1. Verify get_recruitment_sources
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);
    let list = repo
        .get_recruitment_sources(&auth_token, 1, 50)
        .await
        .unwrap();
    assert_eq!(list.0.len(), 1);
    assert_eq!(list.0[0].id(), source_id);

    // 2. Verify get_recruitment_source_by_id
    let fetched = repo
        .get_recruitment_source_by_id(source_id, &auth_token)
        .await
        .unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().name(), "Top Recruiting");

    // 3. Verify update_recruitment_source
    let update_req = apiserver::models::UpdateRecruitmentSourceRequest {
        name: Some("Premium Agency".to_string()),
        referrer_employee_id: None,
        referral_bonus_amount: Some(Decimal::from(100)),
        referral_bonus_paid: Some(true),
        agency_contact_name: Some("Alice".to_string()),
        agency_contact_email: Some("alice@agency.com".to_string()),
        agency_fee_amount: Some(Decimal::from(500)),
        agency_fee_paid: Some(true),
        notes: Some("High tier agency".to_string()),
    };
    let updated = repo
        .update_recruitment_source(source_id, &auth_token, &update_req)
        .await
        .unwrap();
    assert!(updated.is_some());
    let u = updated.unwrap();
    assert_eq!(u.name(), "Premium Agency");
    assert_eq!(u.referral_bonus_amount(), Some(Decimal::from(100)));
    assert_eq!(u.referral_bonus_paid(), true);
    assert_eq!(u.agency_contact_name(), Some("Alice"));
    assert_eq!(u.agency_contact_email(), Some("alice@agency.com"));
    assert_eq!(u.agency_fee_amount(), Some(Decimal::from(500)));
    assert_eq!(u.agency_fee_paid(), true);
    assert_eq!(u.notes(), Some("High tier agency"));

    // Create an employee and set recruitment source
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Sourced Emp")
        .create()
        .await;

    repo.set_employee_recruitment_source(&auth_token, emp_id, Some(source_id))
        .await
        .expect("Failed to link employee to source");

    let breakdown = repo
        .get_recruitment_source_breakdown(&auth_token)
        .await
        .unwrap();

    assert_eq!(breakdown.len(), 1);
    assert_eq!(breakdown[0].source_type, "agency");
    assert_eq!(breakdown[0].source_name, "Premium Agency");
    assert_eq!(breakdown[0].count, 1);

    // 4. Verify delete_recruitment_source
    let affected = repo
        .delete_recruitment_source(source_id, &auth_token)
        .await
        .unwrap();
    assert_eq!(affected, 1);

    let fetched_after = repo
        .get_recruitment_source_by_id(source_id, &auth_token)
        .await
        .unwrap();
    assert!(fetched_after.is_none());
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

    let mut tenant_conn = test_db
        .diesel_pools
        .tenants_tx
        .get_connection(&ws_id)
        .await
        .unwrap();
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

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Update employee to use custom location
    use apiserver::schema::employments;
    diesel::update(employments::table.filter(employments::employee_id.eq(emp_id)))
        .set(employments::primary_location_id.eq(custom_loc_id))
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

    diesel::update(employments::table.filter(employments::employee_id.eq(emp_id)))
        .set(employments::primary_location_id.eq(default_tz_loc_id))
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

#[tokio::test]
async fn test_get_employee_timezone_at_date() {
    let (test_db, factory) = setup_db().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let mut tenant_conn = test_db
        .diesel_pools
        .tenants_tx
        .get_connection(&ws_id)
        .await
        .unwrap();

    use apiserver::schema::sql_types::AssignmentStatus;
    use apiserver::schema::{employee_assignments, employees, work_locations};
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

    let tokyo_loc_id = Uuid::new_v4();
    diesel::insert_into(work_locations::table)
        .values(&NewLoc {
            id: tokyo_loc_id,
            workspace_id: ws_id,
            name: "Tokyo Branch".to_string(),
            timezone: Some("Asia/Tokyo".to_string()),
            is_headquarters: false,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        ws_id
    );
    let hq_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT primary_location_id AS id FROM employments WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Default primary location is HQ (UTC)
    let default_tz = repo
        .get_employee_timezone_at_date(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(default_tz.name(), "Asia/Ho_Chi_Minh");

    // Add active assignment to Tokyo for June 2026
    #[derive(Insertable)]
    #[diesel(table_name = employee_assignments)]
    struct NewAssig {
        id: Uuid,
        workspace_id: Uuid,
        employment_id: Uuid,
        home_location_id: Uuid,
        host_location_id: Uuid,
        start_date: NaiveDate,
        end_date: Option<NaiveDate>,
        status: apiserver::models::employee_assignment::AssignmentStatusEnum,
    }

    diesel::insert_into(employee_assignments::table)
        .values(&NewAssig {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            employment_id,
            home_location_id: hq_id,
            host_location_id: tokyo_loc_id,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            end_date: Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
            status: apiserver::models::employee_assignment::AssignmentStatusEnum::Active,
        })
        .execute(&mut *tenant_conn)
        .await
        .unwrap();

    // Test during assignment -> Asia/Tokyo
    let tz_during = repo
        .get_employee_timezone_at_date(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tz_during.name(), "Asia/Tokyo");

    // Test before assignment -> UTC
    let tz_before = repo
        .get_employee_timezone_at_date(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tz_before.name(), "Asia/Ho_Chi_Minh");

    // Test after assignment -> UTC
    let tz_after = repo
        .get_employee_timezone_at_date(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
            NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(tz_after.name(), "Asia/Ho_Chi_Minh");
}

#[tokio::test]
async fn test_employee_core_crud() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    let (hq_loc_id, employer_id) = {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::employers::dsl::employers;
        use apiserver::schema::work_locations::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        let loc = work_locations
            .filter(workspace_id.eq(ws_id))
            .filter(is_headquarters.eq(true))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap();
        let emp_id = employers
            .filter(apiserver::schema::employers::dsl::workspace_id.eq(ws_id))
            .select(apiserver::schema::employers::dsl::id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap();
        (loc, emp_id)
    };

    // Create
    let new_emp = apiserver::models::Employee::new(
        Uuid::new_v4(),
        ws_id,
        None,
        "Core Employee".to_string(),
        "CORE1".to_string(),
        vec!["Test".to_string()],
        Utc::now(),
        Utc::now(),
        None,
    );
    repo.create_employee(&new_emp).await.unwrap();

    // Read by ID
    let found = repo
        .get_employee_by_id(new_emp.id(), &auth_token)
        .await
        .unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.full_name(), "Core Employee");

    // Read by IDs
    let multi = repo
        .get_employees_by_ids(&[new_emp.id()], &auth_token)
        .await
        .unwrap();
    assert_eq!(multi.len(), 1);

    // Update
    let update_req = apiserver::models::UpdateEmployeeRequest {
        full_name: Some("Core Updated".to_string()),
        employee_id_number: Some("CORE1-U".to_string().into()),
        tags: Some(vec!["Updated".to_string()]),
        user_id: None,
        recruitment_source_id: None,
        max_concurrent_workers: None,
        force_transfer: None,
    };
    let updated = repo
        .update_employee(new_emp.id(), &auth_token, &update_req)
        .await
        .unwrap();
    assert!(updated.is_some());
    let updated = updated.unwrap();
    assert_eq!(updated.full_name(), "Core Updated");
    assert_eq!(updated.is_active(), false);

    // Deactivate the active employment in DB to allow rehire
    crate::test_execute!(
        &pool.api,
        "UPDATE employments SET status = 'terminated', end_date = '2026-03-31' WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active'",
        diesel::sql_types::Uuid, new_emp.id(),
        diesel::sql_types::Uuid, ws_id
    );

    // Rehire
    let rehired = repo
        .rehire_employee(
            &auth_token,
            new_emp.id(),
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            employer_id,
            Some(hq_loc_id),
            None,
        )
        .await
        .unwrap();
    assert!(rehired.is_some());
    let rehired = rehired.unwrap();
    assert_eq!(rehired.is_active(), true);

    let db_employment = {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::employments::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        employments
            .filter(employee_id.eq(new_emp.id()))
            .filter(status.eq("active"))
            .first::<apiserver::models::diesel::employment::EmploymentEntity>(&mut *conn)
            .await
            .unwrap()
    };
    assert_eq!(
        db_employment.start_date,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()
    );
    assert!(db_employment.end_date.is_none());

    // Delete
    let deleted = repo
        .delete_employee(new_emp.id(), &auth_token)
        .await
        .unwrap();
    assert_eq!(deleted, 1);

    let not_found = repo
        .get_employee_by_id(new_emp.id(), &auth_token)
        .await
        .unwrap();
    assert!(not_found.is_none());
}

#[tokio::test]
async fn test_get_all_employees_with_filters() {
    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create 3 employees
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Alice Alpha")
        .with_employee_id_number("EMP01")
        .create()
        .await;
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Bob Bravo")
        .with_employee_id_number("EMP02")
        .create()
        .await;
    factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Charlie Alpha")
        .with_employee_id_number("EMP03")
        .create()
        .await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    // No filters
    let opts_all = apiserver::models::EmployeeFilterOptions {
        limit: 10,
        page: 1,
        search_query: None,
        sort_by: Some("full_name".to_string()),
        sort_dir: Some("asc".to_string()),
        department_ids: None,
        permission: None,
        filter_ids: None,
    };
    let (all, count) = repo.get_all_employees(&auth_token, opts_all).await.unwrap();
    assert_eq!(count, 3);
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].full_name(), "Alice Alpha"); // sorted ascending

    // Filter by search
    let opts_search = apiserver::models::EmployeeFilterOptions {
        limit: 10,
        page: 1,
        search_query: Some("Alpha".to_string()),
        sort_by: None,
        sort_dir: None,
        department_ids: None,
        permission: None,
        filter_ids: None,
    };
    let (search_res, count) = repo
        .get_all_employees(&auth_token, opts_search)
        .await
        .unwrap();
    assert_eq!(count, 2);
    assert_eq!(search_res.len(), 2);
}

#[tokio::test]
async fn test_get_all_employees_filtered_by_permission() {
    use apiserver::repositories::diesel_workspace_repository::DieselWorkspaceRepository;
    use apiserver::repositories::workspace_repository::WorkspaceRepository;

    let (test_db, factory) = setup_db().await;

    let user_1 = factory.user().create().await;
    let user_2 = factory.user().create().await;
    let user_3 = factory.user().create().await;

    let ws_id = factory.workspace().create_with_owner(user_1).await;
    let workspace_repo = DieselWorkspaceRepository::new(test_db.diesel_pools.clone());
    let custom_role = workspace_repo
        .create_workspace_role(
            ws_id,
            "Custom Scoped Role".to_string(),
            None,
            "#000000".to_string(),
            "scoped".to_string(),
        )
        .await
        .unwrap();

    factory.add_member(ws_id, user_2, custom_role.id()).await;
    factory.add_member(ws_id, user_3, custom_role.id()).await;

    // Assign extra permission to user_2
    workspace_repo
        .assign_extra_permission(
            ws_id,
            user_2,
            "overtime:claim_on_behalf".to_string(),
            Utc::now() + chrono::Duration::days(1),
            user_1,
        )
        .await
        .unwrap();

    // Create employees
    let _emp_1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_1)
        .with_full_name("Owner Emp")
        .create()
        .await;

    let _emp_2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_2)
        .with_full_name("Allowed Emp")
        .create()
        .await;

    let _emp_3 = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_3)
        .with_full_name("Forbidden Emp")
        .create()
        .await;

    let repo = DieselEmployeeRepository::new(test_db.diesel_pools.clone());
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    // Query with permission filter
    let opts = apiserver::models::EmployeeFilterOptions {
        limit: 10,
        page: 1,
        search_query: None,
        sort_by: Some("full_name".to_string()),
        sort_dir: Some("asc".to_string()),
        department_ids: None,
        permission: Some("overtime:claim_on_behalf".to_string()),
        filter_ids: None,
    };

    let (employees, count) = repo.get_all_employees(&auth_token, opts).await.unwrap();

    assert_eq!(count, 2);
    let names: Vec<String> = employees
        .into_iter()
        .map(|e| e.full_name().to_string())
        .collect();
    assert!(names.contains(&"Owner Emp".to_string()));
    assert!(names.contains(&"Allowed Emp".to_string()));
    assert!(!names.contains(&"Forbidden Emp".to_string()));
}

#[tokio::test]
async fn test_temporal_segment_splitting_and_propagation() {
    use apiserver::repositories::department_repository::DepartmentRepository;
    use apiserver::repositories::DieselDepartmentRepository;

    let (test_db, factory) = setup_db().await;
    let pool = test_db.diesel_pools.clone();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    let dept_a = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Dept A")
        .create()
        .await;
    let dept_b = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Dept B")
        .create()
        .await;
    let dept_c = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Dept C")
        .create()
        .await;

    let hq_loc_id = {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::work_locations::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        work_locations
            .filter(workspace_id.eq(ws_id))
            .filter(is_headquarters.eq(true))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap()
    };

    let test_employer_id = {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::employers::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        employers
            .filter(workspace_id.eq(ws_id))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap()
    };

    // Create an employee. By default, factory.employee() creates a starting active segment.
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_full_name("Temporal Test Subject")
        .create()
        .await;

    // Set the initial employment segment explicitly in DB
    {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::employments::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        diesel::update(employments.filter(employee_id.eq(emp_id)))
            .set((
                start_date.eq(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
                end_date.eq(None::<NaiveDate>),
                department_id.eq(Some(dept_a)),
            ))
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let dep_repo = DieselDepartmentRepository::new(pool.clone());

    // ----------------------------------------------------
    // Scenario 1: Standard retroactive transfer causing a split
    // Transfer to dept_b on 2026-02-01.
    // Expected segments:
    // 1. 2026-01-01 -> 2026-01-31 (dept_a)
    // 2. 2026-02-01 -> None (dept_b)
    // ----------------------------------------------------
    let _history_entry = dep_repo
        .transfer_employee(
            &auth_token,
            emp_id,
            dept_b,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            Some("Retro transfer to B".to_string()),
        )
        .await
        .unwrap();

    let get_segs = || {
        let pool = pool.clone();
        async move {
            let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
            use apiserver::schema::employments::dsl::*;
            use diesel::prelude::*;
            use diesel_async::RunQueryDsl;
            employments
                .filter(employee_id.eq(emp_id))
                .filter(deleted_at.is_null())
                .order(start_date.asc())
                .load::<apiserver::models::diesel::employment::EmploymentEntity>(&mut *conn)
                .await
                .unwrap()
        }
    };

    let segs = get_segs().await;
    assert_eq!(segs.len(), 2, "Should have split into 2 segments");

    assert_eq!(
        segs[0].start_date,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
    assert_eq!(
        segs[0].end_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap())
    );
    assert_eq!(segs[0].department_id, Some(dept_a));

    assert_eq!(
        segs[1].start_date,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
    );
    assert_eq!(segs[1].end_date, None);
    assert_eq!(segs[1].department_id, Some(dept_b));
    assert_eq!(segs[1].notes.as_deref(), Some("Retro transfer to B"));

    // ----------------------------------------------------
    // Scenario 2: Transfer on exact start date of Segment 2 (no split)
    // Transfer to dept_c on 2026-02-01.
    // Expected: Segment 2 department is updated to dept_c, still 2 segments.
    // ----------------------------------------------------
    let _ = dep_repo
        .transfer_employee(
            &auth_token,
            emp_id,
            dept_c,
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            Some("Corrected transfer to C".to_string()),
        )
        .await
        .unwrap();

    let segs = get_segs().await;
    assert_eq!(segs.len(), 2, "Should still have 2 segments (no new split)");

    assert_eq!(
        segs[0].start_date,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
    assert_eq!(
        segs[0].end_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap())
    );
    assert_eq!(segs[0].department_id, Some(dept_a));

    assert_eq!(
        segs[1].start_date,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
    );
    assert_eq!(segs[1].end_date, None);
    assert_eq!(segs[1].department_id, Some(dept_c));
    assert_eq!(segs[1].notes.as_deref(), Some("Corrected transfer to C"));

    // ----------------------------------------------------
    // Scenario 3: Complex Retroactive Split and Propagation
    // Let's set up 4 segments manually to test propagation & override:
    // Seg 1: 2026-01-01 -> 2026-01-31 (dept_a)
    // Seg 2: 2026-02-01 -> 2026-03-31 (dept_c)
    // Seg 3: 2026-04-01 -> 2026-05-31 (dept_c)  <-- inherited
    // Seg 4: 2026-06-01 -> None       (dept_a)  <-- override (different from dept_c)
    // ----------------------------------------------------
    {
        let mut conn = pool.tenants_tx.get_connection(&ws_id).await.unwrap();
        use apiserver::schema::employments::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        // Delete existing segments
        diesel::delete(employments.filter(employee_id.eq(emp_id)))
            .execute(&mut *conn)
            .await
            .unwrap();

        let make_seg = |start: NaiveDate, end: Option<NaiveDate>, dept: Uuid| {
            apiserver::models::diesel::employment::EmploymentEntity {
                id: Uuid::new_v4(),
                workspace_id: ws_id,
                employee_id: emp_id,
                employer_id: test_employer_id,
                primary_location_id: Some(hq_loc_id),
                department_id: Some(dept),
                status: "active".to_string(),
                start_date: start,
                end_date: end,
                created_at: Utc::now(),
                updated_at: Utc::now(),
                deleted_at: None,
                job_title: None,
                employment_type: "full_time".to_string(),
                manager_employment_id: None,
                notes: None,
            }
        };

        let s1 = make_seg(
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()),
            dept_a,
        );
        let s2 = make_seg(
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 3, 31).unwrap()),
            dept_c,
        );
        let s3 = make_seg(
            NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            Some(NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()),
            dept_c,
        );
        let s4 = make_seg(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(), None, dept_a);

        diesel::insert_into(employments)
            .values(&vec![s1, s2, s3, s4])
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    // Now, transfer to dept_b starting from 2026-03-01 (middle of Seg 2).
    // This should:
    // 1. Split Seg 2 into:
    //    - Seg 2a: 2026-02-01 -> 2026-02-28 (dept_c)
    //    - Seg 2b: 2026-03-01 -> 2026-03-31 (dept_b)
    // 2. Propagate dept_b to Seg 3 (because its department was dept_c, which is the old department of Seg 2).
    // 3. Stop propagation at Seg 4 (because its department is dept_a, which does not match dept_c).
    let _ = dep_repo
        .transfer_employee(
            &auth_token,
            emp_id,
            dept_b,
            NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            Some("Retro split & propagate".to_string()),
        )
        .await
        .unwrap();

    let segs = get_segs().await;
    assert_eq!(segs.len(), 5, "Timeline should now contain 5 segments");

    assert_eq!(
        segs[0].start_date,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );
    assert_eq!(
        segs[0].end_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap())
    );
    assert_eq!(segs[0].department_id, Some(dept_a));

    assert_eq!(
        segs[1].start_date,
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
    );
    assert_eq!(
        segs[1].end_date,
        Some(NaiveDate::from_ymd_opt(2026, 2, 28).unwrap())
    );
    assert_eq!(segs[1].department_id, Some(dept_c));

    assert_eq!(
        segs[2].start_date,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    );
    assert_eq!(
        segs[2].end_date,
        Some(NaiveDate::from_ymd_opt(2026, 3, 31).unwrap())
    );
    assert_eq!(segs[2].department_id, Some(dept_b));
    assert_eq!(segs[2].notes.as_deref(), Some("Retro split & propagate"));

    assert_eq!(
        segs[3].start_date,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()
    );
    assert_eq!(
        segs[3].end_date,
        Some(NaiveDate::from_ymd_opt(2026, 5, 31).unwrap())
    );
    assert_eq!(
        segs[3].department_id,
        Some(dept_b),
        "Propagation failed for Segment 3"
    );

    assert_eq!(
        segs[4].start_date,
        NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()
    );
    assert_eq!(segs[4].end_date, None);
    assert_eq!(
        segs[4].department_id,
        Some(dept_a),
        "Override was not preserved for Segment 4"
    );
}
