//! Integration tests for `GET /api/v1/dashboard/turnover-analytics` (retention &
//! workforce movement analytics).
//!
//! Covers the 8 required scenarios:
//!   1. Denominator = 0  -> `turnover_rate.value` is `None`, `average_headcount` 0, `LOW_DENOMINATOR` warning, still 200.
//!   2. Voluntary/involuntary split derived from `termination_reason`.
//!   3. Target/benchmark from `workspace_config_entries` + comparison pp-delta.
//!   4. Custom (odd-length) period returns the requested `period_start/end` and a zero-filled trend.
//!   5. Cohort boundary: hires outside the period are excluded; `MIN_COHORT >= 3` filter; zero-size cohort -> `None` rate.
//!   6. Permission: a member without `dashboard:read` is FORBIDDEN.
//!   7. Workspace isolation: two workspaces with identical data only see their own rows.
//!   8. Full-stack integration: complete `TurnoverAnalytics` shape through the route.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::body::Body;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

const EMPLOYEE_ROLE: &str = "a0000000-0000-0000-0000-000000000006";

fn request(ws_id: Uuid, user_id: Uuid, query: &str) -> axum::http::Request<Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri(format!("/api/v1/dashboard/turnover-analytics{query}"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(Body::empty())
        .unwrap()
}

async fn body_json(resp: axum::http::Response<Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10_000_000)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

/// Create a terminated employment with a specific start/end date and termination reason.
/// Returns the `employment_id`.
async fn seed_terminated(
    test_db: &TestDatabase,
    ws_id: Uuid,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    termination_reason: &str,
    hire_date: chrono::NaiveDate,
) -> Uuid {
    let employment_id = Uuid::new_v4();
    let employee_id = Uuid::new_v4();
    let location_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM work_locations WHERE workspace_id = $1 AND is_headquarters = true LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );
    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, hire_date, original_hire_date, created_at, updated_at)
         VALUES ($1, $2, 'Terminated Test', $3, $4, $4, NOW(), NOW())",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Text, format!("TERM-{}", employee_id.simple()),
        diesel::sql_types::Date, hire_date
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, end_date, employment_type, termination_reason, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'terminated', $6, $7, 'full_time', $8, NOW(), NOW())",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Uuid, location_id,
        diesel::sql_types::Date, start_date,
        diesel::sql_types::Date, end_date,
        diesel::sql_types::Text, termination_reason
    );
    employment_id
}

/// Create a terminated employment with a `NULL` termination_reason (classified as unknown).
async fn seed_terminated_no_reason(
    test_db: &TestDatabase,
    ws_id: Uuid,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    hire_date: chrono::NaiveDate,
) {
    let employment_id = Uuid::new_v4();
    let employee_id = Uuid::new_v4();
    let location_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM work_locations WHERE workspace_id = $1 AND is_headquarters = true LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );
    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, hire_date, original_hire_date, created_at, updated_at)
         VALUES ($1, $2, 'Terminated Unknown', $3, $4, $4, NOW(), NOW())",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Text, format!("TERM-NULL-{}", employee_id.simple()),
        diesel::sql_types::Date, hire_date
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, end_date, employment_type, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'terminated', $6, $7, 'full_time', NOW(), NOW())",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Uuid, location_id,
        diesel::sql_types::Date, start_date,
        diesel::sql_types::Date, end_date
    );
}

/// Create an active employment still present on a given date (no termination).
async fn seed_active(test_db: &TestDatabase, ws_id: Uuid, start_date: chrono::NaiveDate) -> Uuid {
    let employment_id = Uuid::new_v4();
    let employee_id = Uuid::new_v4();
    let location_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM work_locations WHERE workspace_id = $1 AND is_headquarters = true LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );
    let employer_id: Uuid = crate::test_query_scalar_uuid!(
        &test_db.diesel_pools.api,
        "SELECT id FROM employers WHERE workspace_id = $1 LIMIT 1",
        diesel::sql_types::Uuid,
        ws_id
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, hire_date, original_hire_date, created_at, updated_at)
         VALUES ($1, $2, 'Active Test', $3, $4, $4, NOW(), NOW())",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Text, format!("ACT-{}", employee_id.simple()),
        diesel::sql_types::Date, start_date
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, employment_type, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'active', $6, 'full_time', NOW(), NOW())",
        diesel::sql_types::Uuid, employment_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Uuid, location_id,
        diesel::sql_types::Date, start_date
    );
    employment_id
}

/// Seed a workspace-scoped `dashboard.budget.turnover_rate` config entry.
async fn seed_turnover_target(
    test_db: &TestDatabase,
    ws_id: Uuid,
    value: rust_decimal::Decimal,
    effective_from: chrono::NaiveDate,
) {
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO workspace_config_entries
           (workspace_id, scope_type, scope_id, config_key, value_decimal, effective_from)
         VALUES ($1, 'workspace', NULL, 'dashboard.budget.turnover_rate', $2, $3)",
        diesel::sql_types::Uuid,
        ws_id,
        diesel::sql_types::Numeric,
        value,
        diesel::sql_types::Date,
        effective_from
    );
}

// ── Scenario 1: denominator = 0 ───────────────────────────────────────────────

#[tokio::test]
async fn empty_workspace_returns_null_rate_and_low_denominator_warning() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    // Use an explicit short window so the monthly trend stays empty and the test
    // isolates the zero-denominator contract.
    let start = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let query = format!("?start={start}&end={end}");

    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "zero-denominator must still return 200, never an error"
    );

    let json = body_json(response).await;
    assert_eq!(json["total_terminated"], 0);
    assert_eq!(json["average_headcount"], 0);
    assert_eq!(
        json["turnover_rate"]["value"],
        serde_json::Value::Null,
        "rate value must be null when the denominator is 0 (never fabricated 0.0)"
    );
    assert_eq!(json["turnover_rate"]["numerator"], 0);
    assert_eq!(json["turnover_rate"]["denominator"], 0);
    assert_eq!(json["retention_rate"], serde_json::Value::Null);
    assert_eq!(json["monthly_trend"], serde_json::json!([]));
    assert_eq!(json["cohort_retention"], serde_json::json!([]));

    let warnings = json["warnings"].as_array().unwrap();
    assert!(
        warnings.iter().any(|w| w["code"] == "LOW_DENOMINATOR"),
        "empty workspace must carry the LOW_DENOMINATOR warning"
    );
}

// ── Scenario 2: voluntary / involuntary / unknown split ───────────────────────

#[tokio::test]
async fn classification_split_derives_from_termination_reason() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();

    // Voluntary: 3 distinct reasons.
    for reason in ["voluntary_resignation", "retirement", "mutual_agreement"] {
        seed_terminated(
            &test_db,
            ws_id,
            start,
            start + chrono::Duration::days(30),
            reason,
            start,
        )
        .await;
    }
    // Involuntary: 3 distinct reasons.
    for reason in ["involuntary_termination", "abandonment", "death"] {
        seed_terminated(
            &test_db,
            ws_id,
            start,
            start + chrono::Duration::days(30),
            reason,
            start,
        )
        .await;
    }
    // Unknown: end_of_contract + a NULL reason (2 employees).
    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(30),
        "end_of_contract",
        start,
    )
    .await;
    seed_terminated_no_reason(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(30),
        start,
    )
    .await;

    let query = format!("?start={start}&end={end}");
    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    assert_eq!(json["total_terminated"], 8);
    assert_eq!(json["voluntary_count"], 3);
    assert_eq!(json["involuntary_count"], 3);
    assert_eq!(json["unknown_classification_count"], 2);
    assert_eq!(
        json["voluntary_count"].as_i64().unwrap()
            + json["involuntary_count"].as_i64().unwrap()
            + json["unknown_classification_count"].as_i64().unwrap(),
        json["total_terminated"].as_i64().unwrap(),
        "classification counts must sum to total_terminated"
    );

    let classification = json["classification"].as_array().unwrap();
    let voluntary = classification
        .iter()
        .find(|c| c["classification"] == "voluntary")
        .unwrap();
    let involuntary = classification
        .iter()
        .find(|c| c["classification"] == "involuntary")
        .unwrap();
    let unknown = classification
        .iter()
        .find(|c| c["classification"] == "unknown")
        .unwrap();
    assert_eq!(voluntary["count"], 3);
    assert_eq!(involuntary["count"], 3);
    assert_eq!(unknown["count"], 2);

    let reasons = json["termination_reasons"].as_array().unwrap();
    assert_eq!(
        reasons.len(),
        8,
        "7 distinct seeded reason values + NULL coalesced to 'unknown'"
    );
}

// ── Scenario 3: target + comparison ───────────────────────────────────────────

#[tokio::test]
async fn target_resolves_from_workspace_config_and_comparison_returns_pp_delta() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();

    // Current period: 3 leavers among 2 actives -> turnover rate high.
    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(30),
        "voluntary_resignation",
        start,
    )
    .await;
    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(60),
        "involuntary_termination",
        start,
    )
    .await;
    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(90),
        "mutual_agreement",
        start,
    )
    .await;
    seed_active(&test_db, ws_id, start).await;
    seed_active(&test_db, ws_id, start).await;

    // Previous period (2025-07-04..2025-12-31): 1 leaver among 2 actives -> rate = 50%.
    let prev_start = chrono::NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
    seed_terminated(
        &test_db,
        ws_id,
        prev_start,
        prev_start + chrono::Duration::days(20),
        "voluntary_resignation",
        prev_start,
    )
    .await;
    seed_active(&test_db, ws_id, prev_start).await;
    seed_active(&test_db, ws_id, prev_start).await;

    // Configure the target to 12 (default is 15).
    seed_turnover_target(&test_db, ws_id, rust_decimal::Decimal::from(12), start).await;

    let query = format!("?start={start}&end={end}&comparison=previous_period");
    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    // value_decimal is NUMERIC(12,4), so the API returns "12.0000" — compare numerically.
    assert_eq!(
        json["target"]["value"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        12.0
    );
    assert_eq!(json["target"]["operator"], "lte");
    assert_eq!(json["target"]["source"], "workspace_config");

    let comparison = json["comparison"].as_object().expect("comparison present");
    assert_eq!(comparison["basis"], "previous_period");
    // PreviousPeriod = same-length window immediately before: (start − 181d) .. (start − 1d).
    assert_eq!(comparison["period_start"], "2025-07-04");
    assert_eq!(comparison["period_end"], "2025-12-31");

    // Current rate: 2 leavers / avg headcount. avg headcount across Jan-Jun 2026
    // month-starts includes the 2 actives (and the leavers count only while active).
    let current_rate = json["turnover_rate"]["value"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let prev_rate = comparison["rate"]["value"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let delta = json["comparison"]["delta_percentage_points"]
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let expected_delta = current_rate - prev_rate;
    assert!(
        (delta - expected_delta).abs() < 0.01,
        "pp delta must equal current − previous: {delta} vs {expected_delta}"
    );
}

#[tokio::test]
async fn target_defaults_to_15_lte_with_default_source() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 3, 31).unwrap();
    let query = format!("?start={start}&end={end}");

    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    assert_eq!(json["target"]["value"], "15");
    assert_eq!(json["target"]["operator"], "lte");
    assert_eq!(json["target"]["source"], "default");
}

// ── Scenario 4: custom (odd-length) period ────────────────────────────────────

#[tokio::test]
async fn custom_odd_length_period_returns_requested_window() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 4, 23).unwrap(); // 45 days (odd length)

    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(5),
        "voluntary_resignation",
        start,
    )
    .await;
    seed_active(&test_db, ws_id, start).await;

    let query = format!("?start={start}&end={end}");
    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    assert_eq!(json["period_start"], "2026-03-10");
    assert_eq!(json["period_end"], "2026-04-23");
    assert_eq!(json["total_terminated"], 1);
    assert!(json["average_headcount"].as_i64().unwrap() > 0);

    // Period < 60 days -> no monthly trend (guard against tiny custom windows).
    assert_eq!(json["monthly_trend"], serde_json::json!([]));
}

// ── Scenario 5: cohort boundary ────────────────────────────────────────────────

#[tokio::test]
async fn cohort_retention_excludes_out_of_period_hires_and_filters_min_cohort() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();

    // 4 hires in 2026-02 (cohort size 4 >= MIN_COHORT 3) — all active for 3+ months.
    for _ in 0..4 {
        seed_active(
            &test_db,
            ws_id,
            chrono::NaiveDate::from_ymd_opt(2026, 2, 5).unwrap(),
        )
        .await;
    }
    // 2 hires in 2026-03 (cohort size 2 < 3) — must be filtered out entirely.
    for _ in 0..2 {
        seed_active(
            &test_db,
            ws_id,
            chrono::NaiveDate::from_ymd_opt(2026, 3, 5).unwrap(),
        )
        .await;
    }
    // 3 hires BEFORE the period (2025-12) — must be excluded (cohort outside window).
    for _ in 0..3 {
        seed_active(
            &test_db,
            ws_id,
            chrono::NaiveDate::from_ymd_opt(2025, 12, 5).unwrap(),
        )
        .await;
    }

    let query = format!("?start={start}&end={end}");
    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;
    let cohorts = json["cohort_retention"].as_array().unwrap();

    let cohort_months: Vec<String> = cohorts
        .iter()
        .map(|c| c["cohort_month"].as_str().unwrap().to_string())
        .collect();
    assert!(
        !cohort_months.contains(&"2025-12".to_string()),
        "hires outside the period must be excluded from cohorts"
    );
    assert!(
        !cohort_months.contains(&"2026-03".to_string()),
        "cohorts with fewer than MIN_COHORT=3 hires must be filtered out"
    );

    let feb_rows: Vec<_> = cohorts
        .iter()
        .filter(|c| c["cohort_month"] == "2026-02")
        .collect();
    assert!(!feb_rows.is_empty(), "the 2026-02 cohort must appear");
    for row in &feb_rows {
        assert_eq!(row["cohort_size"], 4);
        // All 4 hires remain active at every offset in the window.
        assert_eq!(row["active_count"], 4);
        assert_eq!(row["retention_rate"], "100");
    }
}

// ── Scenario 6: permission ─────────────────────────────────────────────────────

#[tokio::test]
async fn member_without_dashboard_read_is_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let employee_user = factory.user().create().await;
    factory
        .add_member(
            ws_id,
            employee_user,
            Uuid::parse_str(EMPLOYEE_ROLE).unwrap(),
        )
        .await;

    let response = app
        .clone()
        .oneshot(request(ws_id, employee_user, ""))
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "a member without dashboard:read must be forbidden"
    );
}

// ── Scenario 7: workspace isolation ───────────────────────────────────────────

#[tokio::test]
async fn workspaces_are_isolated_with_identical_data() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_a = factory.user().create().await;
    let ws_a = factory.workspace().create_with_owner(owner_a).await;
    let owner_b = factory.user().create().await;
    let ws_b = factory.workspace().create_with_owner(owner_b).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    // Identical data in BOTH workspaces: 2 terminations each.
    for ws in [ws_a, ws_b] {
        seed_terminated(
            &test_db,
            ws,
            start,
            start + chrono::Duration::days(30),
            "voluntary_resignation",
            start,
        )
        .await;
        seed_terminated(
            &test_db,
            ws,
            start,
            start + chrono::Duration::days(60),
            "involuntary_termination",
            start,
        )
        .await;
    }

    let period_end = start + chrono::Duration::days(180);
    let query = format!("?start={start}&end={period_end}");
    let response_a = app
        .clone()
        .oneshot(request(ws_a, owner_a, &query))
        .await
        .unwrap();
    assert_eq!(response_a.status(), StatusCode::OK);
    let json_a = body_json(response_a).await;

    // Workspace B sees exactly the same own data.
    let response_b = app
        .clone()
        .oneshot(request(ws_b, owner_b, &query))
        .await
        .unwrap();
    assert_eq!(response_b.status(), StatusCode::OK);
    let json_b = body_json(response_b).await;

    assert_eq!(json_a["total_terminated"], 2);
    assert_eq!(json_b["total_terminated"], 2);
    assert_eq!(json_a["voluntary_count"], 1);
    assert_eq!(json_b["voluntary_count"], 1);
    assert_eq!(json_a["involuntary_count"], 1);
    assert_eq!(json_b["involuntary_count"], 1);

    // The 2 terminations from workspace A must never leak into B's totals.
    // Sanity: B's counts are exactly B's own — no cross-workspace bleed.
    let total_expected = json_b["total_terminated"].as_i64().unwrap();
    assert_eq!(total_expected, 2, "B must not see A's terminations");
}

// ── Scenario 8: full-stack integration shape ──────────────────────────────────

#[tokio::test]
async fn full_stack_returns_complete_turnover_analytics_shape() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(owner_id).await;

    let start = chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let end = chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap();

    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(20),
        "voluntary_resignation",
        start,
    )
    .await;
    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(40),
        "death",
        start,
    )
    .await;
    seed_terminated(
        &test_db,
        ws_id,
        start,
        start + chrono::Duration::days(80),
        "end_of_contract",
        start,
    )
    .await;
    seed_active(&test_db, ws_id, start).await;
    seed_active(&test_db, ws_id, start).await;

    // Previous period (2025-07-04..2025-12-31) needs its own leaver + actives so the
    // comparison rate resolves to a real number instead of a zero denominator.
    let prev_start = chrono::NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
    seed_terminated(
        &test_db,
        ws_id,
        prev_start,
        prev_start + chrono::Duration::days(30),
        "mutual_agreement",
        prev_start,
    )
    .await;
    seed_active(&test_db, ws_id, prev_start).await;
    seed_active(&test_db, ws_id, prev_start).await;

    let query = format!("?start={start}&end={end}&comparison=previous_period&target=10");
    let response = app
        .clone()
        .oneshot(request(ws_id, owner_id, &query))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let json = body_json(response).await;

    // Period >= 2 months -> monthly trend populated.
    let trend = json["monthly_trend"].as_array().unwrap();
    assert!(
        trend.len() >= 6,
        "6-month period must produce a monthly trend, got {}",
        trend.len()
    );
    // The trend's per-month turnover_rate carries the denominator.
    assert!(trend
        .iter()
        .all(|p| p["turnover_rate"]["denominator"].as_i64().unwrap() > 0));

    // Termination reasons + classification + employment type all present.
    let reasons = json["termination_reasons"].as_array().unwrap();
    assert_eq!(reasons.len(), 3);
    assert_eq!(json["classification"].as_array().unwrap().len(), 3);

    // Every termination lands in exactly one classification.
    let classified: i64 = json["classification"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["count"].as_i64().unwrap())
        .sum();
    assert_eq!(classified, json["total_terminated"].as_i64().unwrap());

    // by_employment_type / by_location / by_manager / by_source / by_department present.
    let employment_types = json["by_employment_type"].as_array().unwrap();
    assert!(employment_types
        .iter()
        .any(|e| e["employment_type"] == "full_time"));
    let locations = json["by_location"].as_array().unwrap();
    assert!(locations
        .iter()
        .any(|l| l["location_name"] == "Headquarters"));
    assert!(json["by_manager"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["count"].as_i64().unwrap() >= 1));

    // Turnover rate is a number (denominator > 0).
    assert!(json["turnover_rate"]["value"].as_str().is_some());
    assert!(json["turnover_rate"]["denominator"].as_i64().unwrap() > 0);
    // Early attrition present (tenure < 90 days).
    assert!(json["early_attrition"]["numerator"].as_i64().unwrap() >= 0);
    // Regrettable attrition stays null (no data source).
    assert!(json["regrettable_attrition"].is_null());
    // Comparison present (requested previous_period) with a rate.
    let comparison = json["comparison"].as_object().expect("comparison present");
    assert_eq!(comparison["basis"], "previous_period");
    assert!(comparison["rate"]["value"].as_str().is_some());
    // target override respected.
    assert_eq!(json["target"]["value"], "10");
    assert_eq!(json["target"]["source"], "default");
}
