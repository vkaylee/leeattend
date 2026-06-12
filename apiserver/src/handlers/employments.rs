use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::employment::{CreateEmploymentRequest, Employment, UpdateEmploymentRequest},
    utils::i18n::I18nKey,
    AppState,
};

/// Lấy danh sách mối quan hệ công việc của một nhân viên
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Danh sách quan hệ công việc", body = [Employment])
    )
)]
pub async fn list_employee_employments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<Employment>>> {
    let (_ws_id, employee_id) = params;

    let mut list = state
        .employees()
        .employment_repo()
        .list_employee_employments(&ctx.auth_token, employee_id)
        .await?;

    if !list.is_empty() {
        let entity_ids: Vec<Uuid> = list.iter().map(Employment::id).collect();
        let mut custom_fields_map = state
            .core()
            .custom_field_service()
            .get_values_for_entities(ctx.workspace_id, &entity_ids)
            .await?;
        for emp in &mut list {
            if let Some(fields) = custom_fields_map.remove(&emp.id()) {
                *emp = emp.clone().with_custom_fields(fields);
            }
        }
    }

    Ok(Json(list))
}

/// Lấy chi tiết mối quan hệ công việc
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    responses(
        (status = 200, description = "Chi tiết quan hệ công việc", body = Employment)
    )
)]
pub async fn get_employment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<Employment>> {
    let (_ws_id, _employee_id, employment_id) = params;

    let mut emp = state
        .employees()
        .employment_repo()
        .get_employment_by_id(&ctx.auth_token, employment_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmploymentNotFound))?;

    let custom_fields = state
        .core()
        .custom_field_service()
        .get_values_for_entity(ctx.workspace_id, employment_id)
        .await?;

    emp = emp.with_custom_fields(custom_fields);

    Ok(Json(emp))
}

/// Tạo mới một mối quan hệ công việc
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = CreateEmploymentRequest,
    responses(
        (status = 201, description = "Đã tạo quan hệ công việc", body = Employment)
    )
)]
pub async fn create_employment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateEmploymentRequest>,
) -> AppResult<(StatusCode, Json<Employment>)> {
    let (_ws_id, _employee_id) = params;

    let mut emp = state
        .employees()
        .service()
        .create_employment(&ctx.auth_token, &payload)
        .await?;

    let custom_fields = state
        .core()
        .custom_field_service()
        .get_values_for_entity(ctx.workspace_id, emp.id())
        .await?;

    emp = emp.with_custom_fields(custom_fields);

    Ok((StatusCode::CREATED, Json(emp)))
}

/// Cập nhật mối quan hệ công việc
#[utoipa::path(
    patch,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    request_body = UpdateEmploymentRequest,
    responses(
        (status = 200, description = "Đã cập nhật quan hệ công việc", body = Employment)
    )
)]
pub async fn update_employment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateEmploymentRequest>,
) -> AppResult<Json<Employment>> {
    let (_ws_id, _employee_id, employment_id) = params;

    let mut emp = state
        .employees()
        .service()
        .update_employment(&ctx.auth_token, employment_id, &payload)
        .await?;

    let custom_fields = state
        .core()
        .custom_field_service()
        .get_values_for_entity(ctx.workspace_id, employment_id)
        .await?;

    emp = emp.with_custom_fields(custom_fields);

    Ok(Json(emp))
}

/// Xóa (soft delete) mối quan hệ công việc
#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/employments/{employment_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    responses(
        (status = 204, description = "Đã xóa quan hệ công việc")
    )
)]
pub async fn delete_employment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    let (_ws_id, _employee_id, employment_id) = params;

    state
        .employees()
        .employment_repo()
        .delete_employment(&ctx.auth_token, employment_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
