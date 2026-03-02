use crate::common::app::spawn_app_with_factory;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use uuid::Uuid;

#[tokio::test]
async fn test_get_dashboard_preferences_returns_defaults() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{address}/api/dashboard/preferences"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    
    // Should return defaults (nil UUID, overview active view, etc.)
    assert_eq!(body["id"].as_str().unwrap(), Uuid::nil().to_string());
    assert_eq!(body["active_view"].as_str().unwrap(), "overview");
    assert!(body["visible_widgets"]["kpi_cards"].as_bool().unwrap());
    assert!(!body["visible_widgets"]["promo_cards"].as_bool().unwrap());
}

#[tokio::test]
async fn test_update_dashboard_preferences_upserts_and_returns_updated() {
    let (address, ws_id, user_id, _, _db): (String, Uuid, Uuid, TestFactory, TestDatabase) =
        spawn_app_with_factory().await;
    let client = reqwest::Client::new();

    // 1. Update preferences to "attendance" view
    let payload = serde_json::json!({
        "active_view": "attendance",
        "visible_widgets": { "kpi_cards": false, "peak_times": true },
        "widget_order": ["peak_times", "kpi_cards"],
        "collapsed_sections": { "charts": true }
    });

    let res = client
        .put(format!("{address}/api/dashboard/preferences"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .json(&payload)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), reqwest::StatusCode::OK);
    let body: serde_json::Value = res.json().await.unwrap();
    
    // Check that ID is now a real UUID (not nil)
    let pref_id = body["id"].as_str().unwrap();
    assert_ne!(pref_id, Uuid::nil().to_string());
    assert_eq!(body["active_view"].as_str().unwrap(), "attendance");
    assert_eq!(body["visible_widgets"]["peak_times"].as_bool().unwrap(), true);
    assert_eq!(body["collapsed_sections"]["charts"].as_bool().unwrap(), true);

    // 2. GET again should return the persisted data
    let res2 = client
        .get(format!("{address}/api/dashboard/preferences"))
        .header("X-User-Id", user_id.to_string())
        .header("X-Workspace-Id", ws_id.to_string())
        .send()
        .await
        .unwrap();

    assert_eq!(res2.status(), reqwest::StatusCode::OK);
    let body2: serde_json::Value = res2.json().await.unwrap();
    assert_eq!(body2["id"].as_str().unwrap(), pref_id);
    assert_eq!(body2["active_view"].as_str().unwrap(), "attendance");
}
