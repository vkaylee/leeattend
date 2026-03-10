//! Integration tests for WF5: Attendance Summary holiday flag
//! Verifies that the attendance summary API returns is_holiday and holiday_name
//! correctly when holidays exist for the queried date range.

use crate::common::app::spawn_app;
use chrono::{Datelike, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct SummaryRow {
    is_holiday: bool,
    holiday_name: Option<String>,
    actual_work_minutes: i32,
}

/// Create a holiday, trigger recalculation, verify the summary record has is_holiday=true
#[tokio::test]
async fn test_summary_records_holiday_flag_after_recalculate() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);
    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let today = Utc::now().date_naive();

    // Assign a shift
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Create a holiday via API
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Ngày Quốc khánh",
            "date": today.to_string(),
            "holiday_type": "national",
            "is_paid": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Create a time entry (working on holiday)
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
        .with_entry_type("work")
        .create()
        .await;

    // Trigger recalculation directly via service
    apiserver::services::AttendanceSummaryService::new(std::sync::Arc::new(apiserver::repositories::SqlxTimeEntryRepository::new(pool.clone())), std::sync::Arc::new(apiserver::repositories::roster::SqlxRosterRepository::new(pool.clone())), pool.clone()).recalculate_daily_summary(
        workspace_id,
        emp,
        today,
    )
    .await
    .expect("Recalculate failed");

    // Verify the summary record in DB directly
    let row = sqlx::query_as!(
        SummaryRow,
        "SELECT is_holiday, holiday_name, actual_work_minutes FROM attendance_daily_summaries WHERE employee_id = $1 AND date = $2",
        emp,
        today
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(row.is_some(), "Summary record should exist");
    let summary = row.unwrap();
    assert!(summary.is_holiday, "is_holiday should be true for a holiday");
    assert_eq!(summary.holiday_name.as_deref(), Some("Ngày Quốc khánh"));
    assert!(summary.actual_work_minutes > 0, "Should have work minutes");
}
