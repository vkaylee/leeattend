//! Cascade snapshot persistence integration test.
//!
//! Verifies that `recalculate_daily_summary` persists an
//! `EffectiveRulesSnapshot` into `attendance_daily_summaries` when a cascade
//! engine runs in shadow mode during reprocess.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{NaiveDate, TimeZone, Utc};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────────

fn get_json_field(val: &serde_json::Value, path: &str) -> serde_json::Value {
    path.split('.')
        .fold(val.clone(), |acc, seg| match seg.parse::<usize>() {
            Ok(idx) => acc.get(idx).cloned().unwrap_or(serde_json::Value::Null),
            Err(_) => acc.get(seg).cloned().unwrap_or(serde_json::Value::Null),
        })
}

// ── Step 8a ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_cascade_snapshot_persisted_after_reprocess() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    let emp_employment_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM employments WHERE employee_id = $1 AND workspace_id = $2 LIMIT 1",
        diesel::sql_types::Uuid,
        emp_id,
        diesel::sql_types::Uuid,
        ws_id
    );

    // Assign a standard shift (9:00-17:00) so the reprocess has a schedule
    // to pair punches against.
    let base_date = NaiveDate::from_ymd_opt(2026, 7, 20).unwrap();
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, base_date)
        .await;

    // Insert raw logs (UTC times → local 9:00 and 17:00 in Asia/Ho_Chi_Minh)
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let start_time = Utc.with_ymd_and_hms(2026, 7, 20, 2, 0, 0).unwrap(); // 09:00 UTC+7
    let end_time = Utc.with_ymd_and_hms(2026, 7, 20, 10, 0, 0).unwrap(); // 17:00 UTC+7

    let logs = vec![start_time, end_time];
    for punch_time in logs {
        let raw_log = RawAttendanceLog::new(
            Uuid::new_v4(),
            ws_id,
            emp_employment_id,
            punch_time,
            LogSource::Manual,
            None,
            None,
            -1,
            -1,
            Utc::now(),
        );
        crate::common::helpers::create_raw_log(&diesel_pools, &raw_log)
            .await
            .expect("create raw log");
    }

    // Fetch the policy version before reprocess (to compare later).
    let policy_version_before: Option<Uuid> = crate::test_query_scalar_uuid_optional!(
        &factory.diesel_pools.api,
        "SELECT id FROM attendance_policy_versions WHERE workspace_id = $1 AND status = 'ACTIVE' ORDER BY created_at DESC LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );

    // Call the reprocess endpoint.
    let res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/attendance/reprocess/{emp_id}?start_date={base_date}&end_date={base_date}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", user_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .expect("reprocess request");

    assert_eq!(res.status(), StatusCode::OK, "Reprocess should succeed");

    // Query the daily summary for cascade snapshot verification.
    let snapshot_json_val: Option<serde_json::Value> = {
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct SnapshotRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Jsonb>)]
            snapshot: Option<serde_json::Value>,
        }
        let mut conn = factory
            .diesel_pools
            .api
            .get()
            .await
            .expect("get connection");
        let q = diesel::sql_query(
            "SELECT effective_rules_snapshot AS snapshot FROM attendance_daily_summaries WHERE employment_id = $1 AND date = $2"
        )
        .bind::<diesel::sql_types::Uuid, _>(emp_employment_id)
        .bind::<diesel::sql_types::Date, _>(base_date);
        diesel_async::RunQueryDsl::get_result::<SnapshotRow>(q, &mut *conn)
            .await
            .ok()
            .and_then(|r| r.snapshot)
    };

    assert!(
        snapshot_json_val.is_some(),
        "effective_rules_snapshot must NOT be NULL after reprocess"
    );
    let snapshot = snapshot_json_val.unwrap();

    // Verify cascade_trace has 5 elements (L1Base → L2aDayType → L2bShiftTemplate →
    // L2cShiftVersion → L3Exception).
    let trace = get_json_field(&snapshot, "cascade_trace");
    let trace_arr = trace.as_array().expect("cascade_trace must be an array");
    assert_eq!(
        trace_arr.len(),
        5,
        "cascade_trace must have 5 layers (L1→L3); got {}",
        trace_arr.len()
    );

    // Verify layer names.
    let expected_layers = [
        "L1_base",
        "L2a_day_type",
        "L2b_shift_template",
        "L2c_shift_version",
        "L3_exception",
    ];
    for (i, expected) in expected_layers.iter().enumerate() {
        let layer = get_json_field(&trace_arr[i], "layer");
        assert_eq!(
            layer.as_str().unwrap_or(""),
            *expected,
            "cascade_trace[{i}] layer mismatch"
        );
    }

    // Verify final_rules.grace_period_minutes equals the default (15).
    let grace = get_json_field(&snapshot, "final_rules.grace_period_minutes");
    assert_eq!(
        grace.as_i64().unwrap_or(-1),
        apiserver::domain::policy::defaults::DEFAULT_GRACE_PERIOD_MINUTES as i64,
        "final_rules.grace_period_minutes must equal DEFAULT_GRACE_PERIOD_MINUTES (15)"
    );

    // Verify policy_version_id is set and matches the existing ACTIVE version.
    let version_id_col: Option<Uuid> = {
        #[derive(diesel::query_builder::QueryId, diesel::deserialize::QueryableByName)]
        struct VersionRow {
            #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
            policy_version_id: Option<Uuid>,
        }
        let mut conn = factory
            .diesel_pools
            .api
            .get()
            .await
            .expect("get connection");
        let q = diesel::sql_query(
            "SELECT policy_version_id FROM attendance_daily_summaries WHERE employment_id = $1 AND date = $2"
        )
        .bind::<diesel::sql_types::Uuid, _>(emp_employment_id)
        .bind::<diesel::sql_types::Date, _>(base_date);
        diesel_async::RunQueryDsl::get_result::<VersionRow>(q, &mut *conn)
            .await
            .ok()
            .and_then(|r| r.policy_version_id)
    };

    assert!(
        version_id_col.is_some(),
        "policy_version_id must be persisted in daily summary"
    );

    if let Some(pv_before) = policy_version_before {
        assert_eq!(
            version_id_col.unwrap(),
            pv_before,
            "persisted policy_version_id must match the ACTIVE policy version"
        );
    }
}
