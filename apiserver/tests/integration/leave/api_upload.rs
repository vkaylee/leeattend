//! Leave Upload API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::ImportJobStatus;
use apiserver::repositories::leave_request_repository::LeaveRequestRepository;
use apiserver::AppState;
use axum::http::StatusCode;
use calamine::Reader;
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

fn create_multipart_body_binary(filename: &str, content: &[u8]) -> (String, Vec<u8>) {
    let boundary = "boundary123";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\r\n")
            .as_bytes(),
    );
    body.extend_from_slice(content);
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

    let leave_type_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT leave_type_id AS id FROM leave_balances WHERE id = $1",
        diesel::sql_types::Uuid,
        leave_balance_id
    );

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
                .uri(format!("/api/v1/workspaces/{ws_id}/leave/upload-excel"))
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
    let import_service = apiserver::services::leave_import_service::LeaveImportService::new(
        apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
    );

    let job_opt = _state
        .attendance()
        .import_repo()
        .get_job_by_id(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            job_uuid,
        )
        .await
        .unwrap()
        .unwrap();

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        import_service.process_job(job_uuid, ws_id, user_id, &job_opt.file_path.unwrap(), true),
    )
    .await
    .expect("process_job timed out");

    if result.is_err() {
        let job_after = _state
            .attendance()
            .import_repo()
            .get_job_by_id(
                &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
                job_uuid,
            )
            .await
            .unwrap()
            .unwrap();
        panic!("process_job failed! error_log: {:?}", job_after.error_log);
    }

    let job_opt = _state
        .attendance()
        .import_repo()
        .get_job_by_id(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            job_uuid,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        job_opt.status,
        ImportJobStatus::Completed,
        "CSV Job did not complete. error_log: {:?}",
        job_opt.error_log
    );

    // Get job status via API
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/jobs/{job_uuid}/status"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        response.status().is_success() || response.status() == StatusCode::NOT_FOUND,
        "Expected success or not found, got {}",
        response.status()
    );

    let lr_repo = apiserver::repositories::DieselLeaveRequestRepository::new(diesel_pools.clone());
    let requests = lr_repo
        .get_employee_leave_requests(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            emp_id,
        )
        .await
        .unwrap();

    assert_eq!(requests.len(), 1, "Leave request was not created");
    assert_eq!(requests[0].leave_type_id(), leave_type_id);

    // Get latest leave import
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/leave/import/latest"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Download template
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/leave/upload-template"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Download errors
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/leave/import/{job_uuid}/errors"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    // It could be BAD_REQUEST or OK depending on if there are errors. Just check it runs without 500
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
async fn test_leave_import_progress_and_error_formatting_api() {
    let (test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create a processing leave import job
    let job_id = Uuid::new_v4();
    let import_job = apiserver::models::AttendanceImportJob {
        id: job_id,
        workspace_id: ws_id,
        user_id,
        status: ImportJobStatus::Processing,
        total_rows: 10,
        processed_rows: 0,
        file_path: None,
        error_log: None,
        target_type: apiserver::models::enums::ImportTargetType::LeaveImport,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    state
        .attendance()
        .import_repo()
        .create_job(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &import_job,
        )
        .await
        .unwrap();

    // Set progress in Redis
    if let Some(redis) = state.infra().redis() {
        let mut conn = redis.conn();
        let key = format!("import_progress:{}", job_id);
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(4)
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    // 2. Fetch latest leave import job via API and verify processed_rows is fetched from Redis (should be 4)
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/leave/import/latest"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["id"], job_id.to_string());
    assert_eq!(json["status"], "processing");
    assert_eq!(json["total_rows"], 10);
    assert_eq!(json["processed_rows"], 4);

    // 3. Test finalize import with errors formats error_log as JSON
    // Set error in Redis
    if let Some(redis) = state.infra().redis() {
        let mut conn = redis.conn();
        let key = format!("import_errors:{}", job_id);
        let _: () = redis::cmd("HSET")
            .arg(&key)
            .arg("chunk_1_5")
            .arg("Some row failed")
            .query_async(&mut conn)
            .await
            .unwrap();
    }

    // Call FinalizeImportHandler
    let finalize_job = apiserver::queue::Job::with_priority(
        job_id,
        0,
        apiserver::queue::TaskType::FinalizeImport,
        ws_id,
        apiserver::queue::JobPayload::FinalizeImport(apiserver::queue::FinalizeImportPayload {
            import_job_id: job_id,
        }),
        apiserver::queue::JobPriority::High,
    );

    let cache = Arc::new(apiserver::cache::RedisCache::new(
        state.infra().redis().unwrap().clone(),
    ));
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools);

    let res = apiserver::worker::handlers::FinalizeImportHandler::handle(
        &finalize_job,
        &(cache as Arc<dyn apiserver::repositories::CacheRepository>),
        &deps,
    )
    .await;
    assert!(res.is_ok());

    // Verify error_log in DB is formatted as JSON
    let finalized_job = state
        .attendance()
        .import_repo()
        .get_job_by_id(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            job_id,
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(finalized_job.status, ImportJobStatus::Completed);
    let error_log_str = finalized_job.error_log.unwrap();
    let error_log_json: serde_json::Value = serde_json::from_str(&error_log_str).unwrap();
    assert_eq!(error_log_json["success_count"], 4);
    assert_eq!(error_log_json["error_count"], 6);
}

#[tokio::test]
async fn test_import_job_timeout_reconciliation_api() {
    let (test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // 1. Create a processing leave import job from 15 minutes ago
    let leave_job_id = Uuid::new_v4();
    let old_time = chrono::Utc::now() - chrono::Duration::seconds(900);
    let leave_import_job = apiserver::models::AttendanceImportJob {
        id: leave_job_id,
        workspace_id: ws_id,
        user_id,
        status: ImportJobStatus::Processing,
        total_rows: 5,
        processed_rows: 2,
        file_path: None,
        error_log: None,
        target_type: apiserver::models::enums::ImportTargetType::LeaveImport,
        created_at: old_time,
        updated_at: old_time,
    };

    state
        .attendance()
        .import_repo()
        .create_job(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &leave_import_job,
        )
        .await
        .unwrap();

    // Fetch latest leave import job via API and verify it is reconciled to Failed
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!("/api/v1/workspaces/{ws_id}/leave/import/latest"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["id"], leave_job_id.to_string());
    assert_eq!(json["status"], "failed");
    assert!(json["error_log"].as_str().unwrap().contains("timed out"));

    // Verify database record has also been updated to Failed
    let db_job = state
        .attendance()
        .import_repo()
        .get_job_by_id(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            leave_job_id,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(db_job.status, ImportJobStatus::Failed);

    // 2. Create a processing attendance import job from 15 minutes ago
    let att_job_id = Uuid::new_v4();
    let att_import_job = apiserver::models::AttendanceImportJob {
        id: att_job_id,
        workspace_id: ws_id,
        user_id,
        status: ImportJobStatus::Processing,
        total_rows: 10,
        processed_rows: 0,
        file_path: None,
        error_log: None,
        target_type: apiserver::models::enums::ImportTargetType::Log,
        created_at: old_time,
        updated_at: old_time,
    };

    state
        .attendance()
        .import_repo()
        .create_job(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &att_import_job,
        )
        .await
        .unwrap();

    // Fetch latest attendance import job via API and verify it is reconciled to Failed
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/attendance/import-jobs/latest?target_type=log"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["id"], att_job_id.to_string());
    assert_eq!(json["status"], "failed");

    // 3. Re-create another processing attendance import job to test get_import_job_status
    let att_job_id_2 = Uuid::new_v4();
    let att_import_job_2 = apiserver::models::AttendanceImportJob {
        id: att_job_id_2,
        workspace_id: ws_id,
        user_id,
        status: ImportJobStatus::Processing,
        total_rows: 10,
        processed_rows: 0,
        file_path: None,
        error_log: None,
        target_type: apiserver::models::enums::ImportTargetType::Log,
        created_at: old_time,
        updated_at: old_time,
    };

    state
        .attendance()
        .import_repo()
        .create_job(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            &att_import_job_2,
        )
        .await
        .unwrap();

    // Fetch status of specific attendance import job via API and verify it is reconciled to Failed
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/attendance/import-jobs/{att_job_id_2}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["id"], att_job_id_2.to_string());
    assert_eq!(json["status"], "failed");
}

#[tokio::test]
async fn test_download_and_upload_excel_template_roundtrip() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Create employees in DB to match import requirements
    let emp_id_1 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("NV001")
        .create()
        .await;

    let emp_id_2 = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("NV002")
        .create()
        .await;

    // Give employees some leave balance so submit won't fail
    let _leave_balance_id_1 = factory
        .leave_balance()
        .with_employee(emp_id_1)
        .with_entitlement(10)
        .create()
        .await;

    let _leave_balance_id_2 = factory
        .leave_balance()
        .with_employee(emp_id_2)
        .with_entitlement(10)
        .create()
        .await;

    // 1. Download the template
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/leave/upload-template?lang=vi"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Template download failed"
    );

    let template_bytes = axum::body::to_bytes(response.into_body(), 50000)
        .await
        .unwrap();

    // Verify it is a valid XLSX by loading it via calamine
    let mut workbook = calamine::Xlsx::new(std::io::Cursor::new(&template_bytes))
        .expect("Generated template must be a valid XLSX container");
    let sheet_names = workbook.sheet_names();
    assert!(!sheet_names.is_empty(), "Template must have sheets");

    // 2. Upload the downloaded template bytes back
    let (content_type, body) =
        create_multipart_body_binary("leave_import_template.xlsx", &template_bytes);

    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/v1/workspaces/{ws_id}/leave/upload-excel"))
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
        "Excel Template Upload failed"
    );
    let body_bytes = axum::body::to_bytes(response.into_body(), 10000)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let job_id = json["job_id"].as_str().expect("job_id missing");
    let job_uuid = Uuid::parse_str(job_id).unwrap();

    // 3. Process the job manually to check if it parses successfully
    let import_service = apiserver::services::leave_import_service::LeaveImportService::new(
        apiserver::worker::deps::WorkerDeps::new(&diesel_pools.clone()),
    );

    let job_opt = _state
        .attendance()
        .import_repo()
        .get_job_by_id(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            job_uuid,
        )
        .await
        .unwrap()
        .unwrap();

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        import_service.process_job(job_uuid, ws_id, user_id, &job_opt.file_path.unwrap(), true),
    )
    .await
    .expect("process_job timed out");

    if let Err(e) = result {
        let job_after = _state
            .attendance()
            .import_repo()
            .get_job_by_id(
                &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
                job_uuid,
            )
            .await
            .unwrap()
            .unwrap();
        let mut error_msg = format!(
            "process_job failed for downloaded template! error: {e:?}, error_log: {:?}",
            job_after.error_log
        );
        if let Some(ref log) = job_after.error_log {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(log) {
                if let Some(path) = json["error_file_path"].as_str() {
                    if let Ok(file_bytes) = std::fs::read(path) {
                        if let Ok(mut err_wb) =
                            calamine::Xlsx::new(std::io::Cursor::new(&file_bytes))
                        {
                            if let Some(Ok(range)) = err_wb.worksheet_range_at(0) {
                                let mut rows_str = Vec::new();
                                for (idx, r) in range.rows().enumerate() {
                                    let cols: Vec<String> = r
                                        .iter()
                                        .map(|c| {
                                            apiserver::utils::file_parser::cell_to_string(Some(c))
                                        })
                                        .collect();
                                    rows_str.push(format!("Row {idx}: {cols:?}"));
                                }
                                error_msg.push_str(&format!(
                                    "\nError Excel rows:\n{}",
                                    rows_str.join("\n")
                                ));
                            }
                        }
                    }
                }
            }
        }
        panic!("{error_msg}");
    }

    let job_opt = _state
        .attendance()
        .import_repo()
        .get_job_by_id(
            &apiserver::diesel_pool::TenantAuthToken::system_override(ws_id),
            job_uuid,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        job_opt.status,
        ImportJobStatus::Completed,
        "Excel Job did not complete successfully. error_log: {:?}",
        job_opt.error_log
    );
}
