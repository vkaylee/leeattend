use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::MemberContext;
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, EmployeeDepartmentHistory,
    EmploymentPeriod, PaginatedResponse, PaginationParams, RehireEmployeeRequest,
    TransferEmployeeRequest, UpdateEmployeeRequest, WorkingCondition,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn get_working_conditions(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<WorkingCondition>>> {
    let conditions = state
        .employee_service
        .get_working_conditions(id, workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to get working conditions".to_string())))?;
    Ok(Json(conditions))
}

pub async fn create_working_condition(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingConditionRequest>,
) -> AppResult<impl IntoResponse> {
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    let cond = state
        .employee_service
        .create_working_condition(id, ctx.workspace_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to create working condition".to_string())))?;
    Ok((StatusCode::CREATED, Json(cond)))
}

pub async fn delete_working_condition(
    Path((_ws_id, id, cond_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    state
        .employee_service
        .delete_working_condition(id, ctx.workspace_id, cond_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to delete working condition".to_string())))?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Employee Management Handlers ---

pub async fn get_employees(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Employee>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let (data, total) = state
        .employee_service
        .get_all_employees(workspace_id, query.q, page, limit)
        .await
        .map_err(|s| AppError::from((s, "Failed to get employees".to_string())))?;

    Ok(Json(PaginatedResponse {
        data,
        total: total as usize,
        page,
        limit,
    }))
}

pub async fn get_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Employee>> {
    let employee = state
        .employee_service
        .get_employee_by_id(id, workspace_id)
        .await
        .map_err(|_| AppError::NotFound("Employee not found".to_string()))?;
    Ok(Json(employee))
}

pub async fn create_employee(
    Path(_ws_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateEmployeeRequest>,
) -> AppResult<impl IntoResponse> {
    // Since a new employee has no department yet, a scoped-only user cannot create employees.
    ctx.require_scoped_permission(Permission::EmployeesCreate, None)?;
    let employee = state
        .employee_service
        .create_employee(ctx.workspace_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to create employee".to_string())))?;
    Ok((StatusCode::CREATED, Json(employee)))
}

pub async fn update_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateEmployeeRequest>,
) -> AppResult<Json<Employee>> {
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    let employee = state
        .employee_service
        .update_employee(id, ctx.workspace_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to update employee".to_string())))?;
    Ok(Json(employee))
}

pub async fn delete_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::EmployeesDelete, target_dept_id)?;
    let workspace_id = ctx.workspace_id;
    state
        .employee_service
        .delete_employee(id, workspace_id)
        .await
        .map_err(|_| AppError::NotFound("Employee not found".to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_employee_history(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<EmploymentPeriod>>> {
    let history = state
        .employee_service
        .get_employment_history(id, workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to get employment history".to_string())))?;
    Ok(Json(history))
}

pub async fn rehire_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<RehireEmployeeRequest>,
) -> AppResult<Json<Employee>> {
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::EmployeesRehire, target_dept_id)?;
    let employee = state
        .employee_service
        .rehire_employee(id, ctx.workspace_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to rehire employee".to_string())))?;
    Ok(Json(employee))
}

pub async fn get_employee_department_history(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<EmployeeDepartmentHistory>>> {
    let history = state
        .employee_service
        .get_department_history(id, workspace_id)
        .await
        .map_err(|s| AppError::from((s, "Failed to get department history".to_string())))?;
    Ok(Json(history))
}

pub async fn transfer_employee(
    Path((_ws_id, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<TransferEmployeeRequest>,
) -> AppResult<Json<EmployeeDepartmentHistory>> {
    let target_dept_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT department_id FROM employee_department_history WHERE employee_id = $1 ORDER BY effective_date DESC LIMIT 1"
    ).bind(id).fetch_optional(state.db.api()).await.unwrap_or(None);

    ctx.require_scoped_permission(Permission::DepartmentsTransfer, target_dept_id)?;
    let history = state
        .employee_service
        .transfer_employee(id, ctx.workspace_id, req)
        .await
        .map_err(|s| AppError::from((s, "Failed to transfer employee".to_string())))?;
    Ok(Json(history))
}
