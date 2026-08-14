//! Integration tests for dashboard chart endpoints:
//! - GET /api/v1/dashboard/peak-times
//! - GET /api/v1/dashboard/leave-types
//! - GET /api/v1/dashboard/lateness-by-day
//! - GET /api/v1/dashboard/unexcused-absences

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::LeaveStatus;
use chrono::{Duration, NaiveTime, Utc};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════
//  1. Peak Times
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_peak_times_returns_empty_when_no_data() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/peak-times"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.is_empty(), "Expected empty array when no time entries");
}

#[tokio::test]
async fn test_peak_times_returns_slots_with_time_entries() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Compute punch times relative to workspace-local "today" so they
    // always fall within the handler's UTC window.
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let local_today = Utc::now().with_timezone(&tz).date_naive();
    let punch_in = local_today
        .and_hms_opt(8, 15, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap()
        .to_utc();
    let punch_out = local_today
        .and_hms_opt(17, 30, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap()
        .to_utc();

    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/peak-times"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "Expected non-empty peak times data");

    // Check structure
    for slot in &body {
        assert!(slot["time_slot"].is_string());
        assert!(slot["check_in"].is_number());
        assert!(slot["check_out"].is_number());
    }

    // Verify at least one slot has check-in data and at least one has check-out data
    let has_checkin = body.iter().any(|s| s["check_in"].as_i64().unwrap() > 0);
    let has_checkout = body.iter().any(|s| s["check_out"].as_i64().unwrap() > 0);
    assert!(has_checkin, "Expected at least one slot with check-in data");
    assert!(
        has_checkout,
        "Expected at least one slot with check-out data"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  2. Leave Types Breakdown
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_leave_types_returns_empty_when_no_leave() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/leave-types"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(
        body.is_empty(),
        "Expected empty array when no approved leave"
    );
}

#[tokio::test]
async fn test_leave_types_returns_breakdown_with_data() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    let vacation_type_id: Uuid = crate::test_query_scalar_uuid!(
        &_db.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
        diesel::sql_types::Uuid,
        ws_id
    );

    let sick_type_id: Uuid = crate::test_query_scalar_uuid!(
        &_db.diesel_pools.api,
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'sick'",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Create approved leave requests this month (non-overlapping dates to satisfy
    // the leave_requests_no_overlap_active exclusion constraint).
    factory
        .leave_request()
        .with_employee(emp_id)
        .with_leave_type(vacation_type_id)
        .with_dates(today - Duration::days(1), today - Duration::days(1))
        .with_status(LeaveStatus::Approved)
        .create()
        .await;

    factory
        .leave_request()
        .with_employee(emp_id)
        .with_leave_type(sick_type_id)
        .with_dates(today, today)
        .with_status(LeaveStatus::Approved)
        .create()
        .await;

    // Pending leave (should NOT appear)
    factory
        .leave_request()
        .with_employee(emp_id)
        .with_leave_type(vacation_type_id)
        .with_dates(today, today)
        .with_status(LeaveStatus::Pending)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/leave-types"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "Expected non-empty leave types data");

    for item in &body {
        assert!(item["leave_type"].is_string());
        assert!(item["count"].is_number());
        assert!(item["count"].as_i64().unwrap() > 0);
    }

    let type_names: Vec<&str> = body
        .iter()
        .map(|item| item["leave_type"].as_str().unwrap())
        .collect();
    assert!(
        type_names.contains(&"Vacation"),
        "Expected Vacation in breakdown, got: {type_names:?}"
    );
    assert!(
        type_names.contains(&"Sick Leave"),
        "Expected Sick Leave in breakdown, got: {type_names:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  3. Lateness by Day of Week
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_lateness_by_day_returns_empty_when_no_late_entries() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/lateness-by-day"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.is_empty(), "Expected empty when no roster/late data");
}

#[tokio::test]
async fn test_lateness_by_day_returns_data_with_late_entries() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Create a shift 08:00-17:00 and assign to employee (inserts into employee_shifts)
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, today - Duration::days(30))
        .await;

    // Create a LATE time entry (punch in at 08:30 ICT, after shift start 08:00 ICT)
    // Workspace is Asia/Ho_Chi_Minh (UTC+7), so 08:30 ICT = 01:30 UTC, 17:00 ICT = 10:00 UTC
    let punch_in = today.and_hms_opt(1, 30, 0).unwrap().and_utc();
    let punch_out = today.and_hms_opt(10, 0, 0).unwrap().and_utc();

    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/lateness-by-day"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "Expected lateness data for today's DOW");

    for item in &body {
        assert!(item["day"].is_string());
        assert!(item["late_count"].is_number());
        assert!(item["avg_late_minutes"].is_number());

        let day = item["day"].as_str().unwrap();
        assert!(
            ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"].contains(&day),
            "Day '{day}' is not a valid day name"
        );
    }

    let total_late: i64 = body
        .iter()
        .map(|item| item["late_count"].as_i64().unwrap())
        .sum();
    assert!(
        total_late >= 1,
        "Expected at least 1 late count, got {total_late}"
    );

    let first = &body[0];
    let avg = first["avg_late_minutes"].as_f64().unwrap();
    assert!(
        avg > 20.0 && avg < 40.0,
        "Expected avg_late_minutes near 30, got {avg}"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  4. Unexcused Absences
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_unexcused_absences_returns_empty_when_no_employees() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/unexcused-absences"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(
        body.is_empty(),
        "Expected empty array when no employees with departments"
    );
}

#[tokio::test]
async fn test_unexcused_absences_returns_data_with_absent_employees() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let dept_id = factory.department().with_workspace(ws_id).create().await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Assign employee to department
    crate::test_execute!(
        &_db.diesel_pools.api,
        "UPDATE employments SET department_id = $2, start_date = $3 WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        dept_id,
        diesel::sql_types::Date,
        today - Duration::days(30)
    );

    // Assign a shift so they are expected to work today
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, today - Duration::days(30))
        .await;

    // No time entries and no leave for this employee → should be unexcused

    let res = client
        .get(format!("{address}/api/v1/dashboard/unexcused-absences"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(
        !body.is_empty(),
        "Expected non-empty unexcused absences data"
    );

    // Check response structure
    for item in &body {
        assert!(item["department_id"].is_string());
        assert!(item["department_name"].is_string());
        assert!(item["unexcused_count"].is_number());
        assert!(item["consecutive_count"].is_number());
        assert!(item["total_absent_count"].is_number());
        assert!(item["total_leave_count"].is_number());
    }

    // The employee should show up as unexcused
    let total_unexcused: i64 = body
        .iter()
        .map(|item| item["unexcused_count"].as_i64().unwrap())
        .sum();
    assert!(
        total_unexcused > 0,
        "Expected at least 1 unexcused absence, got {total_unexcused}"
    );
}

#[tokio::test]
async fn test_unexcused_absences_supports_date_range_query() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let today = Utc::now().date_naive();
    let start = (today - Duration::days(7)).format("%Y-%m-%d").to_string();
    let end = today.format("%Y-%m-%d").to_string();

    let res = client
        .get(format!(
            "{address}/api/v1/dashboard/unexcused-absences?start_date={start}&end_date={end}"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let _body: Vec<serde_json::Value> = res.json().await.unwrap();
    // Should return successfully even if empty (Vec is always an array)
    // Parsing into Vec<Value> already validates it's an array
}

#[tokio::test]
async fn test_unexcused_absences_group_by_employee() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let dept_id = factory.department().with_workspace(ws_id).create().await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Assign employee to department
    crate::test_execute!(
        &_db.diesel_pools.api,
        "UPDATE employments SET department_id = $2, start_date = $3 WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        dept_id,
        diesel::sql_types::Date,
        today - Duration::days(30)
    );

    // Assign a shift so they are expected to work today
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, today - Duration::days(30))
        .await;

    let res = client
        .get(format!(
            "{address}/api/v1/dashboard/unexcused-absences?group_by=employee"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "Expected employee-level data");

    // Check employee-specific fields are present
    for item in &body {
        assert!(
            item["employee_id"].is_string(),
            "Expected employee_id in group_by=employee response"
        );
        assert!(
            item["employee_name"].is_string(),
            "Expected employee_name in group_by=employee response"
        );
        assert!(item["department_id"].is_string());
        assert!(item["department_name"].is_string());
        assert!(item["unexcused_count"].is_number());
    }
}

#[tokio::test]
async fn test_unexcused_absences_includes_employees_without_department() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Create employee WITHOUT assigning to any department
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();
    // Assign a shift so they are expected to work today
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, today - Duration::days(30))
        .await;

    // group_by=department — should show "Unassigned" group
    let res = client
        .get(format!("{address}/api/v1/dashboard/unexcused-absences"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(
        !body.is_empty(),
        "Expected data for employee without department"
    );

    let has_unassigned = body
        .iter()
        .any(|item| item["department_name"].as_str().unwrap() == "Unassigned");
    assert!(
        has_unassigned,
        "Expected 'Unassigned' department for employee without department, got: {:?}",
        body.iter()
            .map(|i| i["department_name"].as_str().unwrap())
            .collect::<Vec<_>>()
    );

    // group_by=employee — should also include this employee
    let res2 = client
        .get(format!(
            "{address}/api/v1/dashboard/unexcused-absences?group_by=employee"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res2.status(), reqwest::StatusCode::OK);
    let body2: Vec<serde_json::Value> = res2.json().await.unwrap();
    assert!(
        !body2.is_empty(),
        "Expected employee-level data for unassigned employee"
    );

    let has_employee_fields = body2
        .iter()
        .all(|item| item["employee_id"].is_string() && item["employee_name"].is_string());
    assert!(
        has_employee_fields,
        "Expected employee fields in group_by=employee response"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  5. Attendance Trend
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_attendance_trend_returns_30_days_data() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/attendance-trend"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 30, "Expected 30 days of trend data");

    // Check structure: each point has date, present, late, absent, on_leave
    for point in &body {
        assert!(point["date"].is_string());
        assert!(point["present"].is_number());
        assert!(point["late"].is_number());
        assert!(point["absent"].is_number());
        assert!(point["on_leave"].is_number());
    }

    // With no employees, all counts should be zero
    for point in &body {
        assert_eq!(point["present"].as_i64().unwrap(), 0);
        assert_eq!(point["late"].as_i64().unwrap(), 0);
    }
}

#[tokio::test]
async fn test_attendance_trend_includes_present_and_late() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Create shift 08:00-17:00 and assign via shift_rosters (direct roster entry)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (employment_id, template_id, date, status, id) VALUES ($1, $2, $3, 'confirmed', $4)",
        diesel::sql_types::Uuid, emp_employment_id,
        diesel::sql_types::Uuid, shift_id,
        diesel::sql_types::Date, today,
        diesel::sql_types::Uuid, Uuid::new_v4()
    );

    // Create a LATE punch-in (08:30, after shift start 08:00)
    let punch_in = today.and_hms_opt(8, 30, 0).unwrap().and_utc();
    let punch_out = today.and_hms_opt(17, 0, 0).unwrap().and_utc();

    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/attendance-trend"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 30);

    // Find today's entry — should have present=1 and late=1
    let today_str = today.format("%Y-%m-%d").to_string();
    let today_point = body
        .iter()
        .find(|p| p["date"].as_str().unwrap() == today_str);
    assert!(today_point.is_some(), "Expected today in trend data");
    let today_point = today_point.unwrap();
    assert_eq!(
        today_point["present"].as_i64().unwrap(),
        1,
        "Expected 1 present"
    );
    assert_eq!(
        today_point["late"].as_i64().unwrap(),
        1,
        "Expected 1 late (uses shift_rosters join)"
    );
}

#[tokio::test]
async fn test_attendance_trend_late_via_employee_shifts() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Create shift 08:00-17:00 and assign via employee_shifts (NOT shift_rosters)
    // This uses the factory's assign_to_employee which inserts into employee_shifts
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, today - Duration::days(30))
        .await;

    // Create a LATE punch-in (08:30, after shift start 08:00)
    let punch_in = today.and_hms_opt(8, 30, 0).unwrap().and_utc();
    let punch_out = today.and_hms_opt(17, 0, 0).unwrap().and_utc();

    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/attendance-trend"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 30);

    // Find today's entry — should have present=1 and late=1
    let today_str = today.format("%Y-%m-%d").to_string();
    let today_point = body
        .iter()
        .find(|p| p["date"].as_str().unwrap() == today_str);
    assert!(today_point.is_some(), "Expected today in trend data");
    let today_point = today_point.unwrap();
    assert_eq!(
        today_point["present"].as_i64().unwrap(),
        1,
        "Expected 1 present"
    );
    assert_eq!(
        today_point["late"].as_i64().unwrap(),
        1,
        "Expected 1 late via employee_shifts (6-priority chain)"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  6. Department Attendance
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_department_attendance_returns_empty_with_no_departments() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/department-attendance"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(body.is_empty(), "Expected empty when no departments");
}

#[tokio::test]
async fn test_department_attendance_returns_data_with_present_employees() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let dept_id = factory.department().with_workspace(ws_id).create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let today = Utc::now().date_naive();

    // Assign employee to department
    crate::test_execute!(
        &_db.diesel_pools.api,
        "UPDATE employments SET department_id = $2, start_date = $3 WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        dept_id,
        diesel::sql_types::Date,
        today - Duration::days(30)
    );

    // Create time entry that falls within workspace-local "today".
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let local_today = Utc::now().with_timezone(&tz).date_naive();
    let punch_in = local_today
        .and_hms_opt(8, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap()
        .to_utc();
    let punch_out = local_today
        .and_hms_opt(17, 0, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap()
        .to_utc();
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/department-attendance"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "Expected department attendance data");

    // Check structure
    for item in &body {
        assert!(item["department_id"].is_string());
        assert!(item["department_name"].is_string());
        assert!(item["on_time"].is_number());
        assert!(item["late"].is_number());
        assert!(item["absent"].is_number());
        assert!(item["on_leave"].is_number());
    }

    // The employee should show as on_time (no shift_rosters entry → late=0, on_time=present)
    let total_on_time: i64 = body.iter().map(|i| i["on_time"].as_i64().unwrap()).sum();
    assert!(
        total_on_time >= 1,
        "Expected at least 1 on_time, got {total_on_time}"
    );
}

#[tokio::test]
async fn test_department_attendance_supports_week_range() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
            "{address}/api/v1/dashboard/department-attendance?range=week"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let _body: Vec<serde_json::Value> = res.json().await.unwrap();
}

// ═══════════════════════════════════════════════════════════════════
//  7. Workspace Stats
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_workspace_stats_returns_all_fields() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/workspace-stats"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();

    // All expected fields should be present
    assert!(body["total_employees"].is_number());
    assert!(body["present_today"].is_number());
    assert!(body["pending_leave_requests"].is_number());
    assert!(
        body["estimated_monthly_payroll"].is_string()
            || body["estimated_monthly_payroll"].is_number()
    );
    assert!(body["late_today"].is_number());
    assert!(body["absent_today"].is_number());
    assert!(body["on_leave_today"].is_number());
    assert!(body["early_departure_today"].is_number());
    assert!(body["avg_work_hours_today"].is_number());

    // With no employees, all counts should be zero
    assert_eq!(body["total_employees"].as_i64().unwrap(), 0);
    assert_eq!(body["present_today"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_workspace_stats_counts_present_and_absent() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // Create 2 employees
    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let _emp2 = factory.employee().with_workspace(ws_id).create().await;

    // Only emp1 punches in now — Utc::now() always falls within the
    // workspace-local "today" window (regardless of timezone offset).
    let now = Utc::now();
    factory
        .time_entry()
        .with_employee(emp1)
        .with_punch_in(now)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/workspace-stats"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["total_employees"].as_i64().unwrap(), 2);
    assert_eq!(body["present_today"].as_i64().unwrap(), 1);
    assert_eq!(body["absent_today"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_workspace_stats_detects_late_today() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    // Use workspace-local "today" so the roster date matches the handler's date logic.
    let tz: chrono_tz::Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let local_today = Utc::now().with_timezone(&tz).date_naive();

    // Create shift and assign via shift_rosters (the fixed table)
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    crate::test_execute!(&pool.api,
        "INSERT INTO shift_rosters (employment_id, template_id, date, status) VALUES ($1, $2, $3, 'confirmed')",
        diesel::sql_types::Uuid, emp_employment_id,
        diesel::sql_types::Uuid, shift_id,
        diesel::sql_types::Date, local_today
    );

    // Late punch-in: compute a punch time clearly within today's shift window
    // but after the 08:00 shift start (workspace local time).
    let punch_in = local_today
        .and_hms_opt(8, 30, 0)
        .unwrap()
        .and_local_timezone(tz)
        .unwrap()
        .to_utc();
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/workspace-stats"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["late_today"].as_i64().unwrap(), 1);
}

// ═══════════════════════════════════════════════════════════════════
//  8. Overtime Trend
// ═══════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_overtime_trend_returns_empty_with_no_data() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/overtime-trend"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(
        body.is_empty(),
        "Expected empty when no attendance summaries"
    );
}

#[tokio::test]
async fn test_overtime_trend_returns_correct_structure() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let pool = _db.diesel_pools.clone();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let today = Utc::now().date_naive();

    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // Insert a summary row with overtime data
    crate::test_execute!(
        &pool.api,
        r"INSERT INTO attendance_daily_summaries
           (workspace_id, employment_id, date, system_detected_ot_minutes, approved_ot_minutes)
           VALUES ($1, $2, $3, 60, 30)",
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Uuid,
        emp_employment_id,
        diesel::sql_types::Date,
        today
    );

    let res = client
        .get(format!("{address}/api/v1/dashboard/overtime-trend"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert!(!body.is_empty(), "Expected overtime data");

    let point = &body[0];
    assert!(point["date"].is_string());
    assert!(point["system_ot_hours"].is_number());
    assert!(point["approved_ot_hours"].is_number());

    // 60 min = 1.0 hours, 30 min = 0.5 hours
    let system_ot = point["system_ot_hours"].as_f64().unwrap();
    let approved_ot = point["approved_ot_hours"].as_f64().unwrap();
    assert!(
        (system_ot - 1.0).abs() < 0.01,
        "Expected system_ot_hours=1.0, got {system_ot}"
    );
    assert!(
        (approved_ot - 0.5).abs() < 0.01,
        "Expected approved_ot_hours=0.5, got {approved_ot}"
    );
}

// ═══════════════════════════════════════════════════════════════════
//  9. Analytics upgrade — rates, custom range, permission gate
// ═══════════════════════════════════════════════════════════════════

/// Assert the shared `RateQuantity` shape (`value`/`numerator`/`denominator`/`unit`)
/// is present on a JSON object.
fn assert_rate_shape(json: &serde_json::Value, key: &str) {
    let rate = &json[key];
    assert!(
        rate["value"].is_string() || rate["value"].is_null(),
        "rate {key}.value must be a decimal string or null, got {:?}",
        rate["value"]
    );
    assert!(
        rate["numerator"].is_number(),
        "rate {key}.numerator must be a number"
    );
    assert!(
        rate["denominator"].is_number(),
        "rate {key}.denominator must be a number"
    );
    assert!(rate["unit"].is_string(), "rate {key}.unit must be a string");
}

#[tokio::test]
async fn test_workspace_stats_includes_rate_fields() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let emp1 = factory.employee().with_workspace(ws_id).create().await;
    let _emp2 = factory.employee().with_workspace(ws_id).create().await;
    factory
        .time_entry()
        .with_employee(emp1)
        .with_punch_in(Utc::now())
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/workspace-stats"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();

    for key in ["attendance_rate", "absence_rate", "late_rate"] {
        assert_rate_shape(&body, key);
    }

    // 2 employees, 1 present → attendance 50%, absence 50%.
    assert_eq!(
        body["attendance_rate"]["numerator"].as_i64().unwrap(),
        1,
        "attendance numerator = present today"
    );
    assert_eq!(
        body["attendance_rate"]["denominator"].as_i64().unwrap(),
        2,
        "attendance denominator = total employees"
    );
    assert_eq!(
        body["absence_rate"]["numerator"].as_i64().unwrap(),
        1,
        "absence numerator = absent today"
    );
    assert_eq!(
        body["attendance_rate"]["unit"].as_str().unwrap(),
        "employees"
    );
}

#[tokio::test]
async fn test_department_attendance_rates_sorted_desc_with_top_n() {
    let (address, ws_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let today = Utc::now().date_naive();

    let dept_a = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Engineering")
        .create()
        .await;
    let dept_b = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    let emp_a1 = factory.employee().with_workspace(ws_id).create().await;
    let _emp_a2 = factory.employee().with_workspace(ws_id).create().await;
    let emp_b1 = factory.employee().with_workspace(ws_id).create().await;

    for (emp_id, dept_id) in [(emp_a1, dept_a), (_emp_a2, dept_a), (emp_b1, dept_b)] {
        crate::test_execute!(
            &_db.diesel_pools.api,
            "UPDATE employments SET department_id = $2, start_date = $3 WHERE employee_id = $1",
            diesel::sql_types::Uuid,
            emp_id,
            diesel::sql_types::Uuid,
            dept_id,
            diesel::sql_types::Date,
            today - Duration::days(30)
        );
    }

    // 1 present in Engineering (of 2) → 50%; 1 present in Sales (of 1) → 100%.
    let now = Utc::now();
    factory
        .time_entry()
        .with_employee(emp_a1)
        .with_punch_in(now)
        .create()
        .await;
    factory
        .time_entry()
        .with_employee(emp_b1)
        .with_punch_in(now)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/v1/dashboard/department-attendance"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 2, "expected 2 departments");

    // Sorted descending by attendance rate: Sales (100%) before Engineering (50%).
    assert_eq!(
        body[0]["department_name"].as_str().unwrap(),
        "Sales",
        "highest rate first"
    );
    assert_eq!(body[1]["department_name"].as_str().unwrap(), "Engineering");

    for item in &body {
        assert!(item["total_employees"].is_number());
        assert_rate_shape(item, "attendance_rate");
        assert_rate_shape(item, "absence_rate");
        assert_rate_shape(item, "late_rate");
    }
    let sales_rate = body[0]["attendance_rate"].clone();
    assert_eq!(sales_rate["numerator"].as_i64().unwrap(), 1);
    assert_eq!(sales_rate["denominator"].as_i64().unwrap(), 1);

    // Top-N: top_n=1 returns only the highest-rate department.
    let res2 = client
        .get(format!(
            "{address}/api/v1/dashboard/department-attendance?top_n=1"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), reqwest::StatusCode::OK);
    let body2: Vec<serde_json::Value> = res2.json().await.unwrap();
    assert_eq!(body2.len(), 1);
    assert_eq!(body2[0]["department_name"].as_str().unwrap(), "Sales");
}

#[tokio::test]
async fn test_attendance_trend_emits_rate_fields_with_zero_denominator() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/v1/dashboard/attendance-trend"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 30);

    let first = &body[0];
    assert!(first["eligible"].is_number());

    // No employees → denominator is 0, so rate value must be null (never a fake 0%).
    for key in ["attendance_rate", "absence_rate", "late_rate"] {
        let rate = &first[key];
        assert!(
            rate["value"].is_null(),
            "{key}.value must be null when denominator is 0"
        );
        assert_eq!(rate["numerator"].as_i64().unwrap(), 0);
        assert_eq!(rate["denominator"].as_i64().unwrap(), 0);
        assert_eq!(rate["unit"].as_str().unwrap(), "employee_days");
    }
}

#[tokio::test]
async fn test_attendance_trend_custom_range_validation() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let today = Utc::now().date_naive();

    let get = |url: String| {
        let client = &client;
        let ws_id = ws_id;
        let user_id = user_id;
        async move {
            client
                .get(url)
                .header("X-User-Id", user_id.to_string())
                .header("X-Workspace-Id", ws_id.to_string())
                .send()
                .await
                .unwrap()
        }
    };

    // Valid window → 200 with exactly (end - start + 1) days.
    let start = today - Duration::days(6);
    let res = get(format!(
        "{address}/api/v1/dashboard/attendance-trend?start_date={start}&end_date={today}"
    ))
    .await;
    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: Vec<serde_json::Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 7, "a 7-day inclusive window returns 7 points");

    // end < start → 400 (no silent fallback).
    let res = get(format!(
        "{address}/api/v1/dashboard/attendance-trend?start_date={today}&end_date={start}"
    ))
    .await;
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);

    // end > today → 400.
    let future = today + Duration::days(1);
    let res = get(format!(
        "{address}/api/v1/dashboard/attendance-trend?start_date={start}&end_date={future}"
    ))
    .await;
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);

    // Half-specified range → 400 (missing bound must not be silently defaulted).
    let res = get(format!(
        "{address}/api/v1/dashboard/attendance-trend?start_date={start}"
    ))
    .await;
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);

    // Malformed date → 400.
    let res = get(format!(
        "{address}/api/v1/dashboard/attendance-trend?start_date=not-a-date&end_date={today}"
    ))
    .await;
    assert_eq!(res.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_attendance_trend_rejects_invalid_timezone() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!(
            "{address}/api/v1/dashboard/attendance-trend?timezone=Not/AZone"
        ))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        reqwest::StatusCode::BAD_REQUEST,
        "invalid timezone must be rejected, not silently defaulted"
    );
}

#[tokio::test]
async fn test_unexcused_absences_employee_mode_requires_permission() {
    let (address, ws_id, _owner_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // A member on the employee role who has `dashboard:read` but NOT the new
    // `dashboard:employee_absence:read` PII gate.
    let viewer = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            viewer,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;
    crate::test_execute!(
        &_db.diesel_pools.api,
        "INSERT INTO workspace_member_extra_permissions (workspace_id, user_id, permission, expires_at) VALUES ($1, $2, 'dashboard:read', $3)",
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Uuid,
        viewer,
        diesel::sql_types::Timestamptz,
        Utc::now() + Duration::days(1)
    );

    // Employee mode surfaces per-employee PII → 403 without the dedicated permission.
    let res = client
        .get(format!(
            "{address}/api/v1/dashboard/unexcused-absences?group_by=employee"
        ))
        .header("X-User-Id", viewer.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::FORBIDDEN);

    // Department mode only needs `dashboard:read` → 200.
    let res = client
        .get(format!(
            "{address}/api/v1/dashboard/unexcused-absences?group_by=department"
        ))
        .header("X-User-Id", viewer.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), reqwest::StatusCode::OK);
}
