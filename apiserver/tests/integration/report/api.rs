//! Report API integration tests

use crate::common::app::{spawn_app, spawn_app_with_member};
use chrono::{Datelike, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;

#[tokio::test]
async fn test_get_daily_attendance_report_with_break_times() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    // 1. Setup
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // 2. Create Employee
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Break Time Test")
        .with_employee_id_number("BT001")
        .create()
        .await;

    // 3. Create Shift (9:00 - 18:00) with 60 min break
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .with_break_duration(60)
        .assign_to_employee(emp, Utc::now().date_naive())
        .await;

    // 4. Record Attendance
    // Punch In: 9:00
    let now = Utc::now().date_naive();
    let punch_in = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 0, 0)
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 18, 0, 0)
        .unwrap();

    // Break: 12:00 - 13:00
    let break_start = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 12, 0, 0)
        .unwrap();
    let break_end = Utc
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 13, 0, 0)
        .unwrap();

    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(break_start)
        .with_punch_out(break_end)
        .with_entry_type("break")
        .create()
        .await;

    // 5. Fetch Request
    let response = client
        .get(format!("{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={now}&end_date={now}"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_workspace_attendance_summary() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let response = client
        .get(format!(
            "{}/api/v1/workspaces/{}/reports/attendance?type=summary&start_date={}&end_date={}",
            address,
            workspace_id,
            Utc::now().format("%Y-%m-%d"),
            Utc::now().format("%Y-%m-%d")
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_reports_group_adjacent_temporal_employments_by_employer() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);
    let report_end = Utc::now().date_naive();
    let transfer_date = report_end - chrono::Duration::days(1);
    let report_start = transfer_date - chrono::Duration::days(1);

    let employee_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Temporal Report Employee")
        .with_employee_id_number("TEMP-REPORT")
        .create()
        .await;

    let old_employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE workspace_id = $1 AND employee_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Uuid,
        employee_id
    );
    let employer_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT employer_id AS id FROM employments WHERE id = $1",
        diesel::sql_types::Uuid,
        old_employment_id
    );

    crate::test_execute!(
        &pool.api,
        "UPDATE employments SET end_date = $1 WHERE id = $2",
        diesel::sql_types::Nullable<diesel::sql_types::Date>,
        Some(transfer_date - chrono::Duration::days(1)),
        diesel::sql_types::Uuid,
        old_employment_id
    );

    let new_employment_id = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, status, start_date, employment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, 'active', $5, 'full_time', NOW(), NOW())",
        diesel::sql_types::Uuid,
        new_employment_id,
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Date,
        transfer_date
    );

    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Temporal Report Shift")
        .create()
        .await;
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employee_shifts (employment_id, shift_id, start_date, end_date) VALUES ($1, $2, $3, $4), ($5, $2, $6, NULL)",
        diesel::sql_types::Uuid,
        old_employment_id,
        diesel::sql_types::Uuid,
        shift_id,
        diesel::sql_types::Date,
        report_start,
        diesel::sql_types::Nullable<diesel::sql_types::Date>,
        Some(transfer_date - chrono::Duration::days(1)),
        diesel::sql_types::Uuid,
        new_employment_id,
        diesel::sql_types::Date,
        transfer_date
    );

    let summary_response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=summary&start_date={report_start}&end_date={report_end}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(summary_response.status(), StatusCode::OK);
    let summary_body: serde_json::Value = summary_response.json().await.unwrap();
    let summary_rows = summary_body["data"].as_array().unwrap();
    assert_eq!(
        summary_rows.len(),
        1,
        "temporal history for one employer must be grouped"
    );
    assert_eq!(summary_rows[0]["employee_id"], employee_id.to_string());
    assert_eq!(summary_rows[0]["employer_id"], employer_id.to_string());

    let daily_response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={report_start}&end_date={report_end}&employee_ids={employee_id}&employer_id={employer_id}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(daily_response.status(), StatusCode::OK);
    let daily_body: serde_json::Value = daily_response.json().await.unwrap();
    let daily_rows = daily_body["data"].as_array().unwrap();
    assert_eq!(
        daily_rows.len(),
        3,
        "each report date must be emitted exactly once"
    );

    for row in daily_rows {
        let date =
            chrono::NaiveDate::parse_from_str(row["date"].as_str().unwrap(), "%Y-%m-%d").unwrap();
        let row_employment_id = row["employment_id"].as_str().unwrap();
        if date < transfer_date {
            assert_eq!(row_employment_id, old_employment_id.to_string());
        } else {
            assert_eq!(row_employment_id, new_employment_id.to_string());
        }
    }
}

#[tokio::test]
async fn test_get_workspace_attendance_summary_computed_sorting() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let today = Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    // 1. Create Employee A (will work 4 hours)
    let emp_a = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee A")
        .with_employee_id_number("EMP_A")
        .create()
        .await;

    // 2. Create Employee B (will work 8 hours)
    let emp_b = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee B")
        .with_employee_id_number("EMP_B")
        .create()
        .await;

    // 3. Setup shifts for both
    let _shift_a = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Shift A")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .with_break_duration(60)
        .assign_to_employee(emp_a, today)
        .await;

    let _shift_b = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Shift B")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .with_break_duration(60)
        .assign_to_employee(emp_b, today)
        .await;

    // 4. Punch Employee A: 9:00 to 13:00 (4 hours)
    let punch_in_a = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 9, 0, 0)
        .unwrap();
    let punch_out_a = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 13, 0, 0)
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp_a)
        .with_punch_in(punch_in_a)
        .with_punch_out(punch_out_a)
        .create()
        .await;

    // 5. Punch Employee B: 9:00 to 17:00 (8 hours)
    let punch_in_b = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 9, 0, 0)
        .unwrap();
    let punch_out_b = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 17, 0, 0)
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp_b)
        .with_punch_in(punch_in_b)
        .with_punch_out(punch_out_b)
        .create()
        .await;

    // 6. Request page 1 with limit 1, sorted by WorkHoursDesc
    // Expect Employee B to be on page 1 (since 8 hours > 4 hours)
    let response_desc = client
        .get(format!(
            "{}/api/v1/workspaces/{}/reports/attendance?type=summary&start_date={}&end_date={}&sort=work_hours_desc&page=1&limit=1",
            address, workspace_id, today_str, today_str
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response_desc.status(), StatusCode::OK);
    let body_desc: serde_json::Value = response_desc.json().await.unwrap();
    let data_desc = body_desc["data"].as_array().unwrap();
    assert_eq!(data_desc.len(), 1);
    assert_eq!(
        data_desc[0]["employee_name"].as_str().unwrap(),
        "Employee B"
    );

    // 7. Request page 1 with limit 1, sorted by WorkHoursAsc
    // Expect Employee A to be on page 1 (since 4 hours < 8 hours)
    let response_asc = client
        .get(format!(
            "{}/api/v1/workspaces/{}/reports/attendance?type=summary&start_date={}&end_date={}&sort=work_hours_asc&page=1&limit=1",
            address, workspace_id, today_str, today_str
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response_asc.status(), StatusCode::OK);
    let body_asc: serde_json::Value = response_asc.json().await.unwrap();
    let data_asc = body_asc["data"].as_array().unwrap();
    assert_eq!(data_asc.len(), 1);
    assert_eq!(data_asc[0]["employee_name"].as_str().unwrap(), "Employee A");
}

#[tokio::test]
async fn test_get_daily_attendance_report_behalf_filtering() {
    let (address, workspace_id, _owner_id, clerk_user_id, factory, test_db) =
        spawn_app_with_member().await;
    let pool = test_db.diesel_pools.clone();
    let client = reqwest::Client::new();

    let employee_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();

    // 1. Grant 'overtime:claim_on_behalf' permission to employee role
    crate::test_execute!(
        &pool.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, 'overtime:claim_on_behalf') ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid, employee_role_id
    );

    // 2. Create the clerk employee (assignee) linked to clerk_user_id
    let clerk_emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_user(clerk_user_id)
        .with_full_name("Clerk Employee")
        .with_employee_id_number("EMP_CLERK")
        .create()
        .await;

    // 3. Create two target employees
    let emp_assigned_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Assigned Employee")
        .with_employee_id_number("EMP_ASSIGNED")
        .create()
        .await;

    let _emp_unassigned_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Unassigned Employee")
        .with_employee_id_number("EMP_UNASSIGNED")
        .create()
        .await;

    // 4. Create department and transfer emp_assigned to it
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let dept_repo = apiserver::repositories::DieselDepartmentRepository::new(diesel_pools.clone());
    use apiserver::repositories::department_repository::DepartmentRepository;

    let dept = dept_repo
        .create_department(
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
            "Assigned Department".to_string(),
            None,
            None,
        )
        .await
        .unwrap();

    dept_repo
        .transfer_employee(
            &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
            emp_assigned_id,
            dept.id(),
            chrono::Utc::now().date_naive(),
            None,
        )
        .await
        .unwrap();

    // 5. Generate auth token for clerk user
    let clerk_auth_token = factory.auth().create_token(clerk_user_id);
    let today = chrono::Utc::now().date_naive().to_string();

    // CASE A: Clerk has NO behalf assignments
    // Querying report should return empty list (total = 0)
    let response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {clerk_auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 0);
    assert_eq!(body["total"].as_i64().unwrap(), 0);

    // CASE B: Assign the department to the clerk
    let assignment_id = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO behalf_assignments (id, workspace_id, assignee_user_id, target_department_id, assignment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, 'overtime_claim', NOW(), NOW())",
        diesel::sql_types::Uuid, assignment_id,
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Uuid, clerk_user_id,
        diesel::sql_types::Uuid, dept.id()
    );

    // Query report again. This time it should return exactly 1 row (EMP_ASSIGNED)
    let response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {clerk_auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = response.json().await.unwrap();
    let data = body["data"].as_array().unwrap();

    let names: Vec<String> = data
        .iter()
        .map(|row| row["employee_name"].as_str().unwrap().to_string())
        .collect();

    assert!(names.contains(&"Assigned Employee".to_string()));
    assert!(!names.contains(&"Unassigned Employee".to_string()));
    assert!(!names.contains(&"Clerk Employee".to_string()));
}

#[tokio::test]
async fn test_get_attendance_report_behalf_extended() {
    let (address, workspace_id, _owner_id, clerk_user_id, factory, test_db) =
        spawn_app_with_member().await;
    let pool = test_db.diesel_pools.clone();
    let client = reqwest::Client::new();

    let employee_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO workspace_role_permissions (role_id, permission_name) VALUES ($1, 'overtime:claim_on_behalf') ON CONFLICT DO NOTHING",
        diesel::sql_types::Uuid, employee_role_id
    );

    let clerk_auth_token = factory.auth().create_token(clerk_user_id);
    let today = chrono::Utc::now().date_naive().to_string();

    // 1. Create target employee
    let emp_assigned_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Target Assigned Employee")
        .with_employee_id_number("EMP_TARGET")
        .create()
        .await;

    // 2. Create another unassigned employee
    let emp_unassigned_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Target Unassigned Employee")
        .with_employee_id_number("EMP_UNASSIGNED2")
        .create()
        .await;

    // 3. Test: Case E - Query summary type with no assignments
    let response_summary_empty = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=summary&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {clerk_auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response_summary_empty.status(), reqwest::StatusCode::OK);
    let body_summary_empty: serde_json::Value = response_summary_empty.json().await.unwrap();
    assert_eq!(body_summary_empty["data"].as_array().unwrap().len(), 0);

    // 4. Assign the specific employee (emp_assigned_id) to the clerk
    let assignment_id = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO behalf_assignments (id, workspace_id, assignee_user_id, target_employee_id, assignment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, 'overtime_claim', NOW(), NOW())",
        diesel::sql_types::Uuid, assignment_id,
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Uuid, clerk_user_id,
        diesel::sql_types::Uuid, emp_assigned_id
    );

    // 5. Test: Query report with `employee_ids` filter (both assigned & unassigned)
    // The intersection should only return the assigned one
    let response_intersect = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}&employee_ids={},{}",
            emp_assigned_id, emp_unassigned_id
        ))
        .header("Authorization", format!("Bearer {clerk_auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response_intersect.status(), reqwest::StatusCode::OK);
    let body_intersect: serde_json::Value = response_intersect.json().await.unwrap();
    let data_intersect = body_intersect["data"].as_array().unwrap();
    assert_eq!(data_intersect.len(), 1);
    assert_eq!(
        data_intersect[0]["employee_name"].as_str().unwrap(),
        "Target Assigned Employee"
    );

    // 6. Test: Query report with ONLY unassigned `employee_ids` filter (intersection is empty)
    // Should return empty response
    let response_empty_intersect = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}&employee_ids={}",
            emp_unassigned_id
        ))
        .header("Authorization", format!("Bearer {clerk_auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response_empty_intersect.status(), reqwest::StatusCode::OK);
    let body_empty_intersect: serde_json::Value = response_empty_intersect.json().await.unwrap();
    assert_eq!(body_empty_intersect["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_workspace_attendance_summary_sorting_and_filtering() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let today = Utc::now().date_naive();
    let today_str = today.format("%Y-%m-%d").to_string();

    // Create an employer
    let employer_id = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employers (id, workspace_id, name, tax_code, legal_address, created_at, updated_at) VALUES ($1, $2, 'Employer Summary Test', 'MST-2', 'Addr2', NOW(), NOW())",
        diesel::sql_types::Uuid,
        employer_id,
        diesel::sql_types::Uuid,
        workspace_id
    );

    // Create target employee
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Summary Filter Test")
        .with_employee_id_number("EMP_SUM_FIL")
        .create()
        .await;

    // Create employment linked to the employer
    let employment_id = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, status, start_date, employment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, 'active', $5, 'primary', NOW(), NOW())",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, workspace_id,
        diesel::sql_types::Uuid, emp,
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Date, today
    );

    // Create an employee whose employments are outside the requested day.
    let emp_non_overlap = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Non Overlap Test")
        .with_employee_id_number("EMP_NON_OVERLAP")
        .create()
        .await;

    let past_start = today - chrono::Duration::days(10);
    let past_end = today - chrono::Duration::days(5);
    crate::test_execute!(
        &pool.api,
        "UPDATE employments SET start_date = $1, end_date = $2 WHERE employee_id = $3",
        diesel::sql_types::Date,
        past_start,
        diesel::sql_types::Nullable<diesel::sql_types::Date>,
        Some(past_end),
        diesel::sql_types::Uuid,
        emp_non_overlap
    );

    // Insert a second past employment; neither record should create a row for today.
    let default_employer_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        workspace_id
    );
    let emp_non_overlap_employment_2 = uuid::Uuid::new_v4();
    crate::test_execute!(
        &pool.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, status, start_date, end_date, employment_type, created_at, updated_at) VALUES ($1, $2, $3, $4, 'active', $5, $6, 'full_time', NOW(), NOW())",
        diesel::sql_types::Uuid,
        emp_non_overlap_employment_2,
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Uuid,
        emp_non_overlap,
        diesel::sql_types::Uuid,
        default_employer_id,
        diesel::sql_types::Date,
        past_start + chrono::Duration::days(2),
        diesel::sql_types::Nullable<diesel::sql_types::Date>,
        Some(past_end)
    );

    // Create Shift
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Record time entry
    let punch_in = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 9, 0, 0)
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 18, 0, 0)
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    // 1. Query with employer_id filter
    let response_employer = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=summary&start_date={today_str}&end_date={today_str}&employer_id={employer_id}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response_employer.status(), reqwest::StatusCode::OK);
    let body_employer: serde_json::Value = response_employer.json().await.unwrap();
    assert_eq!(body_employer["data"].as_array().unwrap().len(), 1);

    // 2. Query with non-matching employer_id filter
    let response_non_matching_employer = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=summary&start_date={today_str}&end_date={today_str}&employer_id={}",
            uuid::Uuid::new_v4()
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response_non_matching_employer.status(), StatusCode::OK);
    let body_non_matching: serde_json::Value = response_non_matching_employer.json().await.unwrap();
    assert_eq!(body_non_matching["data"].as_array().unwrap().len(), 0);

    // 3. Query with non-matching employment_id filter
    let response_non_matching_employment = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=summary&start_date={today_str}&end_date={today_str}&employment_id={}",
            uuid::Uuid::new_v4()
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response_non_matching_employment.status(), StatusCode::OK);
    let body_non_matching_emp: serde_json::Value =
        response_non_matching_employment.json().await.unwrap();
    assert_eq!(body_non_matching_emp["data"].as_array().unwrap().len(), 0);

    // 4. Test all sorting parameters: leave_days_asc, leave_days_desc, ot_hours_asc, ot_hours_desc, total_hours_asc, total_hours_desc
    let sorts = [
        "leave_days_asc",
        "leave_days_desc",
        "ot_hours_asc",
        "ot_hours_desc",
        "total_hours_asc",
        "total_hours_desc",
    ];

    for sort in sorts {
        let res_sort = client
            .get(format!(
                "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=summary&start_date={today_str}&end_date={today_str}&sort={sort}"
            ))
            .header("Authorization", format!("Bearer {auth_token}"))
            .send()
            .await
            .unwrap();
        assert_eq!(res_sort.status(), StatusCode::OK);
        let body: serde_json::Value = res_sort.json().await.unwrap();
        let data = body["data"].as_array().unwrap();
        assert_eq!(data.len(), 2);
    }
}
