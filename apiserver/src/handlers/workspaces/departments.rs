use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::permissions::Permission;
use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateDepartmentRequest, Department, PaginatedResponse, PaginationParams,
    UpdateDepartmentRequest,
};
use crate::AppState;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedDepartments {
    pub data: Vec<Department>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/departments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of departments", body = PaginatedDepartments)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_departments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    axum::extract::Query(query): axum::extract::Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Department>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (mut departments, total) = state
        .payroll()
        .department_repo()
        .get_departments(&ctx.auth_token, page, limit)
        .await?;

    if !departments.is_empty() {
        let entity_ids: Vec<Uuid> = departments.iter().map(Department::id).collect();
        let mut custom_fields_map = state
            .core()
            .custom_field_service()
            .get_values_for_entities(&ctx.auth_token, &entity_ids)
            .await?;
        for d in &mut departments {
            if let Some(fields) = custom_fields_map.remove(&d.id()) {
                *d = d.clone().with_custom_fields(fields);
            }
        }
    }

    Ok(Json(PaginatedResponse {
        data: departments,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/departments",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateDepartmentRequest,
    responses(
        (status = 201, description = "Department created", body = Department)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateDepartmentRequest>,
) -> AppResult<impl IntoResponse> {
    ctx.require_permission(Permission::DepartmentsCreate)?;
    let mut dept = state
        .payroll()
        .department_repo()
        .create_department(&ctx.auth_token, req.name, req.description, req.manager_id)
        .await?;

    let custom_fields = state
        .core()
        .custom_field_service()
        .get_values_for_entity(&ctx.auth_token, dept.id())
        .await?;
    dept = dept.with_custom_fields(custom_fields);

    Ok((StatusCode::CREATED, Json(dept)))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/departments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Department ID")
    ),
    request_body = UpdateDepartmentRequest,
    responses(
        (status = 200, description = "Department updated", body = Department)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateDepartmentRequest>,
) -> AppResult<Json<Department>> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::DepartmentsUpdate)?;
    let mut dept = state
        .payroll()
        .department_repo()
        .update_department(&ctx.auth_token, id, req)
        .await?;

    let custom_fields = state
        .core()
        .custom_field_service()
        .get_values_for_entity(&ctx.auth_token, id)
        .await?;
    dept = dept.with_custom_fields(custom_fields);

    Ok(Json(dept))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/departments/{id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Department ID to delete")
    ),
    responses(
        (status = 204, description = "Department deleted")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_department(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    let (_, id) = __path_params;
    ctx.require_permission(Permission::DepartmentsDelete)?;
    state
        .payroll()
        .department_repo()
        .delete_department(&ctx.auth_token, id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
