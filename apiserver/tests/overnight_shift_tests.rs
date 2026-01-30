mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::LogSource;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use chrono::{Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use uuid::Uuid;

async fn spawn_app() -> (String, TestDatabase) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let agent_manager = AgentManager::new();
    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository { pool: pool.clone() });
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo,
        workspace_repo,
        agent_service,
        jwt_secret: "test_secret_123".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), test_db)
}

#[tokio::test]
async fn test_overnight_shift_pairing() {
    let (_address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let user_id = factory.user().create().await;
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;
    let employee_id = factory.employee().with_workspace(ws_id).create().await;

    // 1. Create Overnight Shift: 22:00 to 06:00
    let shift_id = factory
        .shift()
        .with_workspace(ws_id)
        .with_name("Night Shift")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

    // Assign the shift manually to employee_shifts table
    sqlx::query!(
        "INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(), employee_id, shift_id, shift_date
    ).execute(&pool).await.unwrap();

    // 2. Insert Raw Logs
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();

    // Punch In: Jan 15, 21:55 Local
    let punch_in_utc = tz
        .from_local_datetime(&shift_date.and_hms_opt(21, 55, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // Punch Out: Jan 16, 06:05 Local (Next Day)
    let next_day = shift_date + Duration::days(1);
    let punch_out_utc = tz
        .from_local_datetime(&next_day.and_hms_opt(6, 5, 0).unwrap())
        .single()
        .unwrap()
        .with_timezone(&Utc);

    // Insert manually into raw logs
    sqlx::query!(
        "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(), ws_id, employee_id, punch_in_utc, LogSource::Device as LogSource
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO raw_attendance_logs (id, workspace_id, employee_id, punch_time, source) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(), ws_id, employee_id, punch_out_utc, LogSource::Device as LogSource
    ).execute(&pool).await.unwrap();

    // 3. Trigger Pairing manually
    apiserver::services::pairing_service::PairingService::reprocess_daily_attendance(
        &pool,
        employee_id,
        shift_date,
        tz,
        "anchor_first",
    )
    .await
    .unwrap();

    // 4. Verify Time Entry
    let entry = sqlx::query!(
        "SELECT punch_in_time, punch_out_time, duration_minutes FROM time_entries WHERE employee_id = $1",
        employee_id
    ).fetch_one(&pool).await.unwrap();

    assert_eq!(entry.punch_in_time, punch_in_utc);
    assert_eq!(entry.punch_out_time, Some(punch_out_utc));
    // 21:55 to 06:05 = 8 hours 10 minutes = 490 minutes
    assert_eq!(entry.duration_minutes, Some(490));

    factory.cleanup().await;
}
