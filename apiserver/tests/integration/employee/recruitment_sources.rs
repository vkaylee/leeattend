//! Recruitment Sources API integration tests

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

fn req(
    method: &str,
    uri: &str,
    ws_id: Uuid,
    user_id: Uuid,
    body: Option<String>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(uri)
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string());

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    builder
        .body(match body {
            Some(b) => axum::body::Body::from(b),
            None => axum::body::Body::empty(),
        })
        .unwrap()
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10000).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_recruitment_source_crud_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // 1. Create recruitment source
    let create_body = serde_json::json!({
        "source_type": "direct",
        "name": "LinkedIn"
    });

    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/recruitment-sources"),
            ws_id,
            admin_id,
            Some(create_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let created = body_json(resp).await;
    let source_id = created["id"].as_str().unwrap();
    assert_eq!(created["name"].as_str().unwrap(), "LinkedIn");

    // 2. List recruitment sources
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/v1/workspaces/{ws_id}/recruitment-sources"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let paginated = body_json(resp).await;
    let list = paginated["data"].as_array().unwrap().clone();
    assert!(list
        .iter()
        .any(|s| s["name"].as_str().unwrap() == "LinkedIn"));

    // 3. Update recruitment source
    let update_body = serde_json::json!({
        "name": "LinkedIn Jobs"
    });
    let resp = app
        .clone()
        .oneshot(req(
            "PUT",
            &format!("/api/v1/workspaces/{ws_id}/recruitment-sources/{source_id}"),
            ws_id,
            admin_id,
            Some(update_body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let updated = body_json(resp).await;
    assert_eq!(updated["name"].as_str().unwrap(), "LinkedIn Jobs");

    // 4. Delete recruitment source
    let resp = app
        .clone()
        .oneshot(req(
            "DELETE",
            &format!("/api/v1/workspaces/{ws_id}/recruitment-sources/{source_id}"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_recruitment_source_dashboard_breakdown() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Dashboard endpoints should return empty data without error
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/dashboard/recruitment-sources",
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_turnover_analytics() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            "/api/v1/dashboard/turnover-analytics",
            ws_id,
            admin_id,
            None,
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_recruitment_source_rbac_member_forbidden() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let body = serde_json::json!({ "source_type": "direct", "name": "Unauthorized Source" });
    let resp = app
        .clone()
        .oneshot(req(
            "POST",
            &format!("/api/v1/workspaces/{ws_id}/recruitment-sources"),
            ws_id,
            member_id,
            Some(body.to_string()),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── Recruitment Effectiveness ────────────────────────────────────────

/// Insert a recruitment source directly and return its id.
async fn seed_recruitment_source(
    test_db: &TestDatabase,
    ws_id: Uuid,
    source_type: &str,
    name: &str,
    fee: Option<rust_decimal::Decimal>,
) -> Uuid {
    let id = Uuid::new_v4();
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO recruitment_sources (id, workspace_id, source_type, name, referral_bonus_paid, agency_fee_paid, created_at, updated_at)
         VALUES ($1, $2, $3::public.recruitment_source_type, $4, false, false, NOW(), NOW())",
        diesel::sql_types::Uuid, id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Text, source_type,
        diesel::sql_types::Text, name
    );
    if let Some(fee) = fee {
        // Apply the fee to the appropriate column based on source type.
        let (col, _) = if source_type == "agency" {
            ("agency_fee_amount", fee)
        } else {
            ("referral_bonus_amount", fee)
        };
        crate::test_execute!(
            &test_db.diesel_pools.api,
            &format!("UPDATE recruitment_sources SET {col} = $1 WHERE id = $2"),
            diesel::sql_types::Numeric,
            fee,
            diesel::sql_types::Uuid,
            id
        );
    }
    id
}

/// Insert an employee with a hire_date, optionally linked to a recruitment source.
/// Returns the employee id.
async fn seed_hired_employee(
    test_db: &TestDatabase,
    ws_id: Uuid,
    hire_date: chrono::NaiveDate,
    source_id: Option<Uuid>,
    termination_date: Option<chrono::NaiveDate>,
) -> Uuid {
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
        "INSERT INTO employees (id, workspace_id, full_name, employee_id_number, hire_date, original_hire_date, recruitment_source_id, termination_date, created_at, updated_at)
         VALUES ($1, $2, 'Effectiveness Test', $3, $4, $4, $5, $6, NOW(), NOW())",
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Text, format!("EFF-{}", employee_id.simple()),
        diesel::sql_types::Date, hire_date,
        diesel::sql_types::Nullable<diesel::sql_types::Uuid>, source_id,
        diesel::sql_types::Nullable<diesel::sql_types::Date>, termination_date
    );

    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO employments (id, workspace_id, employee_id, employer_id, primary_location_id, status, start_date, employment_type, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'active', $6, 'full_time', NOW(), NOW())",
        diesel::sql_types::Uuid, Uuid::new_v4(),
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, employee_id,
        diesel::sql_types::Uuid, employer_id,
        diesel::sql_types::Uuid, location_id,
        diesel::sql_types::Date, hire_date
    );

    employee_id
}

fn effective_req(ws_id: Uuid, user_id: Uuid, query: &str) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri(format!(
            "/api/v1/dashboard/recruitment-effectiveness{query}"
        ))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", user_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn effective_body(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 10_000_000)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_recruitment_effectiveness_empty_source_never_fabricates() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // No employees at all.
    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = effective_body(resp).await;

    assert_eq!(json["total_hires"].as_i64().unwrap(), 0);
    assert_eq!(json["by_source"].as_array().unwrap().len(), 0);
    // Trend is zero-filled for a >= 60-day window: every month present, hires = 0.
    let trend = json["hires_trend"].as_array().unwrap();
    assert_eq!(trend.len(), 6); // 2026-01 .. 2026-06
    assert!(trend.iter().all(|p| p["hires"].as_i64().unwrap() == 0));

    // Funnel: pre-hire stages have null count; hired is null too (never fabricated).
    let funnel = json["funnel"].as_array().unwrap();
    let applicants = funnel.iter().find(|s| s["stage"] == "applicants").unwrap();
    assert!(applicants["count"].is_null());
    assert!(applicants["data_source"].is_null());
    let hired = funnel.iter().find(|s| s["stage"] == "hired").unwrap();
    assert!(hired["count"].is_null() || hired["count"].as_i64().unwrap() == 0);

    // data_gaps non-empty, warnings contain LOW_DENOMINATOR + NO_DATA_SOURCE.
    assert!(!json["data_gaps"].as_array().unwrap().is_empty());
    let codes: Vec<&str> = json["warnings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"LOW_DENOMINATOR"));
    assert!(codes.contains(&"NO_DATA_SOURCE"));

    // overview never fabricates a rate.
    assert!(json["overview"]["retention_90d"].is_null());
    assert!(json["overview"]["average_cost_per_hire"].is_null());
}

#[tokio::test]
async fn test_recruitment_effectiveness_unknown_source_is_an_honest_hire() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Hire with a NULL recruitment_source_id — still a hire, source unknown.
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None,
        None,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = effective_body(resp).await;

    assert_eq!(json["total_hires"].as_i64().unwrap(), 1);
    let sources = json["by_source"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["source_name"], "Unknown source");
    assert_eq!(sources[0]["source_type"], "unknown");
    assert!(sources[0]["source_id"].is_null());
}

#[tokio::test]
async fn test_recruitment_effectiveness_zero_denominator_cost_and_retention() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Direct source with NO fee configured → cost_per_hire not available.
    let direct = seed_recruitment_source(&test_db, ws_id, "direct", "Direct Hire", None).await;
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Some(direct),
        None,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    let json = effective_body(resp).await;

    let sources = json["by_source"].as_array().unwrap();
    assert_eq!(sources[0]["source_type"], "direct");
    assert_eq!(sources[0]["cost_per_hire_available"], false);
    assert!(sources[0]["cost_per_hire"].is_null());

    // Retention: hires=1, retained=1 → 100% (exact division keeps scale 0).
    assert_eq!(sources[0]["retained_90d"].as_i64().unwrap(), 1);
    assert_eq!(sources[0]["retention_90d"].as_str().unwrap(), "100");

    // Overview: cost_available false, average_cost null.
    assert_eq!(json["overview"]["cost_available"], false);
    assert!(json["overview"]["average_cost_per_hire"].is_null());
}

#[tokio::test]
async fn test_recruitment_effectiveness_cost_per_hire_agency() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Agency source with a fee → cost per hire = fee / hires.
    let agency = seed_recruitment_source(
        &test_db,
        ws_id,
        "agency",
        "ABC Recruitment",
        Some(rust_decimal::Decimal::from(10_000)),
    )
    .await;
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Some(agency),
        None,
    )
    .await;
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        Some(agency),
        None,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    let json = effective_body(resp).await;

    assert_eq!(json["total_hires"].as_i64().unwrap(), 2);
    let sources = json["by_source"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["source_type"], "agency");
    assert_eq!(sources[0]["hires"].as_i64().unwrap(), 2);
    assert_eq!(sources[0]["cost_per_hire_available"], true);
    // fee 10000.00 / 2 hires = 5000.00 (numeric(15,2) scale preserved)
    assert_eq!(sources[0]["cost_per_hire"].as_str().unwrap(), "5000.00");
    assert_eq!(json["overview"]["cost_available"], true);
    assert_eq!(
        json["overview"]["average_cost_per_hire"].as_str().unwrap(),
        "5000.00"
    );
}

#[tokio::test]
async fn test_recruitment_effectiveness_date_filters_and_retention() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Hire inside the range, terminated within 90 days → not retained.
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None,
        Some(chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()),
    )
    .await;
    // Hire inside the range, active → retained.
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
        None,
        None,
    )
    .await;
    // Hire OUTSIDE the range → excluded entirely.
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2025, 12, 1).unwrap(),
        None,
        None,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = effective_body(resp).await;

    assert_eq!(json["total_hires"].as_i64().unwrap(), 2);
    assert_eq!(json["overview"]["retained_90d"].as_i64().unwrap(), 1);
    assert_eq!(json["overview"]["retention_90d"].as_str().unwrap(), "50.00");

    // Malformed date → 400 validation error.
    let bad = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=not-a-date&end=2026-06-30",
        ))
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_recruitment_effectiveness_department_filter() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Two departments.
    let dept_a = Uuid::new_v4();
    let dept_b = Uuid::new_v4();
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO departments (id, workspace_id, name, created_at, updated_at) VALUES ($1, $2, 'Dept A', NOW(), NOW()), ($3, $2, 'Dept B', NOW(), NOW())",
        diesel::sql_types::Uuid, dept_a,
        diesel::sql_types::Uuid, ws_id,
        diesel::sql_types::Uuid, dept_b
    );

    let source = seed_recruitment_source(&test_db, ws_id, "direct", "Direct", None).await;
    let emp_a = seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Some(source),
        None,
    )
    .await;
    let emp_b = seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Some(source),
        None,
    )
    .await;

    // Assign departments on the employment rows.
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "UPDATE employments SET department_id = $1 WHERE employee_id = $2",
        diesel::sql_types::Uuid,
        dept_a,
        diesel::sql_types::Uuid,
        emp_a
    );
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "UPDATE employments SET department_id = $1 WHERE employee_id = $2",
        diesel::sql_types::Uuid,
        dept_b,
        diesel::sql_types::Uuid,
        emp_b
    );

    // Filter to Dept A only → 1 hire.
    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            &format!("?start=2026-01-01&end=2026-06-30&department_ids={dept_a}"),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = effective_body(resp).await;
    assert_eq!(json["total_hires"].as_i64().unwrap(), 1);
}

#[tokio::test]
async fn test_recruitment_effectiveness_permission_denied_for_reader() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let member_id = factory
        .user()
        .with_email("member@example.com")
        .create()
        .await;
    factory
        .add_member(
            ws_id,
            member_id,
            uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap(),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            member_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_recruitment_effectiveness_long_source_label() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let long_name = format!("A very long recruitment source name {}", "x".repeat(80));
    let source = seed_recruitment_source(&test_db, ws_id, "referral", &long_name, None).await;
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        Some(source),
        None,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-01-01&end=2026-06-30",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = effective_body(resp).await;
    let sources = json["by_source"].as_array().unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0]["source_name"].as_str().unwrap(), &long_name);
    assert_eq!(sources[0]["source_type"], "referral");
}

#[tokio::test]
async fn test_recruitment_effectiveness_comparison_previous_period() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let source = seed_recruitment_source(&test_db, ws_id, "direct", "Direct", None).await;

    // Current window: 2026-04-01..2026-06-30 → 2 hires.
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        Some(source),
        None,
    )
    .await;
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        Some(source),
        None,
    )
    .await;

    // Previous window: same length (2026-01-31..2026-03-31) → 1 hire.
    seed_hired_employee(
        &test_db,
        ws_id,
        chrono::NaiveDate::from_ymd_opt(2026, 2, 10).unwrap(),
        Some(source),
        None,
    )
    .await;

    let resp = app
        .clone()
        .oneshot(effective_req(
            ws_id,
            admin_id,
            "?start=2026-04-01&end=2026-06-30&comparison=previous_period",
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let json = effective_body(resp).await;

    assert_eq!(json["total_hires"].as_i64().unwrap(), 2);
    assert_eq!(json["overview"]["comparison_hires"].as_i64().unwrap(), 1);
    assert_eq!(
        json["overview"]["comparison_hires_delta"].as_i64().unwrap(),
        1
    );
}
