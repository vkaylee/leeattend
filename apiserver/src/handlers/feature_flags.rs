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
use crate::models::feature_flag::{CreateFeatureFlagRequest, UpdateFeatureFlagRequest};
use crate::AppState;

/// GET /workspaces/:ws_id/feature-flags
pub async fn list_feature_flags(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsRead)?;
    let flags = state
        .payroll
        .feature_flag_repo
        .list_flags(ctx.workspace_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list feature flags: {e}");
            AppError::Internal("Failed to list feature flags".into())
        })?;

    Ok(Json(flags))
}

/// POST /workspaces/:ws_id/feature-flags
pub async fn create_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(body): Json<CreateFeatureFlagRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsManage)?;

    if body.flag_key.is_empty() || body.flag_key.len() > 255 {
        return Err(AppError::Validation(
            "flag_key must be between 1 and 255 characters".into(),
        ));
    }

    let flag = state
        .payroll
        .feature_flag_repo
        .create_flag(
            ctx.workspace_id,
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
pub async fn get_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsRead)?;
    let flag = state
        .payroll
        .feature_flag_repo
        .get_flag_by_id(ctx.workspace_id, id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get feature flag: {e}");
            AppError::Internal("Failed to get feature flag".into())
        })?;

    match flag {
        Some(f) => Ok(Json(f).into_response()),
        None => Err(AppError::NotFound("Feature flag not found".into())),
    }
}

/// GET /workspaces/:ws_id/feature-flags/check/:key
pub async fn check_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, key)): Path<(Uuid, String)>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsRead)?;
    let enabled = state
        .payroll
        .feature_flag_repo
        .is_enabled_flag(ctx.workspace_id, &key)
        .await
        .map_err(|e| {
            tracing::error!("Failed to check feature flag: {e}");
            AppError::Internal("Failed to check feature flag".into())
        })?;

    Ok(Json(serde_json::json!({ "key": key, "enabled": enabled })))
}

/// PATCH /workspaces/:ws_id/feature-flags/:id
pub async fn update_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateFeatureFlagRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsManage)?;
    let flag = state
        .payroll
        .feature_flag_repo
        .update_flag(
            ctx.workspace_id,
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
        None => Err(AppError::NotFound("Feature flag not found".into())),
    }
}

/// PUT /workspaces/:ws_id/feature-flags (upsert)
pub async fn upsert_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(body): Json<CreateFeatureFlagRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsManage)?;

    if body.flag_key.is_empty() || body.flag_key.len() > 255 {
        return Err(AppError::Validation(
            "flag_key must be between 1 and 255 characters".into(),
        ));
    }

    let flag = state
        .payroll
        .feature_flag_repo
        .upsert_flag(
            ctx.workspace_id,
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
pub async fn delete_feature_flag(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_, id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::FeatureFlagsManage)?;
    let deleted = state
        .payroll
        .feature_flag_repo
        .delete_flag(ctx.workspace_id, id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete feature flag: {e}");
            AppError::Internal("Failed to delete feature flag".into())
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Err(AppError::NotFound("Feature flag not found".into()))
    }
}
