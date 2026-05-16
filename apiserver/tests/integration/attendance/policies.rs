use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

// ── Helpers ──────────────────────────────────────────────────────

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
        .expect("build request")
}

async fn body_json(resp: axum::http::Response<axum::body::Body>) -> serde_json::Value {
    let bytes = axum::body::to_bytes(resp.into_body(), 100_000)
        .await
        .expect("read body");
    serde_json::from_slice(&bytes).expect("parse json")
}

// ── 1. Happy-path lifecycle ─────────────────────────────────────

#[tokio::test]
async fn test_default_attendance_policy_lifecycle() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(admin_id).await;

    // GET current rules
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .expect("get rules");
    assert_eq!(resp.status(), StatusCode::OK);
    let rules = body_json(resp).await;
    assert!(rules.get("work_unit_minutes").is_some());
    assert!(rules.get("rounding_policy").is_some());

    // PATCH — create a new version
    let resp = app.clone().oneshot(req(
        "PATCH",
        &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
        ws_id, admin_id,
        Some(serde_json::json!({ "work_unit_minutes": 30, "rounding_policy": "floor", "valid_from": "2026-05-01" }).to_string()),
    )).await.expect("patch rules");
    assert_eq!(resp.status(), StatusCode::OK);
    let version = body_json(resp).await;
    assert_eq!(version["work_unit_minutes"].as_i64().expect("wum"), 30);
    assert_eq!(version["rounding_policy"].as_str().expect("rp"), "floor");
    assert!(version.get("id").is_some());
    assert!(version.get("valid_from").is_some());

    // GET versions list
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default/versions"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .expect("list versions");
    assert_eq!(resp.status(), StatusCode::OK);
    let versions: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 100_000)
            .await
            .expect("bytes"),
    )
    .expect("parse");
    assert!(
        versions.len() >= 2,
        "Expected ≥2 versions, got {}",
        versions.len()
    );
    assert_eq!(versions[0]["work_unit_minutes"].as_i64().expect("wum"), 30);

    // GET default rules should reflect updated values
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .expect("get rules again");
    assert_eq!(resp.status(), StatusCode::OK);
    let fetched = body_json(resp).await;
    assert_eq!(fetched["work_unit_minutes"].as_i64().expect("wum"), 30);
    assert_eq!(fetched["rounding_policy"].as_str().expect("rp"), "floor");
}

// ── 2. Invalid valid_from format → 400 ──────────────────────────

#[tokio::test]
async fn test_patch_invalid_valid_from_returns_400() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            Some(
                serde_json::json!({ "work_unit_minutes": 30, "valid_from": "not-a-date" })
                    .to_string(),
            ),
        ))
        .await
        .expect("patch with bad date");

    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "Invalid date format must return 400"
    );
}

// ── 3. Permission denied → 403 ─────────────────────────────────

#[tokio::test]
async fn test_patch_forbidden_for_regular_member() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Create a regular member (employee role = a0...06)
    let member_id = factory.user().with_email("viewer@test.com").create().await;
    factory
        .add_member(
            ws_id,
            member_id,
            Uuid::parse_str("a0000000-0000-0000-0000-000000000006").expect("role uuid"),
        )
        .await;

    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            member_id,
            Some(serde_json::json!({ "work_unit_minutes": 60 }).to_string()),
        ))
        .await
        .expect("patch as member");

    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "Regular member must not update policy"
    );
}

// ── 4. Partial update preserves other fields ────────────────────

#[tokio::test]
async fn test_partial_update_preserves_existing_fields() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // First set both fields explicitly
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            Some(
                serde_json::json!({ "work_unit_minutes": 30, "rounding_policy": "ceil" })
                    .to_string(),
            ),
        ))
        .await
        .expect("full patch");
    assert_eq!(resp.status(), StatusCode::OK);

    // Now patch only work_unit_minutes — rounding_policy should stay "ceil"
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            Some(serde_json::json!({ "work_unit_minutes": 60 }).to_string()),
        ))
        .await
        .expect("partial patch");
    assert_eq!(resp.status(), StatusCode::OK);
    let v = body_json(resp).await;
    assert_eq!(v["work_unit_minutes"].as_i64().expect("wum"), 60);
    assert_eq!(
        v["rounding_policy"].as_str().expect("rp"),
        "ceil",
        "rounding_policy must be preserved"
    );

    // And the reverse — patch only rounding_policy
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            Some(serde_json::json!({ "rounding_policy": "round" }).to_string()),
        ))
        .await
        .expect("partial patch rp");
    assert_eq!(resp.status(), StatusCode::OK);
    let v = body_json(resp).await;
    assert_eq!(
        v["work_unit_minutes"].as_i64().expect("wum"),
        60,
        "work_unit_minutes must be preserved"
    );
    assert_eq!(v["rounding_policy"].as_str().expect("rp"), "round");
}

// ── 5. Multiple sequential updates — version chain integrity ────

#[tokio::test]
async fn test_multiple_updates_version_chain_integrity() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Apply 3 sequential updates
    for wum in [10, 20, 30] {
        let resp = app
            .clone()
            .oneshot(req(
                "PATCH",
                &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
                ws_id,
                admin_id,
                Some(serde_json::json!({ "work_unit_minutes": wum }).to_string()),
            ))
            .await
            .expect("sequential patch");
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // Get all versions
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default/versions"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .expect("list versions");
    assert_eq!(resp.status(), StatusCode::OK);

    let versions: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 100_000)
            .await
            .expect("bytes"),
    )
    .expect("parse");

    // Migration version + 3 patches ≥ 4
    assert!(
        versions.len() >= 4,
        "Expected ≥4 versions, got {}",
        versions.len()
    );

    // Newest first: 30, 20, 10, migration-default
    assert_eq!(versions[0]["work_unit_minutes"].as_i64().expect("v0"), 30);
    assert_eq!(versions[1]["work_unit_minutes"].as_i64().expect("v1"), 20);
    assert_eq!(versions[2]["work_unit_minutes"].as_i64().expect("v2"), 10);

    // Only the first (newest) should have valid_to = null (active)
    assert!(
        versions[0]["valid_to"].is_null(),
        "Current version must have null valid_to"
    );

    // All expired versions must have valid_to set
    for v in &versions[1..] {
        assert!(
            !v["valid_to"].is_null(),
            "Expired version must have valid_to set: {v:?}"
        );
    }
}

// ── 6. GET versions on empty workspace → [] ─────────────────────

#[tokio::test]
async fn test_get_versions_empty_workspace_returns_empty_list() {
    let (test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Delete migration-seeded policy to simulate truly empty state
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "DELETE FROM attendance_policies WHERE workspace_id = $1",
        diesel::sql_types::Uuid,
        ws_id
    );

    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default/versions"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .expect("get versions empty");
    assert_eq!(resp.status(), StatusCode::OK);

    let versions: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(resp.into_body(), 100_000)
            .await
            .expect("bytes"),
    )
    .expect("parse");
    assert!(
        versions.is_empty(),
        "Expected empty list, got {}",
        versions.len()
    );
}

// ── 7. RFC 3339 valid_from format ───────────────────────────────

#[tokio::test]
async fn test_patch_with_rfc3339_valid_from() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            Some(
                serde_json::json!({
                    "work_unit_minutes": 15,
                    "valid_from": "2026-06-01T00:00:00Z"
                })
                .to_string(),
            ),
        ))
        .await
        .expect("patch rfc3339");
    assert_eq!(resp.status(), StatusCode::OK);

    let v = body_json(resp).await;
    assert_eq!(v["work_unit_minutes"].as_i64().expect("wum"), 15);
    // valid_from should contain the date we specified
    let vf = v["valid_from"].as_str().expect("valid_from");
    assert!(
        vf.contains("2026-06-01"),
        "valid_from should contain 2026-06-01, got {vf}"
    );
}

// ── 8. Update early_arrival_strategy ──────────────────────────────

#[tokio::test]
async fn test_patch_early_arrival_strategy() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let admin_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(admin_id).await;

    // Default strategy should be block
    let resp = app
        .clone()
        .oneshot(req(
            "GET",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            None,
        ))
        .await
        .expect("get rules");
    assert_eq!(resp.status(), StatusCode::OK);
    let rules = body_json(resp).await;
    assert_eq!(
        rules["early_arrival_strategy"].as_str().expect("eas"),
        "block"
    );

    // Patch to clamp
    let resp = app
        .clone()
        .oneshot(req(
            "PATCH",
            &format!("/api/workspaces/{ws_id}/attendance/policies/default"),
            ws_id,
            admin_id,
            Some(
                serde_json::json!({
                    "early_arrival_strategy": "clamp"
                })
                .to_string(),
            ),
        ))
        .await
        .expect("patch early_arrival_strategy");
    assert_eq!(resp.status(), StatusCode::OK);

    let v = body_json(resp).await;
    assert_eq!(v["early_arrival_strategy"].as_str().expect("eas"), "clamp");
}
