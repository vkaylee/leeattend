use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
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
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
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
