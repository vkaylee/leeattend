mod common;

use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{TimeZone, Utc, NaiveDate};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn test_force_reprocess_attendance_api() {
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
    });

    let base_date = NaiveDate::from_ymd_opt(2026, 1, 14).unwrap();
    // Use UTC times that result in the SAME DAY for Asia/Ho_Chi_Minh (UTC+7)
    // 01:00 UTC = 08:00 Local
    // 10:00 UTC = 17:00 Local
    let start_time = Utc.with_ymd_and_hms(2026, 1, 14, 1, 0, 0).unwrap();
    let end_time = Utc.with_ymd_and_hms(2026, 1, 14, 10, 0, 0).unwrap();

    // 1. Create raw logs WITHOUT processing them
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
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &raw_log)
            .await
            .unwrap();
    }

    // 2. Verify time_entries are initially empty
    let entries = sqlx::query!(
        "SELECT id FROM time_entries WHERE employee_id = $1",
        emp_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(entries.is_empty(), "Entries should be empty before reprocessing");

    // 3. Call the reprocess endpoint
    let app = apiserver::create_app(state);
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
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(entries.len(), 1, "One time entry should have been created");
    assert_eq!(entries[0].punch_in_time, start_time);
    assert_eq!(entries[0].punch_out_time, Some(end_time));
}
