//! Employee bulk import integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::ImportJobStatus;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceExt;
use uuid::Uuid;

fn create_multipart_body(filename: &str, content: &str) -> (String, Vec<u8>) {
    let boundary = "boundary123";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(content.as_bytes());
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    (format!("multipart/form-data; boundary={boundary}"), body)
}

#[tokio::test]
async fn test_upload_employee_csv_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // CSV with 2 employees
    let csv_content =
        "ID,FullName,HireDate\nEMP001,John Doe,2026-01-01\nEMP002,Jane Smith,2026-01-15";
    let (content_type, body) = create_multipart_body("employees.csv", csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/employees/upload"))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Employee CSV Upload failed"
    );
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
    let emp1 = apiserver::repositories::employee_repository::get_employee_by_id_number(
        &factory.pool,
        "EMP001",
        ws_id,
    )
    .await
    .unwrap();
    assert!(emp1.is_some());
    let emp1 = emp1.unwrap();
    assert_eq!(emp1.full_name, "John Doe");

    let emp2 = apiserver::repositories::employee_repository::get_employee_by_id_number(
        &factory.pool,
        "EMP002",
        ws_id,
    )
    .await
    .unwrap();
    assert!(emp2.is_some());
}

#[tokio::test]
async fn test_upload_employee_duplicate_id_skips() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Create existing employee with ID "DUP001"
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("DUP001")
        .with_full_name("Old Name")
        .create()
        .await;

    // CSV with same ID "DUP001" (different name) and a new employee "NEW002"
    let csv_content =
        "ID,FullName,HireDate\nDUP001,New John,2026-01-01\nNEW002,Alice Wonder,2026-02-01";
    let (content_type, body) = create_multipart_body("employees.csv", csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/employees/upload"))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let job_uuid = Uuid::parse_str(json["job_id"].as_str().unwrap()).unwrap();

    let mut completed = false;

    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job =
            apiserver::repositories::import_job_repository::get_job_by_id(&factory.pool, job_uuid)
                .await
                .unwrap()
                .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            assert_eq!(job.processed_rows, 1, "Should only process 1 new row");
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed unexpectedly: {:?}", job.error_log);
        }
    }

    assert!(completed, "Job should have completed");

    // Verify "DUP001" is NOT updated
    let emp_dup = apiserver::repositories::employee_repository::get_employee_by_id_number(
        &factory.pool,
        "DUP001",
        ws_id,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(emp_dup.full_name, "Old Name");

    // Verify "NEW002" IS created
    let emp_new = apiserver::repositories::employee_repository::get_employee_by_id_number(
        &factory.pool,
        "NEW002",
        ws_id,
    )
    .await
    .unwrap();
    assert!(emp_new.is_some());
}
