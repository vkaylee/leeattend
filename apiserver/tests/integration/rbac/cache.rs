//! RBAC Permission Cache integration tests
//!
//! Tests the Redis cache-aside pattern for permission lookups
//! and verifies cache invalidation on role/permission mutations.

use crate::common::app::setup_app_tower;
use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::AppState;
use axum::http::StatusCode;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Helper: build a request to read workspace members (requires permissions, triggers MemberContext cache)
fn member_list_request(ws_id: Uuid, caller_id: Uuid) -> axum::http::Request<axum::body::Body> {
    axum::http::Request::builder()
        .method("GET")
        .uri(format!("/api/workspaces/{ws_id}/members"))
        .header("X-Workspace-Id", ws_id.to_string())
        .header("X-User-Id", caller_id.to_string())
        .body(axum::body::Body::empty())
        .unwrap()
}

/// Verify that repeated requests for the same user do NOT fail
/// (i.e. the cache-aside pattern populates and re-uses cached data).
#[tokio::test]
async fn test_cache_aside_repeated_requests_succeed() {
    let (_test_db, factory, _state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;

    // First request — cache miss, should populate cache
    let r1 = app
        .clone()
        .oneshot(member_list_request(ws_id, owner_id))
        .await
        .unwrap();
    assert_eq!(
        r1.status(),
        StatusCode::OK,
        "First request (cache miss) should succeed"
    );

    // Second request — cache hit
    let r2 = app
        .clone()
        .oneshot(member_list_request(ws_id, owner_id))
        .await
        .unwrap();
    assert_eq!(
        r2.status(),
        StatusCode::OK,
        "Second request (cache hit) should succeed"
    );
}

/// After updating a member's primary role, the permission cache for that user
/// must be invalidated so the next request picks up the new permissions.
#[tokio::test]
async fn test_cache_invalidation_on_member_role_update() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;

    // Create a member with 'member' role
    let member_id = factory.user().create().await;
    let member_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory.add_member(ws_id, member_id, member_role_id).await;

    // Warm the cache — member makes a request
    let r1 = app
        .clone()
        .oneshot(member_list_request(ws_id, member_id))
        .await
        .unwrap();
    // member role may or may not have members:read, that's OK — we just want to warm the cache
    let _status_before = r1.status();

    // Owner updates member's role to 'admin'
    let admin_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000002").unwrap();
    let update_body = serde_json::json!({ "role_id": admin_role_id });

    let r_update = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PATCH")
                .uri(format!("/api/workspaces/{ws_id}/members/{member_id}"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r_update.status(),
        StatusCode::OK,
        "Owner should be able to update member role"
    );

    // Verify the cache key was deleted (if Redis is available)
    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = apiserver::cache::RedisCache::new(redis_pool);
        let cache_key = format!("cache:auth:permissions:{ws_id}:{member_id}");
        let cached: Option<serde_json::Value> = cache.get(&cache_key).await;
        assert!(
            cached.is_none(),
            "Cache must be invalidated after role update"
        );
    }

    // Post-invalidation: member with new admin role should now succeed
    let r3 = app
        .clone()
        .oneshot(member_list_request(ws_id, member_id))
        .await
        .unwrap();
    assert_eq!(
        r3.status(),
        StatusCode::OK,
        "After role upgrade to admin, member should access the endpoint"
    );
}

/// After granting an extra permission to a member, the cache must be invalidated
/// so the new permission is picked up on the next request.
#[tokio::test]
async fn test_cache_invalidation_on_extra_permission_grant() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;

    let member_id = factory.user().create().await;
    let member_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory.add_member(ws_id, member_id, member_role_id).await;

    // Warm the cache
    let _ = app
        .clone()
        .oneshot(member_list_request(ws_id, member_id))
        .await
        .unwrap();

    // Owner grants extra permission
    let future_date = chrono::Utc::now() + chrono::Duration::days(1);
    let grant_body = serde_json::json!({
        "permission": "shifts:create",
        "expires_at": future_date
    });

    let r_grant = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/members/{member_id}/permissions"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(grant_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r_grant.status(),
        StatusCode::CREATED,
        "Grant extra permission should succeed"
    );

    // Verify cache invalidation
    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = apiserver::cache::RedisCache::new(redis_pool);
        let cache_key = format!("cache:auth:permissions:{ws_id}:{member_id}");
        let cached: Option<serde_json::Value> = cache.get(&cache_key).await;
        assert!(
            cached.is_none(),
            "Cache must be invalidated after extra permission grant"
        );
    }
}

/// After revoking an extra permission, the cache must be cleared
/// so the revoked permission is no longer in effect.
#[tokio::test]
async fn test_cache_invalidation_on_extra_permission_revoke() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;

    let member_id = factory.user().create().await;
    let member_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory.add_member(ws_id, member_id, member_role_id).await;

    // Grant permission first
    let future_date = chrono::Utc::now() + chrono::Duration::days(1);
    let grant_body = serde_json::json!({
        "permission": "shifts:create",
        "expires_at": future_date
    });

    let r_grant = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!(
                    "/api/workspaces/{ws_id}/members/{member_id}/permissions"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(grant_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r_grant.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r_grant.into_body(), usize::MAX)
        .await
        .unwrap();
    let perm_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let perm_id = perm_json["id"].as_str().unwrap();

    // Warm cache again after grant
    let _ = app
        .clone()
        .oneshot(member_list_request(ws_id, member_id))
        .await
        .unwrap();

    // Revoke the permission
    let r_revoke = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!(
                    "/api/workspaces/{ws_id}/members/{member_id}/permissions/{perm_id}"
                ))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r_revoke.status(),
        StatusCode::NO_CONTENT,
        "Revoke should succeed"
    );

    // Verify cache invalidation
    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = apiserver::cache::RedisCache::new(redis_pool);
        let cache_key = format!("cache:auth:permissions:{ws_id}:{member_id}");
        let cached: Option<serde_json::Value> = cache.get(&cache_key).await;
        assert!(
            cached.is_none(),
            "Cache must be invalidated after permission revoke"
        );
    }
}

/// After updating role permissions (the permission matrix), the cache for ALL
/// members in that workspace must be invalidated (pattern delete).
#[tokio::test]
async fn test_cache_invalidation_on_role_permissions_update() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;

    // Create a custom role
    let create_role_body = serde_json::json!({
        "name": "custom_tester",
        "description": "Test role for cache invalidation",
        "color": "#FF0000"
    });

    let r_create = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(format!("/api/workspaces/{ws_id}/roles"))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(create_role_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r_create.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(r_create.into_body(), usize::MAX)
        .await
        .unwrap();
    let role_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let role_id = role_json["id"].as_str().unwrap();

    // Warm the owner's cache
    let _ = app
        .clone()
        .oneshot(member_list_request(ws_id, owner_id))
        .await
        .unwrap();

    // Update role permissions (triggers workspace-wide invalidation)
    let update_perms_body = serde_json::json!({
        "permissions": ["employees:read", "shifts:create"]
    });

    let r_update = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/workspaces/{ws_id}/roles/{role_id}/permissions"
                ))
                .header("Content-Type", "application/json")
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::from(update_perms_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r_update.status(),
        StatusCode::OK,
        "Update role permissions should succeed"
    );

    // Verify cache invalidation for the entire workspace
    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = apiserver::cache::RedisCache::new(redis_pool);
        let owner_cache_key = format!("cache:auth:permissions:{ws_id}:{owner_id}");
        let cached: Option<serde_json::Value> = cache.get(&owner_cache_key).await;
        assert!(
            cached.is_none(),
            "All workspace caches must be invalidated after role permissions update"
        );
    }
}

/// After removing a member, their cache entry must be deleted.
#[tokio::test]
async fn test_cache_invalidation_on_member_removal() {
    let (_test_db, factory, state, app): (TestDatabase, TestFactory, Arc<AppState>, axum::Router) =
        setup_app_tower().await;

    let owner_id: Uuid = factory.user().create().await;
    let ws_id: Uuid = factory.workspace().create_with_owner(owner_id).await;

    let member_id = factory.user().create().await;
    let member_role_id = uuid::Uuid::parse_str("a0000000-0000-0000-0000-000000000006").unwrap();
    factory.add_member(ws_id, member_id, member_role_id).await;

    // Warm cache for member
    let _ = app
        .clone()
        .oneshot(member_list_request(ws_id, member_id))
        .await
        .unwrap();

    // Owner removes member
    let r_remove = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .method("DELETE")
                .uri(format!("/api/workspaces/{ws_id}/members/{member_id}"))
                .header("X-Workspace-Id", ws_id.to_string())
                .header("X-User-Id", owner_id.to_string())
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        r_remove.status(),
        StatusCode::NO_CONTENT,
        "Remove member should succeed"
    );

    // Verify cache invalidation
    if let Some(redis_pool) = state.infra.redis.clone() {
        let cache = apiserver::cache::RedisCache::new(redis_pool);
        let cache_key = format!("cache:auth:permissions:{ws_id}:{member_id}");
        let cached: Option<serde_json::Value> = cache.get(&cache_key).await;
        assert!(
            cached.is_none(),
            "Removed member's cache must be invalidated"
        );
    }
}
