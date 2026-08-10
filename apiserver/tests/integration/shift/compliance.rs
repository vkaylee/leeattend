//! Compliance engine integration tests — overlap, rest period, override flow

use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use chrono::{NaiveTime, Utc};
use reqwest::StatusCode;
use uuid::Uuid;

/// Assign two different shifts on the same day → 400 (hard overlap violation).
#[tokio::test]
async fn test_compliance_overlap_blocked() {
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
    let date = Utc::now().date_naive();

    // Create two shifts that overlap (both cover 09:00–17:00)
    let shift_a = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Shift A")
        .create()
        .await;

    let shift_b = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Shift B")
        .create()
        .await;

    // Assign first shift
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": date.to_string(), "shift_id": shift_a}],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::ACCEPTED,
        "First assignment should succeed"
    );

    // Attempt to assign second, overlapping shift → must be rejected
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": date.to_string(), "shift_id": shift_b}],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "Overlapping shift assignment must return 400"
    );
}

/// Assign shifts with insufficient rest period → 409 Conflict with violations payload.
#[tokio::test]
async fn test_compliance_rest_period_warning() {
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
    let today = Utc::now().date_naive();
    let tomorrow = today + chrono::Duration::days(1);

    // Create a late-night shift (22:00–06:00) and an early morning shift (08:00–16:00)
    // Gap: 06:00 to 08:00 = 2h rest → insufficient for 12h rule
    let night_shift = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Night Shift")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    let morning_shift = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Morning Shift")
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        )
        .create()
        .await;

    // Assign night shift today
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": today.to_string(), "shift_id": night_shift}],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);

    // Assign morning shift tomorrow → should warn about rest period
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": tomorrow.to_string(), "shift_id": morning_shift}],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::CONFLICT,
        "Insufficient rest period must return 409"
    );

    let body: serde_json::Value = res.json().await.unwrap();
    let error = &body["error"];
    assert!(
        error.get("retry").is_some(),
        "409 response must contain retry payload inside 'error' key"
    );
    assert!(
        error["retry"]["requires_override"] == true,
        "Retry payload must indicate override is required"
    );
}

/// Soft violation with override reason → 202 Accepted (manager override flow).
#[tokio::test]
async fn test_compliance_override_with_reason() {
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
    let today = Utc::now().date_naive();
    let tomorrow = today + chrono::Duration::days(1);

    // Same setup as rest period test
    let night_shift = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Night Shift")
        .with_times(
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(6, 0, 0).unwrap(),
        )
        .create()
        .await;

    let morning_shift = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Morning Shift")
        .with_times(
            NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
        )
        .create()
        .await;

    // Assign night shift today
    client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": today.to_string(), "shift_id": night_shift}],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();

    // Assign morning shift tomorrow WITH override
    let res = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": tomorrow.to_string(), "shift_id": morning_shift}],
            "allow_warnings": true,
            "override_reason": "Critical production deadline — approved by HR director"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::ACCEPTED,
        "Override with reason must succeed"
    );
}

/// A single bulk request can contain multiple shift groups and day-off entries.
#[tokio::test]
async fn test_bulk_compliance_validates_multiple_shift_groups() {
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
    let today = Utc::now().date_naive();
    let later = today + chrono::Duration::days(2);
    let shift_a = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Batch Shift A")
        .create()
        .await;
    let shift_b = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Batch Shift B")
        .create()
        .await;
    let shift_conflict = factory
        .shift()
        .with_workspace(workspace_id)
        .with_name("Batch Shift Conflict")
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .create()
        .await;

    let seed = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [{"date": later.to_string(), "shift_id": shift_b}],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(seed.status(), StatusCode::ACCEPTED);

    let response = client
        .post(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters/bulk"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .json(&serde_json::json!({
            "items": [
                {"date": today.to_string(), "shift_id": shift_a},
                {"date": (today + chrono::Duration::days(1)).to_string(), "shift_id": null},
                {"date": later.to_string(), "shift_id": shift_conflict}
            ],
            "allow_warnings": false
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "a conflict in the second shift group must reject the whole batch"
    );

    let roster_response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/employees/{emp_id}/rosters?start_date={today}&end_date={later}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(roster_response.status(), StatusCode::OK);
    let roster_body: serde_json::Value = roster_response.json().await.unwrap();
    let rosters = roster_body["data"]
        .as_array()
        .expect("roster data must be an array");
    assert_eq!(
        rosters.len(),
        1,
        "failed validation must not partially persist shift A"
    );
}
