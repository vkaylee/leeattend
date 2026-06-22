use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::permissions::Permission,
    errors::AppError,
    handlers::common::MemberContext,
    models::{EmployeePhase, EmploymentPhaseDefinition},
    AppState,
};

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateEmploymentPhaseDefinitionRequest {
    pub name: String,
    pub description: Option<String>,
    pub phase_type: String,
    pub rule_config: serde_json::Value,
    pub display_order: Option<i32>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AssignEmployeePhaseRequest {
    pub phase_definition_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub override_config: Option<serde_json::Value>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateEmployeePhaseStatusRequest {
    pub status: String,
    pub evaluation_notes: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employment-phases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreateEmploymentPhaseDefinitionRequest,
    responses(
        (status = 201, description = "Employment phase definition created", body = EmploymentPhaseDefinition)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_employment_phase_definition(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(payload): Json<CreateEmploymentPhaseDefinitionRequest>,
) -> Result<(StatusCode, Json<EmploymentPhaseDefinition>), AppError> {
    ctx.require_permission(Permission::EmploymentPhasesManage)?;

    let def = state
        .employees()
        .employment_phase_repo()
        .create_definition(
            ctx.workspace_id,
            &payload.name,
            payload.description,
            &payload.phase_type,
            &payload.rule_config,
            payload.display_order.unwrap_or(0),
        )
        .await?;

    Ok((StatusCode::CREATED, Json(def)))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employment-phases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "List of employment phase definitions", body = Vec<EmploymentPhaseDefinition>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_employment_phase_definitions(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<Json<Vec<EmploymentPhaseDefinition>>, AppError> {
    let defs = state
        .employees()
        .employment_phase_repo()
        .get_definitions(ctx.workspace_id)
        .await?;
    Ok(Json(defs))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employees/{emp_id}/phases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("emp_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = AssignEmployeePhaseRequest,
    responses(
        (status = 200, description = "Employee phase assigned", body = EmployeePhase)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn assign_employee_phase(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignEmployeePhaseRequest>,
) -> Result<Json<EmployeePhase>, AppError> {
    let (_ws_id, emp_id) = __path_params;
    ctx.require_permission(Permission::EmploymentPhasesManage)?;

    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(emp_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;
    let employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

    let phase = state
        .employees()
        .employment_phase_repo()
        .assign_phase(
            ctx.workspace_id,
            employment_id,
            payload.phase_definition_id,
            payload.start_date,
            payload.end_date,
            payload.override_config,
        )
        .await?;

    Ok(Json(phase))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/employees/{emp_id}/phases/{phase_id}/status",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("emp_id" = Uuid, Path, description = "Employee ID"),
        ("phase_id" = Uuid, Path, description = "Phase ID")
    ),
    request_body = UpdateEmployeePhaseStatusRequest,
    responses(
        (status = 200, description = "Employee phase status updated", body = EmployeePhase)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_employee_phase_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateEmployeePhaseStatusRequest>,
) -> Result<Json<EmployeePhase>, AppError> {
    let (_ws_id, emp_id, phase_id) = __path_params;
    ctx.require_permission(Permission::EmploymentPhasesManage)?;

    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(emp_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;
    let employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

    let phase = state
        .employees()
        .employment_phase_repo()
        .update_phase_status(
            ctx.workspace_id,
            employment_id,
            phase_id,
            &payload.status,
            payload.evaluation_notes,
            ctx.user_id,
        )
        .await?;

    Ok(Json(phase))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employees/{emp_id}/phases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("emp_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "List of employee phases", body = Vec<EmployeePhase>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_employee_phases(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<EmployeePhase>>, AppError> {
    let (_ws_id, emp_id) = __path_params;

    let employee = state
        .employees()
        .repo()
        .get_employee_by_id(emp_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(crate::utils::i18n::I18nKey::EmployeeNotFound))?;
    let employment_id = employee
        .active_employment_id()
        .ok_or_else(|| AppError::validation(crate::utils::i18n::I18nKey::NoActiveEmployment))?;

    let phases = state
        .employees()
        .employment_phase_repo()
        .get_employment_phases(ctx.workspace_id, employment_id)
        .await?;

    Ok(Json(phases))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employments/{employment_id}/phases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    request_body = AssignEmployeePhaseRequest,
    responses(
        (status = 200, description = "Employment phase assigned", body = EmployeePhase)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn assign_employment_phase(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
    Json(payload): Json<AssignEmployeePhaseRequest>,
) -> Result<Json<EmployeePhase>, AppError> {
    let (_ws_id, employment_id) = __path_params;
    ctx.require_permission(Permission::EmploymentPhasesManage)?;

    let phase = state
        .employees()
        .employment_phase_repo()
        .assign_phase(
            ctx.workspace_id,
            employment_id,
            payload.phase_definition_id,
            payload.start_date,
            payload.end_date,
            payload.override_config,
        )
        .await?;

    Ok(Json(phase))
}

#[utoipa::path(
    put,
    path = "/v1/workspaces/{ws_id}/employments/{employment_id}/phases/{phase_id}/status",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID"),
        ("phase_id" = Uuid, Path, description = "Phase ID")
    ),
    request_body = UpdateEmployeePhaseStatusRequest,
    responses(
        (status = 200, description = "Employment phase status updated", body = EmployeePhase)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_employment_phase_status(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateEmployeePhaseStatusRequest>,
) -> Result<Json<EmployeePhase>, AppError> {
    let (_ws_id, employment_id, phase_id) = __path_params;
    ctx.require_permission(Permission::EmploymentPhasesManage)?;

    let phase = state
        .employees()
        .employment_phase_repo()
        .update_phase_status(
            ctx.workspace_id,
            employment_id,
            phase_id,
            &payload.status,
            payload.evaluation_notes,
            ctx.user_id,
        )
        .await?;

    Ok(Json(phase))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employments/{employment_id}/phases",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employment_id" = Uuid, Path, description = "Employment ID")
    ),
    responses(
        (status = 200, description = "List of employment phases", body = Vec<EmployeePhase>)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_employment_phases(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(__path_params): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<EmployeePhase>>, AppError> {
    let (_ws_id, employment_id) = __path_params;
    let phases = state
        .employees()
        .employment_phase_repo()
        .get_employment_phases(ctx.workspace_id, employment_id)
        .await?;

    Ok(Json(phases))
}
