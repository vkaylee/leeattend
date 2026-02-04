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

    // Setup test storage

    let app_state = Arc::new(AppState {
        agent_manager,
        db: pool.clone(),
        agent_repo: agent_repo.clone(),
        workspace_repo: workspace_repo.clone(),
        agent_service: agent_service.clone(),
        jwt_secret: "test_secret".to_string(),
        attendance_service: Arc::new(AttendanceLogService::new(pool.clone(), 100, 10, 1000).0),
        redis: None,
        coordinator: None,
        roster_cache: None,
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
async fn test_upload_employee_csv_api() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    
    // CSV with 2 employees
    let csv_content = "ID,FullName,HireDate\nEMP001,John Doe,2026-01-01\nEMP002,Jane Smith,2026-01-15";
    let (content_type, body) = create_multipart_body("employees.csv", csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees/upload", ws_id))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK, "Employee CSV Upload failed");
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
            } else if job.status == ImportJobStatus::Failed {
                panic!("Job failed: {:?}", job.error_log);
            }
        }
    }
    assert!(completed, "Employee CSV Job did not complete");

    // Verify employees created
    let emp1 = apiserver::repositories::employee_repository::get_employee_by_id_number(&factory.pool, "EMP001", ws_id).await.unwrap();
    assert!(emp1.is_some());
    let emp1 = emp1.unwrap();
    assert_eq!(emp1.full_name, "John Doe");
    assert_eq!(emp1.hire_date.to_string(), "2026-01-01");

    let emp2 = apiserver::repositories::employee_repository::get_employee_by_id_number(&factory.pool, "EMP002", ws_id).await.unwrap();
    assert!(emp2.is_some());
}

#[tokio::test]
async fn test_upload_employee_duplicate_id_skips() {
    let (_test_db, factory, app) = setup_app().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    
    // Create existing employee with ID "DUP001" and name "Old Name"
    factory.employee()
        .with_workspace(ws_id)
        .with_employee_id_number("DUP001")
        .with_full_name("Old Name")
        .create().await;
    
    // CSV with same ID "DUP001" (different name) and a new employee "NEW002"
    let csv_content = "ID,FullName,HireDate\nDUP001,New John,2026-01-01\nNEW002,Alice Wonder,2026-02-01";
    let (content_type, body) = create_multipart_body("employees.csv", csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{}/employees/upload", ws_id))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let job_uuid = Uuid::parse_str(json["job_id"].as_str().unwrap()).unwrap();

    let mut completed = false;
    let mut final_log = String::new();
    
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = apiserver::repositories::import_job_repository::get_job_by_id(&factory.pool, job_uuid).await.unwrap().unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            final_log = job.error_log.unwrap_or_default();
            
            // Check that processed_rows reflects only the NEW inserted row (1)
            assert_eq!(job.processed_rows, 1, "Should only process 1 new row");
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed unexpectedly: {:?}", job.error_log);
        }
    }
    
    assert!(completed, "Job should have completed");
    assert!(final_log.contains("Đã bỏ qua 1 nhân viên trùng mã"), "Log should mention skipped rows");

    // Verify "DUP001" is NOT updated
    let emp_dup = apiserver::repositories::employee_repository::get_employee_by_id_number(&factory.pool, "DUP001", ws_id).await.unwrap().unwrap();
    assert_eq!(emp_dup.full_name, "Old Name");

    // Verify "NEW002" IS created
    let emp_new = apiserver::repositories::employee_repository::get_employee_by_id_number(&factory.pool, "NEW002", ws_id).await.unwrap();
    assert!(emp_new.is_some());
}