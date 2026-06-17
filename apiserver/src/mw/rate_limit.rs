use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Method, Request, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use redis::AsyncCommands;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::queue::RedisPool;

// ── Configuration ──────────────────────────────────────────────────

/// Rate limit configuration with method-based tiers.
///
/// Read methods (GET, HEAD, OPTIONS) get a higher limit since they are
/// cheap and frequently triggered by UI rendering / React Query refetches.
/// Write methods (POST, PUT, PATCH, DELETE) get a stricter limit to
/// protect against abuse.
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum read requests (GET/HEAD/OPTIONS) per window.
    pub max_read_requests: u64,
    /// Maximum write requests (POST/PUT/PATCH/DELETE) per window.
    pub max_write_requests: u64,
    /// Window duration in seconds.
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_read_requests: 120,
            max_write_requests: 30,
            window_secs: 60,
        }
    }
}

impl RateLimitConfig {
    /// Return the max requests for a given HTTP method.
    pub fn limit_for_method(&self, method: &Method) -> u64 {
        match *method {
            Method::GET | Method::HEAD | Method::OPTIONS => self.max_read_requests,
            _ => self.max_write_requests,
        }
    }

    /// Return a short tier label used in the Redis key.
    #[must_use]
    pub const fn tier_label(method: &Method) -> &'static str {
        match *method {
            Method::GET | Method::HEAD | Method::OPTIONS => "r",
            _ => "w",
        }
    }
}

// ── Layer ──────────────────────────────────────────────────────────

/// Tower Layer that applies Redis-based rate limiting with:
/// - **User-based keys** for authenticated requests (JWT / debug header)
/// - **IP-based keys** for unauthenticated requests
/// - **Method-based tiers** (reads vs writes)
#[derive(Clone)]
pub struct RateLimitLayer {
    redis: Option<RedisPool>,
    config: RateLimitConfig,
    jwt_secret: String,
}

impl RateLimitLayer {
    #[must_use]
    pub const fn new(
        redis: Option<RedisPool>,
        config: RateLimitConfig,
        jwt_secret: String,
    ) -> Self {
        Self {
            redis,
            config,
            jwt_secret,
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            redis: self.redis.clone(),
            config: self.config.clone(),
            jwt_secret: self.jwt_secret.clone(),
        }
    }
}

// ── Service ────────────────────────────────────────────────────────

/// Tower Service that checks rate limits via Redis before forwarding requests.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    redis: Option<RedisPool>,
    config: RateLimitConfig,
    jwt_secret: String,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let redis = self.redis.clone();
        let config = self.config.clone();
        let jwt_secret = self.jwt_secret.clone();
        let mut inner = self.inner.clone();
        // Swap so self.inner is the clone (ready) and inner is the original
        std::mem::swap(&mut self.inner, &mut inner);

        Box::pin(async move {
            // Fail-open: if Redis unavailable, allow request through
            let Some(pool) = redis else {
                return inner.call(req).await;
            };

            // ── Resolve identity: user_id (from JWT) or IP ─────────
            let identity = resolve_identity(&req, &jwt_secret);

            let method = req.method().clone();
            let matched_path = req
                .extensions()
                .get::<axum::extract::MatchedPath>()
                .map(|mp| mp.as_str().to_string())
                .unwrap_or_else(|| req.uri().path().to_string());

            // Bypass rate limiting for health checks to prevent false negatives in load balancers
            if matched_path.ends_with("/health") || matched_path.ends_with("/health/ready") {
                return inner.call(req).await;
            }
            let tier = RateLimitConfig::tier_label(&method);
            let max_requests = config.limit_for_method(&method);

            // Key format: rate:{user_or_ip}:{r|w}:{path_template}
            let key = format!("rate:{identity}:{tier}:{matched_path}");

            // Try rate check — fail-open on Redis errors
            match check_rate(&pool, &key, max_requests, config.window_secs).await {
                Ok(true) => inner.call(req).await,
                Ok(false) => {
                    tracing::warn!(
                        identity = %identity,
                        path = %req.uri().path(),
                        matched_path = %matched_path,
                        tier = %tier,
                        limit = %max_requests,
                        "Rate limit exceeded"
                    );
                    let response = (
                        StatusCode::TOO_MANY_REQUESTS,
                        [("Retry-After", config.window_secs.to_string())],
                        Json(serde_json::json!({
                            "error": {
                                "code": "RATE_LIMITED",
                                "message": "Too many requests. Please try again later."
                            }
                        })),
                    )
                        .into_response();
                    Ok(response)
                }
                Err(e) => {
                    tracing::error!(error = %e, "Rate limiter Redis error — allowing request");
                    inner.call(req).await
                }
            }
        })
    }
}

// ── Identity resolution ────────────────────────────────────────────

/// Extract identity for rate limiting. Priority:
/// 1. JWT Bearer token → `user_id` (lightweight decode, no DB call)
/// 2. X-User-Id debug header (debug builds only)
/// 3. Client IP address (`ConnectInfo` or X-Forwarded-For)
fn resolve_identity(req: &Request<Body>, jwt_secret: &str) -> String {
    // 1. Try JWT token
    if let Some(auth_header) = req.headers().get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                if let Ok(user_id) =
                    crate::services::auth_service::AuthService::verify_token(token, jwt_secret)
                {
                    return format!("u:{user_id}");
                }
            }
        }
    }

    // 2. Debug: X-User-Id header (test/dev only)
    #[cfg(debug_assertions)]
    {
        if let Some(user_id_str) = req.headers().get("x-user-id").and_then(|h| h.to_str().ok()) {
            if let Ok(user_id) = uuid::Uuid::parse_str(user_id_str) {
                return format!("u:{user_id}");
            }
        }
    }

    // 3. Fall back to IP
    let ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .or_else(|| {
            req.headers()
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .map(|s| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".to_string());

    format!("ip:{ip}")
}

/// Check if the request is within rate limits using Redis INCR + EXPIRE.
async fn check_rate(
    pool: &RedisPool,
    key: &str,
    max_requests: u64,
    window_secs: u64,
) -> Result<bool, redis::RedisError> {
    let mut conn = pool.conn();
    let count: u64 = conn.incr(key, 1u64).await?;
    if count == 1 {
        // First request in window — set expiry
        conn.expire::<_, ()>(key, i64::try_from(window_secs).unwrap_or(i64::MAX))
            .await?;
    }
    Ok(count <= max_requests)
}

// ── Unit tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_read_requests, 120);
        assert_eq!(config.max_write_requests, 30);
        assert_eq!(config.window_secs, 60);
    }

    #[test]
    fn test_limit_for_method_reads() {
        let config = RateLimitConfig::default();
        assert_eq!(config.limit_for_method(&Method::GET), 120);
        assert_eq!(config.limit_for_method(&Method::HEAD), 120);
        assert_eq!(config.limit_for_method(&Method::OPTIONS), 120);
    }

    #[test]
    fn test_limit_for_method_writes() {
        let config = RateLimitConfig::default();
        assert_eq!(config.limit_for_method(&Method::POST), 30);
        assert_eq!(config.limit_for_method(&Method::PUT), 30);
        assert_eq!(config.limit_for_method(&Method::PATCH), 30);
        assert_eq!(config.limit_for_method(&Method::DELETE), 30);
    }

    #[test]
    fn test_tier_label() {
        assert_eq!(RateLimitConfig::tier_label(&Method::GET), "r");
        assert_eq!(RateLimitConfig::tier_label(&Method::HEAD), "r");
        assert_eq!(RateLimitConfig::tier_label(&Method::OPTIONS), "r");
        assert_eq!(RateLimitConfig::tier_label(&Method::POST), "w");
        assert_eq!(RateLimitConfig::tier_label(&Method::PUT), "w");
        assert_eq!(RateLimitConfig::tier_label(&Method::PATCH), "w");
        assert_eq!(RateLimitConfig::tier_label(&Method::DELETE), "w");
    }

    #[test]
    fn test_resolve_identity_with_debug_header() {
        let req = Request::builder()
            .uri("/test")
            .header("x-user-id", "550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let identity = resolve_identity(&req, "test_secret");
        assert_eq!(identity, "u:550e8400-e29b-41d4-a716-446655440000");
    }

    #[test]
    fn test_resolve_identity_with_invalid_debug_header() {
        let req = Request::builder()
            .uri("/test")
            .header("x-user-id", "not-a-uuid")
            .body(Body::empty())
            .unwrap();

        let identity = resolve_identity(&req, "test_secret");
        // Should fall back to IP (which will be "unknown" without ConnectInfo)
        assert!(identity.starts_with("ip:"));
    }

    #[test]
    fn test_resolve_identity_with_forwarded_for() {
        let req = Request::builder()
            .uri("/test")
            .header("x-forwarded-for", "203.0.113.50, 70.41.3.18")
            .body(Body::empty())
            .unwrap();

        let identity = resolve_identity(&req, "test_secret");
        assert_eq!(identity, "ip:203.0.113.50");
    }

    #[test]
    fn test_resolve_identity_no_headers() {
        let req = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let identity = resolve_identity(&req, "test_secret");
        assert_eq!(identity, "ip:unknown");
    }

    #[test]
    fn test_resolve_identity_jwt_takes_priority() {
        use crate::services::auth_service::AuthService;

        let user_id = uuid::Uuid::new_v4();
        let secret = "test_jwt_secret";
        let token = AuthService::generate_token(user_id, secret, 60).unwrap();

        let req = Request::builder()
            .uri("/test")
            .header("authorization", format!("Bearer {token}"))
            .header("x-forwarded-for", "203.0.113.50")
            .body(Body::empty())
            .unwrap();

        let identity = resolve_identity(&req, secret);
        assert_eq!(identity, format!("u:{user_id}"));
    }

    #[test]
    fn test_resolve_identity_invalid_jwt_falls_through() {
        let req = Request::builder()
            .uri("/test")
            .header("authorization", "Bearer invalid.jwt.token")
            .header("x-forwarded-for", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let identity = resolve_identity(&req, "test_secret");
        // Invalid JWT should fall through to IP
        assert_eq!(identity, "ip:10.0.0.1");
    }

    #[test]
    fn test_custom_config() {
        let config = RateLimitConfig {
            max_read_requests: 200,
            max_write_requests: 50,
            window_secs: 30,
        };
        assert_eq!(config.limit_for_method(&Method::GET), 200);
        assert_eq!(config.limit_for_method(&Method::POST), 50);
        assert_eq!(config.window_secs, 30);
    }

    #[tokio::test]
    async fn test_rate_limiter_matched_path() {
        use axum::{routing::get, Router};
        use tower::ServiceExt;

        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/0".to_string());
        let redis_pool = if let Ok(Ok(p)) = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            RedisPool::new(&redis_url),
        )
        .await
        {
            Some(p)
        } else {
            println!(
                "Redis not available, skipping rate limiter matched path integration assertions"
            );
            return;
        };

        let config = RateLimitConfig {
            max_read_requests: 100,
            max_write_requests: 100,
            window_secs: 60,
        };

        let layer = RateLimitLayer::new(redis_pool.clone(), config, "test_secret".to_string());

        let app = Router::new()
            .route("/api/v1/employees/:id", get(|| async { "OK" }))
            .layer(layer);

        let req = Request::builder()
            .method(Method::GET)
            .uri("/api/v1/employees/550e8400-e29b-41d4-a716-446655440000")
            .header("x-user-id", "550e8400-e29b-41d4-a716-446655440000")
            .body(Body::empty())
            .unwrap();

        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);

        if let Some(ref pool) = redis_pool {
            let mut conn = pool.get_async_connection().await.unwrap();
            let key = "rate:u:550e8400-e29b-41d4-a716-446655440000:r:/api/v1/employees/:id";
            let exists: bool = redis::cmd("EXISTS")
                .arg(key)
                .query_async(&mut conn)
                .await
                .unwrap();
            assert!(
                exists,
                "Redis rate limit key should be based on MatchedPath template"
            );

            let _: () = redis::cmd("DEL")
                .arg(key)
                .query_async(&mut conn)
                .await
                .unwrap();
        }
    }
}
