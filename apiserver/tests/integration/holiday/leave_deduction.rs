//! Integration tests for holiday leave deduction feature
//!
//! Tests the 3-way holiday status in reports:
//! - `holiday_paid`: paid holiday, no punch → normal paid day off
//! - `holiday_leave`: unpaid holiday + approved leave request → leave deducted
//! - `holiday_unpaid`: unpaid holiday + NO leave request → unpaid day
//!
//! Also tests the process-leave API endpoint and CRUD with `leave_type_id`.

use crate::common::app::spawn_app;
use apiserver::models::LeaveStatus;
use chrono::{NaiveTime, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct PaginatedReport {
    data: Vec<ReportRow>,
}

#[derive(Debug, Deserialize)]
struct ReportRow {
    status: String,
    is_holiday: bool,
    holiday_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct HolidayResponse {
    id: Uuid,
    is_paid: bool,
    leave_type_id: Option<Uuid>,
}

// =============================================================================
// CRUD: leave_type_id field
// =============================================================================

/// Creating an unpaid holiday with `leave_type_id` should persist correctly
#[tokio::test]
async fn test_create_holiday_with_leave_type_id() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // Get the default vacation leave type
    let leave_type_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let today = Utc::now().date_naive();
    let body = serde_json::json!({
        "name": "Company Day Off",
        "date": today.to_string(),
        "holiday_type": "company",
        "is_paid": false,
        "leave_type_id": leave_type_id.to_string(),
    });

    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let holiday: HolidayResponse = resp.json().await.unwrap();
    assert!(!holiday.is_paid);
    assert_eq!(holiday.leave_type_id, Some(leave_type_id));
}

/// Updating a holiday to set `leave_type_id`
#[tokio::test]
async fn test_update_holiday_with_leave_type_id() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let leave_type_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let today = Utc::now().date_naive();

    // Create paid holiday first
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Flex Day",
            "date": today.to_string(),
            "is_paid": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let holiday: HolidayResponse = resp.json().await.unwrap();
    assert!(holiday.is_paid);
    assert_eq!(holiday.leave_type_id, None);

    // Update to unpaid with leave_type_id
    let resp = client
        .put(format!(
            "{address}/api/workspaces/{workspace_id}/holidays/{}",
            holiday.id
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Flex Day",
            "date": today.to_string(),
            "is_paid": false,
            "leave_type_id": leave_type_id.to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated: HolidayResponse = resp.json().await.unwrap();
    assert!(!updated.is_paid);
    assert_eq!(updated.leave_type_id, Some(leave_type_id));
}

// =============================================================================
// Report status: holiday_unpaid
// =============================================================================

/// Unpaid holiday + no leave request → status should be "`holiday_unpaid`"
#[tokio::test]
async fn test_report_status_holiday_unpaid() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();

    // Assign shift so employee is "expected"
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Create unpaid holiday (no leave_type_id, is_paid=false)
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Company Anniversary",
            "date": today.to_string(),
            "holiday_type": "company",
            "is_paid": false,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // No leave request exists → should be "holiday_unpaid"
    let report_resp = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(report_resp.status(), StatusCode::OK);

    let report: PaginatedReport = report_resp.json().await.unwrap();
    assert_eq!(report.data.len(), 1);

    let row = &report.data[0];
    assert_eq!(
        row.status, "holiday_unpaid",
        "Unpaid holiday + no leave → status should be 'holiday_unpaid'"
    );
    assert!(row.is_holiday);
    assert_eq!(row.holiday_name.as_deref(), Some("Company Anniversary"));
}

// =============================================================================
// Report status: holiday_leave
// =============================================================================

/// Unpaid holiday + approved leave request → status should be "`holiday_leave`"
#[tokio::test]
async fn test_report_status_holiday_leave_deducted() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();

    // Assign shift
    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(18, 0, 0).unwrap(),
        )
        .assign_to_employee(emp, today)
        .await;

    // Create unpaid holiday
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Team Building Day",
            "date": today.to_string(),
            "holiday_type": "company",
            "is_paid": false,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Create an approved leave request covering today
    factory
        .leave_request()
        .with_employee(emp)
        .with_dates(today, today)
        .with_status(LeaveStatus::Approved)
        .create()
        .await;

    // Should be "holiday_leave" — leave was deducted for this day
    let report_resp = client
        .get(format!(
            "{address}/api/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(report_resp.status(), StatusCode::OK);

    let report: PaginatedReport = report_resp.json().await.unwrap();
    assert_eq!(report.data.len(), 1);

    let row = &report.data[0];
    assert_eq!(
        row.status, "holiday_leave",
        "Unpaid holiday + approved leave → status should be 'holiday_leave'"
    );
    assert!(row.is_holiday);
    assert_eq!(row.holiday_name.as_deref(), Some("Team Building Day"));
}

// =============================================================================
// Process-leave API endpoint
// =============================================================================

/// POST /holidays/:id/process-leave — should create leave requests for employees
#[tokio::test]
async fn test_process_holiday_leave_api() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();

    let vacation_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Give employee leave balance (12 days)
    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    // Create unpaid holiday
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Founder's Day",
            "date": today.to_string(),
            "holiday_type": "company",
            "is_paid": false,
            "leave_type_id": vacation_id.to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let holiday: HolidayResponse = resp.json().await.unwrap();

    // Process leave deductions
    let resp = client
        .post(format!(
            "{address}/api/workspaces/{workspace_id}/holidays/{}/process-leave",
            holiday.id
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let results: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(
        !results.is_empty(),
        "Should have processed at least 1 employee"
    );

    // Verify a leave request was created for the employee
    let leave_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) as "count!" FROM leave_requests
           WHERE employee_id = $1 AND status = 'approved'
             AND start_date = $2 AND end_date = $2"#,
    )
    .bind(emp)
    .bind(today)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        leave_count, 1,
        "Should have created exactly 1 approved leave request"
    );
}

/// POST /holidays/:id/process-leave — employee with no balance → marked unpaid
#[tokio::test]
async fn test_process_holiday_leave_insufficient_balance() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();

    let vacation_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Give employee 0 days balance
    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(0)
        .create()
        .await;

    // Create unpaid holiday
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Extra Day Off",
            "date": today.to_string(),
            "holiday_type": "company",
            "is_paid": false,
            "leave_type_id": vacation_id.to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let holiday: HolidayResponse = resp.json().await.unwrap();

    // Process leave deductions
    let resp = client
        .post(format!(
            "{address}/api/workspaces/{workspace_id}/holidays/{}/process-leave",
            holiday.id
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Should NOT have created a leave request (balance insufficient)
    let leave_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) as "count!" FROM leave_requests
           WHERE employee_id = $1 AND start_date = $2"#,
    )
    .bind(emp)
    .bind(today)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        leave_count, 0,
        "No leave request should be created when balance is 0"
    );
}

/// POST /holidays/:id/process-leave — idempotency: running twice should not duplicate
#[tokio::test]
async fn test_process_holiday_leave_idempotent() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();

    let vacation_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM leave_types WHERE workspace_id = $1 AND code = 'vacation'",
    )
    .bind(workspace_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    factory
        .leave_balance()
        .with_employee(emp)
        .with_entitlement(12)
        .create()
        .await;

    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "Team Day",
            "date": today.to_string(),
            "holiday_type": "company",
            "is_paid": false,
            "leave_type_id": vacation_id.to_string(),
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let holiday: HolidayResponse = resp.json().await.unwrap();
    let hid = holiday.id;

    // Process twice
    for _ in 0..2 {
        let resp = client
            .post(format!(
                "{address}/api/workspaces/{workspace_id}/holidays/{hid}/process-leave"
            ))
            .header("Authorization", format!("Bearer {auth_token}"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Should still have exactly 1 leave request
    let leave_count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) as "count!" FROM leave_requests
           WHERE employee_id = $1 AND start_date = $2"#,
    )
    .bind(emp)
    .bind(today)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        leave_count, 1,
        "Idempotent: should still have exactly 1 leave request"
    );
}

/// Process-leave on a PAID holiday should return empty results (no deductions needed)
#[tokio::test]
async fn test_process_leave_on_paid_holiday_noop() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.pool();
    let factory = crate::common::factory::TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let _emp = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let today = Utc::now().date_naive();

    // Create PAID holiday
    let resp = client
        .post(format!("{address}/api/workspaces/{workspace_id}/holidays"))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "name": "National Day",
            "date": today.to_string(),
            "is_paid": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let holiday: HolidayResponse = resp.json().await.unwrap();

    // Process leave — should be no-op for paid holiday
    let resp = client
        .post(format!(
            "{address}/api/workspaces/{workspace_id}/holidays/{}/process-leave",
            holiday.id
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let results: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(
        results.len(),
        1,
        "Paid holiday should return a single result"
    );
    assert_eq!(
        results[0],
        serde_json::json!("Paid"),
        "Paid holiday result should be explicitly Paid"
    );
}
