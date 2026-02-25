use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use redis::AsyncCommands;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::queue::RedisPool;

/// Rate limit configuration.
#[derive(Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests allowed per window.
    pub max_requests: u64,
    /// Window duration in seconds.
    pub window_secs: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 5,
            window_secs: 60,
        }
    }
}

/// Tower Layer that applies Redis-based sliding window rate limiting.
#[derive(Clone)]
pub struct RateLimitLayer {
    redis: Option<RedisPool>,
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(redis: Option<RedisPool>, config: RateLimitConfig) -> Self {
        Self { redis, config }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            redis: self.redis.clone(),
            config: self.config.clone(),
        }
    }
}

/// Tower Service that checks rate limits via Redis before forwarding requests.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    redis: Option<RedisPool>,
    config: RateLimitConfig,
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
        let mut inner = self.inner.clone();
        // Swap so self.inner is the clone (ready) and inner is the original
        std::mem::swap(&mut self.inner, &mut inner);

        Box::pin(async move {
            // Fail-open: if Redis unavailable, allow request through
            let Some(pool) = redis else {
                return inner.call(req).await;
            };

            // Extract client IP from ConnectInfo or forwarded headers
            let ip = req
                .extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ci| ci.0.ip().to_string())
                .unwrap_or_else(|| {
                    req.headers()
                        .get("x-forwarded-for")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.split(',').next())
                        .unwrap_or("unknown")
                        .trim()
                        .to_string()
                });

            let path = req.uri().path().to_string();
            let key = format!("rate:{ip}:{path}");

            // Try rate check — fail-open on Redis errors
            match check_rate(&pool, &key, &config).await {
                Ok(true) => inner.call(req).await,
                Ok(false) => {
                    tracing::warn!(ip = %ip, path = %path, "Rate limit exceeded");
                    let response = (
                        StatusCode::TOO_MANY_REQUESTS,
                        [(
                            "Retry-After",
                            config.window_secs.to_string(),
                        )],
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

/// Check if the request is within rate limits using Redis INCR + EXPIRE.
async fn check_rate(
    pool: &RedisPool,
    key: &str,
    config: &RateLimitConfig,
) -> Result<bool, redis::RedisError> {
    let mut conn = pool.conn();
    let count: u64 = conn.incr(key, 1u64).await?;
    if count == 1 {
        // First request in window — set expiry
        conn.expire::<_, ()>(key, config.window_secs as i64).await?;
    }
    Ok(count <= config.max_requests)
}
