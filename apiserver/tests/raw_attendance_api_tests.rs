mod common;

use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{Duration, TimeZone, Utc};
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_test_context() -> (
    TestDatabase,
    sqlx::PgPool,
    TestFactory,
    Arc<AppState>,
    Uuid,
    Uuid,
    Uuid,
    chrono::DateTime<Utc>,
) {
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
        redis: None,
    });

    let base_time = Utc.with_ymd_and_hms(2026, 1, 14, 8, 0, 0).unwrap();

    for i in 0..3 {
        let raw_log = RawAttendanceLog {
            id: Uuid::new_v4(),
            workspace_id: ws_id,
            employee_id: emp_id,
            punch_time: base_time + Duration::hours(i as i64),
            source: LogSource::Manual,
            device_id: None,
            raw_data: None,
            created_at: Utc::now(),
        };
        apiserver::repositories::raw_attendance_log_repository::create_raw_log(&pool, &raw_log)
            .await
            .unwrap();
    }

    (
        _test_db, pool, factory, state, ws_id, emp_id, user_id, base_time,
    )
}

#[tokio::test]
async fn test_get_raw_attendance_logs() {
    let (_db, _pool, _factory, state, ws_id, _emp_id, user_id, _base_time) =
        setup_test_context().await;
    let app = apiserver::create_app(state);

    let res = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/workspaces/{}/attendance/raw-logs", ws_id))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    let body = axum::body::to_bytes(res.into_body(), 1000000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["data"].as_array().unwrap().len() >= 3);
    assert_eq!(json["total"], 3);
}
