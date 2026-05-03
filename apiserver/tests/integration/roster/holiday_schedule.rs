//! Tests for holiday metadata in `EffectiveSchedule`

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::EmployeeSchedule;
use chrono::NaiveDate;
use reqwest::StatusCode;
use rust_decimal::Decimal;
use uuid::Uuid;

#[tokio::test]
async fn test_schedule_shows_holiday_metadata() {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let auth_token = factory.auth().create_token(user_id);

    let emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // 1. Create a holiday on 2026-05-01
    let holiday_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    let _holiday_id: Uuid = sqlx::query_scalar(
        "INSERT INTO holidays (workspace_id, name, date, holiday_type, is_paid, ot_multiplier) 
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(workspace_id)
    .bind("Labor Day")
    .bind(holiday_date)
    .bind("national")
    .bind(true)
    .bind(Decimal::from(2)) // 200% logic
    .fetch_one(&_db.pool())
    .await
    .unwrap();

    // 2. Fetch schedule via API for a week that includes the holiday
    let start_date = "2026-04-27"; // Monday
    let end_date = "2026-05-03"; // Sunday

    let res = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/schedules?start_date={start_date}&end_date={end_date}&employee_ids={emp_id}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let schedules: Vec<EmployeeSchedule> = res.json().await.unwrap();

    // 3. Verification
    assert_eq!(schedules.len(), 1);
    let emp_sched = &schedules[0];

    // Find the schedule for the holiday date
    let holiday_sched = emp_sched
        .schedules
        .iter()
        .find(|s| s.date == holiday_date)
        .expect("Schedule for holiday date should exist");

    // Assert holiday metadata is correctly populated
    assert!(holiday_sched.is_holiday, "is_holiday should be true");
    assert_eq!(
        holiday_sched.holiday_name,
        Some("Labor Day".to_string()),
        "holiday_name should match"
    );

    // Find a non-holiday schedule
    let regular_date = NaiveDate::from_ymd_opt(2026, 4, 30).unwrap();
    let regular_sched = emp_sched
        .schedules
        .iter()
        .find(|s| s.date == regular_date)
        .expect("Schedule for regular date should exist");

    // Assert holiday metadata is default/false for non-holidays
    assert!(
        !regular_sched.is_holiday,
        "is_holiday should be false for regular days"
    );
    assert_eq!(
        regular_sched.holiday_name, None,
        "holiday_name should be None for regular days"
    );
}
