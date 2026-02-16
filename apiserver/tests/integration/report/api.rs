//! Report API integration tests

use crate::common::app::spawn_app;
use chrono::{Datelike, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;

#[tokio::test]
async fn test_get_daily_attendance_report_with_break_times() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    // 1. Setup
    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // 2. Create Employee
    let emp = factory.employee()
        .with_workspace(workspace_id)
        .with_full_name("Break Time Test")
        .with_employee_id_number("BT001")
        .create()
        .await;

    // 3. Create Shift (9:00 - 18:00) with 60 min break
    factory.shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap()
        )
        .with_break_duration(60)
        .assign_to_employee(emp, Utc::now().date_naive())
        .await;

    // 4. Record Attendance
    // Punch In: 9:00
    let now = Utc::now().date_naive();
    let punch_in = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 0, 0).unwrap();
    let punch_out = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 18, 0, 0).unwrap();

    // Break: 12:00 - 13:00
    let break_start = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 12, 0, 0).unwrap();
    let break_end = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 13, 0, 0).unwrap();

    factory.time_entry().with_employee(emp).with_punch_in(punch_in).with_punch_out(punch_out).with_entry_type("work").create().await;
    factory.time_entry().with_employee(emp).with_punch_in(break_start).with_punch_out(break_end).with_entry_type("break").create().await;



    // 5. Fetch Request
    let response = client
        .get(format!("{address}/api/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={now}&end_date={now}"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_workspace_attendance_summary() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let response = client
        .get(format!("{}/api/workspaces/{}/reports/attendance?type=summary&start_date={}&end_date={}", address, workspace_id, Utc::now().format("%Y-%m-%d"), Utc::now().format("%Y-%m-%d")))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
