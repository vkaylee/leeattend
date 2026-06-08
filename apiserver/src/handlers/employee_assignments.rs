use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    errors::AppError,
    handlers::common::MemberContext,
    models::employee_assignment::{
        AssignmentStatusEnum, CreateEmployeeAssignmentRequest, EmployeeAssignment,
        NewEmployeeAssignment, UpdateEmployeeAssignmentRequest,
    },
    utils::i18n::I18nKey,
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
        (status = 200, description = "List of assignments", body = [EmployeeAssignment])
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip(state, ctx))]
pub async fn list_employee_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<EmployeeAssignment>>, AppError> {
    let (_ws_id, employee_id) = params;
    let repo = state.employees().repo();

    let employments = state
        .employees()
        .employment_repo()
        .list_employee_employments(&ctx.auth_token, employee_id)
        .await?;
    let employment_ids: Vec<Uuid> = employments.into_iter().map(|e| e.id()).collect();

    let mut assignments = Vec::new();
    if !employment_ids.is_empty() {
        let mut batch_map = repo
            .list_assignments_batch(&ctx.auth_token, &employment_ids)
            .await?;
        for id in employment_ids {
            if let Some(mut list) = batch_map.remove(&id) {
                assignments.append(&mut list);
            }
        }
    }

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
#[tracing::instrument(skip(state, ctx))]
pub async fn get_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<EmployeeAssignment>, AppError> {
    let (_ws_id, employee_id, assignment_id) = params;
    let repo = state.employees().repo();

    let employments = state
        .employees()
        .employment_repo()
        .list_employee_employments(&ctx.auth_token, employee_id)
        .await?;

    let mut assignment = None;
    for emp in employments {
        if let Ok(Some(a)) = repo
            .get_employee_assignment(&ctx.auth_token, emp.id(), assignment_id)
            .await
        {
            assignment = Some(a);
            break;
        }
    }

    let assignment =
        assignment.ok_or_else(|| AppError::not_found(I18nKey::EmployeeAssignmentNotFound))?;
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
#[tracing::instrument(skip(state, ctx))]
pub async fn create_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateEmployeeAssignmentRequest>,
) -> Result<Json<EmployeeAssignment>, AppError> {
    let (_ws_id, employee_id) = params;
    let repo = state.employees().repo();

    // Ensure the employee exists
    repo.get_employee_by_id(employee_id, &ctx.auth_token)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeNotFound))?;

    let active_employment = state
        .employees()
        .employment_repo()
        .get_active_employment_on_date(
            &ctx.auth_token,
            employee_id,
            chrono::Utc::now().date_naive(),
        )
        .await?
        .ok_or_else(|| AppError::validation_raw("Employee has no active employment profile"))?;

    let home_location_id = active_employment.primary_location_id().ok_or_else(|| {
        AppError::validation_raw("Employee active employment has no primary location assigned")
    })?;

    // Validate that home and host locations are not the same
    if home_location_id == payload.host_location_id {
        return Err(AppError::validation(I18nKey::HomeHostLocationSame));
    }

    // Additional validations can be done here (e.g. checking overlapping active assignments)

    let new_assignment = NewEmployeeAssignment {
        id: Uuid::new_v4(),
        workspace_id: ctx.workspace_id,
        employment_id: active_employment.id(),
        home_location_id,
        host_location_id: payload.host_location_id,
        start_date: payload.start_date,
        end_date: payload.end_date,
        reason: payload.reason,
        status: AssignmentStatusEnum::Active,
    };

    let created = repo
        .create_employee_assignment(&ctx.auth_token, &new_assignment)
        .await?;
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
#[tracing::instrument(skip(state, ctx))]
pub async fn update_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateEmployeeAssignmentRequest>,
) -> Result<Json<EmployeeAssignment>, AppError> {
    let (_ws_id, employee_id, assignment_id) = params;
    let repo = state.employees().repo();

    let employments = state
        .employees()
        .employment_repo()
        .list_employee_employments(&ctx.auth_token, employee_id)
        .await?;

    let mut found_employment_id = None;
    for emp in &employments {
        if let Ok(Some(_)) = repo
            .get_employee_assignment(&ctx.auth_token, emp.id(), assignment_id)
            .await
        {
            found_employment_id = Some(emp.id());
            break;
        }
    }

    let employment_id_val = found_employment_id
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeAssignmentNotFound))?;

    let updated = repo
        .update_employee_assignment(&ctx.auth_token, employment_id_val, assignment_id, &payload)
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
#[tracing::instrument(skip(state, ctx))]
pub async fn delete_employee_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<()>, AppError> {
    let (_ws_id, employee_id, assignment_id) = params;
    let repo = state.employees().repo();

    let employments = state
        .employees()
        .employment_repo()
        .list_employee_employments(&ctx.auth_token, employee_id)
        .await?;

    let mut found_employment_id = None;
    for emp in &employments {
        if let Ok(Some(_)) = repo
            .get_employee_assignment(&ctx.auth_token, emp.id(), assignment_id)
            .await
        {
            found_employment_id = Some(emp.id());
            break;
        }
    }

    let employment_id_val = found_employment_id
        .ok_or_else(|| AppError::not_found(I18nKey::EmployeeAssignmentNotFound))?;

    let deleted_count = repo
        .delete_employee_assignment(&ctx.auth_token, employment_id_val, assignment_id)
        .await?;

    if deleted_count == 0 {
        return Err(AppError::not_found(I18nKey::EmployeeAssignmentNotFound));
    }

    Ok(Json(()))
}
