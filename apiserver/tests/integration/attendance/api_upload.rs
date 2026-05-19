//! Attendance Upload API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use crate::common::helpers;
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
async fn test_upload_attendance_csv_api() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id_number = "API-TEST-CSV-001";
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    let now = chrono::Utc::now();
    let csv_date = now.format("%d/%m/%Y %H:%M").to_string();
    let csv_content =
        format!("employee_code,full_name,punch_time\n{emp_id_number},Test User,{csv_date}");
    let (content_type, body) = create_multipart_body("test.csv", &csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/attendance/upload"))
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
        sleep(Duration::from_secs(1)).await;
        let job_opt = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
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
    assert!(completed, "CSV Job did not complete");

    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&_test_db.url());
    let entries = helpers::get_employee_time_entries(
        &diesel_pools,
        ws_id,
        emp_id,
        chrono::Utc::now() - chrono::Duration::days(30),
        chrono::Utc::now() + chrono::Duration::days(30),
    )
    .await
    .unwrap();
    assert_eq!(entries.len(), 1);
}
