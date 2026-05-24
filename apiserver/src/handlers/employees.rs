use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::{self, MemberContext};
use crate::models::{
    CreateEmployeeRequest, CreateWorkingConditionRequest, Employee, EmployeeDepartmentHistory,
    PaginatedResponse, PaginationParams, RehireEmployeeRequest, TransferEmployeeRequest,
    UpdateEmployeeRequest, WorkingCondition,
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
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<WorkingCondition>>> {
    let conditions = state
        .employees
        .service
        .get_working_conditions(id, workspace_id)
        .await?;
    Ok(Json(conditions))
}

pub async fn create_working_condition(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingConditionRequest>,
) -> AppResult<impl IntoResponse> {
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    let cond = state
        .employees
        .service
        .create_working_condition(id, ctx.workspace_id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(cond)))
}

pub async fn delete_working_condition(
    Path((_, id, cond_id)): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    state
        .employees
        .service
        .delete_working_condition(id, ctx.workspace_id, cond_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Employee Management Handlers ---

pub async fn get_employees(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Query(query): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Employee>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let department_ids: Option<Vec<Uuid>> = query
        .department_id
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.split(',')
                .filter_map(|id| id.trim().parse::<Uuid>().ok())
                .collect()
        })
        .filter(|v: &Vec<Uuid>| !v.is_empty());
    let (data, total) = state
        .employees
        .service
        .get_all_employees(
            workspace_id,
            crate::models::EmployeeFilterOptions {
                search_query: query.q,
                page,
                limit,
                sort_by: query.sort_by,
                sort_dir: query.sort_dir,
                department_ids,
            },
        )
        .await?;

    Ok(Json(PaginatedResponse {
        data,
        total: total as usize,
        page,
        limit,
    }))
}

pub async fn get_employee(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Employee>> {
    let employee = state
        .employees
        .service
        .get_employee_by_id(id, workspace_id)
        .await
        .map_err(|_e| AppError::NotFound("Employee not found".to_string()))?;
    Ok(Json(employee))
}

pub async fn create_employee(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateEmployeeRequest>,
) -> AppResult<impl IntoResponse> {
    // Since a new employee has no department yet, a scoped-only user cannot create employees.
    ctx.require_scoped_permission(Permission::EmployeesCreate, None)?;
    let employee = state
        .employees
        .service
        .create_employee(ctx.workspace_id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(employee)))
}

pub async fn update_employee(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateEmployeeRequest>,
) -> AppResult<Json<Employee>> {
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    let employee = state
        .employees
        .service
        .update_employee(id, ctx.workspace_id, req)
        .await?;
    Ok(Json(employee))
}

pub async fn delete_employee(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesDelete, target_dept_id)?;
    let workspace_id = ctx.workspace_id;
    state
        .employees
        .service
        .delete_employee(id, workspace_id)
        .await
        .map_err(|_e| AppError::NotFound("Employee not found".to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn rehire_employee(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<RehireEmployeeRequest>,
) -> AppResult<Json<Employee>> {
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesRehire, target_dept_id)?;
    let employee = state
        .employees
        .service
        .rehire_employee(id, ctx.workspace_id, req)
        .await?;
    Ok(Json(employee))
}

pub async fn get_employee_department_history(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeDepartmentHistory>>> {
    let history = state
        .employees
        .service
        .get_department_history(id, &ctx.auth_token)
        .await?;
    Ok(Json(history))
}

pub async fn transfer_employee(
    Path((_, id)): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<TransferEmployeeRequest>,
) -> AppResult<Json<EmployeeDepartmentHistory>> {
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::DepartmentsTransfer, target_dept_id)?;
    let history = state
        .employees
        .service
        .transfer_employee(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(history))
}
