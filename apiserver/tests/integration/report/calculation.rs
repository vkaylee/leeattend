//! Report calculation tests

use crate::integration::common::app::spawn_app;

use chrono::{Datelike, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;

#[tokio::test]
async fn test_report_calculation_grace_period_and_break() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::integration::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);
    let emp = factory.employee().with_workspace(workspace_id).create().await;
    let now = Utc::now().date_naive();

    // Shift: 9:00 - 18:00
    factory.shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap()
        )
        .assign_to_employee(emp, now)
        .await;

    // Late Punch In: 9:16 (Late)
    let punch_in = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 16, 0).unwrap();
    let punch_out = Utc.with_ymd_and_hms(now.year(), now.month(), now.day(), 18, 0, 0).unwrap();

    factory.time_entry().with_employee(emp).with_punch_in(punch_in).with_punch_out(punch_out).with_entry_type("work").create().await;



    let response = client
        .get(format!("{}/api/workspaces/{}/reports/daily-attendance?start_date={}&end_date={}", address, workspace_id, now, now))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
