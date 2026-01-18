mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::{DailyAttendanceReport, LeaveStatus};
use apiserver::AppState;
use chrono::{Duration, NaiveTime, Utc};
use common::factory::TestFactory;
use common::db::TestDatabase;
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
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let app = create_app(state);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    (format!("http://127.0.0.1:{}", port), test_db)
}


#[tokio::test]
async fn test_get_daily_attendance_report_with_break_times() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    let test_date = Utc::now().date_naive();
    let scheduled_start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let scheduled_end = NaiveTime::from_hms_opt(17, 0, 0).unwrap();
    let break_start = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let break_end = NaiveTime::from_hms_opt(13, 0, 0).unwrap();
    let break_duration_minutes = 60;

    // Create a shift with explicit break times
    let shift_id = sqlx::query_scalar!(
        "INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, break_start, break_end, is_break_auto_deduct, break_is_overtime, early_punch_in_grace_period_minutes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING id",
        ws_id,
        "Day Shift with Break",
        scheduled_start,
        scheduled_end,
        break_duration_minutes,
        break_start,
        break_end,
        false, // is_break_auto_deduct
        false, // break_is_overtime
        0
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Assign the shift to the employee
    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date, end_date) VALUES ($1, $2, $3, $4)",
        employee_id,
        shift_id,
        test_date,
        test_date
    )
    .execute(&pool)
    .await
    .unwrap();

    // Punch in and out
    let punch_in_time = test_date
        .and_time(scheduled_start)
        .and_local_timezone(Utc)
        .unwrap();
    let punch_out_time = test_date
        .and_time(scheduled_end)
        .and_local_timezone(Utc)
        .unwrap();

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time)
        .with_punch_out(punch_out_time)
        .create()
        .await;

    // Act
    let res = client
        .get(&format!(
            "{}/api/workspaces/{}/reports/attendance?start_date={}&end_date={}&type=daily",
            address, ws_id, test_date, test_date
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string()) // Dev mode auth
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(res.status(), 200);
    let report: Vec<DailyAttendanceReport> = res.json().await.unwrap();

    assert_eq!(report.len(), 1);
    let entry = &report[0];

    assert_eq!(entry.employee_id, employee_id);
    assert_eq!(entry.date, test_date);
    assert_eq!(entry.scheduled_start, Some(scheduled_start));
    assert_eq!(entry.scheduled_end, Some(scheduled_end));
    assert_eq!(entry.break_duration_minutes, Some(break_duration_minutes));
    assert_eq!(entry.break_start, Some(break_start));
    assert_eq!(entry.break_end, Some(break_end));
    assert_eq!(entry.is_break_auto_deduct, Some(false));

    factory.cleanup().await;
}

#[tokio::test]
async fn test_get_daily_attendance_report_no_break_auto_deduct() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    let test_date = Utc::now().date_naive() - Duration::days(1);
    let scheduled_start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let scheduled_end = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
    let break_duration_minutes = 30;

    // Create a shift with auto_deduct = true
    let shift_id = sqlx::query_scalar!(
        "INSERT INTO shifts (workspace_id, name, start_time, end_time, break_duration_minutes, is_break_auto_deduct, break_is_overtime, early_punch_in_grace_period_minutes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        ws_id,
        "Shift Auto Deduct Break",
        scheduled_start,
        scheduled_end,
        break_duration_minutes,
        true, // is_break_auto_deduct
        false,
        0
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Assign the shift
    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
        employee_id,
        shift_id,
        test_date
    )
    .execute(&pool)
    .await
    .unwrap();

    // Punch in and out
    let punch_in_time = test_date
        .and_time(scheduled_start)
        .and_local_timezone(Utc)
        .unwrap();
    let punch_out_time = test_date
        .and_time(scheduled_end)
        .and_local_timezone(Utc)
        .unwrap();

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_time)
        .with_punch_out(punch_out_time)
        .create()
        .await;

    // Act
    let res = client
        .get(&format!(
            "{}/api/workspaces/{}/reports/attendance?start_date={}&end_date={}&type=daily",
            address, ws_id, test_date, test_date
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(res.status(), 200);
    let report: Vec<DailyAttendanceReport> = res.json().await.unwrap();

    assert_eq!(report.len(), 1);
    let entry = &report[0];

    assert_eq!(entry.employee_id, employee_id);
    assert_eq!(entry.date, test_date);
    assert_eq!(entry.break_duration_minutes, Some(break_duration_minutes));
    assert_eq!(entry.break_start, None);
    assert_eq!(entry.break_end, None);
    assert_eq!(entry.is_break_auto_deduct, Some(true));

    factory.cleanup().await;
}

#[tokio::test]
async fn test_get_workspace_attendance_summary() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Create a time entry
    factory.time_entry()
        .with_employee(employee_id)
        .with_punch_in(Utc::now() - Duration::hours(8))
        .with_punch_out(Utc::now())
        .create()
        .await;

    // Create an approved leave request
    let start_date = Utc::now().date_naive();
    let end_date = start_date;
    sqlx::query!(
        "INSERT INTO leave_requests (id, employee_id, leave_type, start_date, end_date, status) VALUES ($1, $2, $3, $4, $5, $6)",
        Uuid::new_v4(),
        employee_id,
        "vacation",
        start_date,
        end_date,
        LeaveStatus::Approved as LeaveStatus
    )
    .execute(&pool)
    .await
    .unwrap();

    let start_date_str = (Utc::now() - Duration::days(1)).date_naive().to_string();
    let end_date_str = (Utc::now() + Duration::days(1)).date_naive().to_string();

    // Act
    let res = client
        .get(&format!(
            "{}/api/workspaces/{}/reports/attendance?start_date={}&end_date={}&type=summary",
            address, ws_id, start_date_str, end_date_str
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

        // Assert

        assert_eq!(res.status(), 200);

        let summary: Vec<apiserver::models::AttendanceSummary> = res.json().await.unwrap();

        assert_eq!(summary.len(), 1);

        assert!(summary[0].total_work_units >= 480);

        assert_eq!(summary[0].total_leave_days, 1);

    }

    