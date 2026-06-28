use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::handlers;
use crate::mw;
use crate::AppState;

pub fn router(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/auth/login", post(handlers::login))
        .route("/auth/register", post(handlers::register))
        .route_layer(mw::RateLimitLayer::new(
            state.infra().redis().cloned(),
            mw::rate_limit::RateLimitConfig::default(),
            state.core().jwt_secret().to_string(),
        ))
        // ── Below the rate-limiter: these have their own internal rate limiting ──
        .route("/auth/forgot-password", post(handlers::forgot_password))
        .route("/auth/reset-password", post(handlers::reset_password))
        .route("/health", get(handlers::health_check))
        .route("/health/ready", get(handlers::readiness_check))
        .route("/auth/refresh", post(handlers::refresh_token))
        .route("/auth/logout", post(handlers::logout))
        .route("/auth/change-password", post(handlers::change_password))
}
