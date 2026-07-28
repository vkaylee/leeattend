use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{CreateFeatureFlagRequest, FeatureFlag, UpdateFeatureFlagRequest, CascadeDualRunStatus};
use crate::utils::i18n::I18nKey;
use crate::AppState;

/// GET /workspaces/:ws_id/feature-flags
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/feature-flags",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of feature flags", body = Vec<FeatureFlag>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_feature_flags(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsRead)?;
    let flags = state
        .payroll()
        .feature_flag_repo()
        .list_flags(&ctx.auth_token)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list feature flags: {e}");
            AppError::Internal("Failed to list feature flags".into())
        })?;

    Ok(Json(flags))
}

/// POST /workspaces/:ws_id/feature-flags
#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/feature-flags",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateFeatureFlagRequest,
    responses(
        (status = 201, description = "Feature flag created", body = FeatureFlag)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(body): Json<CreateFeatureFlagRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsManage)?;

    if body.flag_key.is_empty() || body.flag_key.len() > 255 {
        return Err(AppError::validation(I18nKey::InvalidFeatureFlagKey));
    }

    let flag = state
        .payroll()
        .feature_flag_repo()
        .create_flag(
            &ctx.auth_token,
            &body.flag_key,
            body.enabled,
            body.description.as_deref(),
            &body.metadata,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create feature flag: {e}");
            AppError::Internal("Failed to create feature flag".into())
        })?;

    Ok((StatusCode::CREATED, Json(flag)))
}

/// GET /workspaces/:ws_id/feature-flags/:id
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/feature-flags/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Feature Flag ID")
    ),
    responses(
        (status = 200, description = "Feature flag details", body = FeatureFlag),
        (status = 404, description = "Feature flag not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::FeatureFlagsRead)?;
    let flag = state
        .payroll()
        .feature_flag_repo()
        .get_flag_by_id(&ctx.auth_token, id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get feature flag: {e}");
            AppError::Internal("Failed to get feature flag".into())
        })?;

    match flag {
        Some(f) => Ok(Json(f).into_response()),
        None => Err(AppError::not_found(I18nKey::FeatureFlagNotFound)),
    }
}

/// GET /workspaces/:ws_id/feature-flags/check/:key
#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/feature-flags/check/{key}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("key" = String, Path, description = "Feature Flag Key")
    ),
    responses(
        (status = 200, description = "Feature flag status", body = crate::models::feature_flag::FeatureFlagCheckResponse)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn check_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, String)>,
) -> AppResult<impl IntoResponse> {
    let (_, key) = __path_params;
    ctx.require_permission(Permission::FeatureFlagsRead)?;
    let enabled = state
        .payroll()
        .feature_flag_repo()
        .is_enabled_flag(&ctx.auth_token, &key)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check feature flag: {e}");
            AppError::Internal("Failed to check feature flag".into())
        })?;

    Ok(Json(serde_json::json!({ "key": key, "enabled": enabled })))
}

/// PATCH /workspaces/:ws_id/feature-flags/:id
#[utoipa::path(
    patch,
    path = "/v1/workspaces/{ws_id}/feature-flags/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Feature Flag ID")
    ),
    request_body = UpdateFeatureFlagRequest,
    responses(
        (status = 200, description = "Feature flag updated", body = FeatureFlag),
        (status = 404, description = "Feature flag not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateFeatureFlagRequest>,
) -> AppResult<impl IntoResponse> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::FeatureFlagsManage)?;
    let flag = state
        .payroll()
        .feature_flag_repo()
        .update_flag(
            &ctx.auth_token,
            id,
            body.enabled,
            body.description.as_deref(),
            body.metadata.as_ref(),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to update feature flag: {e}");
            AppError::Internal("Failed to update feature flag".into())
        })?;

    match flag {
        Some(f) => Ok(Json(f).into_response()),
        None => Err(AppError::not_found(I18nKey::FeatureFlagNotFound)),
    }
}

/// PUT /workspaces/:ws_id/feature-flags (upsert)
#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/feature-flags",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateFeatureFlagRequest,
    responses(
        (status = 200, description = "Feature flag upserted", body = FeatureFlag)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn upsert_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(body): Json<CreateFeatureFlagRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsManage)?;

    if body.flag_key.is_empty() || body.flag_key.len() > 255 {
        return Err(AppError::validation(I18nKey::InvalidFeatureFlagKey));
    }

    let flag = state
        .payroll()
        .feature_flag_repo()
        .upsert_flag(
            &ctx.auth_token,
            &body.flag_key,
            body.enabled,
            body.description.as_deref(),
            &body.metadata,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to upsert feature flag: {e}");
            AppError::Internal("Failed to upsert feature flag".into())
        })?;

    Ok(Json(flag))
}

/// DELETE /workspaces/:ws_id/feature-flags/:id
#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/feature-flags/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Feature Flag ID")
    ),
    responses(
        (status = 204, description = "Feature flag deleted"),
        (status = 404, description = "Feature flag not found")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::FeatureFlagsManage)?;
    let deleted = state
        .payroll()
        .feature_flag_repo()
        .delete_flag(&ctx.auth_token, id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete feature flag: {e}");
            AppError::Internal("Failed to delete feature flag".into())
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AppError::not_found(I18nKey::FeatureFlagNotFound))
    }
}

// ── ADR-005 Dual-Run Status ──────────────────────────────────────────────

/// GET /workspaces/:ws_id/cascade-dual-run/status
///
/// Returns the current dual-run window status for the ADR-005 cascade cutover.
/// Computes days remaining from the `cutover_at` in the `cascade_as_primary`
/// feature flag metadata (30-day window per ADR-005 §3).
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/cascade-dual-run/status",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
    ),
    responses(
        (status = 200, description = "Dual-run status", body = crate::models::feature_flag::CascadeDualRunStatus)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[allow(clippy::unused_async)]
pub async fn get_cascade_dual_run_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsRead)?;

    const WINDOW_DAYS: i64 = 30;

    let flag = state
        .payroll()
        .feature_flag_repo()
        .get_flag_by_key(&ctx.auth_token, "cascade_as_primary")
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch cascade_as_primary flag: {e}");
            AppError::Internal("Failed to fetch dual-run status".into())
        })?;

    let now = chrono::Utc::now();

    let (enabled, cutover_at, window_closes_at, days_remaining, phase, message) = match flag {
        Some(f) if f.enabled => {
            let cutover_str = f
                .metadata
                .get("cutover_at")
                .and_then(|v| v.as_str())
                .map(String::from);

            let closes_dt = cutover_str
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .or_else(|| {
                    // Fallback: use flag updated_at + 30 days
                    Some(f.updated_at + chrono::Duration::days(WINDOW_DAYS))
                });

            let days = closes_dt
                .as_ref()
                .map(|closes| {
                    let duration = *closes - now;
                    duration.num_days().max(0)
                });

            let (phase_str, msg) = match days {
                Some(0) => ("closed", "Dual-run window closed — rollback path disabled.".into()),
                Some(1) => ("final", "1 day remaining in dual-run window. Rollback path closes tomorrow!".into()),
                Some(d) if d <= 3 => ("critical", format!("{d} days remaining — rollback window closing soon!")),
                Some(d) if d <= 7 => ("closing_soon", format!("{d} days remaining in dual-run window.")),
                Some(d) => ("active", format!("Dual-run active — {d} days remaining. Rollback available.")),
                None => ("active", "Dual-run active. Rollback available.".into()),
            };

            let closes_str = closes_dt.map(|dt| dt.to_rfc3339());

            (true, cutover_str, closes_str, days, phase_str.to_string(), msg)
        }
        _ => {
            (
                false,
                None,
                None,
                None,
                "closed".to_string(),
                "Dual-run window not active.".to_string(),
            )
        }
    };

    Ok(Json(CascadeDualRunStatus {
        enabled,
        cutover_at,
        window_closes_at,
        days_remaining,
        window_phase: phase,
        message,
    }))
}
