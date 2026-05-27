use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    handlers::common::MemberContext,
    errors::AppError,
    models::employee_assignment::{
        CreateEmployeeAssignmentRequest, EmployeeAssignment, NewEmployeeAssignment,
        UpdateEmployeeAssignmentRequest, AssignmentStatusEnum
    },
    AppState,
};

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments",
    tag = "Employees",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "List of assignments", body = Vec<EmployeeAssignment>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_employee_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<EmployeeAssignment>>, AppError> {
    let repo = &state.employees.repo;
    let assignments = repo.list_employee_assignments(&ctx.auth_token, employee_id).await?;
    Ok(Json(assignments))
}

#[utoipa::path(
    get,
    path = "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}",
    tag = "Employees",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("assignment_id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Employee assignment", body = EmployeeAssignment),
        (status = 404, description = "Not found")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, employee_id, assignment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<EmployeeAssignment>, AppError> {
    let repo = &state.employees.repo;
    let assignment = repo
        .get_employee_assignment(&ctx.auth_token, employee_id, assignment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Employee assignment not found".to_string()))?;
    Ok(Json(assignment))
}

#[utoipa::path(
    post,
    path = "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments",
    tag = "Employees",
    request_body = CreateEmployeeAssignmentRequest,
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Assignment created", body = EmployeeAssignment)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((ws_id, employee_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateEmployeeAssignmentRequest>,
) -> Result<Json<EmployeeAssignment>, AppError> {
    let repo = &state.employees.repo;

    // Ensure the employee exists and fetch their current primary_location_id.
    let employee = repo
        .get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::NotFound("Employee not found".to_string()))?;

    let home_location_id = employee.primary_location_id;

    // Validate that home and host locations are not the same
    if home_location_id == payload.host_location_id {
        return Err(AppError::Validation("Home location and host location cannot be the same".to_string()));
    }

    // Additional validations can be done here (e.g. checking overlapping active assignments)

    let new_assignment = NewEmployeeAssignment {
        id: Uuid::new_v4(),
        workspace_id: ws_id,
        employee_id,
        home_location_id,
        host_location_id: payload.host_location_id,
        start_date: payload.start_date,
        end_date: payload.end_date,
        reason: payload.reason,
        status: AssignmentStatusEnum::Active,
    };

    let created = repo.create_employee_assignment(&ctx.auth_token, &new_assignment).await?;
    Ok(Json(created))
}

#[utoipa::path(
    patch,
    path = "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}",
    tag = "Employees",
    request_body = UpdateEmployeeAssignmentRequest,
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("assignment_id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Assignment updated", body = EmployeeAssignment)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, employee_id, assignment_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateEmployeeAssignmentRequest>,
) -> Result<Json<EmployeeAssignment>, AppError> {
    let repo = &state.employees.repo;
    
    // Ensure it exists
    repo.get_employee_assignment(&ctx.auth_token, employee_id, assignment_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Employee assignment not found".to_string()))?;

    let updated = repo
        .update_employee_assignment(&ctx.auth_token, employee_id, assignment_id, &payload)
        .await?;
    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}",
    tag = "Employees",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("assignment_id" = Uuid, Path, description = "Assignment ID")
    ),
    responses(
        (status = 200, description = "Assignment deleted")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, employee_id, assignment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    let repo = &state.employees.repo;
    
    let deleted_count = repo
        .delete_employee_assignment(&ctx.auth_token, employee_id, assignment_id)
        .await?;
        
    if deleted_count == 0 {
        return Err(AppError::NotFound("Employee assignment not found".to_string()));
    }
    
    Ok(Json(()))
}
