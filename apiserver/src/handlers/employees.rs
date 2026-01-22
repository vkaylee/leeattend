use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, EmployeeDepartmentHistory, EmployeeSearchQuery,
    RehireEmployeeRequest, TransferEmployeeRequest, UpdateEmployeeRequest,
};

pub async fn get_working_conditions(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match EmployeeService::get_working_conditions(&state.db, id, workspace_id).await {
        Ok(conditions) => Json(conditions).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn create_working_condition(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingConditionRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can manage policies").into_response();
    }
    match EmployeeService::create_working_condition(&state.db, id, ctx.workspace_id, req).await {
        Ok(cond) => (StatusCode::CREATED, Json(cond)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn delete_working_condition(
    Path((_ws_id, id, cond_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return StatusCode::FORBIDDEN.into_response();
    }
    match EmployeeService::delete_working_condition(&state.db, id, ctx.workspace_id, cond_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(status) => status.into_response(),
    }
}
use crate::services::employee_service::EmployeeService;
use crate::AppState;

// --- Employee Management Handlers ---

pub async fn get_employees(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<EmployeeSearchQuery>,
) -> impl IntoResponse {
    match EmployeeService::get_all_employees(&state.db, workspace_id, query.q).await {
        Ok(employees) => Json(employees).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> Result<Json<Employee>, StatusCode> {
    match EmployeeService::get_employee_by_id(&state.db, id, workspace_id).await {
        Ok(employee) => Ok(Json(employee)),
        Err(status) => Err(status),
    }
}

pub async fn create_employee(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateEmployeeRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can create employees").into_response();
    }
    match EmployeeService::create_employee(&state.db, ctx.workspace_id, req).await {
        Ok(employee) => (StatusCode::CREATED, Json(employee)).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn update_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateEmployeeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Ok((StatusCode::FORBIDDEN, "Only admins can update employees").into_response());
    }
    match EmployeeService::update_employee(&state.db, id, ctx.workspace_id, req).await {
        Ok(employee) => Ok(Json(employee).into_response()),
        Err(status) => Err(status),
    }
}

pub async fn delete_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> Result<StatusCode, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Err(StatusCode::FORBIDDEN);
    }
    let workspace_id = ctx.workspace_id;
    match EmployeeService::delete_employee(&state.db, id, workspace_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(status) => Err(status),
    }
}

pub async fn get_employee_history(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> impl IntoResponse {
    match EmployeeService::get_employment_history(&state.db, id, workspace_id).await {
        Ok(history) => Json(history).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn rehire_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<RehireEmployeeRequest>,
) -> impl IntoResponse {
    if !ctx.is_admin_or_owner() {
        return (StatusCode::FORBIDDEN, "Only admins can re-hire employees").into_response();
    }
    match EmployeeService::rehire_employee(&state.db, id, ctx.workspace_id, req).await {
        Ok(employee) => Json(employee).into_response(),
        Err(status) => status.into_response(),
    }
}

pub async fn get_employee_department_history(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> Result<Json<Vec<EmployeeDepartmentHistory>>, StatusCode> {
    match EmployeeService::get_department_history(&state.db, id, workspace_id).await {
        Ok(history) => Ok(Json(history)),
        Err(status) => Err(status),
    }
}

pub async fn transfer_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<TransferEmployeeRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    if !ctx.is_admin_or_owner() {
        return Ok((StatusCode::FORBIDDEN, "Only admins can transfer employees").into_response());
    }
    match EmployeeService::transfer_employee(&state.db, id, ctx.workspace_id, req).await {
        Ok(history) => Ok(Json(history).into_response()),
        Err(status) => Err(status),
    }
}
