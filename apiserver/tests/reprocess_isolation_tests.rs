mod common;

use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{TimeZone, Utc, NaiveDate};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_reprocess_should_not_delete_next_day_entries() {
    let _test_db = TestDatabase::new().await;
    let pool = _test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Initialize AppState
    let agent_manager = apiserver::agent_manager::AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));
    let jwt_secret = "dev_secret_key".to_string();

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret,
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
    });

    // Day 1: 2026-01-14
    let day1 = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    let day1_in = Utc.with_ymd_and_hms(2026, 1, 14, 8, 0, 0).unwrap();
    let day1_out = Utc.with_ymd_and_hms(2026, 1, 14, 17, 0, 0).unwrap();

    // Day 2: 2026-01-15 (Morning log)
    let _day2 = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let day2_in = Utc.with_ymd_and_hms(2026, 1, 15, 9, 0, 0).unwrap();
    let day2_out = Utc.with_ymd_and_hms(2026, 1, 15, 18, 0, 0).unwrap();

    // 1. Create logs for BOTH days
    let logs = vec![day1_in, day1_out, day2_in, day2_out];
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
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &raw_log)
            .await
            .unwrap();
    }

    // 2. Pre-populate Day 2 entry (to simulate existing data)
    sqlx::query!(
        "INSERT INTO time_entries (id, employee_id, workspace_id, punch_in_time, punch_out_time, duration_minutes, entry_type) 
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        Uuid::new_v4(),
        emp_id,
        ws_id,
        day2_in,
        Some(day2_out),
        540,
        "work"
    )
    .execute(&pool)
    .await
    .unwrap();

    // 3. Call the reprocess endpoint for Day 1 ONLY
    let app = apiserver::create_app(state);
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{}/attendance/reprocess/{}?start_date={}&end_date={}",
                    ws_id, emp_id, day1, day1
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Verify Day 2 entry still exists
    let day2_entries = sqlx::query!(
        "SELECT id FROM time_entries WHERE employee_id = $1 AND punch_in_time = $2",
        emp_id,
        day2_in
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(day2_entries.len(), 1, "Day 2 entry should NOT have been deleted when reprocessing Day 1");
    
    // 5. Verify Day 1 entry was created
    let day1_entries = sqlx::query!(
        "SELECT id FROM time_entries WHERE employee_id = $1 AND punch_in_time = $2",
        emp_id,
        day1_in
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(day1_entries.len(), 1, "Day 1 entry should have been created");
}
