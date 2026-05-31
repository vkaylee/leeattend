//! Report API integration tests

use crate::common::app::spawn_app;
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
