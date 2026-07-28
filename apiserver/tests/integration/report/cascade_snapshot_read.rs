//! Report reads from cascade snapshot integration test.
//!
//! Verifies that GET /reports/attendance?type=daily uses
//! `effective_rules_snapshot.final_rules.grace_period_minutes`
//! when a snapshot is present, instead of the legacy recompute value.

use crate::common::app::spawn_app;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Utc};
use reqwest::StatusCode;
use uuid::Uuid;

// ── Step 8b ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_report_reads_grace_period_from_cascade_snapshot() {
    let (address, test_db) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();
    let factory = TestFactory::new(pool.clone());
    let client = reqwest::Client::new();

    let user_id = factory.user().create().await;
    let workspace_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = factory.auth().create_token(user_id);

    // Create employee + shift (needed for the report to find the employee in the
    // roster/schedule layer and produce a row).
    let emp_id = factory
        .employee()
        .with_workspace(workspace_id)
        .with_full_name("Snapshot Read Test")
        .with_employee_id_number("SNAP-001")
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

    // Pick a distinct grace_period_minutes value (30) that differs from DEFAULT (15).
    let snapshot_json = serde_json::json!({
        "final_rules": {
            "grace_period_minutes": 30,
            "authorized_early_leave_minutes": 0,
            "is_early_leave_paid": false,
            "early_arrival_strategy": "block",
            "is_break_auto_deduct": true,
            "is_break_overtime": false,
            "allow_early_ot": false
        },
        "cascade_trace": [
            {
                "layer": "L1_base",
                "contributions": {}
            },
            {
                "layer": "L2a_day_type",
                "contributions": {}
            },
            {
                "layer": "L2b_shift_template",
                "contributions": {}
            },
            {
                "layer": "L2c_shift_version",
                "contributions": {}
            },
            {
                "layer": "L3_exception",
                "contributions": {}
            }
        ]
    });

    // Get the ACTIVE policy_version_id for the FK.
    let policy_version_id: Option<Uuid> = crate::test_query_scalar_uuid_optional!(
        &pool.api,
        "SELECT id FROM attendance_policy_versions WHERE workspace_id = $1 AND status = 'ACTIVE' ORDER BY created_at DESC LIMIT 1",
        diesel::sql_types::Uuid,
        workspace_id
    );

    // Insert a daily_summary row with our snapshot JSON so the report service
    // finds it via the dual-path logic.
    crate::test_execute!(
        &pool.api,
        "INSERT INTO attendance_daily_summaries \
         (workspace_id, employment_id, date, actual_work_minutes, system_detected_ot_minutes, \
          ot_status, is_holiday, effective_rules_snapshot, policy_version_id) \
         VALUES ($1, $2, $3, 480, 0, 'none', false, $4, $5) \
         ON CONFLICT (employment_id, date) \
         DO UPDATE SET effective_rules_snapshot = EXCLUDED.effective_rules_snapshot, \
                       policy_version_id = EXCLUDED.policy_version_id",
        diesel::sql_types::Uuid,
        workspace_id,
        diesel::sql_types::Uuid,
        employment_id,
        diesel::sql_types::Date,
        today,
        diesel::sql_types::Jsonb,
        snapshot_json,
        diesel::sql_types::Nullable<diesel::sql_types::Uuid>,
        policy_version_id
    );

    // Add a matching time_entry so the punch-in/out cells are filled and the
    // report path produces a valid row for this employee-day.
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
    // target our employee (bypasses the paginated employee listing).
    let response = client
        .get(format!(
            "{address}/api/v1/workspaces/{workspace_id}/reports/attendance?type=daily&start_date={today}&end_date={today}&employment_id={employment_id}"
        ))
        .header("Authorization", format!("Bearer {auth_token}"))
        .send()
        .await
        .expect("GET /reports/attendance");

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.expect("parse response body");
    let data = body["data"].as_array().expect("response has data array");

    // Our employee should be directly included via employment_id filter, so
    // there should be exactly one row.
    assert!(!data.is_empty(), "report must return at least one row");

    let our_row = &data[0];

    let actual_grace = our_row["grace_period_minutes"]
        .as_i64()
        .expect("grace_period_minutes must be present and numeric");

    assert_eq!(
        actual_grace, 30,
        "grace_period_minutes must be 30 (from snapshot), NOT 15 (default). Got {actual_grace}"
    );
}
