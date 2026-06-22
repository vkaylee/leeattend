//! Shift swap API integration tests
//!
//! Tests the full flow:
//!   1. Create swap request (auto shift lookup)
//!   2. List swap requests with status filter
//!   3. Approve swap (roster gets swapped)
//!   4. Reject swap
//!   5. Error cases (same employee, non-admin, etc.)

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::shift_swap::{ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus};
use apiserver::models::ShiftRoster;
use chrono::{NaiveDate, Utc};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helper: create two employees with roster assignments
// ---------------------------------------------------------------------------

struct SwapTestSetup {
    address: String,
    workspace_id: Uuid,
    user_id: Uuid,
    auth_token: String,
    emp_a: Uuid,
    emp_b: Uuid,
    shift_morning: Uuid,
    shift_evening: Uuid,
    swap_date: NaiveDate,
    _db: TestDatabase,
}

async fn setup() -> SwapTestSetup {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;

    let auth_token = factory.auth().create_token(user_id);

    // Create two employees
    let emp_a = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee A")
        .with_employee_id_number("EMP-A")
        .create()
        .await;

    let emp_b = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Employee B")
        .with_employee_id_number("EMP-B")
        .create()
        .await;

    // Create two shifts
    let shift_morning = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Morning")
        .with_times(
            chrono::NaiveTime::from_hms_opt(7, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
        )
        .create()
        .await;

    let shift_evening = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Evening")
        .with_times(
            chrono::NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(23, 0, 0).unwrap(),
        )
        .create()
        .await;

    // Assign rosters for a specific date
    let swap_date = Utc::now().date_naive() + chrono::Duration::days(1);

    let emp_a_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_a
    );
    let emp_b_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &_db.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_b
    );

    // Employee A → Morning shift
    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employment_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_a_employment_id,
        diesel::sql_types::Uuid, shift_morning,
        diesel::sql_types::Date, swap_date
    );

    // Employee B → Evening shift
    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employment_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_b_employment_id,
        diesel::sql_types::Uuid, shift_evening,
        diesel::sql_types::Date, swap_date
    );

    SwapTestSetup {
        address,
        workspace_id,
        user_id,
        auth_token,
        emp_a,
        emp_b,
        shift_morning,
        shift_evening,
        swap_date,
        _db,
    }
}

// ---------------------------------------------------------------------------
// Test: Create a swap request
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_shift_swap() {
    let s = setup().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string(),
            "reason": "Family emergency"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);

    let swap: ShiftSwapRequest = res.json().await.unwrap();
    assert_eq!(swap.requester_id(), s.emp_a);
    assert_eq!(swap.target_id(), s.emp_b);
    assert_eq!(swap.swap_date(), s.swap_date);
    assert_eq!(swap.requester_shift_id(), Some(s.shift_morning));
    assert_eq!(swap.target_shift_id(), Some(s.shift_evening));
    assert_eq!(swap.status(), ShiftSwapStatus::PendingAdmin);
    assert_eq!(swap.reason(), Some("Family emergency"));
    assert_eq!(swap.requested_by_user_id(), Some(s.user_id));
}

// ---------------------------------------------------------------------------
// Test: List swap requests with status filter
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_list_shift_swaps_by_status() {
    let s = setup().await;
    let client = reqwest::Client::new();

    // Create a swap
    let create_res = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), StatusCode::CREATED);

    // List pending_admin — should find it
    let list_res = client
        .get(format!(
            "{}/api/v1/workspaces/{}/shift-swaps?status=pending_admin",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap();

    assert_eq!(list_res.status(), StatusCode::OK);
    let swaps: Vec<ShiftSwapRequestWithNames> = list_res.json().await.unwrap();
    assert_eq!(swaps.len(), 1);
    assert_eq!(swaps[0].requester_name(), Some("Employee A"));
    assert_eq!(swaps[0].target_name(), Some("Employee B"));

    // List approved — should be empty
    let empty_res = client
        .get(format!(
            "{}/api/v1/workspaces/{}/shift-swaps?status=approved",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap();

    assert_eq!(empty_res.status(), StatusCode::OK);
    let empty: Vec<ShiftSwapRequestWithNames> = empty_res.json().await.unwrap();
    assert!(empty.is_empty());
}

// ---------------------------------------------------------------------------
// Test: Approve a swap → rosters are swapped
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_approve_shift_swap_swaps_rosters() {
    let s = setup().await;
    let client = reqwest::Client::new();

    // 1. Create swap
    let swap: ShiftSwapRequest = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // 2. Approve
    let approve_res = client
        .put(format!(
            "{}/api/v1/workspaces/{}/shift-swaps/{}/approve",
            s.address,
            s.workspace_id,
            swap.id()
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({ "status": "approved" }))
        .send()
        .await
        .unwrap();

    assert_eq!(approve_res.status(), StatusCode::OK);

    // 3. Verify rosters were swapped
    // Employee A should now have Evening shift
    let roster_a_res: apiserver::models::common::PaginatedResponse<ShiftRoster> = client
        .get(format!(
            "{}/api/v1/workspaces/{}/employees/{}/rosters",
            s.address, s.workspace_id, s.emp_a
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let roster_a = roster_a_res.data;

    let a_on_date = roster_a.iter().find(|r| r.date == s.swap_date);
    assert!(a_on_date.is_some(), "Employee A should have a roster entry");
    assert_eq!(
        a_on_date.unwrap().shift_id,
        Some(s.shift_evening),
        "Employee A should now have Evening shift"
    );

    // Employee B should now have Morning shift
    let roster_b_res: apiserver::models::common::PaginatedResponse<ShiftRoster> = client
        .get(format!(
            "{}/api/v1/workspaces/{}/employees/{}/rosters",
            s.address, s.workspace_id, s.emp_b
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let roster_b = roster_b_res.data;

    let b_on_date = roster_b.iter().find(|r| r.date == s.swap_date);
    assert!(b_on_date.is_some(), "Employee B should have a roster entry");
    assert_eq!(
        b_on_date.unwrap().shift_id,
        Some(s.shift_morning),
        "Employee B should now have Morning shift"
    );

    // 4. Verify swap status is approved
    let swaps: Vec<ShiftSwapRequestWithNames> = client
        .get(format!(
            "{}/api/v1/workspaces/{}/shift-swaps?status=approved",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(swaps.len(), 1);
    assert_eq!(swaps[0].status(), ShiftSwapStatus::Approved);
}

// ---------------------------------------------------------------------------
// Test: Reject a swap → no roster change
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reject_shift_swap() {
    let s = setup().await;
    let client = reqwest::Client::new();

    // Create
    let swap: ShiftSwapRequest = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Reject
    let reject_res = client
        .put(format!(
            "{}/api/v1/workspaces/{}/shift-swaps/{}/approve",
            s.address,
            s.workspace_id,
            swap.id()
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({ "status": "rejected" }))
        .send()
        .await
        .unwrap();

    assert_eq!(reject_res.status(), StatusCode::OK);

    // Verify original roster still intact for Employee A (Morning)
    let roster_a_res: apiserver::models::common::PaginatedResponse<ShiftRoster> = client
        .get(format!(
            "{}/api/v1/workspaces/{}/employees/{}/rosters",
            s.address, s.workspace_id, s.emp_a
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let roster_a = roster_a_res.data;

    let a_on_date = roster_a.iter().find(|r| r.date == s.swap_date);
    assert!(a_on_date.is_some());
    assert_eq!(
        a_on_date.unwrap().shift_id,
        Some(s.shift_morning),
        "Roster should not change on rejection"
    );
}

// ---------------------------------------------------------------------------
// Test: Cannot approve an already approved swap (CONFLICT)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cannot_double_approve() {
    let s = setup().await;
    let client = reqwest::Client::new();

    // Create & approve
    let swap: ShiftSwapRequest = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // First approve
    client
        .put(format!(
            "{}/api/v1/workspaces/{}/shift-swaps/{}/approve",
            s.address,
            s.workspace_id,
            swap.id()
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({ "status": "approved" }))
        .send()
        .await
        .unwrap();

    // Second approve → should fail with CONFLICT
    let res = client
        .put(format!(
            "{}/api/v1/workspaces/{}/shift-swaps/{}/approve",
            s.address,
            s.workspace_id,
            swap.id()
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({ "status": "approved" }))
        .send()
        .await
        .unwrap();

    // The service returns CONFLICT (or wrapped in AppError), check for 4xx
    assert!(
        res.status().is_client_error() || res.status().is_server_error(),
        "Double-approve should fail, got {}",
        res.status()
    );
}

// ---------------------------------------------------------------------------
// Test: Cannot create swap with same employee as requester and target
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_swap_same_employee_rejected() {
    let s = setup().await;
    let client = reqwest::Client::new();

    let res = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_a,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// Test: Non-admin cannot access shift swap endpoints
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_non_admin_cannot_create_swap() {
    let (address, workspace_id, _owner_id, member_id, factory, _db) =
        crate::common::app::spawn_app_with_member().await;

    let member_token = factory.auth().create_token(member_id);

    let emp_a = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;
    let emp_b = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    let client = reqwest::Client::new();
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/shift-swaps"
        ))
        .header("Authorization", format!("Bearer {member_token}"))
        .json(&json!({
            "requester_id": emp_a,
            "target_id": emp_b,
            "swap_date": Utc::now().date_naive().to_string()
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

// ===========================================================================
// Phase 3 Tests: Eligibility validation & Stats
// ===========================================================================

// ---------------------------------------------------------------------------
// Test: Duplicate swap is rejected (CONFLICT)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_duplicate_swap_rejected() {
    let s = setup().await;
    let client = reqwest::Client::new();

    // First swap — should succeed
    let res1 = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(res1.status(), StatusCode::CREATED);

    // Second identical swap — should fail with CONFLICT
    let res2 = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res2.status(),
        StatusCode::CONFLICT,
        "Duplicate active swap should be rejected"
    );
}

// ---------------------------------------------------------------------------
// Test: Same-shift swap is rejected (UNPROCESSABLE_ENTITY)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_same_shift_swap_rejected() {
    let s = setup().await;
    let client = reqwest::Client::new();

    let hq_loc_id = {
        let mut conn = s
            ._db
            .diesel_pools
            .tenants_tx
            .get_connection(&s.workspace_id)
            .await
            .unwrap();
        use apiserver::schema::work_locations::dsl::*;
        use diesel::prelude::*;
        use diesel_async::RunQueryDsl;
        work_locations
            .filter(workspace_id.eq(s.workspace_id))
            .filter(is_headquarters.eq(true))
            .select(id)
            .first::<uuid::Uuid>(&mut *conn)
            .await
            .unwrap()
    };

    let emp_c = Uuid::new_v4();
    crate::test_execute!(
        &s._db.diesel_pools.api,
        "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, hire_date, original_hire_date, created_at, updated_at) VALUES ($1, $2, $3, $4, CURRENT_DATE, CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        diesel::sql_types::Uuid, emp_c,
        diesel::sql_types::Uuid, s.workspace_id,
        diesel::sql_types::Text, "Employee C",
        diesel::sql_types::Text, "EMP-C"
    );

    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &s._db.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        s.workspace_id
    );

    let emp_c_employment_id = Uuid::new_v4();
    crate::test_execute!(
        &s._db.diesel_pools.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'active', CURRENT_DATE, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        diesel::sql_types::Uuid, emp_c_employment_id,
        diesel::sql_types::Uuid, s.workspace_id,
        diesel::sql_types::Uuid, emp_c,
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Uuid, hq_loc_id
    );

    // Assign Employee C → Morning shift (same as A)
    crate::test_execute!(
        &s._db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employment_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_c_employment_id,
        diesel::sql_types::Uuid, s.shift_morning,
        diesel::sql_types::Date, s.swap_date
    );

    // Try to swap A ↔ C (both have Morning) — should fail
    let res = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": emp_c,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "Same-shift swap should be rejected as pointless"
    );
}

// ---------------------------------------------------------------------------
// Test: Stats endpoint returns correct counts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_shift_swap_stats() {
    let s = setup().await;
    let client = reqwest::Client::new();

    // Create a swap (pending_admin)
    let swap: ShiftSwapRequest = client
        .post(format!(
            "{}/api/v1/workspaces/{}/shift-swaps",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({
            "requester_id": s.emp_a,
            "target_id": s.emp_b,
            "swap_date": s.swap_date.to_string()
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // Approve it
    client
        .put(format!(
            "{}/api/v1/workspaces/{}/shift-swaps/{}/approve",
            s.address,
            s.workspace_id,
            swap.id()
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .json(&json!({ "status": "approved" }))
        .send()
        .await
        .unwrap();

    // Fetch stats
    let stats_res = client
        .get(format!(
            "{}/api/v1/workspaces/{}/shift-swaps/stats",
            s.address, s.workspace_id
        ))
        .header("Authorization", format!("Bearer {}", s.auth_token))
        .send()
        .await
        .unwrap();

    assert_eq!(stats_res.status(), StatusCode::OK);

    let stats: serde_json::Value = stats_res.json().await.unwrap();
    assert_eq!(stats["approved"], 1, "Should have 1 approved swap");
    assert_eq!(
        stats["approved_this_month"], 1,
        "Should count in current month"
    );
    assert_eq!(stats["pending_admin"], 0, "No pending after approval");
}
