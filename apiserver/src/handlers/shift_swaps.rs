use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::shift_swap::{
    ApproveShiftSwapRequest, CreateShiftSwapRequest, ShiftSwapRequest, ShiftSwapRequestWithNames,
    ShiftSwapStatus,
};
use crate::repositories::shift_swap_repository::SwapStats;

use crate::AppState;
use crate::UserId;

// ---------------------------------------------------------------------------
// POST /workspaces/:ws_id/shift-swaps
// ---------------------------------------------------------------------------

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/shift-swaps",
    request_body = CreateShiftSwapRequest,
    responses(
        (status = 201, description = "Shift swap request created", body = ShiftSwapRequest)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_shift_swap(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<CreateShiftSwapRequest>,
) -> AppResult<(
    StatusCode,
    Json<crate::models::shift_swap::ShiftSwapRequest>,
)> {
    ctx.require_permission(Permission::ShiftSwapsManage)?;

    let swap = state
        .shifts()
        .swap_service()
        .create_swap(ctx.workspace_id, user_id, req)
        .await?;

    Ok((StatusCode::CREATED, Json(swap)))
}

// ---------------------------------------------------------------------------
// GET /workspaces/:ws_id/shift-swaps?status=pending_admin
// ---------------------------------------------------------------------------

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ShiftSwapQuery {
    pub status: Option<ShiftSwapStatus>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/shift-swaps",
    responses(
        (status = 200, description = "List of shift swap requests", body = [ShiftSwapRequestWithNames])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ShiftSwapQuery
    )
)]
pub async fn list_shift_swaps(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ShiftSwapQuery>,
) -> AppResult<Json<Vec<ShiftSwapRequestWithNames>>> {
    ctx.require_permission(Permission::ShiftSwapsManage)?;

    let swaps = state
        .shifts()
        .swap_service()
        .get_workspace_swaps(ctx.workspace_id, query.status)
        .await?;

    Ok(Json(swaps))
}

// ---------------------------------------------------------------------------
// PUT /workspaces/:ws_id/shift-swaps/:id/approve
// ---------------------------------------------------------------------------

#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/shift-swaps/{id}/approve",
    request_body = ApproveShiftSwapRequest,
    responses(
        (status = 200, description = "Shift swap request approved/rejected")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Shift Swap Request ID")
    )
)]
pub async fn approve_shift_swap(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    ctx: MemberContext,
    Json(req): Json<ApproveShiftSwapRequest>,
) -> AppResult<StatusCode> {
    let (_ws_id, id) = params;
    ctx.require_permission(Permission::ShiftSwapsApprove)?;

    state
        .shifts()
        .swap_service()
        .approve_swap(ctx.workspace_id, user_id, id, req.status)
        .await?;

    Ok(StatusCode::OK)
}

// ---------------------------------------------------------------------------
// GET /workspaces/:ws_id/shift-swaps/stats
// ---------------------------------------------------------------------------

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/shift-swaps/stats",
    responses(
        (status = 200, description = "Workspace shift swap stats", body = SwapStats)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn get_shift_swap_stats(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<crate::repositories::shift_swap_repository::SwapStats>> {
    ctx.require_permission(Permission::ShiftSwapsManage)?;

    let swap_stats = state
        .shifts()
        .swap_repo()
        .get_workspace_swap_stats(&ctx.auth_token)
        .await
        .map_err(|e| {
            AppError::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get swap stats: {e}"),
            ))
        })?;

    Ok(Json(swap_stats))
}
