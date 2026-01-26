mod common;

use apiserver::agent_manager::AgentManager;
use apiserver::models::ImportJobStatus;
use apiserver::services::AttendanceLogService;
use apiserver::AppState;
use axum::http::StatusCode;
use common::db::TestDatabase;
use common::factory::TestFactory;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_app() -> (TestDatabase, TestFactory, axum::Router) {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool().clone();
    let factory = TestFactory::new(pool.clone());

    let agent_repo = Arc::new(apiserver::repositories::SqlxAgentRepository::new(
        pool.clone(),
    ));
    let workspace_repo =
        Arc::new(apiserver::repositories::SqlxWorkspaceRepository { pool: pool.clone() });
    let agent_manager = AgentManager::new();
    let agent_service = Arc::new(apiserver::services::AgentService::new(
        agent_repo.clone(),
        agent_manager.clone(),
    ));

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
    });

    let app = apiserver::create_app(app_state);
    (test_db, factory, app)
}

fn create_multipart_body(filename: &str, content: &str) -> (String, Vec<u8>) {
    let boundary = "boundary123";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n\r\n",
            filename
        )
        .as_bytes(),
    );
    body.extend_from_slice(content.as_bytes());
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    (format!("multipart/form-data; boundary={}", boundary), body)
}

#[tokio::test]
async fn test_upload_attendance_csv_api() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_number = "API-TEST-CSV-001";
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let csv_content = format!(
        "empNo,empName,empDt\n{},Test User,11/01/2026 14:00",
        emp_id_number
    );
    let (content_type, body) = create_multipart_body("test.csv", &csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/upload", ws_id))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "CSV Upload failed");
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let job_id = json["job_id"].as_str().expect("job_id missing");
    let job_uuid = Uuid::parse_str(job_id).unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job_opt =
            apiserver::repositories::import_job_repository::get_job_by_id(&factory.pool, job_uuid)
                .await
                .unwrap();
        if let Some(job) = job_opt {
            if job.status == ImportJobStatus::Completed {
                completed = true;
                break;
            }
        }
    }
    assert!(completed, "CSV Job did not complete");

    let entries = apiserver::repositories::time_entry_repository::get_employee_time_entries(
        &factory.pool,
        emp_id,
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now() + chrono::Duration::days(30),
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
}

#[tokio::test]
async fn test_upload_attendance_xls_fallback_api() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_number = "API-TEST-XLS-001";
    let _emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let csv_content = format!(
        "empNo,empName,empDt\n{},Xls User,11/01/2026 15:30",
        emp_id_number
    );
    let (content_type, body) = create_multipart_body("test.xls", &csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/attendance/upload", ws_id))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "XLS Upload failed");
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let job_id = json["job_id"].as_str().unwrap();
    let job_uuid = Uuid::parse_str(job_id).unwrap();

    let mut success = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job =
            apiserver::repositories::import_job_repository::get_job_by_id(&factory.pool, job_uuid)
                .await
                .unwrap()
                .unwrap();
        if job.status == ImportJobStatus::Completed {
            success = true;
            break;
        }
    }
    assert!(success, "XLS Fallback Job did not complete");
}
