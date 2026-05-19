//! Employee bulk import integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::ImportJobStatus;
use apiserver::AppState;
use axum::http::StatusCode;
use rust_xlsxwriter::Workbook;
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
        "employee_code,full_name,hire_date,department_name\nEMP001,John Doe,2026-01-01,\nEMP002,Jane Smith,2026-01-15,";
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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
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
    assert!(completed, "Employee CSV Job did not complete");

    // Verify employees created
    let emp1 = _state
        .employees
        .repo
        .find_by_number(ws_id, "EMP001")
        .await
        .unwrap();
    assert!(emp1.is_some());
    let emp1 = emp1.unwrap();
    assert_eq!(emp1.full_name, "John Doe");

    let emp2 = _state
        .employees
        .repo
        .find_by_number(ws_id, "EMP002")
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
        "employee_code,full_name,hire_date,department_name\nDUP001,New John,2026-01-01,\nNEW002,Alice Wonder,2026-02-01,";
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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;

    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
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

    // Check error_log for DL_LINK
    let job = _state
        .attendance
        .import_repo
        .get_job_by_id(ws_id, job_uuid)
        .await
        .unwrap()
        .unwrap();
    let error_log = job.error_log.expect("Expected error_log for skipped row");
    assert!(
        error_log.contains("DL_LINK:"),
        "Error log should contain DL_LINK: {}",
        error_log
    );

    // Check that the error file exists on disk
    let storage_dir = apiserver::utils::get_storage_dir();
    let error_file_path = format!("{}/imports/error_{job_uuid}.xlsx", storage_dir);
    assert!(
        tokio::fs::try_exists(&error_file_path).await.unwrap(),
        "Error Excel file should exist"
    );

    // Verify "DUP001" is NOT updated
    let emp_dup = _state
        .employees
        .repo
        .find_by_number(ws_id, "DUP001")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(emp_dup.full_name, "Old Name");

    // Verify "NEW002" IS created
    let emp_new = _state
        .employees
        .repo
        .find_by_number(ws_id, "NEW002")
        .await
        .unwrap();
    assert!(emp_new.is_some());
}

#[tokio::test]
async fn test_upload_employee_csv_with_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // CSV with 3 employees, 2 in "Engineering", 1 in "Marketing", column D = department
    let csv_content = "employee_code,full_name,hire_date,department_name\n\
                       DEPT001,Alice Eng,2026-01-01,Engineering\n\
                       DEPT002,Bob Eng,2026-01-15,Engineering\n\
                       DEPT003,Carol Mkt,2026-02-01,Marketing";
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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    // Wait for job completion
    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            assert_eq!(job.processed_rows, 3, "All 3 employees should be processed");
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete in time");

    // Verify departments were created
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let dept_names: Vec<&str> = departments.iter().map(|d| d.name.as_str()).collect();
    assert!(
        dept_names.contains(&"Engineering"),
        "Engineering department should be created"
    );
    assert!(
        dept_names.contains(&"Marketing"),
        "Marketing department should be created"
    );

    // Verify employee DEPT001 is assigned to Engineering
    let emp1 = _state
        .employees
        .repo
        .find_by_number(ws_id, "DEPT001")
        .await
        .unwrap()
        .expect("Employee DEPT001 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp1.id)
        .await
        .unwrap();
    assert!(
        current_dept.is_some(),
        "DEPT001 should have a department assignment"
    );

    let eng_dept = departments
        .iter()
        .find(|d| d.name == "Engineering")
        .unwrap();
    assert_eq!(
        current_dept.unwrap().department_id,
        eng_dept.id,
        "DEPT001 should be assigned to Engineering"
    );

    // Verify employee DEPT003 is assigned to Marketing
    let emp3 = _state
        .employees
        .repo
        .find_by_number(ws_id, "DEPT003")
        .await
        .unwrap()
        .expect("Employee DEPT003 should exist");

    let current_dept3 = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp3.id)
        .await
        .unwrap();
    assert!(current_dept3.is_some());

    let mkt_dept = departments.iter().find(|d| d.name == "Marketing").unwrap();
    assert_eq!(
        current_dept3.unwrap().department_id,
        mkt_dept.id,
        "DEPT003 should be assigned to Marketing"
    );
}

#[tokio::test]
async fn test_upload_employee_csv_reuses_existing_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Pre-create a department named "Sales"
    let pre_existing_dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    // CSV with an employee in the same department "Sales" (case-insensitive match!)
    let csv_content =
        "employee_code,full_name,hire_date,department_name\nRE001,Dana Sales,2026-03-01,sales";
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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete");

    // Verify NO duplicate department was created — should still be only 1 "Sales"
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let sales_depts: Vec<_> = departments
        .iter()
        .filter(|d| d.name.to_lowercase() == "sales")
        .collect();
    assert_eq!(
        sales_depts.len(),
        1,
        "Should reuse existing department, not create duplicate"
    );
    assert_eq!(
        sales_depts[0].id, pre_existing_dept_id,
        "Should be the same pre-existing department"
    );

    // Verify employee is assigned to it
    let emp = _state
        .employees
        .repo
        .find_by_number(ws_id, "RE001")
        .await
        .unwrap()
        .expect("Employee RE001 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp.id)
        .await
        .unwrap();
    assert!(current_dept.is_some());
    assert_eq!(
        current_dept.unwrap().department_id,
        pre_existing_dept_id,
        "Employee should be assigned to the pre-existing Sales department"
    );
}

/// Create multipart body with binary content (for real .xlsx files)
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

/// Generate a real .xlsx file with employee data including department column
fn generate_employee_xlsx(rows: &[(&str, &str, &str, &str)]) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // Header row
    worksheet.write_string(0, 0, "employee_code").unwrap();
    worksheet.write_string(0, 1, "full_name").unwrap();
    worksheet.write_string(0, 2, "hire_date").unwrap();
    worksheet.write_string(0, 3, "department_name").unwrap();

    // Data rows
    for (idx, (id, name, date, dept)) in rows.iter().enumerate() {
        let row = (idx + 1) as u32;
        worksheet.write_string(row, 0, *id).unwrap();
        worksheet.write_string(row, 1, *name).unwrap();
        worksheet.write_string(row, 2, *date).unwrap();
        if !dept.is_empty() {
            worksheet.write_string(row, 3, *dept).unwrap();
        }
    }

    workbook.save_to_buffer().unwrap()
}

#[tokio::test]
async fn test_upload_employee_xlsx_with_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Generate a real .xlsx file with 3 employees in 2 departments
    let xlsx_bytes = generate_employee_xlsx(&[
        ("XLS001", "Alice Excel", "2026-01-01", "Engineering"),
        ("XLS002", "Bob Excel", "2026-01-15", "Engineering"),
        ("XLS003", "Carol Excel", "2026-02-01", "Marketing"),
    ]);
    let (content_type, body) = create_multipart_body_binary("employees.xlsx", &xlsx_bytes);

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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    // Wait for job completion
    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            assert_eq!(job.processed_rows, 3, "All 3 employees should be processed");
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete in time");

    // Verify departments were created
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let dept_names: Vec<&str> = departments.iter().map(|d| d.name.as_str()).collect();
    assert!(
        dept_names.contains(&"Engineering"),
        "Engineering department should be created from xlsx"
    );
    assert!(
        dept_names.contains(&"Marketing"),
        "Marketing department should be created from xlsx"
    );

    // Verify employee XLS001 is assigned to Engineering
    let emp1 = _state
        .employees
        .repo
        .find_by_number(ws_id, "XLS001")
        .await
        .unwrap()
        .expect("Employee XLS001 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp1.id)
        .await
        .unwrap();
    assert!(
        current_dept.is_some(),
        "XLS001 should have a department assignment"
    );

    let eng_dept = departments
        .iter()
        .find(|d| d.name == "Engineering")
        .unwrap();
    assert_eq!(
        current_dept.unwrap().department_id,
        eng_dept.id,
        "XLS001 should be assigned to Engineering"
    );

    // Verify employee XLS003 is assigned to Marketing
    let emp3 = _state
        .employees
        .repo
        .find_by_number(ws_id, "XLS003")
        .await
        .unwrap()
        .expect("Employee XLS003 should exist");

    let current_dept3 = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp3.id)
        .await
        .unwrap();
    assert!(current_dept3.is_some());

    let mkt_dept = departments.iter().find(|d| d.name == "Marketing").unwrap();
    assert_eq!(
        current_dept3.unwrap().department_id,
        mkt_dept.id,
        "XLS003 should be assigned to Marketing"
    );
}

#[tokio::test]
async fn test_upload_employee_xlsx_reuses_existing_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Pre-create a department named "Sales"
    let pre_existing_dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Sales")
        .create()
        .await;

    // Generate a real .xlsx — department name is lowercase "sales" (case-insensitive match!)
    let xlsx_bytes = generate_employee_xlsx(&[("XLSRE001", "Dana Xlsx", "2026-03-01", "sales")]);
    let (content_type, body) = create_multipart_body_binary("employees.xlsx", &xlsx_bytes);

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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete");

    // Verify NO duplicate department was created — should still be only 1 "Sales"
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let sales_depts: Vec<_> = departments
        .iter()
        .filter(|d| d.name.to_lowercase() == "sales")
        .collect();
    assert_eq!(
        sales_depts.len(),
        1,
        "Should reuse existing department, not create duplicate"
    );
    assert_eq!(
        sales_depts[0].id, pre_existing_dept_id,
        "Should be the same pre-existing department"
    );

    // Verify employee is assigned to it
    let emp = _state
        .employees
        .repo
        .find_by_number(ws_id, "XLSRE001")
        .await
        .unwrap()
        .expect("Employee XLSRE001 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp.id)
        .await
        .unwrap();
    assert!(current_dept.is_some());
    assert_eq!(
        current_dept.unwrap().department_id,
        pre_existing_dept_id,
        "Employee should be assigned to the pre-existing Sales department"
    );
}

// =============================================================================
// Scenario 3: Existing employee + New department
// =============================================================================

#[tokio::test]
async fn test_upload_csv_existing_employee_new_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Pre-create employee
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EXIST001")
        .with_full_name("Existing Alice")
        .create()
        .await;

    // Upload CSV with same employee ID + a NEW department
    let csv =
        "employee_code,full_name,hire_date,department_name\nEXIST001,Existing Alice,2026-01-01,R&D";
    let (content_type, body) = create_multipart_body("employees.csv", csv);

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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete");

    // Verify "R&D" department was created
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let rd_dept = departments
        .iter()
        .find(|d| d.name == "R&D")
        .expect("R&D department should be created");

    // Verify existing employee is assigned to the new department
    let emp = _state
        .employees
        .repo
        .find_by_number(ws_id, "EXIST001")
        .await
        .unwrap()
        .expect("Employee EXIST001 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp.id)
        .await
        .unwrap();
    assert!(
        current_dept.is_some(),
        "Existing employee should be assigned to the new department"
    );
    assert_eq!(
        current_dept.unwrap().department_id,
        rd_dept.id,
        "Existing employee should be in R&D"
    );
}

#[tokio::test]
async fn test_upload_csv_existing_employee_existing_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Pre-create employee
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EXIST002")
        .with_full_name("Existing Bob")
        .create()
        .await;

    // Pre-create department
    let pre_dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("Finance")
        .create()
        .await;

    // Upload CSV with existing employee + existing department
    let csv =
        "employee_code,full_name,hire_date,department_name\nEXIST002,Existing Bob,2026-01-01,Finance";
    let (content_type, body) = create_multipart_body("employees.csv", csv);

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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete");

    // Verify no duplicate department
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let finance_depts: Vec<_> = departments.iter().filter(|d| d.name == "Finance").collect();
    assert_eq!(
        finance_depts.len(),
        1,
        "Should not create duplicate Finance"
    );

    // Verify existing employee is assigned to existing department
    let emp = _state
        .employees
        .repo
        .find_by_number(ws_id, "EXIST002")
        .await
        .unwrap()
        .expect("Employee EXIST002 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp.id)
        .await
        .unwrap();
    assert!(current_dept.is_some());
    assert_eq!(
        current_dept.unwrap().department_id,
        pre_dept_id,
        "Existing employee should be assigned to existing Finance department"
    );
}

// =============================================================================
// Scenario 3 & 4 (XLSX): Existing employee + New/Existing department
// =============================================================================

#[tokio::test]
async fn test_upload_xlsx_existing_employee_new_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Pre-create employee
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("XLSEX001")
        .with_full_name("Existing Carol")
        .create()
        .await;

    // Generate xlsx with existing employee + new department
    let xlsx_bytes =
        generate_employee_xlsx(&[("XLSEX001", "Existing Carol", "2026-01-01", "DevOps")]);
    let (content_type, body) = create_multipart_body_binary("employees.xlsx", &xlsx_bytes);

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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete");

    // Verify DevOps department was created
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let devops_dept = departments
        .iter()
        .find(|d| d.name == "DevOps")
        .expect("DevOps department should be created");

    // Verify existing employee is assigned
    let emp = _state
        .employees
        .repo
        .find_by_number(ws_id, "XLSEX001")
        .await
        .unwrap()
        .expect("Employee XLSEX001 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp.id)
        .await
        .unwrap();
    assert!(current_dept.is_some());
    assert_eq!(
        current_dept.unwrap().department_id,
        devops_dept.id,
        "Existing employee should be in DevOps"
    );
}

#[tokio::test]
async fn test_upload_xlsx_existing_employee_existing_department() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;

    // Pre-create employee
    factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("XLSEX002")
        .with_full_name("Existing Dana")
        .create()
        .await;

    // Pre-create department
    let pre_dept_id = factory
        .department()
        .with_workspace(ws_id)
        .with_name("HR")
        .create()
        .await;

    // Generate xlsx with existing employee + existing department (lowercase for case-insensitive)
    let xlsx_bytes = generate_employee_xlsx(&[("XLSEX002", "Existing Dana", "2026-01-01", "hr")]);
    let (content_type, body) = create_multipart_body_binary("employees.xlsx", &xlsx_bytes);

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
    tokio::time::timeout(
        std::time::Duration::from_secs(10),
        _state.attendance.import_service.process_job(ws_id, job_uuid),
    )
    .await
    .expect("process_job timed out")
    .unwrap();

    let mut completed = false;
    for _ in 0..30 {
        sleep(Duration::from_millis(200)).await;
        let job = _state
            .attendance
            .import_repo
            .get_job_by_id(ws_id, job_uuid)
            .await
            .unwrap()
            .unwrap();
        if job.status == ImportJobStatus::Completed {
            completed = true;
            break;
        } else if job.status == ImportJobStatus::Failed {
            panic!("Job failed: {:?}", job.error_log);
        }
    }
    assert!(completed, "Job did not complete");

    // Verify no duplicate
    let departments = _state
        .payroll
        .department_repo
        .get_departments(ws_id)
        .await
        .unwrap();
    let hr_depts: Vec<_> = departments
        .iter()
        .filter(|d| d.name.to_lowercase() == "hr")
        .collect();
    assert_eq!(hr_depts.len(), 1, "Should not create duplicate HR");
    assert_eq!(hr_depts[0].id, pre_dept_id);

    // Verify existing employee is assigned
    let emp = _state
        .employees
        .repo
        .find_by_number(ws_id, "XLSEX002")
        .await
        .unwrap()
        .expect("Employee XLSEX002 should exist");

    let current_dept = _state
        .payroll
        .department_repo
        .get_current_department(ws_id, emp.id)
        .await
        .unwrap();
    assert!(current_dept.is_some());
    assert_eq!(
        current_dept.unwrap().department_id,
        pre_dept_id,
        "Existing employee should be assigned to existing HR department"
    );
}
