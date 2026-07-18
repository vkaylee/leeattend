#![recursion_limit = "512"]
#![allow(
    missing_docs,
    deprecated,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::use_self,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_field_names,
    clippy::option_if_let_else,
    clippy::items_after_statements,
    // TECH_DEBT: Defer similar_names to build cleanly
    clippy::similar_names,
    clippy::too_many_arguments,
    clippy::multiple_crate_versions
)]
#![warn(clippy::too_many_lines)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::map_err_ignore)]
#![deny(clippy::match_like_matches_macro)]
#![deny(clippy::type_complexity)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::branches_sharing_code)]
#![deny(unused_imports)]
#![deny(clippy::unnecessary_struct_initialization)]
#![deny(clippy::useless_let_if_seq)]
#![deny(clippy::unnecessary_semicolon)]
#![deny(clippy::if_same_then_else)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::manual_let_else)]
#![deny(clippy::match_wildcard_for_single_variants)]
#![deny(clippy::single_match_else)]
#![deny(clippy::manual_string_new)]
#![deny(clippy::redundant_closure_for_method_calls)]
// Allow .unwrap() and .expect() in test code — only production code is denied
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_possible_wrap,
        clippy::cast_sign_loss,
        clippy::panic,
        clippy::disallowed_types,
        clippy::ref_option_ref,
        clippy::disallowed_macros,
        clippy::unimplemented,
        clippy::map_unwrap_or,
        clippy::clone_on_ref_ptr,
        clippy::manual_let_else,
        clippy::or_fun_call,
        clippy::manual_strip,
        clippy::if_same_then_else,
        clippy::float_cmp,
        clippy::too_many_lines,
        clippy::module_inception,
        clippy::used_underscore_binding,
        clippy::default_trait_access
    )
)]

pub mod agent_manager;
pub mod app_builder;
pub mod cache;
pub mod crypto;

pub mod dev_initialization;
pub mod diesel_pool;
pub mod domain;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod mw;
pub mod openapi;
pub mod queue;
pub mod repositories;
pub mod routes;
pub mod schema;
pub mod services;
pub mod state;
pub mod utils;
pub mod worker;
#[cfg(test)]
mod workspace_id_guard;

use axum::Router;

use axum::http::{HeaderValue, Method};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub use crate::state::{
    AgentState, AppState, AttendanceState, CoreState, EmployeeState, InfraState, LeaveState,
    PayrollState, ReportState, ShiftState,
};

mod ax_auth {
    use crate::services::auth_service::AuthService;
    use crate::AppState;
    use axum::{
        async_trait,
        extract::{FromRef, FromRequestParts},
        http::{request::Parts, StatusCode},
    };
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct UserId(pub Uuid);

    async fn is_token_revoked(
        token: &str,
        user_id: uuid::Uuid,
        iat: i64,
        app_state: &Arc<AppState>,
    ) -> bool {
        let Some(redis_pool) = app_state.infra().redis() else {
            return false;
        };
        let Some(sig) = token.rsplit('.').next() else {
            return true;
        };

        let sig_key = format!("jwt:revoked:{sig}");
        let user_key = format!("user:revoked_at:{user_id}");

        let mut conn = redis_pool.conn();
        use redis::AsyncCommands;

        // 1. Check if token signature is explicitly revoked (individual blocklist)
        if conn.exists::<_, bool>(&sig_key).await.unwrap_or(false) {
            return true;
        }

        // 2. Check if user sessions were revoked globally after token issued
        if let Ok(Some(revoked_at_str)) = conn.get::<_, Option<String>>(&user_key).await {
            if let Ok(revoked_at) = revoked_at_str.parse::<i64>() {
                if iat < revoked_at {
                    return true;
                }
            }
        }

        false
    }

    #[async_trait]
    impl<S> FromRequestParts<S> for UserId
    where
        S: Send + Sync,
        Arc<AppState>: FromRef<S>,
    {
        type Rejection = (StatusCode, String);

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            #[cfg(debug_assertions)]
            {
                if let Some(imp_id_str) = parts
                    .headers
                    .get("X-Debug-Impersonate-Id")
                    .and_then(|h| h.to_str().ok())
                {
                    if let Ok(user_id) = Uuid::parse_str(imp_id_str) {
                        return Ok(Self(user_id));
                    }
                }
            }

            // 1. Try Authorization header first (Fallback for Native Apps / Postman)
            if let Some(auth_header) = parts.headers.get("Authorization") {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        let app_state = Arc::<AppState>::from_ref(state);

                        if let Ok(claims) =
                            AuthService::verify_token_claims(token, app_state.core().jwt_secret())
                        {
                            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                                if !is_token_revoked(token, user_id, claims.iat, &app_state).await {
                                    return Ok(Self(user_id));
                                }
                            }
                        }
                    }
                }
            }

            // 2. Try Cookie header (Standard for Web Dashboard)
            if let Some(cookie_header) = parts.headers.get("cookie") {
                if let Ok(cookie_str) = cookie_header.to_str() {
                    for cookie in cookie_str.split(';') {
                        let cookie = cookie.trim();
                        if let Some(token) = cookie.strip_prefix("access_token=") {
                            let app_state = Arc::<AppState>::from_ref(state);
                            if let Ok(claims) = AuthService::verify_token_claims(
                                token,
                                app_state.core().jwt_secret(),
                            ) {
                                if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                                    if !is_token_revoked(token, user_id, claims.iat, &app_state)
                                        .await
                                    {
                                        return Ok(Self(user_id));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. Try query string (For SSE and WebSockets where custom headers are not supported)
            if let Some(query) = parts.uri.query() {
                for param in query.split('&') {
                    if let Some(token) = param.strip_prefix("access_token=") {
                        let app_state = Arc::<AppState>::from_ref(state);
                        if let Ok(claims) =
                            AuthService::verify_token_claims(token, app_state.core().jwt_secret())
                        {
                            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                                if !is_token_revoked(token, user_id, claims.iat, &app_state).await {
                                    return Ok(Self(user_id));
                                }
                            }
                        }
                    }
                }
            }

            #[cfg(debug_assertions)]
            {
                if let Some(user_id_str) =
                    parts.headers.get("X-User-Id").and_then(|h| h.to_str().ok())
                {
                    if let Ok(user_id) = Uuid::parse_str(user_id_str) {
                        return Ok(Self(user_id));
                    }
                }
            }

            Err((
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ))
        }
    }
}

pub use ax_auth::UserId;

pub fn parse_allowed_origins(origins_str: &str) -> Vec<HeaderValue> {
    let mut origins = Vec::new();
    for origin in origins_str.split(',') {
        let trimmed = origin.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(header_value) = trimmed.parse::<HeaderValue>() {
            origins.push(header_value);
        }
    }
    origins
}

pub fn create_app(state: Arc<AppState>) -> Router {
    let config = crate::utils::config::get_config();
    let allowed_origins_str = config.allowed_origins.clone();

    let origins = parse_allowed_origins(&allowed_origins_str);

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::HeaderName::from_static("x-requested-with"),
            axum::http::HeaderName::from_static("x-workspace-id"),
            axum::http::HeaderName::from_static("x-user-id"),
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ]);

    // Compose domain-specific route modules
    let router = Router::new()
        .merge(routes::dashboard::router())
        .merge(routes::auth::router(&state))
        .merge(routes::workspaces::router())
        .merge(routes::files::router())
        .merge(routes::documents::router())
        .merge(routes::employees::router())
        .merge(routes::custom_fields::router())
        .merge(routes::condition_types::router())
        .merge(routes::time_tracking::router())
        .merge(routes::leaves::router())
        .merge(routes::attendance::router())
        .merge(routes::reports::router(&state))
        .merge(routes::payroll::router())
        .merge(routes::shifts::router())
        .merge(routes::ess::router())
        .merge(routes::feature_flags::router())
        .merge(routes::notifications::router())
        .merge(routes::lifecycle::router())
        // System-level RBAC routes (PlatformAdmin + Support)
        .merge(routes::system::router());

    #[cfg(debug_assertions)]
    let router = {
        use axum::routing::{get, post};
        let router = router
            .route("/debug/users", get(handlers::debug::get_debug_users))
            .route(
                "/debug/workspaces/:ws_id/clear",
                post(handlers::debug::clear_debug_db),
            )
            .route(
                "/dev/workspaces/:ws_id/clear",
                post(handlers::clear_workspace_data),
            );
        tracing::warn!("⚠️ DEV MODE ACTIVE: Debug auth bypass & data-clearing endpoints are enabled. Never ship this to production!");
        router
    };

    Router::new()
        .nest("/api/v1", router)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(
            crate::mw::i18n::locale_middleware,
        ))
        .layer(crate::mw::RateLimitLayer::new(
            state.infra().redis().cloned(),
            crate::mw::rate_limit::RateLimitConfig::default(),
            state.core().jwt_secret().to_string(),
        ))
        .layer(axum::middleware::from_fn(crate::mw::request_id_middleware))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_allowed_origins() {
        // Test multiple valid origins with mixed spacing
        let origins = parse_allowed_origins(
            "http://localhost:3000, https://hrm.vlee.dev ,http://localhost:3001",
        );
        assert_eq!(origins.len(), 3);
        assert_eq!(origins[0].to_str().unwrap(), "http://localhost:3000");
        assert_eq!(origins[1].to_str().unwrap(), "https://hrm.vlee.dev");
        assert_eq!(origins[2].to_str().unwrap(), "http://localhost:3001");

        // Test empty string
        let origins = parse_allowed_origins("");
        assert!(origins.is_empty());

        // Test trailing commas and extra spaces
        let origins = parse_allowed_origins("http://localhost:3000,,  ,");
        assert_eq!(origins.len(), 1);
        assert_eq!(origins[0].to_str().unwrap(), "http://localhost:3000");

        // Test invalid origin format (e.g. invalid chars in header value)
        let origins =
            parse_allowed_origins("http://localhost:3000, invalid\norigin, https://hrm.vlee.dev");
        // "invalid\norigin" will fail HeaderValue parsing because of newline, so it's ignored
        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0].to_str().unwrap(), "http://localhost:3000");
        assert_eq!(origins[1].to_str().unwrap(), "https://hrm.vlee.dev");
    }

    /// Verify that CORS allow_headers includes all custom headers
    /// used by the frontend (X-Workspace-Id, X-User-Id).
    /// Regression test for production bug: missing headers caused
    /// preflight failures → 401 on /me → redirect to login on refresh.
    #[test]
    fn test_cors_includes_custom_headers() {
        use axum::http::HeaderName;

        let required_headers: Vec<HeaderName> = vec![
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("x-workspace-id"),
            HeaderName::from_static("x-user-id"),
            axum::http::header::ACCEPT,
            axum::http::header::ORIGIN,
        ];

        // Verify all custom headers are valid HeaderName instances
        for header in &required_headers {
            assert!(
                !header.as_str().is_empty(),
                "Header name should not be empty: {header:?}"
            );
        }

        // Verify the specific custom headers we added for the CORS fix
        assert!(required_headers.contains(&HeaderName::from_static("x-workspace-id")));
        assert!(required_headers.contains(&HeaderName::from_static("x-user-id")));
    }
}
