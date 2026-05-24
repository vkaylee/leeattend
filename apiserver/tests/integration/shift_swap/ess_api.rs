//! ESS Shift Swap API integration tests
//!
//! Tests the full ESS flow:
//!   1. Create ESS swap request
//!   2. Target employee responds to swap (accept/reject)
//!   3. Requester cancels swap
//!   4. List my swaps

use crate::common::app::{spawn_app_with_factory, spawn_app_with_member};
use apiserver::models::shift_swap::{ShiftSwapRequest, ShiftSwapRequestWithNames, ShiftSwapStatus};
use apiserver::models::ShiftRoster;
use chrono::{NaiveDate, Utc};
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_ess_shift_swap_flow() {
    let (address, workspace_id, owner_id, requester_user_id, factory, _db) =
        spawn_app_with_member().await;
    let target_user_id = factory.user().create().await;

    factory
        .add_member(
            workspace_id,
            target_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let requester_token = factory.auth().create_token(requester_user_id);
    let target_token = factory.auth().create_token(target_user_id);

    // Create two employees
    let emp_a = factory
        .employee()
        .with_user(requester_user_id)
        .with_workspace(workspace_id)
        .with_full_name("Employee Requester")
        .create()
        .await;

    let emp_b = factory
        .employee()
        .with_user(target_user_id)
        .with_workspace(workspace_id)
        .with_full_name("Employee Target")
        .create()
        .await;

    // Create two shifts
    let shift_morning = factory.shift().with_workspace(workspace_id).create().await;
    let shift_evening = factory.shift().with_workspace(workspace_id).create().await;

    let swap_date = Utc::now().date_naive() + chrono::Duration::days(1);

    // Employee A → Morning shift
    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_a,
        diesel::sql_types::Uuid, shift_morning,
        diesel::sql_types::Date, swap_date
    );

    // Employee B → Evening shift
    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_b,
        diesel::sql_types::Uuid, shift_evening,
        diesel::sql_types::Date, swap_date
    );

    let client = reqwest::Client::new();

    // 1. Employee A creates swap
    let res = client
        .post(format!("{address}/api/ess/shift-swaps"))
        .header("Authorization", format!("Bearer {requester_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .json(&json!({
            "target_id": emp_b,
            "swap_date": swap_date.to_string(),
            "reason": "Need morning off"
        }))
        .send()
        .await
        .unwrap();

    let status = res.status();
    let body = res.text().await.unwrap();
    println!("Response: {}", body);
    assert_eq!(status, StatusCode::CREATED);
    let swap: ShiftSwapRequest = serde_json::from_str(&body).unwrap();
    assert_eq!(swap.status, ShiftSwapStatus::PendingTarget);

    // 2. Target rejects the swap
    let res = client
        .put(format!("{address}/api/ess/shift-swaps/{}/respond", swap.id))
        .header("Authorization", format!("Bearer {target_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .json(&json!({ "accept": false }))
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // List swaps for A
    let res = client
        .get(format!("{address}/api/ess/shift-swaps"))
        .header("Authorization", format!("Bearer {requester_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .send()
        .await
        .unwrap();
    let swaps: Vec<ShiftSwapRequestWithNames> = res.json().await.unwrap();
    assert_eq!(swaps[0].status, ShiftSwapStatus::Rejected);
}

#[tokio::test]
async fn test_ess_shift_swap_cancel_and_accept() {
    let (address, workspace_id, owner_id, requester_user_id, factory, _db) =
        spawn_app_with_member().await;
    let target_user_id = factory.user().create().await;

    factory
        .add_member(
            workspace_id,
            target_user_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let requester_token = factory.auth().create_token(requester_user_id);
    let target_token = factory.auth().create_token(target_user_id);
    let emp_a = factory
        .employee()
        .with_user(requester_user_id)
        .with_workspace(workspace_id)
        .create()
        .await;
    let emp_b = factory
        .employee()
        .with_user(target_user_id)
        .with_workspace(workspace_id)
        .create()
        .await;
    let shift_morning = factory.shift().with_workspace(workspace_id).create().await;
    let shift_evening = factory.shift().with_workspace(workspace_id).create().await;
    let swap_date = Utc::now().date_naive() + chrono::Duration::days(1);

    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_a,
        diesel::sql_types::Uuid, shift_morning,
        diesel::sql_types::Date, swap_date
    );
    crate::test_execute!(&_db.diesel_pools.api,
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'active')",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, emp_b,
        diesel::sql_types::Uuid, shift_evening,
        diesel::sql_types::Date, swap_date
    );

    let client = reqwest::Client::new();

    // 1. Employee A creates swap
    let swap: ShiftSwapRequest = client
        .post(format!("{address}/api/ess/shift-swaps"))
        .header("Authorization", format!("Bearer {requester_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .json(&json!({ "target_id": emp_b, "swap_date": swap_date.to_string() }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // 2. Employee A cancels swap
    let res = client
        .delete(format!("{address}/api/ess/shift-swaps/{}", swap.id))
        .header("Authorization", format!("Bearer {requester_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // 3. Employee A creates another swap
    let res = client
        .post(format!("{address}/api/ess/shift-swaps"))
        .header("Authorization", format!("Bearer {requester_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .json(&json!({ "target_id": emp_b, "swap_date": swap_date.to_string() }))
        .send()
        .await
        .unwrap();

    let status2 = res.status();
    let body2 = res.text().await.unwrap();
    println!("Response2: {}", body2);
    assert_eq!(status2, StatusCode::CREATED);
    let swap2: ShiftSwapRequest = serde_json::from_str(&body2).unwrap();

    // 4. Employee B accepts it
    let res = client
        .put(format!(
            "{address}/api/ess/shift-swaps/{}/respond",
            swap2.id
        ))
        .header("Authorization", format!("Bearer {target_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .json(&json!({ "accept": true }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Verify it is now PendingAdmin
    let swaps: Vec<ShiftSwapRequestWithNames> = client
        .get(format!("{address}/api/ess/shift-swaps"))
        .header("Authorization", format!("Bearer {requester_token}"))
        .header("x-workspace-id", workspace_id.to_string())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let accepted = swaps.iter().find(|s| s.id == swap2.id).unwrap();
    assert_eq!(accepted.status, ShiftSwapStatus::PendingAdmin);
}
