use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    BulkCreateWorkSegmentsRequest, CreateWorkSegmentRequest, UpdateWorkSegmentRequest, WorkSegment,
    WorkSegmentFilter, WorkSegmentListResponse,
};
use crate::utils::i18n::I18nKey;
use crate::AppState;

/// Maximum number of segments in a single bulk-create request.
const MAX_BULK_SEGMENTS: usize = 1000;

// ── Query Params ────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, utoipa::IntoParams)]
pub struct ListWorkSegmentsQuery {
    /// Comma-separated employment UUIDs
    pub employment_ids: Option<String>,
    /// Start date in YYYY-MM-DD format
    pub start_date: Option<String>,
    /// End date in YYYY-MM-DD format
    pub end_date: Option<String>,
    /// Segment type filter: work | standby | on_call | training | travel
    pub segment_type: Option<String>,
    /// Segment status filter: scheduled | in_progress | completed | cancelled
    pub segment_status: Option<String>,
    /// Source filter: roster | pattern | rule | fixed | manual | auto
    pub source: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

// ── Handlers ────────────────────────────────────────────────────────────────

/// List work segments with optional filters and pagination.
#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/work-segments",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ListWorkSegmentsQuery,
    ),
    responses(
        (status = 200, description = "Paginated list of work segments", body = WorkSegmentListResponse)
    ),
)]
pub async fn list_work_segments(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListWorkSegmentsQuery>,
    ctx: MemberContext,
) -> AppResult<Json<WorkSegmentListResponse>> {
    ctx.require_permission(Permission::RostersRead)?;

    let filter = WorkSegmentFilter {
        employment_ids: query.employment_ids.map(|s| {
            s.split(',')
                .filter_map(|id| Uuid::parse_str(id.trim()).ok())
                .collect()
        }),
        start_date: query
            .start_date
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        end_date: query
            .end_date
            .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        segment_type: query.segment_type,
        segment_status: query.segment_status,
        source: query.source,
    };

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (segments, total_count) = state
        .shifts()
        .work_segment_repo()
        .list_segments(&ctx.auth_token, &filter, page, limit)
        .await?;

    Ok(Json(WorkSegmentListResponse {
        segments,
        total_count,
    }))
}

/// Get a single work segment by ID.
#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/work-segments/{segment_id}",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("segment_id" = Uuid, Path, description = "Work segment ID"),
    ),
    responses(
        (status = 200, description = "Work segment details", body = WorkSegment),
        (status = 404, description = "Work segment not found"),
    ),
)]
pub async fn get_work_segment(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, segment_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<WorkSegment>> {
    ctx.require_permission(Permission::RostersRead)?;

    let segment = state
        .shifts()
        .work_segment_repo()
        .get_segment_by_id(&ctx.auth_token, segment_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::WorkSegmentNotFound))?;

    Ok(Json(segment))
}

/// Create a new work segment.
#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/work-segments",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = CreateWorkSegmentRequest,
    responses(
        (status = 201, description = "Work segment created", body = WorkSegment),
    ),
)]
pub async fn create_work_segment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(mut req): Json<CreateWorkSegmentRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::RostersManage)?;

    // Force source = "manual" for user-created segments
    req.source = "manual".to_string();

    let segment = state
        .shifts()
        .work_segment_repo()
        .create_segment(&ctx.auth_token, &req)
        .await?;

    // Invalidate roster cache because segments changed
    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(segment)))
}

/// Update an existing work segment.
#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/work-segments/{segment_id}",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("segment_id" = Uuid, Path, description = "Work segment ID"),
    ),
    request_body = UpdateWorkSegmentRequest,
    responses(
        (status = 200, description = "Work segment updated", body = WorkSegment),
        (status = 404, description = "Work segment not found"),
    ),
)]
pub async fn update_work_segment(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, segment_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateWorkSegmentRequest>,
) -> AppResult<Json<WorkSegment>> {
    ctx.require_permission(Permission::RostersManage)?;

    let segment = state
        .shifts()
        .work_segment_repo()
        .update_segment(&ctx.auth_token, segment_id, &req)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok(Json(segment))
}

/// Delete a work segment.
#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/work-segments/{segment_id}",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("segment_id" = Uuid, Path, description = "Work segment ID"),
    ),
    responses(
        (status = 204, description = "Work segment deleted"),
        (status = 404, description = "Work segment not found"),
    ),
)]
pub async fn delete_work_segment(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, segment_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::RostersManage)?;

    // Verify existence before delete
    state
        .shifts()
        .work_segment_repo()
        .get_segment_by_id(&ctx.auth_token, segment_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::WorkSegmentNotFound))?;

    state
        .shifts()
        .work_segment_repo()
        .delete_segment(&ctx.auth_token, segment_id)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Bulk-create work segments, optionally replacing existing ones.
#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/work-segments/bulk",
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    ),
    request_body = BulkCreateWorkSegmentsRequest,
    responses(
        (status = 201, description = "Work segments created", body = Vec<WorkSegment>),
    ),
)]
pub async fn bulk_create_work_segments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<BulkCreateWorkSegmentsRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::RostersManage)?;

    if req.segments.len() > MAX_BULK_SEGMENTS {
        return Err(AppError::validation(I18nKey::WorkSegmentBatchTooLarge));
    }

    let segments = state
        .shifts()
        .work_segment_repo()
        .bulk_create_segments(&ctx.auth_token, &req.segments, req.replace_existing)
        .await?;

    if let Some(cache) = state.infra().roster_cache() {
        cache.evict_local(ctx.workspace_id).await;
    }

    Ok((StatusCode::CREATED, Json(segments)))
}
