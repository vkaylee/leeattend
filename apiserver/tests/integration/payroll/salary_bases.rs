//! Integration tests for salary bases endpoints.
//!
//! Validates that:
//! - GET /workspaces/:ws_id/salary-bases returns groups and members
//! - POST /workspaces/:ws_id/salary-bases/:base_id/members assigns a component to a base
//! - DELETE /workspaces/:ws_id/salary-bases/:base_id/members/:component_id unassigns it

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn body_json(response: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn make_req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    let req = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .header("Content-Type", "application/json");

    if let Some(b) = body {
        req.body(axum::body::Body::from(serde_json::to_string(&b).unwrap()))
            .unwrap()
    } else {
        req.body(axum::body::Body::empty()).unwrap()
    }
}

async fn create_salary_base_group(
    pool: &apiserver::diesel_pool::DieselPools,
    ws_id: Uuid,
    name: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO salary_base_groups (id, workspace_id, name, slug, is_system) \
         VALUES ($1, $2, $3, $4, false)",
        diesel::sql_types::Uuid,
        id,
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Text,
        name,
        diesel::sql_types::Text,
        name.to_lowercase().replace(' ', "_")
    );
    id
}

async fn create_payroll_component(
    pool: &apiserver::diesel_pool::DieselPools,
    ws_id: Uuid,
    name: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    // Insert a dummy payroll component definition
    crate::test_execute!(
        &pool.api,
        "INSERT INTO salary_component_definitions \
         (id, workspace_id, name, code, component_type) \
         VALUES ($1, $2, $3, $4, 'basic_salary')",
        diesel::sql_types::Uuid,
        id,
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Text,
        name,
        diesel::sql_types::Text,
        name.to_lowercase().replace(' ', "_")
    );
    id
}

#[tokio::test]
async fn test_salary_bases_crud() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create a base group
    let base_id = create_salary_base_group(&factory.diesel_pools, ws_id, "Standard Base").await;

    // Create a component
    let comp_id = create_payroll_component(&factory.diesel_pools, ws_id, "Basic Salary").await;

    // 1. Get salary bases (empty members)
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let mut body = body_json(res).await;
    let groups = body["groups"].as_array().unwrap();
    assert!(groups.iter().any(|g| g["id"] == base_id.to_string()));

    // 2. Assign component to base
    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members"),
            ws_id,
            user_id,
            Some(json!({ "component_definition_id": comp_id })),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Get salary bases (should contain the member)
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    body = body_json(res).await;
    let members = body["members"].as_array().unwrap();
    assert!(members
        .iter()
        .any(|m| m["component_definition_id"] == comp_id.to_string()
            && m["base_group_id"] == base_id.to_string()));

    // 4. Unassign component
    let res = app
        .clone()
        .oneshot(make_req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases/{base_id}/members/{comp_id}"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 5. Get salary bases (should be empty again)
    let res = app
        .clone()
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();
    body = body_json(res).await;
    let members = body["members"].as_array().unwrap();
    assert!(!members
        .iter()
        .any(|m| m["component_definition_id"] == comp_id.to_string()
            && m["base_group_id"] == base_id.to_string()));
}

#[tokio::test]
async fn test_create_custom_salary_base() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let payload = json!({
        "name": "Custom Base",
        "slug": "custom_base",
        "description": "My custom base"
    });

    let res = app
        .clone()
        .oneshot(make_req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            Some(payload),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body = body_json(res).await;
    assert_eq!(body["name"], "Custom Base");
    assert_eq!(body["slug"], "custom_base");
    assert_eq!(body["is_system"], false);

    // Verify it appears in GET
    let res = app
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    let groups = body["groups"].as_array().unwrap();
    assert!(groups
        .iter()
        .any(|g| g["name"] == "Custom Base" && g["slug"] == "custom_base"));
}

#[tokio::test]
async fn test_system_salary_bases_seeded() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Fetch salary bases
    let res = app
        .oneshot(make_req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/salary-bases"),
            ws_id,
            user_id,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = body_json(res).await;
    let groups = body["groups"].as_array().unwrap();

    // Verify the 4 seeded system bases are present
    let system_slugs = vec!["gross", "net", "bhxh", "taxable"];
    for slug in system_slugs {
        let found = groups
            .iter()
            .find(|g| g["slug"] == slug && g["is_system"] == true);
        assert!(
            found.is_some(),
            "System base with slug '{}' should be seeded and returned",
            slug
        );
    }
}

#[tokio::test]
async fn test_overtime_calculation_with_salary_base_group() {
    use apiserver::services::payroll_service::PayrollService;
    use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    let test_db = TestDatabase::new().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;

    // Set work_unit_minutes to 60 for consistent hourly-like testing
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

    // Create the "ot_base" base group in the DB
    let ot_base_group_id =
        create_salary_base_group(&factory.diesel_pools, workspace_id, "ot_base").await;

    // Create Component A (Base Salary)
    let comp_a_id =
        create_payroll_component(&factory.diesel_pools, workspace_id, "Base Salary").await;

    // Create Component B (Allowance)
    let comp_b_id =
        create_payroll_component(&factory.diesel_pools, workspace_id, "Allowance").await;

    // Assign Component A to OT Base group
    crate::test_execute!(
        &pool.api,
        "INSERT INTO salary_component_base_members (base_group_id, component_definition_id, created_at) \
         VALUES ($1, $2, NOW())",
        diesel::sql_types::Uuid,
        ot_base_group_id,
        diesel::sql_types::Uuid,
        comp_a_id
    );

    // Assign salary component values to the employee:
    // Component A = 2600
    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, comp_a_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Component B = 2600
    crate::test_execute!(&pool.api,
        "INSERT INTO employee_salary_components (employment_id, component_definition_id, amount, effective_date) VALUES ($1, $2, $3, $4)",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, comp_b_id,
        diesel::sql_types::Numeric, Decimal::from(2600),
        diesel::sql_types::Date, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Create overtime policy definition
    let ot_policy_id = Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO overtime_policy_definitions (id, workspace_id, name, description) \
         VALUES ($1, $2, 'OT Policy', 'Test')",
        diesel::sql_types::Uuid,
        ot_policy_id,
        diesel::sql_types::Uuid,
        workspace_id
    );

    // Create overtime rule with multiplier 1.5x, all days of week
    let rule_id = Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO overtime_rules (id, policy_id, priority, multiplier, days_of_week, is_holiday) \
         VALUES ($1, $2, 1, $3, 127, false)",
        diesel::sql_types::Uuid,
        rule_id,
        diesel::sql_types::Uuid,
        ot_policy_id,
        diesel::sql_types::Numeric,
        Decimal::from_str("1.5").unwrap()
    );

    // Assign overtime policy to workspace
    let assignment_id = Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO overtime_policy_assignments (id, workspace_id, assignee_type, policy_id, effective_from) \
         VALUES ($1, $2, 'workspace', $3, $4)",
        diesel::sql_types::Uuid,
        assignment_id,
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Uuid,
        ot_policy_id,
        diesel::sql_types::Date,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
    );

    // Create a shift: 09:00 - 17:00 (8 hours shift, 480 minutes)
    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Day Shift")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .with_break_duration(0)
        .with_allow_early_ot(false)
        .create()
        .await;

    // Assign shift for Jan 5
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employee_shifts (employment_id, template_id, start_date) VALUES ($1, $2, $3)",
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Uuid,
        shift_id,
        diesel::sql_types::Date,
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap()
    );

    // Create time entry: Jan 5 09:00 -> Jan 5 19:00 (10 hours, which is 2 hours of overtime)
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(
            tz.with_ymd_and_hms(2026, 1, 5, 9, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .with_punch_out(
            tz.with_ymd_and_hms(2026, 1, 5, 19, 0, 0)
                .unwrap()
                .with_timezone(&Utc),
        )
        .create()
        .await;

    // Calculate payroll
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
        let er = Arc::new(apiserver::repositories::DieselEmployeeRepository::new(
            diesel_pools.clone(),
        ));
        let atr = Arc::new(
            apiserver::repositories::DieselAttendancePolicyRepository::new(diesel_pools.clone()),
        );

        PayrollService::new(apiserver::services::payroll_service::PayrollServiceArgs {
            payroll_repo: pr,
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
    .calculate_monthly_payslip(employee_id, &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id), 2026, 1)
    .await
    .unwrap();

    // Verify payroll calculations:
    // Total components: Component A + Component B = 2600 + 2600 = 5200.
    // Total standard days: 26. Standard hours/day: 8.
    // Base hourly rate: 5200 / (26 * 8) = 25.
    // Worked 1 day = 8 standard hours -> Base pay amount = 8 * 25 = 200.
    assert_eq!(payslip.base_pay_amount().round_dp(2), Decimal::from(200));

    // Overtime base components: Component A only (2600).
    // OT base = 2600.
    // OT hourly rate: 2600 / (26 * 8) = 12.5.
    // Worked 2 hours of overtime with multiplier 1.5x:
    // OT pay = 2 * 1.5 * 12.5 = 37.5.
    // (If Component B were also included, OT hourly rate would be 25, and OT pay would be 2 * 1.5 * 25 = 75).
    assert_eq!(
        payslip.overtime_pay_amount().round_dp(2),
        Decimal::from_str("37.5").unwrap()
    );
}
