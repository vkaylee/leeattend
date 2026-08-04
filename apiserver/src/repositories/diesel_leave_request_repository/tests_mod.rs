use super::*;
use crate::diesel_pool::{DieselPools, TenantAuthToken};
use crate::models::{LeaveRequest, LeaveStatus, NewLeaveRequestParams};
use chrono::{NaiveDate, Utc};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

#[tokio::test]
async fn test_leave_request_repository_errors() {
    let test_db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/leeattend_test_db".to_string());
    let pools = DieselPools::for_test(&test_db_url);
    let repo = DieselLeaveRequestRepository::new(pools);
    let ws_id = Uuid::new_v4();
    let emp_id = Uuid::new_v4();
    let lt_id = Uuid::new_v4();
    let auth_token = TenantAuthToken(ws_id);

    let req = LeaveRequest::new(NewLeaveRequestParams {
        id: Uuid::new_v4(),
        employment_id: emp_id,
        leave_type_id: lt_id,
        start_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(),
        reason: Some("test".to_string()),
        status: LeaveStatus::Pending,
        duration_type: crate::models::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        requested_by_user_id: None,
        approved_by_user_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        attachment_file_ids: None,
    });

    let _ = repo.create_leave_request(&auth_token, &req).await;
    let _ = repo
        .get_leave_request_by_id(&auth_token, Uuid::new_v4())
        .await;
    let _ = repo
        .update_leave_request_status(&auth_token, Uuid::new_v4(), LeaveStatus::Approved, None)
        .await;
    let _ = repo.get_employee_leave_requests(&auth_token, emp_id).await;
    let _ = repo.get_workspace_leave_requests(&auth_token, None).await;
    let _ = repo
        .get_approved_leave_breakdown(
            emp_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            &auth_token,
        )
        .await;
    let _ = repo
        .get_employee_department_for_request(Uuid::new_v4(), &auth_token)
        .await;
    let _ = repo.get_leave_types(&auth_token).await;
    let _ = repo
        .get_system_generated_holiday_leave(
            &auth_token,
            emp_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        )
        .await;
    let _ = repo.cancel_leave_request(&auth_token, Uuid::new_v4()).await;
    let _ = repo
        .get_employee_leave_history_requests(&auth_token, emp_id, 2023, None)
        .await;
    let _ = repo.check_leave_type_exists(&auth_token, lt_id).await;
    let _ = repo.get_leave_type_name(&auth_token, lt_id).await;
    let _ = repo
        .get_unpaid_holiday_days_without_leave(
            &auth_token,
            emp_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await;
    let _ = repo
        .get_batch_leave_days(
            &auth_token,
            &[emp_id],
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await;
    let _ = repo
        .get_approved_leave_dates(
            &auth_token,
            &[emp_id],
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await;
    let _ = repo
        .get_approved_leave_periods(
            &auth_token,
            emp_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await;
}

#[tokio::test]
async fn test_leave_request_repository_flow() {
    let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://test_user:test_password@db_system:5432/leeattend_test_template".to_string()
    });
    let pools = DieselPools::for_test(&test_db_url);
    let repo = DieselLeaveRequestRepository::new(pools.clone());
    let ws_id = Uuid::new_v4();
    let token = TenantAuthToken::system_override(ws_id);

    let mut conn = if let Ok(c) = pools.tenant_transaction_conn(&token).await {
        c
    } else {
        println!("Database not available, skipping integration assertions");
        return;
    };

    // Insert Workspace
    let workspace_entity = crate::models::diesel::core::WorkspaceEntity {
        id: ws_id,
        name: "Test Workspace".to_string(),
        created_at: Some(chrono::Utc::now()),
        date_format: "YYYY-MM-DD".to_string(),
        time_format: "HH:mm".to_string(),
        timezone: "UTC".to_string(),
        theme_primary_color: None,
        default_language: "en".to_string(),
        logo_url: None,
        tagline: None,
        custom_db_url_encrypted: None,
    };
    diesel::insert_into(crate::schema::workspaces::table)
        .values(&workspace_entity)
        .execute(&mut *conn)
        .await
        .expect("insert workspace");

    let employer_id = Uuid::new_v4();
    let dept_id = Uuid::new_v4();
    let employee_id = Uuid::new_v4();
    let employment_id = Uuid::new_v4();
    let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();

    // Insert Employer
    diesel::sql_query("INSERT INTO employers (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
        .bind::<diesel::sql_types::Uuid, _>(employer_id)
        .bind::<diesel::sql_types::Uuid, _>(ws_id)
        .bind::<diesel::sql_types::Text, _>("Test Employer")
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert employer");

    // Insert Department
    diesel::sql_query("INSERT INTO departments (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)")
        .bind::<diesel::sql_types::Uuid, _>(dept_id)
        .bind::<diesel::sql_types::Uuid, _>(ws_id)
        .bind::<diesel::sql_types::Text, _>("Test Dept")
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert dept");

    // Insert Employee
    diesel::sql_query("INSERT INTO employees (id, workspace_id, employee_id_number, full_name, hire_date, original_hire_date, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(ws_id)
        .bind::<diesel::sql_types::Text, _>("EMP-LEAVE-TEST")
        .bind::<diesel::sql_types::Text, _>("Test Leave Employee")
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Bool, _>(true)
        .execute(&mut *conn)
        .await
        .expect("insert employee");

    // Insert Employment
    diesel::sql_query("INSERT INTO employments (id, workspace_id, employee_id, employer_id, department_id, status, start_date, created_at, updated_at, employment_type) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
        .bind::<diesel::sql_types::Uuid, _>(employment_id)
        .bind::<diesel::sql_types::Uuid, _>(ws_id)
        .bind::<diesel::sql_types::Uuid, _>(employee_id)
        .bind::<diesel::sql_types::Uuid, _>(employer_id)
        .bind::<diesel::sql_types::Uuid, _>(dept_id)
        .bind::<diesel::sql_types::Text, _>("active")
        .bind::<diesel::sql_types::Date, _>(start_date)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Text, _>("full_time")
        .execute(&mut *conn)
        .await
        .expect("insert employment");

    // Insert Leave Type
    let lt_id = Uuid::new_v4();
    diesel::sql_query("INSERT INTO leave_types (id, workspace_id, code, name, is_paid, created_at, policy_config) VALUES ($1, $2, $3, $4, $5, $6, $7)")
        .bind::<diesel::sql_types::Uuid, _>(lt_id)
        .bind::<diesel::sql_types::Uuid, _>(ws_id)
        .bind::<diesel::sql_types::Text, _>("vacation")
        .bind::<diesel::sql_types::Text, _>("Vacation")
        .bind::<diesel::sql_types::Bool, _>(true)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Jsonb, _>(serde_json::json!({}))
        .execute(&mut *conn)
        .await
        .expect("insert leave type");

    // 1. Check leave type existence and name
    assert!(repo.check_leave_type_exists(&token, lt_id).await.unwrap());
    assert_eq!(
        repo.get_leave_type_name(&token, lt_id).await.unwrap(),
        Some("Vacation".to_string())
    );
    assert!(!repo.get_leave_types(&token).await.unwrap().is_empty());

    // 2. Create leave request (normal request)
    let request_id = Uuid::new_v4();
    let request = LeaveRequest::new(NewLeaveRequestParams {
        id: request_id,
        employment_id,
        leave_type_id: lt_id,
        start_date: NaiveDate::from_ymd_opt(2023, 1, 15).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2023, 1, 16).unwrap(),
        reason: Some("vacation trip".to_string()),
        status: LeaveStatus::Pending,
        duration_type: crate::models::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        requested_by_user_id: Some(Uuid::new_v4()),
        approved_by_user_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        attachment_file_ids: None,
    });
    repo.create_leave_request(&token, &request).await.unwrap();

    // 3. Query leave request by ID
    let fetched = repo
        .get_leave_request_by_id(&token, request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id(), request_id);
    assert_eq!(fetched.status(), LeaveStatus::Pending);

    // 4. Update leave request status to PreApproved / Approved
    repo.update_leave_request_status(
        &token,
        request_id,
        LeaveStatus::Approved,
        Some(Uuid::new_v4()),
    )
    .await
    .unwrap();
    let fetched_approved = repo
        .get_leave_request_by_id(&token, request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched_approved.status(), LeaveStatus::Approved);

    // 5. Get employee and workspace leave requests
    let emp_requests = repo
        .get_employee_leave_requests(&token, employee_id)
        .await
        .unwrap();
    assert_eq!(emp_requests.len(), 1);
    let ws_requests = repo
        .get_workspace_leave_requests(&token, Some(LeaveStatus::Approved))
        .await
        .unwrap();
    assert!(!ws_requests.is_empty());

    // 6. Get approved leave breakdown
    let breakdown = repo
        .get_approved_leave_breakdown(
            employee_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
            &token,
        )
        .await
        .unwrap();
    assert_eq!(breakdown.total_days, 2.0);

    // 7. Get employee department for request
    let dept = repo
        .get_employee_department_for_request(request_id, &token)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(dept, dept_id);

    // 8. Create a system-generated leave request
    let system_req_id = Uuid::new_v4();
    let system_request = LeaveRequest::new(NewLeaveRequestParams {
        id: system_req_id,
        employment_id,
        leave_type_id: lt_id,
        start_date: NaiveDate::from_ymd_opt(2023, 1, 20).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2023, 1, 20).unwrap(),
        reason: Some("system holiday".to_string()),
        status: LeaveStatus::Approved,
        duration_type: crate::models::LeaveDurationType::FullDay,
        start_time: None,
        end_time: None,
        requested_by_user_id: None, // requested_by_user_id is None -> system generated
        approved_by_user_id: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        attachment_file_ids: None,
    });
    repo.create_leave_request(&token, &system_request)
        .await
        .unwrap();

    let sys_generated = repo
        .get_system_generated_holiday_leave(
            &token,
            employee_id,
            NaiveDate::from_ymd_opt(2023, 1, 20).unwrap(),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sys_generated.0, system_req_id);
    assert_eq!(sys_generated.1, lt_id);
    assert_eq!(sys_generated.2, "vacation");

    // 9. Unpaid holidays
    // Let's insert an unpaid holiday on 2023-01-10 (not covered by leave request)
    diesel::sql_query("INSERT INTO holidays (id, workspace_id, name, date, holiday_type, is_paid, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)")
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(ws_id)
        .bind::<diesel::sql_types::Text, _>("Unpaid Holiday")
        .bind::<diesel::sql_types::Date, _>(NaiveDate::from_ymd_opt(2023, 1, 10).unwrap())
        .bind::<diesel::sql_types::Text, _>("national")
        .bind::<diesel::sql_types::Bool, _>(false)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .execute(&mut *conn)
        .await
        .expect("insert holiday");

    let unpaid_count = repo
        .get_unpaid_holiday_days_without_leave(
            &token,
            employee_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unpaid_count, 1);

    // 10. Batch leave days, dates, periods
    let batch_days = repo
        .get_batch_leave_days(
            &token,
            &[employee_id],
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(batch_days.get(&employee_id), Some(&3)); // 2 days vacation + 1 day system holiday

    let approved_dates = repo
        .get_approved_leave_dates(
            &token,
            &[employee_id],
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await
        .unwrap();
    assert!(approved_dates.contains(&(employee_id, NaiveDate::from_ymd_opt(2023, 1, 15).unwrap())));
    assert!(approved_dates.contains(&(employee_id, NaiveDate::from_ymd_opt(2023, 1, 16).unwrap())));
    assert!(approved_dates.contains(&(employee_id, NaiveDate::from_ymd_opt(2023, 1, 20).unwrap())));

    let periods = repo
        .get_approved_leave_periods(
            &token,
            employee_id,
            NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2023, 1, 31).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(periods.len(), 2);

    // 11. Leave history and cancel
    let history = repo
        .get_employee_leave_history_requests(&token, employee_id, 2023, Some(lt_id))
        .await
        .unwrap();
    assert_eq!(history.len(), 2);

    repo.cancel_leave_request(&token, request_id).await.unwrap();
    let fetched_cancelled = repo
        .get_leave_request_by_id(&token, request_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched_cancelled.status(), LeaveStatus::Cancelled);
}
