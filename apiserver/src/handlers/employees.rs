use crate::domain::permissions::Permission;
use crate::errors::{AppError, AppResult};
use crate::handlers::common::{self, MemberContext};
use crate::models::{
    CompensationHistoryResponse, CreateEmployeeRequest, CreateWorkingConditionRequest, Employee,
    EmployeeDepartmentHistory, PaginatedResponse, PaginationParams, RehireEmployeeRequest,
    TransferEmployeeRequest, UpdateEmployeeRequest, WorkingCondition,
};
use crate::utils::i18n::I18nKey;
use crate::AppState;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Datelike;
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
#[tracing::instrument(skip(state, ctx))]
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
        .employees()
        .service()
        .get_working_conditions(id, &ctx.auth_token, page, limit)
        .await?;
    Ok(Json(PaginatedResponse {
        data: conditions,
        total: usize::try_from(total).unwrap_or(0),
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
#[tracing::instrument(skip(state, ctx))]
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
        .employees()
        .service()
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
#[tracing::instrument(skip(state, ctx))]
pub async fn delete_working_condition(
    Path(params): Path<(Uuid, Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_workspace_id, id, cond_id) = params;
    let target_dept_id = common::current_department_id(&state, &ctx.auth_token, id).await;
    ctx.require_scoped_permission(Permission::EmployeesUpdate, target_dept_id)?;
    state
        .employees()
        .service()
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
#[tracing::instrument(skip(state, ctx))]
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

    let mut filter_ids = None;
    if let Some(ref cf_filters_str) = query.custom_field_filters {
        if !cf_filters_str.is_empty() {
            let definitions = state
                .core()
                .custom_field_service()
                .list_definitions(ctx.workspace_id, Some("employee".to_string()))
                .await?;
            let def_map: std::collections::HashMap<String, Uuid> = definitions
                .into_iter()
                .map(|def| (def.field_key().to_string(), def.id()))
                .collect();

            let mut current_intersect: Option<std::collections::HashSet<Uuid>> = None;
            for filter_part in cf_filters_str.split(',') {
                let parts: Vec<&str> = filter_part.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let key = parts[0].trim();
                    let val = parts[1].trim().to_string();
                    if let Some(&def_id) = def_map.get(key) {
                        let matching_entity_ids = state
                            .core()
                            .custom_field_service()
                            .find_entity_ids_by_value(ctx.workspace_id, def_id, val)
                            .await?;
                        let matching_set: std::collections::HashSet<Uuid> =
                            matching_entity_ids.into_iter().collect();
                        if let Some(ref mut set) = current_intersect {
                            *set = set.intersection(&matching_set).copied().collect();
                        } else {
                            current_intersect = Some(matching_set);
                        }
                    }
                }
            }

            if let Some(set) = current_intersect {
                filter_ids = Some(set.into_iter().collect::<Vec<Uuid>>());
            } else {
                filter_ids = Some(vec![]);
            }
        }
    }

    let (mut data, total) = state
        .employees()
        .service()
        .get_all_employees(
            &ctx.auth_token,
            crate::models::EmployeeFilterOptions::new(
                query.q,
                page,
                limit,
                query.sort_by,
                query.sort_dir,
                department_ids,
                query.permission,
                filter_ids,
            ),
        )
        .await?;

    if !data.is_empty() {
        let entity_ids: Vec<Uuid> = data.iter().map(Employee::id).collect();
        let mut custom_fields_map = state
            .core()
            .custom_field_service()
            .get_values_for_entities(ctx.workspace_id, &entity_ids)
            .await?;
        for emp in &mut data {
            if let Some(fields) = custom_fields_map.remove(&emp.id()) {
                *emp = emp.clone().with_custom_fields(fields);
            }
        }
    }

    Ok(Json(PaginatedResponse {
        data,
        total: usize::try_from(total).unwrap_or(0),
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
#[tracing::instrument(skip(state, ctx))]
pub async fn get_employee(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Employee>> {
    let (_workspace_id, id) = params;
    let _workspace_id = ctx.workspace_id;

    let mut employee = state
        .employees()
        .service()
        .get_employee_by_id(id, &ctx.auth_token)
        .await
        .map_err(|_e| AppError::not_found(I18nKey::EmployeeNotFound))?;

    let custom_fields = state
        .core()
        .custom_field_service()
        .get_values_for_entity(ctx.workspace_id, id)
        .await?;

    employee = employee.with_custom_fields(custom_fields);

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
#[tracing::instrument(skip(state, ctx))]
pub async fn create_employee(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreateEmployeeRequest>,
) -> AppResult<impl IntoResponse> {
    // Since a new employee has no department yet, a scoped-only user cannot create employees.
    ctx.require_scoped_permission(Permission::EmployeesCreate, None)?;
    let employee = state
        .employees()
        .service()
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
#[tracing::instrument(skip(state, ctx))]
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
        .employees()
        .service()
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
#[tracing::instrument(skip(state, ctx))]
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
        .employees()
        .service()
        .delete_employee(id, &ctx.auth_token)
        .await
        .map_err(|_e| AppError::not_found(I18nKey::EmployeeNotFound))?;
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
#[tracing::instrument(skip(state, ctx))]
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
        .employees()
        .service()
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
#[tracing::instrument(skip(state, ctx))]
pub async fn get_employee_department_history(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<EmployeeDepartmentHistory>>> {
    let (_workspace_id, id) = params;
    let history = state
        .employees()
        .service()
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
#[tracing::instrument(skip(state, ctx))]
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
        .employees()
        .service()
        .transfer_employee(id, &ctx.auth_token, req)
        .await?;
    Ok(Json(history))
}

#[derive(Debug, serde::Deserialize)]
pub struct CompensationHistoryQuery {
    pub employment_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/{id}/compensation-history",
    responses(
        (status = 200, description = "Successfully retrieved employee compensation history", body = CompensationHistoryResponse)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        ("id" = Uuid, Path, description = "Employee ID"),
        ("employment_id" = Option<Uuid>, Query, description = "Optional filter by employment ID")
    )
)]
#[tracing::instrument(skip(state, ctx))]
pub async fn get_compensation_history(
    Path(params): Path<(Uuid, Uuid)>,
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<CompensationHistoryQuery>,
) -> AppResult<Json<crate::models::CompensationHistoryResponse>> {
    let (_workspace_id, id) = params;
    ctx.require_permission(Permission::EmployeesRead)?;

    let timeline = state
        .employees()
        .service()
        .get_compensation_history(&ctx.auth_token, id, query.employment_id)
        .await?;

    Ok(Json(crate::models::CompensationHistoryResponse {
        timeline,
    }))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ProratedLeavePreview {
    pub leave_type_id: Uuid,
    pub leave_type_code: String,
    pub leave_type_name: String,
    pub old_entitlement: i32,
    pub new_entitlement: i32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ValidTerminationRow {
    pub row_number: usize,
    pub employee_id: Uuid,
    pub employee_code: String,
    pub full_name: String,
    pub start_date: chrono::NaiveDate,
    pub termination_date: chrono::NaiveDate,
    pub employment_id: Uuid,
    pub prorated_leaves: Vec<ProratedLeavePreview>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct InvalidTerminationRow {
    pub row_number: usize,
    pub employee_code: Option<String>,
    pub raw_termination_date: Option<String>,
    pub reason: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct TerminationPreviewResponse {
    pub valid_rows: Vec<ValidTerminationRow>,
    pub invalid_rows: Vec<InvalidTerminationRow>,
    pub can_commit: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ConfirmTerminationRow {
    pub employment_id: Uuid,
    pub termination_date: chrono::NaiveDate,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ConfirmTerminationRequest {
    pub rows: Vec<ConfirmTerminationRow>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ConfirmTerminationResponse {
    pub success_count: usize,
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/import-termination/preview",
    request_body(content = String, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully generated bulk termination preview", body = TerminationPreviewResponse)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::too_many_lines
)]
pub async fn import_termination_preview(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    mut multipart: Multipart,
) -> AppResult<Json<TerminationPreviewResponse>> {
    ctx.require_permission(Permission::EmployeesUpdate)?;

    let path = crate::utils::upload::extract_uploaded_file(
        &state,
        Some(&ctx.auth_token),
        &mut multipart,
        "file",
        crate::utils::upload::MAX_UPLOAD_FILE_SIZE,
        "imports",
    )
    .await?;

    let file_bytes = match tokio::fs::read(&path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = tokio::fs::remove_file(&path).await;
            return Err(AppError::Internal(format!(
                "Failed to read temporary file: {e}"
            )));
        }
    };
    let _ = tokio::fs::remove_file(&path).await;

    let rows = crate::utils::file_parser::extract_mapped_rows(
        &file_bytes,
        &["employee_code", "termination_date"],
    )?;

    let mut valid_rows = Vec::new();
    let mut invalid_rows = Vec::new();

    for (line_number, mapped_row) in rows {
        let raw_code = mapped_row.get("employee_code").cloned().unwrap_or_default();
        let raw_date = mapped_row
            .get("termination_date")
            .cloned()
            .unwrap_or_default();

        if raw_code.trim().is_empty() {
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code),
                raw_termination_date: Some(raw_date),
                reason: crate::utils::i18n::translate(
                    &crate::utils::i18n::get_locale(),
                    "error.full_name_required",
                ),
            });
            continue;
        }

        let Some(termination_date) = crate::utils::datetime::parse_date(&raw_date) else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code.clone()),
                raw_termination_date: Some(raw_date.clone()),
                reason: crate::utils::i18n::translate(
                    &crate::utils::i18n::get_locale(),
                    "error.invalid_date_format",
                ),
            });
            continue;
        };

        let normalized_code = crate::utils::normalize_employee_id(&raw_code);
        let Some(employee) = state
            .employees()
            .repo()
            .get_employee_by_id_number(&normalized_code, &ctx.auth_token)
            .await?
        else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code.clone()),
                raw_termination_date: Some(raw_date.clone()),
                reason: crate::utils::i18n::translate(
                    &crate::utils::i18n::get_locale(),
                    "error.employee_not_found",
                ),
            });
            continue;
        };

        let employments = state
            .employees()
            .employment_repo()
            .list_employee_employments(&ctx.auth_token, employee.id())
            .await?;

        let active_employment = employments.into_iter().find(|emp| emp.status() == "active");

        let Some(active_employment) = active_employment else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code.clone()),
                raw_termination_date: Some(raw_date.clone()),
                reason: crate::utils::i18n::translate(
                    &crate::utils::i18n::get_locale(),
                    "error.no_active_employment",
                ),
            });
            continue;
        };

        if termination_date < active_employment.start_date() {
            let locale = crate::utils::i18n::get_locale();
            let reason = if locale == "vi" {
                format!(
                    "Ngày nghỉ việc không được trước ngày bắt đầu làm việc ({})",
                    active_employment.start_date().format("%d/%m/%Y")
                )
            } else {
                format!(
                    "Termination date cannot be before employment start date ({})",
                    active_employment.start_date().format("%d/%m/%Y")
                )
            };
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code.clone()),
                raw_termination_date: Some(raw_date.clone()),
                reason,
            });
            continue;
        }

        // Calculate leave proration preview
        let year = termination_date.year();
        let Some(start_of_year) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code.clone()),
                raw_termination_date: Some(raw_date.clone()),
                reason: "Invalid start of year".to_string(),
            });
            continue;
        };
        let Some(end_of_year) = chrono::NaiveDate::from_ymd_opt(year, 12, 31) else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code.clone()),
                raw_termination_date: Some(raw_date.clone()),
                reason: "Invalid end of year".to_string(),
            });
            continue;
        };

        let eff_start = std::cmp::max(start_of_year, active_employment.start_date());
        let days_worked = (termination_date - eff_start).num_days() + 1;
        let total_days = (end_of_year - start_of_year).num_days() + 1;
        let proration_fraction = days_worked as f64 / total_days as f64;

        let balances = state
            .leaves()
            .balance_repo()
            .get_employee_balances(&ctx.auth_token, employee.id(), year)
            .await?;

        let mut prorated_leaves = Vec::new();
        for emp_bal in balances {
            let prorated_entitlement =
                (f64::from(emp_bal.entitlement) * proration_fraction).round() as i32;

            prorated_leaves.push(ProratedLeavePreview {
                leave_type_id: emp_bal.leave_type_id,
                leave_type_code: emp_bal.leave_type_code.clone(),
                leave_type_name: emp_bal.leave_type_name.clone(),
                old_entitlement: emp_bal.entitlement,
                new_entitlement: prorated_entitlement,
            });
        }

        valid_rows.push(ValidTerminationRow {
            row_number: line_number,
            employee_id: employee.id(),
            employee_code: raw_code.clone(),
            full_name: employee.full_name().to_string(),
            start_date: active_employment.start_date(),
            termination_date,
            employment_id: active_employment.id(),
            prorated_leaves,
        });
    }

    let can_commit = !valid_rows.is_empty();

    Ok(Json(TerminationPreviewResponse {
        valid_rows,
        invalid_rows,
        can_commit,
    }))
}

#[utoipa::path(
    post,
    path = "/workspaces/{workspace_id}/employees/import-termination/confirm",
    request_body = ConfirmTerminationRequest,
    responses(
        (status = 200, description = "Successfully confirmed bulk termination", body = ConfirmTerminationResponse)
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn import_termination_confirm(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<ConfirmTerminationRequest>,
) -> AppResult<Json<ConfirmTerminationResponse>> {
    ctx.require_permission(Permission::EmployeesUpdate)?;

    let mut success_count = 0;
    for row in req.rows {
        state
            .employees()
            .service()
            .terminate_employment(&ctx.auth_token, row.employment_id, row.termination_date)
            .await?;
        success_count += 1;
    }

    Ok(Json(ConfirmTerminationResponse { success_count }))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/import-termination/template",
    responses(
        (status = 200, description = "Download bulk termination template Excel file", body = [u8])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
pub async fn import_termination_template(
    State(_state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<axum::response::Response> {
    ctx.require_permission(Permission::EmployeesUpdate)?;

    let mut workbook = rust_xlsxwriter::Workbook::new();
    let worksheet = workbook.add_worksheet();

    worksheet
        .write_string(0, 0, "employee_code")
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string(0, 1, "termination_date")
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    // Write a dummy row as an example
    worksheet
        .write_string(1, 0, "EMP001")
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string(1, 1, "2026-06-30")
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    let bytes = workbook
        .save_to_buffer()
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    use axum::http::header;

    axum::response::Response::builder()
        .header(
            header::CONTENT_TYPE,
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"employee_termination_template.xlsx\"",
        )
        .body(axum::body::Body::from(bytes))
        .map_err(|e| AppError::TemplateError(format!("Failed to build response: {e}")))
}
