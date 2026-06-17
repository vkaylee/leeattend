use apiserver::mw::rate_limit::RateLimitConfig;
use apiserver::mw::RateLimitLayer;
use axum::http::{Request, StatusCode};
use axum::{routing::get, Router};
use tower::ServiceExt;

#[tokio::test]
async fn test_rate_limit_ip_unknown_fallback() {
    let redis_pool = match crate::common::app::create_test_redis_pool().await {
        Some(pool) => pool,
        None => {
            println!("Skipping test_rate_limit_ip_unknown_fallback because REDIS_URL is not set");
            return;
        }
    };

    // Set a very strict rate limit config: 2 read requests max per 10 seconds.
    let config = RateLimitConfig {
        max_read_requests: 2,
        max_write_requests: 2,
        window_secs: 10,
    };

    let layer = RateLimitLayer::new(Some(redis_pool.clone()), config, "test_secret".to_string());

    let app = Router::new()
        .route("/api/v1/test-route", get(|| async { "OK" }))
        .layer(layer);

    // Helper to send request with no client IP headers
    let send_req = || {
        let req = Request::builder()
            .method("GET")
            .uri("/api/v1/test-route")
            .body(axum::body::Body::empty())
            .unwrap();
        app.clone().oneshot(req)
    };

    // First request should succeed
    let res1 = send_req().await.unwrap();
    assert_eq!(res1.status(), StatusCode::OK);

    // Second request should succeed (reaches limit of 2)
    let res2 = send_req().await.unwrap();
    assert_eq!(res2.status(), StatusCode::OK);

    // Third request should be rate-limited (returns 429) since it falls back to the same "ip:unknown" key
    let res3 = send_req().await.unwrap();
    assert_eq!(res3.status(), StatusCode::TOO_MANY_REQUESTS);

    // Clean up Redis key to keep test environment clean
    let mut conn = redis_pool.get_async_connection().await.unwrap();
    let key = "rate:ip:unknown:r:/api/v1/test-route";
    let _: () = redis::cmd("DEL")
        .arg(key)
        .query_async(&mut conn)
        .await
        .unwrap();
}
