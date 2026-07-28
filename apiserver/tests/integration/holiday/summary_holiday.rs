//! Integration tests for WF5: Attendance Summary holiday flag
//! Verifies that the attendance summary API returns `is_holiday` and `holiday_name`
//! correctly when holidays exist for the queried date range.

use crate::common::app::spawn_app;
use chrono::{Datelike, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;
use serde::Deserialize;

/// Create a holiday, trigger recalculation, verify the summary record has `is_holiday=true`
#[tokio::test]
async fn test_summary_records_holiday_flag_after_recalculate() {
    let (address, test_db) = spawn_app().await;
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let pool = test_db.diesel_pools.clone();
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
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/holidays"
        ))
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

    let employment_id: uuid::Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp
    );

    // Trigger recalculation directly via service
    apiserver::services::AttendanceSummaryService::new(
        apiserver::services::attendance_summary_service::AttendanceSummaryServiceArgs {
            feature_flag_repo: None,
            time_entry_repo: std::sync::Arc::new(
                apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone()),
            ),
            roster_repo: std::sync::Arc::new(apiserver::repositories::DieselRosterRepository::new(
                diesel_pools.clone(),
            )),
            policy_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendancePolicyRepository::new(
                    diesel_pools.clone(),
                ),
            ),
            condition_type_repo: std::sync::Arc::new(
                apiserver::repositories::DieselConditionTypeRepository::new(diesel_pools.clone()),
            ),
            workspace_repo: std::sync::Arc::new(
                apiserver::repositories::DieselWorkspaceRepository::new(diesel_pools.clone()),
            ),
            employee_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmployeeRepository::new(diesel_pools.clone()),
            ),
            employment_repo: std::sync::Arc::new(
                apiserver::repositories::DieselEmploymentRepository::new(diesel_pools.clone()),
            ),
            holiday_repo: std::sync::Arc::new(
                apiserver::repositories::DieselHolidayRepository::new(diesel_pools.clone()),
            ),
            leave_request_repo: std::sync::Arc::new(
                apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone()),
            ),
            daily_summary_repo: std::sync::Arc::new(
                apiserver::repositories::DieselAttendanceDailySummaryRepository::new(
                    diesel_pools.clone(),
                ),
            ),
        },
    )
    .recalculate_daily_summary(
        &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id),
        employment_id,
        today,
    )
    .await
    .expect("Recalculate failed");

    let repo =
        apiserver::repositories::DieselAttendanceDailySummaryRepository::new(diesel_pools.clone());
    let summaries = apiserver::repositories::attendance_daily_summary_repository::AttendanceDailySummaryRepository::get_summaries(
        &repo, &apiserver::diesel_pool::TenantAuthToken::system_override(workspace_id), Some(employment_id), today, today
    ).await.unwrap();

    let summary = summaries.into_iter().next().unwrap();
    assert!(
        summary.is_holiday,
        "is_holiday should be true for a holiday"
    );
    assert_eq!(summary.holiday_name.as_deref(), Some("Ngày Quốc khánh"));
    assert!(summary.actual_work_minutes > 0, "Should have work minutes");
}
