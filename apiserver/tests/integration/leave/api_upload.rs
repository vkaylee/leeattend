//! Leave Upload API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::ImportJobStatus;
use apiserver::repositories::leave_request_repository::LeaveRequestRepository;
use apiserver::AppState;
use axum::http::StatusCode;
use chrono::Datelike;
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
async fn test_upload_leave_csv_api() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    let emp_id_number = "1";
    let emp_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number(emp_id_number)
        .create()
        .await;

    // Give employee some leave balance so submit won't fail
    let leave_balance_id = factory
        .leave_balance()
        .with_employee(emp_id)
        .with_entitlement(10)
        .create()
        .await;

    let leave_type_id: Uuid =
        sqlx::query_scalar("SELECT leave_type_id FROM leave_balances WHERE id = $1")
            .bind(leave_balance_id)
            .fetch_one(&factory.pool)
            .await
            .unwrap();

    let start_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let end_date = start_date.clone();
    let csv_content = format!(
        "employee_code,leave_type_code,start_date,end_date,reason,duration_type\n\
        {emp_id_number},vacation,{start_date},{end_date},Vacation,full_day"
    );

    let (content_type, body) = create_multipart_body("leaves.csv", &csv_content);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/leave/upload-excel"))
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

    // The background worker is not running in the setup_app_tower context, so we process it manually.
    let import_service =
        apiserver::services::leave_import_service::LeaveImportService::new(diesel_pools.clone());

    let job_opt = _state
        .attendance
        .import_repo
        .get_job_by_id(job_uuid)
        .await
        .unwrap()
        .unwrap();

    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        import_service.process_job(job_uuid, ws_id, user_id, &job_opt.file_path.unwrap(), true),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let job_opt = _state
        .attendance
        .import_repo
        .get_job_by_id(job_uuid)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        job_opt.status,
        ImportJobStatus::Completed,
        "CSV Job did not complete. error_log: {:?}",
        job_opt.error_log
    );

    let lr_repo = apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone());
    let requests = lr_repo
        .get_employee_leave_requests(ws_id, emp_id)
        .await
        .unwrap();

    assert_eq!(requests.len(), 1, "Leave request was not created");
    assert_eq!(requests[0].leave_type_id, leave_type_id);
}
