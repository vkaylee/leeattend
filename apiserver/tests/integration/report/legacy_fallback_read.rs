//! Legacy fallback integration test.
//!
//! Verifies that GET /reports/attendance?type=daily still works when
//! `effective_rules_snapshot IS NULL`, falling back to the legacy
//! recomputation path.

use crate::common::app::spawn_app;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;
use uuid::Uuid;

// ── Step 8c ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_report_legacy_fallback_when_snapshot_is_null() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    let emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Legacy Fallback Test")
        .with_employee_id_number("LEGACY-001")
        .create()
        .await;

    let today = Utc::now().date_naive();

    factory
        .shift()
        .with_workspace(workspace_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, today)
        .await;

    let employment_id: Uuid = crate::test_query_scalar_uuid!(
        &pool.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        workspace_id
    );

    // Insert daily_summary row with NULL effective_rules_snapshot.
    // The report service must fall back to the legacy recomputation path.
    crate::test_execute!(
        &pool.api,
        "INSERT INTO attendance_daily_summaries \
         (workspace_id, employment_id, date, actual_work_minutes, system_detected_ot_minutes, \
          ot_status, is_holiday, effective_rules_snapshot) \
         VALUES ($1, $2, $3, 480, 0, 'none', false, NULL) \
         ON CONFLICT (employment_id, date) \
         DO UPDATE SET effective_rules_snapshot = NULL",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Date,
        today
    );

    // Add a matching time_entry so the report path produces a valid row.
    let punch_in = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 2, 0, 0)
        .unwrap();
    let punch_out = Utc
        .with_ymd_and_hms(today.year(), today.month(), today.day(), 10, 0, 0)
        .unwrap();
    factory
        .time_entry()
        .with_employee(emp_id)
        .with_punch_in(punch_in)
        .with_punch_out(punch_out)
        .with_entry_type("work")
        .create()
        .await;

    // Call GET /reports/attendance?type=daily with employment_id to directly
    // target our employee.
    let response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}&employment_id={employment_id}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .expect("GET /reports/attendance");

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Report must succeed even when snapshot is NULL (legacy path)"
    );

    let body: serde_json::Value = response.json().await.expect("parse response body");
    let data = body["data"].as_array().expect("response has data array");

    // Our employee should be directly included via employment_id filter.
    assert!(!data.is_empty(), "report must return at least one row");

    let our_row = &data[0];

    // grace_period_minutes should be present (from legacy recomputation).
    // The exact value comes from PolicyResolver → AttendanceRules defaults + conditions.
    let actual_grace = our_row["grace_period_minutes"]
        .as_i64()
        .expect("grace_period_minutes must be present in legacy path");

    // Legacy path should return the DEFAULT_GRACE_PERIOD_MINUTES (15) or the
    // shift-level grace period. As long as it's a valid value > 0, the legacy
    // fallback is working correctly.
    assert!(
        actual_grace > 0,
        "grace_period_minutes must be positive (got {actual_grace}) from legacy path"
    );
}
