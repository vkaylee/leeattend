//! Attendance Reprocess API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use crate::common::helpers;
use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_force_reprocess_attendance_api() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let base_date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    // Use UTC times that result in the SAME DAY for Asia/Ho_Chi_Minh (UTC+7)
    // 01:00 UTC = 08:00 Local
    // 10:00 UTC = 17:00 Local
    let start_time = Utc.with_ymd_and_hms(2026, 1, 14, 1, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2026, 1, 14, 10, 0, 0).unwrap();

    // 1. Create raw logs WITHOUT processing them (bypass service)
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let logs = vec![start_time, end_time];
    for punch_time in logs {
        let raw_log = RawAttendanceLog::new(
            Uuid::new_v4(),
            ws_id,
            emp_employment_id,
            punch_time,
            LogSource::Manual,
            None,
            None,
            -1,
            -1,
            Utc::now(),
        );
        helpers::create_raw_log(&diesel_pools, &raw_log)
            .await
            .unwrap();
    }

    let time_entry_repo =
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());

    let start_dt = base_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_dt = base_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    use apiserver::repositories::time_entry_repository::TimeEntryRepository;

    // 2. Verify time_entries are initially empty
    let entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_employment_id,
            start_dt,
            end_dt,
        )
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
                    "/api/v1/workspaces/{ws_id}/attendance/reprocess/{emp_id}?start_date={base_date}&end_date={base_date}"
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
    let entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_employment_id,
            start_dt,
            end_dt,
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 1, "One time entry should have been created");
    assert_eq!(entries[0].punch_in_time(), start_time);
    assert_eq!(entries[0].punch_out_time(), Some(end_time));
}

#[tokio::test]
async fn test_force_reprocess_all_attendance_api() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_1 = factory.employee().with_workspace(ws_id).create().await;
    let emp_id_2 = factory.employee().with_workspace(ws_id).create().await;
    let emp_employment_id_1: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id_1
    );
    let emp_employment_id_2: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id_2
    );

    let base_date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    // Use UTC times that result in the SAME DAY for Asia/Ho_Chi_Minh (UTC+7)
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone());
    let start_time = Utc.with_ymd_and_hms(2026, 1, 14, 1, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2026, 1, 14, 10, 0, 0).unwrap();

    // 1. Create raw logs WITHOUT processing them (bypass service) for BOTH employees
    for (emp_id, emp_employment_id) in [
        (emp_id_1, emp_employment_id_1),
        (emp_id_2, emp_employment_id_2),
    ] {
        let logs = vec![start_time, end_time];
        for punch_time in logs {
            let raw_log = RawAttendanceLog::new(
                Uuid::new_v4(),
                ws_id,
                emp_employment_id,
                punch_time,
                LogSource::Manual,
                None,
                None,
                -1,
                -1,
                Utc::now(),
            );
            helpers::create_raw_log(&diesel_pools, &raw_log)
                .await
                .unwrap();
        }
    }

    // 2. Verify time_entries are initially empty
    let time_entry_repo =
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());
    use apiserver::repositories::time_entry_repository::TimeEntryRepository;
    let start_dt = base_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_dt = base_date.and_hms_opt(23, 59, 59).unwrap().and_utc();
    for (emp_id, emp_employment_id) in [
        (emp_id_1, emp_employment_id_1),
        (emp_id_2, emp_employment_id_2),
    ] {
        let entries = time_entry_repo
            .get_employment_time_entries(
                &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
                emp_employment_id,
                start_dt,
                end_dt,
            )
            .await
            .unwrap();
        assert!(
            entries.is_empty(),
            "Entries should be empty before bulk reprocessing"
        );
    }

    // 3. Call the bulk reprocess endpoint
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/attendance/reprocess-all?start_date={base_date}&end_date={base_date}"
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
    let redis = _state.infra().redis().unwrap();
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
    let queue_repo: Arc<dyn apiserver::repositories::queue_repository::QueueRepository> = Arc::new(
        apiserver::repositories::queue_repository::RedisQueueRepository::new(redis.clone()),
    );
    let outbox_repo = Arc::new(apiserver::repositories::DieselOutboxJobRepository::new(
        diesel_pools.clone(),
    ));
    let outbox_service = Arc::new(apiserver::services::OutboxJobService::new(outbox_repo));
    let coordinator = apiserver::services::ReportCoordinator::new(
        queue_repo.clone(),
        _state.employees().repo().clone(),
        _state.core().workspace_repo().clone(),
        outbox_service,
    );
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
            &queue_repo,
            &job,
            &deps,
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
    for (emp_id, emp_employment_id) in [
        (emp_id_1, emp_employment_id_1),
        (emp_id_2, emp_employment_id_2),
    ] {
        let entries = time_entry_repo
            .get_employment_time_entries(
                &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
                emp_employment_id,
                start_dt,
                end_dt,
            )
            .await
            .unwrap();

        assert_eq!(
            entries.len(),
            1,
            "One time entry should have been created for employee"
        );
        assert_eq!(entries[0].punch_in_time(), start_time);
        assert_eq!(entries[0].punch_out_time(), Some(end_time));
    }
}

#[tokio::test]
async fn test_force_reprocess_attendance_no_location_fallback_api() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    // Create workspace with a specific timezone
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    // Set primary_location_id to NULL so that get_employee_timezone methods fall back to workspace timezone
    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE employments SET primary_location_id = NULL WHERE id = $1",
        diesel::sql_types::Uuid,
        emp_employment_id
    );

    let base_date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
    // Use UTC times that result in the SAME DAY for Asia/Ho_Chi_Minh (UTC+7)
    // 01:00 UTC = 08:00 Local
    // 10:00 UTC = 17:00 Local
    let start_time = Utc.with_ymd_and_hms(2026, 5, 1, 1, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2026, 5, 1, 10, 0, 0).unwrap();

    // 1. Create raw logs WITHOUT processing them (bypass service)
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let logs = vec![start_time, end_time];
    for punch_time in logs {
        let raw_log = RawAttendanceLog::new(
            Uuid::new_v4(),
            ws_id,
            emp_employment_id,
            punch_time,
            LogSource::Manual,
            None,
            None,
            -1,
            -1,
            Utc::now(),
        );
        helpers::create_raw_log(&diesel_pools, &raw_log)
            .await
            .unwrap();
    }

    let time_entry_repo =
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());

    let start_dt = base_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_dt = base_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    use apiserver::repositories::time_entry_repository::TimeEntryRepository;

    // 2. Verify time_entries are initially empty
    let entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_employment_id,
            start_dt,
            end_dt,
        )
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
                    "/api/v1/workspaces/{ws_id}/attendance/reprocess/{emp_id}?start_date={base_date}&end_date={base_date}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Verify time_entries are now created correctly (verifying fallback worked)
    let entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_employment_id,
            start_dt,
            end_dt,
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 1, "One time entry should have been created");
    assert_eq!(entries[0].punch_in_time(), start_time);
    assert_eq!(entries[0].punch_out_time(), Some(end_time));
}

/// Regression test for the "4 sessions" bug.
///
/// Scenario: scheduled shift 07:30-16:30 with break 11:30-12:30, Asia/Ho_Chi_Minh
/// timezone. Employee records five raw punches in the SAME LOCAL DAY: one check-in
/// at 07:22, three duplicate device retransmits at 15:06, and one check-out at
/// 16:45. Two concurrent HTTP POST /reprocess requests for the same (employee,
/// day) simulate the race (e.g. webhook + manual click before React isPending
/// flips). Before the advisory-lock fix `replace_time_entries` allowed the two
/// transactions to interleave their DELETE+INSERT, producing 4 rows (2 per
/// transaction) in `time_entries`. After the fix the lock serialises them and the
/// day must end up with exactly 2 session rows (morning + afternoon).
#[tokio::test]
async fn test_concurrent_reprocess_does_not_double_insert_sessions() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id
    );

    let base_date = NaiveDate::from_ymd_opt(2026, 7, 3).unwrap();

    // 1. Create a shift 07:30-16:30 with break 11:30-12:30.
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let shift_id: Uuid = crate::test_query_scalar_uuid!(
        &diesel_pools.api,
        "INSERT INTO shifts (workspace_id, name) VALUES ($1, $2) RETURNING id",
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Text,
        "0730-1630 with break"
    );
    crate::test_execute!(
        &diesel_pools.api,
        "INSERT INTO shift_default_intervals \
            (shift_id, sort_order, start_time, end_time, \
             break_duration_minutes, break_start, break_end, \
             is_break_auto_deduct, break_is_overtime, \
             early_punch_in_grace_period_minutes, allow_early_ot) \
         VALUES ($1, 0, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        diesel::sql_types::Uuid,                      shift_id,
        diesel::sql_types::Time,                      NaiveTime::from_hms_opt(7, 30, 0).unwrap(),
        diesel::sql_types::Time,                      NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
        diesel::sql_types::Integer,                   60,
        diesel::sql_types::Time,                      NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
        diesel::sql_types::Time,                      NaiveTime::from_hms_opt(12, 30, 0).unwrap(),
        diesel::sql_types::Bool,                      false,
        diesel::sql_types::Bool,                      false,
        diesel::sql_types::Integer,                   0,
        diesel::sql_types::Bool,                      false
    );

    let employment_id_for_assign = emp_employment_id;
    crate::test_execute!(
        &diesel_pools.api,
        "INSERT INTO employee_shifts (employment_id, shift_id, start_date) VALUES ($1, $2, $3)",
        diesel::sql_types::Uuid,  employment_id_for_assign,
        diesel::sql_types::Uuid,  shift_id,
        diesel::sql_types::Date,  base_date
    );

    // 2. Create 5 raw logs WITHOUT processing through the service:
    //    check-in 07:22, 3x duplicate 15:06, check-out 16:45 (all LOCAL).
    //    Asia/Ho_Chi_Minh is UTC+7, so LOCAL time = UTC+7 -> UTC = LOCAL - 7h.
    let local_punches = [
        (7, 22),    // 07:22 local
        (15, 6),    // 15:06 duplicate #1
        (15, 6),    // 15:06 duplicate #2
        (15, 6),    // 15:06 duplicate #3
        (16, 45),   // 16:45 local
    ];
    for (h, m) in local_punches {
        let local_dt = base_date.and_hms_opt(h, m, 0).unwrap();
        let punch_utc = local_dt - chrono::Duration::hours(7); // UTC+7 -> UTC
        let raw_log = RawAttendanceLog::new(
            Uuid::new_v4(),
            ws_id,
            emp_employment_id,
            punch_utc.and_utc(),
            LogSource::Manual,
            None,
            None,
            -1,
            -1,
            Utc::now(),
        );
        helpers::create_raw_log(&diesel_pools, &raw_log)
            .await
            .unwrap();
    }

    let time_entry_repo =
        apiserver::repositories::DieselTimeEntryRepository::new(diesel_pools.clone());

    let start_dt = base_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_dt = base_date.and_hms_opt(23, 59, 59).unwrap().and_utc();

    use apiserver::repositories::time_entry_repository::TimeEntryRepository;

    // Verify time_entries are initially empty.
    let entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_employment_id,
            start_dt,
            end_dt,
        )
        .await
        .unwrap();
    assert!(
        entries.is_empty(),
        "Entries should be empty before reprocessing"
    );

    // 3. Fire two HTTP reprocess requests for the SAME (employee, day) at the
    //    same time. This is the race that previously quadrupled the sessions.
    let make_request = {
        let app = app.clone();
        let ws_id = ws_id.to_string();
        let user_id = user_id.to_string();
        let emp_id_str = emp_id.to_string();
        let base_date_str = base_date.to_string();
        move || {
            let app = app.clone();
            let ws_id = ws_id.clone();
            let user_id = user_id.clone();
            let emp_id_str = emp_id_str.clone();
            let base_date_str = base_date_str.clone();
            async move {
                app.oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!(
                            "/api/v1/workspaces/{ws_id}/attendance/reprocess/{emp_id_str}?start_date={base_date_str}&end_date={base_date_str}"
                        ))
                        .header("X-Workspace-Id", ws_id)
                        .header("X-User-Id", user_id)
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
                .status()
            }
        }
    };

    let (status1, status2) = tokio::join!(make_request(), make_request());
    assert_eq!(status1, StatusCode::OK, "First reprocess must succeed");
    assert_eq!(status2, StatusCode::OK, "Second reprocess must succeed");

    // 4. Verify exactly 2 time entries were written: morning (07:22 -> 11:30)
    //    and afternoon (12:30 -> 16:45). No quadrupling.
    let entries = time_entry_repo
        .get_employment_time_entries(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_employment_id,
            start_dt,
            end_dt,
        )
        .await
        .unwrap();

    assert_eq!(
        entries.len(),
        2,
        "Concurrent reprocess must produce exactly 2 sessions (morning + afternoon), got {}",
        entries.len()
    );

    // 5. Verify the two sessions are split on the configured break:
    //    morning ends no later than break_start (11:30) and afternoon starts
    //    no earlier than break_end (12:30).
    let mut sorted = entries;
    sorted.sort_by_key(|e| e.punch_in_time());

    let morning_in_local = sorted[0].punch_in_time().with_timezone(&chrono_tz::Asia::Ho_Chi_Minh);
    let afternoon_out_local = sorted[1]
        .punch_out_time()
        .unwrap()
        .with_timezone(&chrono_tz::Asia::Ho_Chi_Minh);

    assert_eq!(
        (morning_in_local.hour(), morning_in_local.minute()),
        (7, 22),
        "Morning session must start at 07:22 local"
    );
    assert_eq!(
        (afternoon_out_local.hour(), afternoon_out_local.minute()),
        (16, 45),
        "Afternoon session must end at 16:45 local"
    );

    let morning_end = sorted[0].punch_out_time().unwrap();
    let afternoon_start = sorted[1].punch_in_time();

    let morning_end_local = morning_end.with_timezone(&chrono_tz::Asia::Ho_Chi_Minh);
    let afternoon_start_local = afternoon_start.with_timezone(&chrono_tz::Asia::Ho_Chi_Minh);

    assert!(
        (morning_end_local.hour(), morning_end_local.minute()) <= (11, 30),
        "Morning session must end before or at break start (11:30), got {:02}:{:02}",
        morning_end_local.hour(),
        morning_end_local.minute()
    );
    assert!(
        (afternoon_start_local.hour(), afternoon_start_local.minute()) >= (12, 30),
        "Afternoon session must start at or after break end (12:30), got {:02}:{:02}",
        afternoon_start_local.hour(),
        afternoon_start_local.minute()
    );

    drop(test_db);
}

