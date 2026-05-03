//! Integration tests for holiday-aware report status
//!
//! Verifies that the daily attendance report returns correct status values:
//! - `holiday`: no punch on a holiday
//! - `holiday_work`: punch exists on a holiday
//! - `present`/`late`/`absent`/`off`: non-holiday days (unchanged)

use crate::common::app::spawn_app;
use chrono::{Datelike, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PaginatedReport {
    data: Vec<ReportRow>,
}

#[derive(Debug, Deserialize)]
struct ReportRow {
    status: String,
    is_holiday: bool,
    holiday_name: Option<String>,
}

/// Create a holiday for today, employee has NO punch → status should be "holiday"
#[tokio::test]
async fn test_report_status_holiday_no_punch() {
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

    // Assign a shift so the employee is "expected" to work
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Create a holiday for today via API
    let create_resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Test Holiday",
            "date": today.to_string(),
            "holiday_type": "national",
            "is_paid": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    // Fetch the daily attendance report
    let report_resp = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(report_resp.status(), StatusCode::OK);

    let report: PaginatedReport = report_resp.json().await.unwrap();
    assert_eq!(report.data.len(), 1);

    let row = &report.data[0];
    assert_eq!(
        row.status, "holiday_paid",
        "No punch on paid holiday → status should be 'holiday_paid'"
    );
    assert!(row.is_holiday, "is_holiday should be true");
    assert_eq!(row.holiday_name.as_deref(), Some("Test Holiday"));
}

/// Create a holiday for today, employee HAS a punch → status should be "`holiday_work`"
#[tokio::test]
async fn test_report_status_holiday_with_punch() {
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

    // Assign shift
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Create holiday
    let create_resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Quốc tế Lao động",
            "date": today.to_string(),
            "holiday_type": "national",
            "is_paid": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    // Create a time entry (employee worked on holiday)
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

    // Fetch report
    let report_resp = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(report_resp.status(), StatusCode::OK);

    let report: PaginatedReport = report_resp.json().await.unwrap();
    assert_eq!(report.data.len(), 1);

    let row = &report.data[0];
    assert_eq!(
        row.status, "holiday_work",
        "Punch on holiday → status should be 'holiday_work'"
    );
    assert!(row.is_holiday, "is_holiday should be true");
    assert_eq!(row.holiday_name.as_deref(), Some("Quốc tế Lao động"));
}
