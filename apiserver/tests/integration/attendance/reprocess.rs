//! Attendance Reprocess API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{TimeZone, Utc, NaiveDate};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_force_reprocess_attendance_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let base_date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    // Use UTC times that result in the SAME DAY for Asia/Ho_Chi_Minh (UTC+7)
    // 01:00 UTC = 08:00 Local
    // 10:00 UTC = 17:00 Local
    let start_time = Utc.with_ymd_and_hms(2026, 1, 14, 1, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2026, 1, 14, 10, 0, 0).unwrap();

    // 1. Create raw logs WITHOUT processing them (bypass service)
    let logs = vec![start_time, end_time];
    for punch in logs {
        let raw_log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            employee_id: emp_id,
            punch_time: punch,
            source: LogSource::Manual,
            device_id: None,
            raw_data: None,
            created_at: Utc::now(),
        };
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(&factory.pool, &raw_log)
            .await
            .unwrap();
    }

    // 2. Verify time_entries are initially empty
    let entries = sqlx::query!(
        "SELECT id FROM time_entries WHERE employee_id = $1",
        emp_id
    )
    .fetch_all(&factory.pool)
    .await
    .unwrap();
    assert!(entries.is_empty(), "Entries should be empty before reprocessing");

    // 3. Call the reprocess endpoint
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/reprocess/{}?start_date={}&end_date={}",
                    ws_id, emp_id, base_date, base_date
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Verify time_entries are now created correctly
    let entries = sqlx::query!(
        "SELECT punch_in_time, punch_out_time FROM time_entries WHERE employee_id = $1 ORDER BY punch_in_time",
        emp_id
    )
    .fetch_all(&factory.pool)
    .await
    .unwrap();

    assert_eq!(entries.len(), 1, "One time entry should have been created");
    assert_eq!(entries[0].punch_in_time, start_time);
    assert_eq!(entries[0].punch_out_time, Some(end_time));
}
