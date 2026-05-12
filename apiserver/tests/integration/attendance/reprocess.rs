//! Attendance Reprocess API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();
}

#[tokio::test]
async fn test_force_reprocess_attendance_api() {
    init_tracing();
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

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
    for punch_time in logs {
        let raw_log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            employee_id: emp_id,
            punch_time,
            source: LogSource::Manual,
            device_id: None,
            raw_data: None,
            punch: -1,
            status: -1,
            created_at: Utc::now(),
        };
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(
            &factory.pool,
            &raw_log,
        )
        .await
        .unwrap();
    }

    // 2. Verify time_entries are initially empty
    let entries = sqlx::query!("SELECT id FROM time_entries WHERE employee_id = $1", emp_id)
        .fetch_all(&factory.pool)
        .await
        .unwrap();
    assert!(
        entries.is_empty(),
        "Entries should be empty before reprocessing"
    );

    // 3. Call the reprocess endpoint
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/attendance/reprocess/{emp_id}?start_date={base_date}&end_date={base_date}"
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

#[tokio::test]
async fn test_force_reprocess_all_attendance_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_1 = factory.employee().with_workspace(ws_id).create().await;
    let emp_id_2 = factory.employee().with_workspace(ws_id).create().await;

    let base_date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    // Use UTC times that result in the SAME DAY for Asia/Ho_Chi_Minh (UTC+7)
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(
        &std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/leeattend".to_string()
        }),
    );
    let start_time = Utc.with_ymd_and_hms(2026, 1, 14, 1, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2026, 1, 14, 10, 0, 0).unwrap();

    // 1. Create raw logs WITHOUT processing them (bypass service) for BOTH employees
    for emp_id in [emp_id_1, emp_id_2] {
        let logs = vec![start_time, end_time];
        for punch_time in logs {
            let raw_log = RawAttendanceLog {
                id: Uuid::new_v4(),
                workspace_id: ws_id,
                employee_id: emp_id,
                punch_time,
                source: LogSource::Manual,
                device_id: None,
                raw_data: None,
                punch: -1,
                status: -1,
                created_at: Utc::now(),
            };
            apiserver::repositories::raw_attendance_log_repository::create_raw_log(
                &factory.pool,
                &raw_log,
            )
            .await
            .unwrap();
        }
    }

    // 2. Verify time_entries are initially empty
    let entries = sqlx::query!("SELECT id FROM time_entries WHERE workspace_id = $1", ws_id)
        .fetch_all(&factory.pool)
        .await
        .unwrap();
    assert!(
        entries.is_empty(),
        "Entries should be empty before bulk reprocessing"
    );

    // 3. Call the bulk reprocess endpoint
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/attendance/reprocess-all?start_date={base_date}&end_date={base_date}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let parent_job_id = body_json["job_id"]
        .as_str()
        .unwrap()
        .parse::<Uuid>()
        .unwrap();

    // 4. Verify job metadata was created correctly
    let redis = _state.infra.redis.as_ref().unwrap();
    let queue = apiserver::queue::JobQueue::new(redis.clone());

    let meta = queue
        .get_job_metadata(parent_job_id)
        .await
        .unwrap()
        .expect("Job metadata must exist");
    assert_eq!(meta.total_batches, 2);
    assert_eq!(meta.status, apiserver::queue::JobStatus::Pending);

    // 5. Construct and execute jobs manually against test DB
    //    The background Docker worker may consume the queued jobs but processes them
    //    against a different DB. We construct identical jobs and process them directly
    //    against the test's isolated DB to verify the handler logic.
    let coordinator =
        apiserver::services::ReportCoordinator::new(redis.clone(), _state.infra.db.api().clone());
    for (idx, emp_id) in [emp_id_1, emp_id_2].iter().enumerate() {
        let job = apiserver::queue::Job::with_priority(
            parent_job_id,
            idx as u32,
            apiserver::queue::TaskType::ReprocessAttendance,
            ws_id,
            apiserver::queue::JobPayload::ReprocessAttendance(apiserver::queue::ReprocessPayload {
                employee_id: *emp_id,
                date_range: apiserver::queue::DateRange {
                    start: base_date,
                    end: base_date,
                },
            }),
            apiserver::queue::JobPriority::Normal,
        );

        apiserver::worker::handlers::reprocess_attendance::ReprocessAttendanceHandler::handle(
            _state.infra.db.api(),
            &diesel_pools,
            &job,
            redis,
        )
        .await
        .unwrap();

        coordinator
            .on_batch_complete(parent_job_id, idx as u32, true)
            .await
            .unwrap();
    }

    // 6. Verify job status is now Completed
    let meta = queue
        .get_job_metadata(parent_job_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(meta.status, apiserver::queue::JobStatus::Completed);

    // 7. Verify time_entries are now created correctly for both employees
    for emp_id in [emp_id_1, emp_id_2] {
        let entries = sqlx::query!(
            "SELECT punch_in_time, punch_out_time FROM time_entries WHERE employee_id = $1 ORDER BY punch_in_time",
            emp_id
        )
        .fetch_all(&factory.pool)
        .await
        .unwrap();

        assert_eq!(
            entries.len(),
            1,
            "One time entry should have been created for employee"
        );
        assert_eq!(entries[0].punch_in_time, start_time);
        assert_eq!(entries[0].punch_out_time, Some(end_time));
    }
}
