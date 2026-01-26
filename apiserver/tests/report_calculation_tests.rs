mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::models::{DailyAttendanceReport, PaginatedResponse};
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use chrono::{Local, NaiveTime, TimeZone, Utc};
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
async fn test_report_calculation_grace_period_and_break() {
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
    let start_time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(17, 0, 0).unwrap();

    // Create Shift: 08:00 - 17:00, Break 60m (Auto), Grace 15m
    let shift_id = sqlx::query_scalar!(
        r#"
        INSERT INTO shifts (
            workspace_id, name, start_time, end_time, 
            break_duration_minutes, is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
        RETURNING id
        "#,
        ws_id,
        "Standard Shift",
        start_time,
        end_time,
        60,   // 60 min break
        true, // Auto deduct
        false,
        15 // 15 min grace
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Assign Shift
    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
        employee_id,
        shift_id,
        test_date
    )
    .execute(&pool)
    .await
    .unwrap();

    // Scenario:
    // Punch In: 07:50 (Within 15m grace of 08:00) -> Effective 08:00
    // Punch Out: 17:00
    // Raw Duration: 08:00 -> 17:00 = 9h (540m)
    // Break Deduction: 60m (Auto)
    // Net: 540 - 60 = 480m

    let punch_in = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(7, 50, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(17, 0, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    // Act
    let res = client
        .get(format!(
            "{}/api/workspaces/{}/reports/attendance?start_date={}&end_date={}&type=daily",
            address, ws_id, test_date, test_date
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let report_res: PaginatedResponse<DailyAttendanceReport> = res.json().await.unwrap();
    let report = report_res.data;

    // Assert

    // Assert
    assert_eq!(report.len(), 1);
    // Punch 07:45 -> effective 08:00 (due to 30m grace).
    // Out 17:00. 08:00 to 17:00 = 9h = 540m.
    // Break 60m. Net = 480m.
    // Unit size 1 -> 480 units.
    assert_eq!(
        report[0].work_units, 480,
        "Expected 480 units (8h) after grace period and break deduction"
    );
}

#[tokio::test]
async fn test_report_calculation_rounding() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;

    // Workspace with 15m Floor Rounding
    let ws_id = sqlx::query_scalar!(
        "INSERT INTO workspaces (id, name, work_unit_minutes, rounding_policy) VALUES ($1, $2, $3, $4) RETURNING id",
        Uuid::new_v4(),
        "Rounding WS",
        15,
        "floor"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Add owner
    sqlx::query(
        "INSERT INTO workspace_members (workspace_id, user_id, role) VALUES ($1, $2, 'owner')",
    )
    .bind(ws_id)
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();

    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    let test_date = Utc::now().date_naive();
    let start_time = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let end_time = NaiveTime::from_hms_opt(18, 0, 0).unwrap();

    // Create Shift: 09:00 - 18:00, Break 0m
    let shift_id = sqlx::query_scalar!(
        r#"
        INSERT INTO shifts (
            workspace_id, name, start_time, end_time, 
            break_duration_minutes, is_break_auto_deduct, break_is_overtime, 
            early_punch_in_grace_period_minutes
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
        RETURNING id
        "#,
        ws_id,
        "Simple Shift",
        start_time,
        end_time,
        0,
        false,
        false,
        0
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO employee_shifts (employee_id, shift_id, start_date) VALUES ($1, $2, $3)",
        employee_id,
        shift_id,
        test_date
    )
    .execute(&pool)
    .await
    .unwrap();

    // Scenario:
    // Worked 9h 14m (554m)
    // Rounding: Floor 15m -> 540m (9h)

    let punch_in = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(9, 0, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);
    let punch_out = Local
        .from_local_datetime(&test_date.and_time(NaiveTime::from_hms_opt(18, 14, 0).unwrap()))
        .single()
        .unwrap()
        .with_timezone(&Utc);

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .create()
        .await;

    // Act
    let res = client
        .get(format!(
            "{}/api/workspaces/{}/reports/attendance?start_date={}&end_date={}&type=daily",
            address, ws_id, test_date, test_date
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let report_res: PaginatedResponse<DailyAttendanceReport> = res.json().await.unwrap();
    let report = report_res.data;

    // Assert

    assert_eq!(report.len(), 1);
    // Work 09:00 to 18:14 = 9h14m = 554m.
    // Break 0m. Net = 554m.
    // Rounding 15m Floor: 554 -> 540m.
    // 540 / 15 = 36 units.
    assert_eq!(
        report[0].work_units, 36,
        "Expected 36 units after floor rounding (554m -> 540m / 15)"
    );
}
