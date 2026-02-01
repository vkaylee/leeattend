mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::create_app;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use chrono::{Duration, NaiveDate, TimeZone, Utc};
use chrono_tz::Tz;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;

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

    let storage_provider = apiserver::services::create_storage_provider(&storage)


    // Setup test storage



    let storage = apiserver::StorageBackend::Local {



        base_path: std::path::PathBuf::from("./test_storage"),



    };



    let storage_provider = apiserver::services::create_storage_provider(&storage)



        .await



        .expect("Failed to create test storage");




    // Setup test storage
    let storage = apiserver::StorageBackend::Local {
        base_path: std::path::PathBuf::from("./test_storage"),
    };
    let storage_provider = apiserver::services::create_storage_provider(&storage)
        .await
        .expect("Failed to create test storage");

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
        storage,
        storage_provider,
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
async fn test_report_uses_workspace_timezone_boundary() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    // Create workspace with UTC+7 (Asia/Ho_Chi_Minh)
    // 00:00 Local = 17:00 Previous Day UTC
    let ws_id = factory
        .workspace()
        .with_timezone("Asia/Ho_Chi_Minh")
        .create_with_owner(user_id)
        .await;

    let employee_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_user(user_id)
        .create()
        .await;

    // Target Date: 2026-05-10
    let target_date = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();

    // 1. Create a Time Entry at 01:00 AM Local Time on target_date
    // 01:00 AM UTC+7 = 18:00 PM UTC (Previous Day, 2026-05-09)
    let tz: Tz = "Asia/Ho_Chi_Minh".parse().unwrap();
    let punch_in_local = tz
        .from_local_datetime(&target_date.and_hms_opt(1, 0, 0).unwrap())
        .single()
        .unwrap();

    let punch_in_utc = punch_in_local.with_timezone(&Utc);
    let punch_out_utc = punch_in_utc + Duration::minutes(60);

    // Sanity check: ensure it really is previous day in UTC
    assert_eq!(
        punch_in_utc.date_naive(),
        NaiveDate::from_ymd_opt(2026, 5, 9).unwrap()
    );

    factory
        .time_entry()
        .with_employee(employee_id)
        .with_punch_in(punch_in_utc)
        .with_punch_out(punch_out_utc)
        .create()
        .await;

    // 2. Request Report for target_date (Summary)
    // If the server uses UTC, it sees '2026-05-10 00:00:00Z' to '2026-05-10 23:59:59Z'.
    // The entry (May 9th 18:00 UTC) would be EXCLUDED.
    // If the server uses Timezone, it converts target_date to '2026-05-09 17:00:00Z' (Start of May 10th Local).
    // The entry (May 9th 18:00 UTC) should be INCLUDED.

    let res = client
        .get(format!(
            "{}/api/workspaces/{}/reports/attendance?start_date={}&end_date={}&type=summary",
            address, ws_id, target_date, target_date
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    let summary_res: apiserver::models::PaginatedResponse<apiserver::models::AttendanceSummary> =
        res.json().await.unwrap();
    let summary = summary_res.data;

    assert_eq!(summary.len(), 1, "Should return 1 summary row");
    assert_eq!(summary[0].employee_id, employee_id);
    assert!(
        summary[0].total_work_units > 0,
        "Total work units should be > 0 (found {}), proving the entry was included",
        summary[0].total_work_units
    );

    factory.cleanup().await;
}
