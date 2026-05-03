//! Integration tests for dashboard chart endpoints:
//! - GET /api/dashboard/peak-times
//! - GET /api/dashboard/leave-types
//! - GET /api/dashboard/lateness-by-day
//! - GET /api/dashboard/unexcused-absences

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
        .get(format!("{address}/api/dashboard/peak-times"))
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

    let today = Utc::now().date_naive();
    let punch_in = today.and_hms_opt(8, 15, 0).unwrap().and_utc();
    let punch_out = today.and_hms_opt(17, 30, 0).unwrap().and_utc();

    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/dashboard/peak-times"))
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

    // Verify check-in slot
    let has_checkin_slot = body.iter().any(|s| {
        s["time_slot"].as_str().unwrap() == "08:00-08:30" && s["check_in"].as_i64().unwrap() > 0
    });
    assert!(has_checkin_slot, "Expected a check-in in 08:00-08:30 slot");

    // Verify check-out slot
    let has_checkout_slot = body.iter().any(|s| {
        s["time_slot"].as_str().unwrap() == "17:30-18:00" && s["check_out"].as_i64().unwrap() > 0
    });
    assert!(
        has_checkout_slot,
        "Expected a check-out in 17:30-18:00 slot"
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
        .get(format!("{address}/api/dashboard/leave-types"))
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
    let pool = _db.pool();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    let vacation_type_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(ws_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let sick_type_id: Uuid =
        sqlx::query_scalar("SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'sick'")
            .bind(ws_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    // Create approved leave requests this month (keep within current month)
    factory
        .leave_request()
        .with_employee(emp_id)
        .with_leave_type(vacation_type_id)
        .with_dates(today - Duration::days(1), today)
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
        .get(format!("{address}/api/dashboard/leave-types"))
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
        .get(format!("{address}/api/dashboard/lateness-by-day"))
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
        .get(format!("{address}/api/dashboard/lateness-by-day"))
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
        .get(format!("{address}/api/dashboard/unexcused-absences"))
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
    let pool = _db.pool();

    let dept_id = factory.department().with_workspace(ws_id).create().await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Assign employee to department
    sqlx::query("INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, $3)")
        .bind(emp_id)
        .bind(dept_id)
        .bind(today - Duration::days(30))
        .execute(&pool)
        .await
        .unwrap();

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
        .get(format!("{address}/api/dashboard/unexcused-absences"))
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
            "{address}/api/dashboard/unexcused-absences?start_date={start}&end_date={end}"
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
    let pool = _db.pool();

    let dept_id = factory.department().with_workspace(ws_id).create().await;

    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let today = Utc::now().date_naive();

    // Assign employee to department
    sqlx::query("INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, $3)")
        .bind(emp_id)
        .bind(dept_id)
        .bind(today - Duration::days(30))
        .execute(&pool)
        .await
        .unwrap();

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
            "{address}/api/dashboard/unexcused-absences?group_by=employee"
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
        .get(format!("{address}/api/dashboard/unexcused-absences"))
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
            "{address}/api/dashboard/unexcused-absences?group_by=employee"
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
        .get(format!("{address}/api/dashboard/attendance-trend"))
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
    let pool = _db.pool();

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

    // Insert into shift_rosters directly (the table that was fixed)
    sqlx::query(
        "INSERT INTO shift_rosters (employee_id, shift_id, date, status) VALUES ($1, $2, $3, 'confirmed')"
    )
    .bind(emp_id)
    .bind(shift_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

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
        .get(format!("{address}/api/dashboard/attendance-trend"))
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
        .get(format!("{address}/api/dashboard/attendance-trend"))
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
        .get(format!("{address}/api/dashboard/department-attendance"))
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
    let pool = _db.pool();

    let dept_id = factory.department().with_workspace(ws_id).create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let today = Utc::now().date_naive();

    // Assign employee to department
    sqlx::query("INSERT INTO employee_department_history (employee_id, department_id, effective_date) VALUES ($1, $2, $3)")
        .bind(emp_id)
        .bind(dept_id)
        .bind(today - Duration::days(30))
        .execute(&pool)
        .await
        .unwrap();

    // Create time entry for today
    let punch_in = today.and_hms_opt(8, 0, 0).unwrap().and_utc();
    let punch_out = today.and_hms_opt(17, 0, 0).unwrap().and_utc();
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/dashboard/department-attendance"))
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
            "{address}/api/dashboard/department-attendance?range=week"
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
        .get(format!("{address}/api/dashboard/workspace-stats"))
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

    // Only emp1 punches in today
    let today = Utc::now().date_naive();
    let punch_in = today.and_hms_opt(8, 0, 0).unwrap().and_utc();
    factory
        .time_entry()
        .with_employee(emp1)
        .with_punch_in(punch_in)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/dashboard/workspace-stats"))
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
    let pool = _db.pool();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let today = Utc::now().date_naive();

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

    sqlx::query(
        "INSERT INTO shift_rosters (employee_id, shift_id, date, status) VALUES ($1, $2, $3, 'confirmed')"
    )
    .bind(emp_id)
    .bind(shift_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    // Late punch-in at 08:30 (after shift start 08:00)
    let punch_in = today.and_hms_opt(8, 30, 0).unwrap().and_utc();
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .create()
        .await;

    let res = client
        .get(format!("{address}/api/dashboard/workspace-stats"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    assert_eq!(body["present_today"].as_i64().unwrap(), 1);
    assert_eq!(
        body["late_today"].as_i64().unwrap(),
        1,
        "Expected late_today=1 (exercises shift_rosters join)"
    );
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
        .get(format!("{address}/api/dashboard/overtime-trend"))
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
    let pool = _db.pool();

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let today = Utc::now().date_naive();

    // Insert a summary row with overtime data
    sqlx::query(
        r"INSERT INTO attendance_daily_summaries
           (workspace_id, employee_id, date, system_detected_ot_minutes, approved_ot_minutes)
           VALUES ($1, $2, $3, 60, 30)",
    )
    .bind(ws_id)
    .bind(emp_id)
    .bind(today)
    .execute(&pool)
    .await
    .unwrap();

    let res = client
        .get(format!("{address}/api/dashboard/overtime-trend"))
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
