use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use calamine::Reader;
use rust_xlsxwriter::Workbook;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

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

fn generate_termination_xlsx(rows: &[(&str, &str)]) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet.write_string(0, 0, "employee_code").unwrap();
    worksheet.write_string(0, 1, "termination_date").unwrap();

    for (idx, &(id, date)) in rows.iter().enumerate() {
        let row = (idx + 1) as u32;
        worksheet.write_string(row, 0, id).unwrap();
        worksheet.write_string(row, 1, date).unwrap();
    }

    workbook.save_to_buffer().unwrap()
}

#[tokio::test]
async fn test_bulk_termination_flow() {
    let (test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(user_id).await;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    // Create 3 employees:
    // 1. EMP001 (Valid termination)
    // 2. EMP002 (Termination date before start date -> invalid)
    // 3. EMP003 (Already terminated -> invalid)
    let emp1_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP001")
        .with_full_name("John Active")
        .create()
        .await;

    let emp2_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP002")
        .with_full_name("Jane Early")
        .create()
        .await;

    let emp3_id = factory
        .employee()
        .with_workspace(ws_id)
        .with_employee_id_number("EMP003")
        .with_full_name("Bob Terminated")
        .create()
        .await;

    // Wait, let's get the employment records created by the factory
    let employments1 = state
        .employees()
        .employment_repo()
        .list_employee_employments(&auth_token, emp1_id)
        .await
        .unwrap();
    let emp1_employment = employments1
        .iter()
        .find(|e| e.status() == "active")
        .unwrap()
        .clone();

    let employments2 = state
        .employees()
        .employment_repo()
        .list_employee_employments(&auth_token, emp2_id)
        .await
        .unwrap();
    let emp2_employment_orig = employments2
        .iter()
        .find(|e| e.status() == "active")
        .unwrap()
        .clone();

    let update_req = apiserver::models::UpdateEmploymentRequest {
        employer_id: None,
        primary_location_id: None,
        department_id: None,
        status: None,
        start_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        end_date: None,
        job_title: None,
        employment_type: None,
        manager_employment_id: None,
        notes: None,
        termination_reason: None,
    };
    let emp2_employment = state
        .employees()
        .employment_repo()
        .update_employment(&auth_token, emp2_employment_orig.id(), &update_req)
        .await
        .unwrap();

    let employments3 = state
        .employees()
        .employment_repo()
        .list_employee_employments(&auth_token, emp3_id)
        .await
        .unwrap();
    let emp3_employment = employments3
        .iter()
        .find(|e| e.status() == "active")
        .unwrap()
        .clone();

    // Terminate EMP003 in advance
    state
        .employees()
        .service()
        .terminate_employment(
            &auth_token,
            emp3_employment.id(),
            chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
            "voluntary_resignation".to_string(),
            None,
        )
        .await
        .unwrap();

    // Setup a leave balance for EMP001 so we can verify leave proration preview
    // Note: leave_type_id needs to exist. We can query a workspace leave type or create one.
    let leave_types = state
        .leaves()
        .policy_repo()
        .get_leave_types(ws_id)
        .await
        .unwrap();
    let annual_leave_type = leave_types
        .iter()
        .find(|t| t.code() == "AL" || t.code() == "annual")
        .cloned();

    let leave_type_id = if let Some(lt) = annual_leave_type {
        lt.id()
    } else {
        let new_lt = state
            .leaves()
            .policy_repo()
            .create_leave_type(
                ws_id,
                "AL".to_string(),
                "Annual Leave".to_string(),
                true,
                apiserver::models::LeaveTypeConfig::default(),
            )
            .await
            .unwrap();
        new_lt.id()
    };

    // Insert leave balance of 12 days for EMP001 for 2026
    let initial_balance = apiserver::models::LeaveBalance::new(
        Uuid::new_v4(),
        emp1_employment.id(),
        leave_type_id,
        2026,
        12,
        rust_decimal::Decimal::ZERO,
        0,
        chrono::Utc::now(),
        chrono::Utc::now(),
    );
    state
        .leaves()
        .balance_repo()
        .upsert_leave_balance(&auth_token, &initial_balance)
        .await
        .unwrap();

    // Generate sheet content:
    // Row 1: EMP001 -> terminated on 2026-06-30 (start of year to June 30 is ~181 days out of 365, proration ~6 days)
    // Row 2: EMP002 -> terminated on 2025-12-31 (before start_date of employment which is in 2026)
    // Row 3: EMP003 -> terminated on 2026-06-30 (already terminated)
    // Row 4: EMP999 -> non-existent employee
    let xlsx_bytes = generate_termination_xlsx(&[
        ("EMP001", "2026-06-30"),
        ("EMP002", "2025-12-31"),
        ("EMP003", "2026-06-30"),
        ("EMP999", "2026-06-30"),
    ]);

    let (content_type, body) = create_multipart_body_binary("terminations.xlsx", &xlsx_bytes);

    // 1. Post to Preview Endpoint
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/import-termination/preview"
                ))
                .header("Content-Type", content_type)
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = axum::body::to_bytes(response.into_body(), 50000)
        .await
        .unwrap();
    let preview: apiserver::handlers::employees::TerminationPreviewResponse =
        serde_json::from_slice(&body_bytes).unwrap();

    // Verify valid rows: Only EMP001 is valid
    assert_eq!(preview.valid_rows.len(), 1);
    let valid_row = &preview.valid_rows[0];
    assert_eq!(valid_row.employee_code, "EMP001");
    assert_eq!(valid_row.employment_id, emp1_employment.id());
    assert_eq!(
        valid_row.termination_date,
        chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()
    );

    // Verify prorated leaves preview
    assert_eq!(valid_row.prorated_leaves.len(), 1);
    let leave_preview = &valid_row.prorated_leaves[0];
    assert_eq!(leave_preview.old_entitlement, 12);
    // 2026 has 365 days. 2026-01-01 to 2026-06-30 is 181 days. 12 * 181 / 365 = 5.95 -> round to 6
    assert_eq!(leave_preview.new_entitlement, 6);

    // Verify invalid rows
    assert_eq!(preview.invalid_rows.len(), 3);

    // Check EMP002 (termination before start date)
    let inv_row_emp2 = preview
        .invalid_rows
        .iter()
        .find(|r| r.employee_code.as_deref() == Some("EMP002"))
        .unwrap();
    assert!(
        inv_row_emp2
            .reason
            .contains("không được trước ngày bắt đầu")
            || inv_row_emp2.reason.contains("cannot be before")
    );

    // Check EMP003 (no active employment)
    let inv_row_emp3 = preview
        .invalid_rows
        .iter()
        .find(|r| r.employee_code.as_deref() == Some("EMP003"))
        .unwrap();
    assert_eq!(
        inv_row_emp3.reason,
        apiserver::utils::i18n::translate("vi", "error.no_active_employment")
    );

    // Check EMP999 (employee not found)
    let inv_row_emp999 = preview
        .invalid_rows
        .iter()
        .find(|r| r.employee_code.as_deref() == Some("EMP999"))
        .unwrap();
    assert_eq!(
        inv_row_emp999.reason,
        apiserver::utils::i18n::translate("vi", "error.employee_not_found")
    );

    assert!(preview.can_commit);

    // 2. Post to Confirm Endpoint
    let confirm_payload = apiserver::handlers::employees::ConfirmTerminationRequest {
        rows: vec![apiserver::handlers::employees::ConfirmTerminationRow {
            employment_id: emp1_employment.id(),
            termination_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
            termination_reason: "voluntary_resignation".to_string(),
            termination_notes: None,
        }],
    };

    let confirm_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/import-termination/confirm"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::from(
                    serde_json::to_vec(&confirm_payload).unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(confirm_response.status(), StatusCode::OK);
    let confirm_bytes = axum::body::to_bytes(confirm_response.into_body(), 10000)
        .await
        .unwrap();
    let confirm_res: apiserver::handlers::employees::ConfirmTerminationResponse =
        serde_json::from_slice(&confirm_bytes).unwrap();
    assert_eq!(confirm_res.success_count, 1);

    // Dequeue job from redis and process it manually
    let redis = state.infra().redis().unwrap().clone();
    let queue = apiserver::queue::JobQueue::new(redis.clone());
    let target_job_id = confirm_res.job_id.as_deref().unwrap();
    let mut skipped_jobs = Vec::new();
    let mut target_job = None;

    for _ in 0..20 {
        if let Some(j) = queue.dequeue_job(5.0).await.unwrap() {
            if j.parent_job_id.to_string() == target_job_id {
                target_job = Some(j);
                break;
            } else {
                skipped_jobs.push(j);
            }
        } else {
            break;
        }
    }

    // Re-enqueue skipped jobs so other tests can process them
    if !skipped_jobs.is_empty() {
        queue.enqueue_jobs(skipped_jobs).await.unwrap();
    }

    let job = target_job.expect("Job not found in queue");
    let queue_repo: std::sync::Arc<dyn apiserver::repositories::QueueRepository> =
        std::sync::Arc::new(
            apiserver::repositories::queue_repository::RedisQueueRepository::new(redis.clone()),
        );
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let deps = apiserver::worker::deps::WorkerDeps::new(&diesel_pools);
    apiserver::worker::handlers::BulkTerminationHandler::handle(&queue_repo, &job, &deps)
        .await
        .unwrap();

    // 3. Verify Database State
    let emp1_employment_updated = state
        .employees()
        .employment_repo()
        .get_employment_by_id(&auth_token, emp1_employment.id())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(emp1_employment_updated.status(), "terminated");
    assert_eq!(
        emp1_employment_updated.end_date(),
        Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap())
    );

    let updated_balances = state
        .leaves()
        .balance_repo()
        .get_employee_balances(&auth_token, emp1_id, 2026)
        .await
        .unwrap();
    assert_eq!(updated_balances.len(), 1);
    assert_eq!(updated_balances[0].entitlement, 6);
}

#[tokio::test]
async fn test_import_termination_template_download() {
    let (_test_db, factory, _state, app) = setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;

    // Call template download endpoint
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/import-termination/template"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check headers
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .unwrap();
    assert_eq!(
        content_type,
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );

    let content_disposition = response
        .headers()
        .get(axum::http::header::CONTENT_DISPOSITION)
        .unwrap();
    assert_eq!(
        content_disposition,
        "attachment; filename=\"employee_termination_template.xlsx\""
    );

    // Read body as bytes
    let body = axum::body::to_bytes(response.into_body(), 10_000_000)
        .await
        .unwrap();
    assert!(!body.is_empty());

    // Excel/ZIP file signature check
    assert!(body.starts_with(b"PK\x03\x04"));

    // Verify it is a valid XLSX by loading it via calamine
    let mut workbook = calamine::Xlsx::new(std::io::Cursor::new(&body))
        .expect("Generated template must be a valid XLSX container");

    let sheet_names = workbook.sheet_names();
    assert_eq!(sheet_names.len(), 2);
    assert_eq!(sheet_names[0], "Template");
    assert_eq!(sheet_names[1], "Instructions");

    // Verify Template sheet headers
    let template_range = workbook.worksheet_range("Template").unwrap();
    let header = template_range
        .rows()
        .next()
        .expect("Template sheet must have a header row");
    let header_cols: Vec<String> = header
        .iter()
        .map(|c| apiserver::utils::file_parser::cell_to_string(Some(c)))
        .collect();
    assert_eq!(
        header_cols,
        vec![
            "employee_code".to_string(),
            "termination_date".to_string(),
            "termination_reason".to_string(),
            "termination_notes".to_string(),
        ]
    );

    // Verify Instructions sheet contains valid termination reason keys
    let instructions_range = workbook.worksheet_range("Instructions").unwrap();
    let mut found_voluntary = false;
    let mut found_involuntary = false;
    let mut found_end_of_contract = false;
    let mut found_retirement = false;
    let mut found_mutual = false;
    let mut found_abandonment = false;
    let mut found_death = false;
    let mut found_other = false;

    for row in instructions_range.rows() {
        for cell in row {
            let val = apiserver::utils::file_parser::cell_to_string(Some(cell));
            match val.as_str() {
                "voluntary_resignation" => found_voluntary = true,
                "involuntary_termination" => found_involuntary = true,
                "end_of_contract" => found_end_of_contract = true,
                "retirement" => found_retirement = true,
                "mutual_agreement" => found_mutual = true,
                "abandonment" => found_abandonment = true,
                "death" => found_death = true,
                "other" => found_other = true,
                _ => {}
            }
        }
    }

    assert!(
        found_voluntary,
        "Instructions must list 'voluntary_resignation'"
    );
    assert!(
        found_involuntary,
        "Instructions must list 'involuntary_termination'"
    );
    assert!(
        found_end_of_contract,
        "Instructions must list 'end_of_contract'"
    );
    assert!(found_retirement, "Instructions must list 'retirement'");
    assert!(found_mutual, "Instructions must list 'mutual_agreement'");
    assert!(found_abandonment, "Instructions must list 'abandonment'");
    assert!(found_death, "Instructions must list 'death'");
    assert!(found_other, "Instructions must list 'other'");

    // Call template download endpoint with Vietnamese language parameter
    let response_vi = app
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/employees/import-termination/template?lang=vi"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response_vi.status(), StatusCode::OK);

    let body_vi = axum::body::to_bytes(response_vi.into_body(), 10_000_000)
        .await
        .unwrap();

    let mut workbook_vi = calamine::Xlsx::new(std::io::Cursor::new(&body_vi))
        .expect("Generated template must be a valid XLSX container");

    let sheet_names_vi = workbook_vi.sheet_names();
    assert_eq!(sheet_names_vi.len(), 2);
    assert_eq!(sheet_names_vi[0], "Mẫu nhập");
    assert_eq!(sheet_names_vi[1], "Hướng dẫn");

    let template_range_vi = workbook_vi.worksheet_range("Mẫu nhập").unwrap();
    let header_vi = template_range_vi
        .rows()
        .next()
        .expect("Template sheet must have a header row");
    let header_cols_vi: Vec<String> = header_vi
        .iter()
        .map(|c| apiserver::utils::file_parser::cell_to_string(Some(c)))
        .collect();
    assert_eq!(
        header_cols_vi,
        vec![
            "employee_code".to_string(),
            "termination_date".to_string(),
            "termination_reason".to_string(),
            "termination_notes".to_string(),
        ]
    );

    let instructions_range_vi = workbook_vi.worksheet_range("Hướng dẫn").unwrap();
    let mut found_voluntary_vi = false;
    for row in instructions_range_vi.rows() {
        for cell in row {
            let val = apiserver::utils::file_parser::cell_to_string(Some(cell));
            if val == "voluntary_resignation" {
                found_voluntary_vi = true;
            }
        }
    }
    assert!(
        found_voluntary_vi,
        "Instructions must list 'voluntary_resignation' in VI template"
    );
}
