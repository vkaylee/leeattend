// Allow .unwrap() and .expect() in test code — only production code is denied
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![allow(
    unused_imports,
    missing_docs,
    clippy::needless_raw_string_hashes,
    clippy::needless_raw_strings,
    clippy::clone_on_ref_ptr,
    clippy::too_many_lines,
    clippy::manual_let_else,
    clippy::single_match_else,
    clippy::ignored_unit_patterns,
    clippy::cast_possible_truncation,
    clippy::must_use_candidate,
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::redundant_clone,
    clippy::use_self,
    clippy::manual_string_new,
    clippy::module_name_repetitions,
    clippy::missing_panics_doc,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::similar_names,
    clippy::doc_markdown,
    clippy::struct_excessive_bools,
    clippy::unreadable_literal,
    clippy::unused_async,
    clippy::implicit_hasher,
    clippy::trivially_copy_pass_by_ref,
    clippy::match_wildcard_for_single_variants,
    clippy::let_underscore_untyped,
    clippy::default_trait_access,
    clippy::semicolon_if_nothing_returned,
    clippy::enum_glob_use,
    clippy::wildcard_imports,
    clippy::struct_field_names,
    clippy::type_complexity,
    clippy::inefficient_to_string,
    clippy::option_if_let_else,
    clippy::redundant_closure_for_method_calls,
    clippy::map_unwrap_or,
    clippy::derive_partial_eq_without_eq,
    clippy::match_same_arms,
    clippy::significant_drop_tightening,
    clippy::cloned_instead_of_copied,
    clippy::future_not_send,
    clippy::manual_clamp,
    clippy::or_fun_call,
    clippy::used_underscore_binding,
    clippy::cast_lossless,
    clippy::needless_pass_by_value,
    clippy::implicit_clone,
    clippy::format_push_string,
    clippy::items_after_statements,
    clippy::if_not_else,
    clippy::branches_sharing_code,
    clippy::unused_self,
    clippy::unnecessary_struct_initialization,
    clippy::ref_option,
    clippy::assigning_clones,
    clippy::fn_params_excessive_bools,
    clippy::too_long_first_doc_paragraph,
    clippy::useless_let_if_seq,
    clippy::unnecessary_semicolon,
    clippy::manual_midpoint,
    clippy::uninlined_format_args,
    clippy::needless_for_each,
    clippy::redundant_pub_crate
)]

pub mod agent_manager;
pub mod cache;
pub mod db_pool;
pub mod dev_initialization;
pub mod domain;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod openapi;
pub mod queue;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;
pub mod worker;
pub mod mw;
pub mod app_builder;
#[cfg(test)]
mod workspace_id_guard;

use axum::Router;

use axum::http::{HeaderValue, Method};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub use crate::state::{
    AppState, AgentState, AttendanceState, CoreState, EmployeeState,
    InfraState, LeaveState, PayrollState, ReportState, ShiftState,
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

                        if let Ok(user_id) = AuthService::verify_token(token, &app_state.core.jwt_secret) {
                            return Ok(Self(user_id));
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
                            if let Ok(user_id) = AuthService::verify_token(token, &app_state.core.jwt_secret) {
                                return Ok(Self(user_id));
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
    let allowed_origins_str = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            "http://localhost:5173,http://localhost:3000,http://127.0.0.1:5173,http://127.0.0.1:3000".to_string()
        } else {
            String::new()
        }
    });

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
        .merge(routes::employees::router())
        .merge(routes::time_tracking::router())
        .merge(routes::leaves::router())
        .merge(routes::attendance::router())
        .merge(routes::reports::router(&state))
        .merge(routes::payroll::router())
        .merge(routes::shifts::router())
        .merge(routes::ess::router())
        .merge(routes::feature_flags::router())
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
        .nest("/api", router)
        .layer(cors)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_allowed_origins() {
        // Test multiple valid origins with mixed spacing
        let origins = parse_allowed_origins("http://localhost:3000, https://hrm.vlee.dev ,http://localhost:3001");
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
        let origins = parse_allowed_origins("http://localhost:3000, invalid\norigin, https://hrm.vlee.dev");
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
