//! Time Tracking integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::services::auth_service::AuthService;
use apiserver::AppState;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::{Duration, NaiveDate, NaiveTime, Utc};
use chrono_tz::Tz;
use serde_json::json;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_punch_in_grace_period_enforcement_smoke() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Use a fixed date/time to avoid midnight rollover issues
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 10).unwrap();
    let shift_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();

    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_times(shift_start, NaiveTime::from_hms_opt(17, 0, 0).unwrap())
        .with_grace_period(5) // 5 mins grace -> earliest is 08:55
        .assign_to_employee(employee_id, test_date)
        .await;

    let token = AuthService::generate_token(user_id, "test_secret_tower", 3600).unwrap();

    // Try to punch in at 08:30 -> Should fail (400) because 08:30 < 08:55
    let punch_payload = json!({
        "employee_id": employee_id,
        "entry_type": "work"
    });

    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let real_now = Utc::now().with_timezone(&tz);
    let today = real_now.date_naive();
    let future_start = NaiveTime::from_hms_opt(23, 59, 0).unwrap(); // Far in the future today

    // Ensure we don't rollover if it's already very late
    let (test_date, test_start) = if real_now.time() > NaiveTime::from_hms_opt(23, 0, 0).unwrap() {
        (
            today + Duration::days(1),
            NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
        )
    } else {
        (today, future_start)
    };

    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE shifts SET start_time = $1, end_time = $2 WHERE id = $3",
        diesel::sql_types::Time,
        test_start,
        diesel::sql_types::Time,
        NaiveTime::from_hms_opt(23, 59, 59).unwrap(),
        diesel::sql_types::Uuid,
        shift_id
    );

    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE employee_shifts SET start_date = $1 WHERE shift_id = $2",
        diesel::sql_types::Date,
        test_date,
        diesel::sql_types::Uuid,
        shift_id
    );

    // Now set up for the fail case
    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE employee_shifts SET start_date = $1 WHERE shift_id = $2",
        diesel::sql_types::Date,
        today,
        diesel::sql_types::Uuid,
        shift_id
    );

    // Set start_time to now + 4 hours (definitely in future, so grace period prevents punch in)
    let test_start_time = (real_now + Duration::hours(4)).time();

    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE shifts SET start_time = $1 WHERE id = $2",
        diesel::sql_types::Time,
        test_start_time,
        diesel::sql_types::Uuid,
        shift_id
    );

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{ws_id}/time/punch-in"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&punch_payload).unwrap()))
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();

    // Only assert if we didn't wrap around midnight within 4 hours
    if test_start_time > real_now.time() {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // Clean up any accidental punch-in from the first attempt
    crate::test_execute!(
        &factory.diesel_pools.api,
        "DELETE FROM time_entries WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id
    );

    // Now update shift to have a large grace period (6 hours)
    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE shifts SET early_punch_in_grace_period_minutes = 360 WHERE id = $1",
        diesel::sql_types::Uuid,
        shift_id
    );

    let req2 = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/workspaces/{ws_id}/time/punch-in"))
        .header("Authorization", format!("Bearer {token}"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_string(&punch_payload).unwrap()))
        .unwrap();

    let response2 = app.clone().oneshot(req2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::CREATED);
}
