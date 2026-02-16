//! Roster API integration tests

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{BulkRosterRequest, RosterItem, ShiftRoster};
use reqwest::StatusCode;
use chrono::Utc;
use uuid::Uuid;

#[tokio::test]
async fn test_bulk_upsert_roster_api() {
    let (address, workspace_id, user_id, factory, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) = spawn_app_with_factory().await;
    let client = reqwest::Client::new();
    let auth_token = factory.auth().create_token(user_id);
    let emp_id = factory.employee().with_workspace(workspace_id).create().await;

    // 1. Create a shift
    let shift_id = factory.shift()
        .with_workspace(workspace_id)
        .with_name("Morning Shift")
        .create().await;

    // 2. Prepare Bulk Upsert Request
    let date = Utc::now().date_naive();
    let req = BulkRosterRequest {
        items: vec![
            RosterItem {
                date,
                shift_id: Some(shift_id),
                notes: None,
            }
        ],
    };

    // 3. Send Request
    let res = client
        .post(format!("{}/api/workspaces/{}/employees/{}/rosters/bulk", address, workspace_id, emp_id))
        .header("Authorization", format!("Bearer {}", auth_token))
        .json(&req)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 4. Verify via GET (handler returns no body, just status code)
    let get_res = client
        .get(format!("{}/api/workspaces/{}/employees/{}/rosters", address, workspace_id, emp_id))
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .unwrap();

    assert_eq!(get_res.status(), StatusCode::OK);
    let rosters: Vec<ShiftRoster> = get_res.json().await.unwrap();
    assert_eq!(rosters.len(), 1);
    assert_eq!(rosters[0].shift_id, Some(shift_id));
}
