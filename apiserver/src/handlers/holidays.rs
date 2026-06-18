use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{CreateHolidayRequest, Holiday, UpdateHolidayRequest};
use crate::repositories::holiday_repository::HolidayLeaveResult;
use crate::utils::i18n::I18nKey;

use crate::AppState;

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListHolidaysQuery {
    pub year: Option<i32>,
    pub location_id: Option<Uuid>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedHolidays {
    pub data: Vec<Holiday>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/holidays",
    responses(
        (status = 200, description = "List of holidays", body = PaginatedHolidays)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ListHolidaysQuery
    )
)]
pub async fn list_holidays(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ListHolidaysQuery>,
) -> AppResult<Json<crate::models::PaginatedResponse<Holiday>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (holidays, total) = state
        .leaves()
        .holiday_repo()
        .list(&ctx.auth_token, query.year, query.location_id, page, limit)
        .await?;

    Ok(Json(crate::models::PaginatedResponse {
        data: holidays,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/holidays/{id}",
    responses(
        (status = 200, description = "Holiday details", body = Holiday)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Holiday ID")
    )
)]
pub async fn get_holiday(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Holiday>> {
    let (_ws_id, id) = params;
    let holiday = state
        .leaves()
        .holiday_repo()
        .get_by_id(&ctx.auth_token, id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::HolidayNotFound))?;

    Ok(Json(holiday))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/holidays",
    request_body = CreateHolidayRequest,
    responses(
        (status = 201, description = "Holiday created", body = Holiday)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_holiday(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateHolidayRequest>,
) -> AppResult<(axum::http::StatusCode, Json<Holiday>)> {
    ctx.require_permission(Permission::HolidaysManage)?;

    let holiday = state
        .leaves()
        .holiday_repo()
        .create_holiday(&ctx.auth_token, &req)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") || e.to_string().contains("duplicate") {
                AppError::conflict(I18nKey::HolidayAlreadyExists)
            } else {
                AppError::Internal(e.to_string())
            }
        })?;

    Ok((axum::http::StatusCode::CREATED, Json(holiday)))
}

#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/holidays/{id}",
    request_body = UpdateHolidayRequest,
    responses(
        (status = 200, description = "Holiday updated", body = Holiday)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Holiday ID")
    )
)]
pub async fn update_holiday(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<UpdateHolidayRequest>,
) -> AppResult<Json<Holiday>> {
    let (_ws_id, id) = params;
    ctx.require_permission(Permission::HolidaysManage)?;

    let holiday = state
        .leaves()
        .holiday_repo()
        .update_holiday(&ctx.auth_token, id, &req)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::HolidayNotFound))?;

    Ok(Json(holiday))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/holidays/{id}",
    responses(
        (status = 204, description = "Holiday deleted")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Holiday ID")
    )
)]
pub async fn delete_holiday(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<axum::http::StatusCode> {
    let (_ws_id, id) = params;
    ctx.require_permission(Permission::HolidaysManage)?;

    let deleted = state
        .leaves()
        .holiday_repo()
        .delete(&ctx.auth_token, id)
        .await?;

    if deleted {
        Ok(axum::http::StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found(I18nKey::HolidayNotFound))
    }
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/holidays/{id}/process-leave",
    responses(
        (status = 200, description = "Holiday leave processed", body = [HolidayLeaveResult])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Holiday ID")
    )
)]
pub async fn process_holiday_leave(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<HolidayLeaveResult>>> {
    let (_ws_id, id) = params;
    ctx.require_permission(Permission::HolidaysManage)?;

    let holiday = state
        .leaves()
        .holiday_repo()
        .get_by_id(&ctx.auth_token, id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::HolidayNotFound))?;

    let results = state
        .leaves()
        .holiday_leave_service()
        .process_holiday_leave(ctx.workspace_id, &holiday)
        .await?;

    Ok(Json(results))
}
