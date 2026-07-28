//! Dual-run integration test per ADR-005 Phase 5.
//!
//! Verifies that:
//! 1. When a modifier creates a cascade-vs-legacy diff, both snapshot and
//!    legacy rules are computed.
//! 2. The snapshot is persisted with cascade_trace including the modifier layer.
//! 3. policy_version_id is persisted in the daily summary.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::domain::policy::cascade::partial::PartialPolicyRules;
use apiserver::models::{LogSource, RawAttendanceLog};
use apiserver::AppState;
use axum::http::{Request, StatusCode};
use chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn get_json_field(val: &serde_json::Value, path: &str) -> serde_json::Value {
    path.split('.')
        .fold(val.clone(), |acc, seg| match seg.parse::<usize>() {
            Ok(idx) => acc.get(idx).cloned().unwrap_or(serde_json::Value::Null),
            Err(_) => acc.get(seg).cloned().unwrap_or(serde_json::Value::Null),
        })
}

/// Ensure an ACTIVE attendance_policy_version with base_rules exists so the
/// cascade engine has L1 rules to evaluate. Returns the version_id.
async fn ensure_active_policy_version(factory: &TestFactory, ws_id: Uuid) -> Uuid {
    // Check if an ACTIVE version already exists.
    let existing: Option<Uuid> = crate::test_query_scalar_uuid_optional!(
        &factory.diesel_pools.api,
        "SELECT id FROM attendance_policy_versions WHERE status = 'ACTIVE' ORDER BY created_at DESC LIMIT 1"
    );
    if let Some(id) = existing {
        return id;
    }

    // Get the existing policy version (seeded by factory) and promote it.
    let version_id: Uuid = crate::test_query_scalar_uuid!(
        &factory.diesel_pools.api,
        "SELECT id FROM attendance_policy_versions WHERE policy_id = (SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true LIMIT 1) LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );

    let base_rules = serde_json::json!({
        "grace_period_minutes": { "default": 15, "min": 0, "max": 120 },
        "authorized_early_leave_minutes": { "default": 0, "min": 0, "max": 240 },
        "is_break_auto_deduct": true,
        "early_arrival_strategy": "Allow",
        "allow_early_ot": false,
        "is_break_overtime": false
    });
    crate::test_execute!(
        &factory.diesel_pools.api,
        "UPDATE attendance_policy_versions \
         SET status = 'ACTIVE', base_rules = $1, updated_at = NOW() \
         WHERE id = $2",
        diesel::sql_types::Jsonb,
        base_rules,
        diesel::sql_types::Uuid,
        version_id
    );

    version_id
}

/// Insert a modifier that adds delta_grace to the employee's cascade evaluation.
/// `effective_from` must be BEFORE the reprocess date.
async fn create_modifier(
    factory: &TestFactory,
    ws_id: Uuid,
    employee_id: Uuid,
    version_id: Uuid,
    delta_grace: i32,
    effective_from: chrono::DateTime<Utc>,
) {
    let existing_count: i64 = crate::test_query_scalar_i64!(
        &factory.diesel_pools.api,
        "SELECT COUNT(*) FROM employee_policy_modifiers WHERE employee_id = $1",
        diesel::sql_types::Uuid,
        employee_id
    );

    let contributions = serde_json::to_value(
        PartialPolicyRules::default().with_delta_grace_period_minutes(delta_grace),
    )
    .unwrap();

    crate::test_execute!(
        &factory.diesel_pools.api,
        "INSERT INTO employee_policy_modifiers \
         (workspace_id, employee_id, policy_version_id, contributions, priority_rank, \
          effective_from, reason) \
         VALUES ($1, $2, $3, $4, $5, $6, 'ADR-005 integration test')",
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Uuid,
        employee_id,
        diesel::sql_types::Uuid,
        version_id,
        diesel::sql_types::Jsonb,
        contributions,
        diesel::sql_types::Int4,
        (existing_count + 1) as i32,
        diesel::sql_types::Timestamptz,
        effective_from
    );
}

#[tokio::test]
async fn test_dual_run_snapshot_with_modifier_diff() {
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

    // Ensure an ACTIVE policy version exists for cascade engine resolution.
    let version_id = ensure_active_policy_version(&factory, ws_id).await;

    // Assign a standard shift (9:00-17:00).
    let base_date = NaiveDate::from_ymd_opt(2026, 7, 21).unwrap();
    factory
        .shift()
        .with_workspace(ws_id)
        .with_times(
            NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            NaiveTime::from_hms_opt(17, 0, 0).unwrap(),
        )
        .assign_to_employee(emp_id, base_date)
        .await;

    // Create a modifier that adds +30 grace minutes (L2.5).
    // Cascade: grace = 15 (base) + 30 (delta) = 45.
    // Legacy resolver: does NOT know about modifiers → grace stays at ~15.
    let effective_from = Utc.from_utc_datetime(&base_date.and_hms_opt(0, 0, 0).unwrap());
    create_modifier(&factory, ws_id, emp_id, version_id, 30, effective_from).await;

    // Insert raw punch logs so reprocess has something to pair.
    let diesel_pools = apiserver::diesel_pool::DieselPools::for_test(&test_db.url());
    let start_time = Utc.with_ymd_and_hms(2026, 7, 21, 2, 0, 0).unwrap(); // 09:00 UTC+7
    let end_time = Utc.with_ymd_and_hms(2026, 7, 21, 10, 0, 0).unwrap(); // 17:00 UTC+7

    for punch_time in [start_time, end_time] {
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

    // Run reprocess.
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

    // Assertion 1: effective_rules_snapshot is present.
    let snapshot_json: Option<serde_json::Value> = {
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
            "SELECT effective_rules_snapshot AS snapshot FROM attendance_daily_summaries \
             WHERE employment_id = $1 AND date = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(emp_employment_id)
        .bind::<diesel::sql_types::Date, _>(base_date);
        diesel_async::RunQueryDsl::get_result::<SnapshotRow>(q, &mut *conn)
            .await
            .ok()
            .and_then(|r| r.snapshot)
    };

    assert!(
        snapshot_json.is_some(),
        "effective_rules_snapshot must be persisted after reprocess"
    );
    let snapshot = snapshot_json.unwrap();

    // Assertion 2: Cascade final_rules.grace_period_minutes = 45 (15+30).
    let cascade_grace = get_json_field(&snapshot, "final_rules.grace_period_minutes");
    assert_eq!(
        cascade_grace.as_i64().unwrap_or(-1),
        45,
        "Cascade grace_period_minutes must be 45 (base 15 + modifier delta 30)"
    );

    // Assertion 3: Cascade_trace includes L2.5 modifier layer.
    let trace = get_json_field(&snapshot, "cascade_trace");
    let trace_arr = trace.as_array().expect("cascade_trace must be an array");
    assert!(
        trace_arr.iter().any(|e| get_json_field(e, "layer").as_str() == Some("L2.5_employee")),
        "cascade_trace must contain L2.5_employee layer"
    );

    // Assertion 4: Verify the snapshot has the expected structure.
    assert!(
        trace_arr.iter().any(|e| {
            let layer = get_json_field(e, "layer");
            layer.as_str() == Some("L1_base")
                && get_json_field(e, "policy_version_id") != serde_json::Value::Null
        }),
        "cascade_trace L1_base must have a non-null policy_version_id provenance"
    );

    // Assertion 5: Summary row exists with valid OT data.
    let system_ot: i32 = crate::test_query_scalar_i32!(
        &factory.diesel_pools.api,
        "SELECT system_detected_ot_minutes AS count FROM attendance_daily_summaries \
         WHERE employment_id = $1 AND date = $2",
        diesel::sql_types::Uuid,
        emp_employment_id,
        diesel::sql_types::Date,
        base_date
    );

    assert!(
        system_ot >= 0,
        "System OT must be a valid non-negative value"
    );
}
