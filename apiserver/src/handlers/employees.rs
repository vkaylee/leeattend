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

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedEmployees {
    pub data: Vec<Employee>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedWorkingConditions {
    pub data: Vec<WorkingCondition>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}/conditions",
    responses(
        (status = 200, description = "Successfully retrieved working conditions", body = PaginatedWorkingConditions)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
        PaginationParams
    )
)]
pub async fn get_working_conditions(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<WorkingCondition>>> {
    let (_workspace_id, id) = params;
    let _workspace_id = ctx.workspace_id;
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (conditions, total) = state
        .employees
        .service
        .get_working_conditions(id, &ctx.auth_token, page, limit)
        .await?;
    Ok(Json(PaginatedResponse {
        data: conditions,
        total: total as usize,
        page,
        limit,
    }))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/{id}/conditions",
    request_body = CreateWorkingConditionRequest,
    responses(
        (status = 201, description = "Successfully created working condition", body = WorkingCondition)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn create_working_condition(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateWorkingConditionRequest>,
) -> AppResult<impl IntoResponse> {
    let (_workspace_id, id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    let cond = state
        .employees
        .service
        .create_working_condition(id, &ctx.auth_token, req)
        .await?;
    Ok((StatusCode::CREATED, Json(cond)))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/employees/{id}/conditions/{cond_id}",
    responses(
        (status = 204, description = "Successfully deleted working condition")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
        ("cond_id" = Uuid, Path, description = "Working Condition ID"),
    )
)]
pub async fn delete_working_condition(
    Path(params): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_workspace_id, id, cond_id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    state
        .employees
        .service
        .delete_working_condition(id, &ctx.auth_token, cond_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Employee Management Handlers ---

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees",
    responses(
        (status = 200, description = "Successfully retrieved employees list", body = PaginatedEmployees)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        PaginationParams
    )
)]
pub async fn get_employees(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Employee>>> {
    let _workspace_id = ctx.workspace_id;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);
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
            &ctx.auth_token,
            crate::models::EmployeeFilterOptions {
                search_query: query.q,
                page,
                limit,
                sort_by: query.sort_by,
                sort_dir: query.sort_dir,
                department_ids,
                permission: query.permission,
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

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}",
    responses(
        (status = 200, description = "Successfully retrieved employee details", body = Employee)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn get_employee(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Employee>> {
    let (_workspace_id, id) = params;
    let _workspace_id = ctx.workspace_id;

    let employee = state
        .employees
        .service
        .get_employee_by_id(id, &ctx.auth_token)
        .await
        .map_err(|_e| AppError::NotFound("Employee not found".to_string()))?;
    Ok(Json(employee))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees",
    request_body = CreateEmployeeRequest,
    responses(
        (status = 201, description = "Successfully created employee", body = Employee)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    )
)]
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
        .create_employee(&ctx.auth_token, req)
        .await?;
    Ok((StatusCode::CREATED, Json(employee)))
}

#[utoipa::path(
    put,
    path = "/workspaces/{workspace_id}/employees/{id}",
    request_body = UpdateEmployeeRequest,
    responses(
        (status = 200, description = "Successfully updated employee details", body = Employee)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn update_employee(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<UpdateEmployeeRequest>,
) -> AppResult<Json<Employee>> {
    let (_workspace_id, id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    let employee = state
        .employees
        .service
        .update_employee(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(employee))
}

#[utoipa::path(
    delete,
    path = "/workspaces/{workspace_id}/employees/{id}",
    responses(
        (status = 204, description = "Successfully deleted employee")
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn delete_employee(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_workspace_id, id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesDelete, target_dept_id)?;
    let _workspace_id = ctx.workspace_id;
    state
        .employees
        .service
        .delete_employee(id, &ctx.auth_token)
        .await
        .map_err(|_e| AppError::NotFound("Employee not found".to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/{id}/rehire",
    request_body = RehireEmployeeRequest,
    responses(
        (status = 200, description = "Successfully rehired employee", body = Employee)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn rehire_employee(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<RehireEmployeeRequest>,
) -> AppResult<Json<Employee>> {
    let (_workspace_id, id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesRehire, target_dept_id)?;
    let employee = state
        .employees
        .service
        .rehire_employee(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(employee))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}/departments",
    responses(
        (status = 200, description = "Successfully retrieved employee department history", body = [EmployeeDepartmentHistory])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn get_employee_department_history(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeDepartmentHistory>>> {
    let (_workspace_id, id) = params;
    let history = state
        .employees
        .service
        .get_department_history(id, &ctx.auth_token)
        .await?;
    Ok(Json(history))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/{id}/departments",
    request_body = TransferEmployeeRequest,
    responses(
        (status = 200, description = "Successfully transferred employee to department", body = EmployeeDepartmentHistory)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
    )
)]
pub async fn transfer_employee(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<TransferEmployeeRequest>,
) -> AppResult<Json<EmployeeDepartmentHistory>> {
    let (_workspace_id, id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::DepartmentsTransfer, target_dept_id)?;
    let history = state
        .employees
        .service
        .transfer_employee(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(history))
}
