//! Roster API integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{BulkRosterRequest, RosterItem, ShiftRoster};
use chrono::Utc;
use reqwest::StatusCode;
use uuid::Uuid;

#[tokio::test]
async fn test_bulk_upsert_roster_api_smoke() {
    let (address, workspace_id, user_id, factory, _db): (
        String,
        Uuid,
        Uuid,
        TestFactory,
        TestDatabase,
    ) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let auth_token = factory.auth().create_token(user_id);
    let emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .create()
        .await;

    // 1. Create a shift
    let shift_id = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Morning Shift")
        .create()
        .await;

    // 2. Prepare Bulk Upsert Request
    let date = Utc::now().date_naive();
    let req = BulkRosterRequest {
        items: vec![RosterItem {
            date,
            shift_id: Some(shift_id),
            notes: None,
        }],
        employment_id: None,
        allow_warnings: false,
        override_reason: None,
    };

    // 3. Send Request
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&req)
        .send()
        .await
        .unwrap();

    // Roster write is synchronous; attendance reprocess is enqueued to the worker,
    // so the handler returns 202 Accepted instead of 200 OK.
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    // 4. Verify via GET with explicit date range to avoid TZ boundary issues
    let date_str = date.format("%Y-%m-%d").to_string();
    let get_res = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters?start_date={date_str}&end_date={date_str}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(get_res.status(), StatusCode::OK);
    let rosters_res: apiserver::models::common::PaginatedResponse<ShiftRoster> =
        get_res.json().await.unwrap();
    let rosters = rosters_res.data;
    assert_eq!(rosters.len(), 1);
    assert_eq!(rosters[0].shift_id, Some(shift_id));
}
