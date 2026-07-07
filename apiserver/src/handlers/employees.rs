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
    pub termination_reason: String,
    pub termination_notes: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
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
    pub termination_reason: String,
    pub termination_notes: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ConfirmTerminationRequest {
    pub rows: Vec<ConfirmTerminationRow>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema, Clone)]
pub struct ConfirmTerminationResponse {
    pub success_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
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
        &["employee_code", "termination_date", "termination_reason"],
    )?;

    let mut valid_rows = Vec::new();
    let mut invalid_rows = Vec::new();

    // 1. Initial parsing step to filter empty rows/bad dates, and gather all unique normalized employee codes.
    const VALID_TERMINATION_REASONS: &[&str] = &[
        "voluntary_resignation",
        "involuntary_termination",
        "end_of_contract",
        "retirement",
        "mutual_agreement",
        "abandonment",
        "death",
        "other",
    ];

    struct ParsedRow {
        row_number: usize,
        raw_code: String,
        normalized_code: String,
        raw_date: String,
        termination_date: chrono::NaiveDate,
        termination_reason: String,
        termination_notes: Option<String>,
    }

    let mut parsed_rows = Vec::new();
    let mut normalized_codes = Vec::new();
    let mut seen_codes = std::collections::HashSet::new();

    for (line_number, mapped_row) in rows {
        let raw_code = mapped_row.get("employee_code").cloned().unwrap_or_default();
        let raw_date = mapped_row
            .get("termination_date")
            .cloned()
            .unwrap_or_default();
        let raw_reason = mapped_row
            .get("termination_reason")
            .cloned()
            .unwrap_or_default();
        let termination_notes = mapped_row
            .get("termination_notes")
            .cloned()
            .filter(|s| !s.trim().is_empty());

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

        let termination_reason = raw_reason.trim().to_lowercase();
        if !VALID_TERMINATION_REASONS.contains(&termination_reason.as_str()) {
            let locale = crate::utils::i18n::get_locale();
            let reason = if locale == "vi" {
                format!(
                    "Lý do nghỉ việc không hợp lệ: '{}'. Các giá trị hợp lệ: {}",
                    raw_reason,
                    VALID_TERMINATION_REASONS.join(", ")
                )
            } else {
                format!(
                    "Invalid termination reason: '{}'. Valid values: {}",
                    raw_reason,
                    VALID_TERMINATION_REASONS.join(", ")
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

        let normalized_code = crate::utils::normalize_employee_id(&raw_code);
        if !seen_codes.insert(normalized_code.clone()) {
            let locale = crate::utils::i18n::get_locale();
            let reason = if locale == "vi" {
                "Mã nhân viên bị lặp lại trong tệp".to_string()
            } else {
                "Duplicate employee code in the file".to_string()
            };
            invalid_rows.push(InvalidTerminationRow {
                row_number: line_number,
                employee_code: Some(raw_code),
                raw_termination_date: Some(raw_date),
                reason,
            });
            continue;
        }
        normalized_codes.push(normalized_code.clone());

        parsed_rows.push(ParsedRow {
            row_number: line_number,
            raw_code,
            normalized_code,
            raw_date,
            termination_date,
            termination_reason,
            termination_notes,
        });
    }

    // 2. Batch fetch Employees
    let employees_list = state
        .employees()
        .repo()
        .get_employees_by_id_numbers(&normalized_codes, &ctx.auth_token)
        .await?;

    let employees_by_code: std::collections::HashMap<String, Employee> = employees_list
        .into_iter()
        .map(|emp| (emp.employee_id_number().to_string(), emp))
        .collect();

    let employee_ids: Vec<Uuid> = employees_by_code.values().map(Employee::id).collect();

    // 3. Batch fetch Employments
    let employments = state
        .employees()
        .employment_repo()
        .get_employments_for_employees(&ctx.auth_token, &employee_ids)
        .await?;

    let mut employments_by_employee_id: std::collections::HashMap<
        Uuid,
        Vec<crate::models::employment::Employment>,
    > = std::collections::HashMap::new();
    for emp in employments {
        employments_by_employee_id
            .entry(emp.employee_id())
            .or_default()
            .push(emp);
    }

    // 4. Batch fetch Leave Balances
    let years: std::collections::HashSet<i32> = parsed_rows
        .iter()
        .map(|r| r.termination_date.year())
        .collect();

    let mut balances_by_year_and_employee: std::collections::HashMap<
        i32,
        std::collections::HashMap<Uuid, Vec<crate::models::EmployeeLeaveBalance>>,
    > = std::collections::HashMap::new();
    for year in years {
        let batch_balances = state
            .leaves()
            .balance_repo()
            .get_employee_balances_batch(&ctx.auth_token, &employee_ids, year)
            .await?;
        balances_by_year_and_employee.insert(year, batch_balances);
    }

    // 5. Final loop in memory over the parsed rows to validate and build proration previews
    for row in parsed_rows {
        let Some(employee) = employees_by_code.get(&row.normalized_code) else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: row.row_number,
                employee_code: Some(row.raw_code.clone()),
                raw_termination_date: Some(row.raw_date.clone()),
                reason: crate::utils::i18n::translate(
                    &crate::utils::i18n::get_locale(),
                    "error.employee_not_found",
                ),
            });
            continue;
        };

        let employee_employments = employments_by_employee_id
            .get(&employee.id())
            .cloned()
            .unwrap_or_default();

        // Accept any non-terminated employment (active, probation, onboarding, etc.).
        // This is consistent with `terminate_employment` in lifecycle.rs which only
        // blocks when status == "terminated". If multiple candidates exist, prefer
        // the one with the latest start_date.
        let mut warnings = Vec::new();
        let active_employment = {
            let mut candidates: Vec<_> = employee_employments
                .clone()
                .into_iter()
                .filter(|emp| emp.status() != "terminated")
                .collect();
            candidates.sort_by(|a, b| b.start_date().cmp(&a.start_date()));
            if candidates.len() > 1 {
                let locale = crate::utils::i18n::get_locale();
                let warning_msg = if locale == "vi" {
                    format!("Tìm thấy {} hợp đồng chưa kết thúc. Đã tự động chọn hợp đồng mới nhất (bắt đầu ngày {}).", candidates.len(), candidates[0].start_date().format("%d/%m/%Y"))
                } else {
                    format!("Found {} non-terminated employments. Automatically selected the latest one (starting {}).", candidates.len(), candidates[0].start_date().format("%d/%m/%Y"))
                };
                warnings.push(warning_msg);
            }
            candidates.into_iter().next()
        };

        let Some(active_employment) = active_employment else {
            let error_key = if employee_employments.is_empty() {
                "error.no_active_employment"
            } else {
                "error.employee_already_terminated"
            };
            invalid_rows.push(InvalidTerminationRow {
                row_number: row.row_number,
                employee_code: Some(row.raw_code.clone()),
                raw_termination_date: Some(row.raw_date.clone()),
                reason: crate::utils::i18n::translate(
                    &crate::utils::i18n::get_locale(),
                    error_key,
                ),
            });
            continue;
        };

        if row.termination_date < active_employment.start_date() {
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
                row_number: row.row_number,
                employee_code: Some(row.raw_code.clone()),
                raw_termination_date: Some(row.raw_date.clone()),
                reason,
            });
            continue;
        }

        // Calculate leave proration preview
        let year = row.termination_date.year();
        let Some(start_of_year) = chrono::NaiveDate::from_ymd_opt(year, 1, 1) else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: row.row_number,
                employee_code: Some(row.raw_code.clone()),
                raw_termination_date: Some(row.raw_date.clone()),
                reason: "Invalid start of year".to_string(),
            });
            continue;
        };
        let Some(end_of_year) = chrono::NaiveDate::from_ymd_opt(year, 12, 31) else {
            invalid_rows.push(InvalidTerminationRow {
                row_number: row.row_number,
                employee_code: Some(row.raw_code.clone()),
                raw_termination_date: Some(row.raw_date.clone()),
                reason: "Invalid end of year".to_string(),
            });
            continue;
        };

        let eff_start = std::cmp::max(start_of_year, active_employment.start_date());
        let days_worked = (row.termination_date - eff_start).num_days() + 1;
        let total_days = (end_of_year - start_of_year).num_days() + 1;
        let proration_fraction = days_worked as f64 / total_days as f64;

        let balances = balances_by_year_and_employee
            .get(&year)
            .and_then(|m| m.get(&employee.id()))
            .cloned()
            .unwrap_or_default();

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
            row_number: row.row_number,
            employee_id: employee.id(),
            employee_code: row.raw_code.clone(),
            full_name: employee.full_name().to_string(),
            start_date: active_employment.start_date(),
            termination_date: row.termination_date,
            employment_id: active_employment.id(),
            prorated_leaves,
            termination_reason: row.termination_reason,
            termination_notes: row.termination_notes,
            warnings,
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
        (status = 202, description = "Successfully enqueued bulk termination job", body = ConfirmTerminationResponse)
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

    let employment_ids: Vec<Uuid> = req.rows.iter().map(|r| r.employment_id).collect();
    let employments = state
        .employees()
        .employment_repo()
        .get_employments_by_ids(&ctx.auth_token, &employment_ids)
        .await?;

    for emp in employments {
        if emp.status() == "terminated" {
            return Err(AppError::validation(I18nKey::EmploymentInactive));
        }
    }

    use crate::queue::{Job, JobMetadata, JobPayload, TaskType};

    let job_id = crate::utils::id::new_id();
    let total_batches = u32::try_from(req.rows.len()).unwrap_or(0);

    // Set up job metadata in Redis for progress tracking
    let payload_val = serde_json::to_value(&req).ok();
    let meta = JobMetadata::new(
        job_id,
        total_batches,
        ctx.workspace_id,
        ctx.user_id,
        Some(TaskType::BulkTermination),
        None,
        payload_val,
    );

    let queue = state.infra().job_queue()?;
    queue
        .create_job_metadata(&meta)
        .await
        .map_err(|e| AppError::BackgroundJobError(format!("Failed to create job metadata: {e}")))?;

    // Map confirm rows to bulk termination rows
    let bulk_rows = req
        .rows
        .iter()
        .map(|r| crate::queue::BulkTerminationRow {
            employment_id: r.employment_id,
            termination_date: r.termination_date,
            termination_reason: r.termination_reason.clone(),
            termination_notes: r.termination_notes.clone(),
        })
        .collect();

    let job = Job::new(
        job_id,
        0,
        TaskType::BulkTermination,
        ctx.workspace_id,
        JobPayload::BulkTermination(crate::queue::BulkTerminationPayload { rows: bulk_rows }),
    );

    queue.enqueue_jobs(vec![job]).await.map_err(|e| {
        AppError::BackgroundJobError(format!("Failed to enqueue bulk termination job: {e}"))
    })?;

    Ok(Json(ConfirmTerminationResponse {
        success_count: req.rows.len(),
        job_id: Some(job_id.to_string()),
    }))
}

#[utoipa::path(
    get,
    path = "/workspaces/{workspace_id}/employees/import-termination/template",
    responses(
        (status = 200, description = "Download bulk termination template Excel file", body = [u8])
    ),
    params(
        ("workspace_id" = Uuid, Path, description = "Workspace ID"),
        crate::handlers::leaves::DownloadTemplateQuery,
    )
)]
#[tracing::instrument(skip_all, fields(workspace_id = %ctx.workspace_id, user_id = %ctx.user_id))]
#[allow(clippy::too_many_lines)]
pub async fn import_termination_template(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<
        crate::handlers::leaves::DownloadTemplateQuery,
    >,
    ctx: MemberContext,
) -> AppResult<axum::response::Response> {
    ctx.require_permission(Permission::EmployeesUpdate)?;

    let lang = query.lang.as_deref().unwrap_or("en");
    let msgs = crate::services::import_messages::TerminationImportMessages::for_lang(lang);

    let mut workbook = rust_xlsxwriter::Workbook::new();

    // 1. Template sheet
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name(msgs.template_sheet_name)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    let header_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::header_format();
    let example_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::example_row_format();

    worksheet
        .write_string_with_format(0, 0, "employee_code", &header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string_with_format(0, 1, "termination_date", &header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string_with_format(0, 2, "termination_reason", &header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string_with_format(0, 3, "termination_notes", &header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    // Write a dummy row as an example
    worksheet
        .write_string_with_format(1, 0, "EMP001", &example_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string_with_format(1, 1, "2026-06-30", &example_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string_with_format(1, 2, "voluntary_resignation", &example_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .write_string_with_format(1, 3, msgs.example_notes, &example_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    worksheet
        .set_column_width(0, 20)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .set_column_width(1, 20)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .set_column_width(2, 25)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    worksheet
        .set_column_width(3, 40)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    // 2. Instructions sheet
    let instructions = workbook.add_worksheet();
    instructions
        .set_name(msgs.instructions_sheet_name)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    let title_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::title_format();
    let ref_header_fmt = crate::utils::excel_format_utils::ExcelFormatUtils::ref_header_format();

    instructions
        .write_string_with_format(0, 0, msgs.title_instructions, &title_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    // Columns description section
    instructions
        .write_string_with_format(2, 0, msgs.header_column, &ref_header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    instructions
        .write_string_with_format(2, 1, msgs.header_required, &ref_header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    instructions
        .write_string_with_format(2, 2, msgs.header_desc, &ref_header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    let cols = vec![
        ("employee_code", msgs.yes_label, msgs.col_employee_code_desc),
        (
            "termination_date",
            msgs.yes_label,
            msgs.col_termination_date_desc,
        ),
        (
            "termination_reason",
            msgs.yes_label,
            msgs.col_termination_reason_desc,
        ),
        (
            "termination_notes",
            msgs.no_label,
            msgs.col_termination_notes_desc,
        ),
    ];

    for (idx, (col, req, desc)) in cols.into_iter().enumerate() {
        let row = u32::try_from(idx + 3).unwrap_or(0);
        instructions
            .write_string(row, 0, col)
            .map_err(|e| AppError::TemplateError(e.to_string()))?;
        instructions
            .write_string(row, 1, req)
            .map_err(|e| AppError::TemplateError(e.to_string()))?;
        instructions
            .write_string(row, 2, desc)
            .map_err(|e| AppError::TemplateError(e.to_string()))?;
    }

    // Reasons table section
    instructions
        .write_string_with_format(9, 0, msgs.title_valid_reasons, &title_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    instructions
        .write_string_with_format(11, 0, msgs.reason_header_key, &ref_header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    instructions
        .write_string_with_format(11, 1, msgs.reason_header_desc, &ref_header_fmt)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;

    let reasons = vec![
        ("voluntary_resignation", msgs.reason_voluntary),
        ("involuntary_termination", msgs.reason_involuntary),
        ("end_of_contract", msgs.reason_end_of_contract),
        ("retirement", msgs.reason_retirement),
        ("mutual_agreement", msgs.reason_mutual_agreement),
        ("abandonment", msgs.reason_abandonment),
        ("death", msgs.reason_death),
        ("other", msgs.reason_other),
    ];

    for (idx, (key, desc)) in reasons.into_iter().enumerate() {
        let row = u32::try_from(idx + 12).unwrap_or(0);
        instructions
            .write_string(row, 0, key)
            .map_err(|e| AppError::TemplateError(e.to_string()))?;
        instructions
            .write_string(row, 1, desc)
            .map_err(|e| AppError::TemplateError(e.to_string()))?;
    }

    instructions
        .set_column_width(0, 25)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    instructions
        .set_column_width(1, 20)
        .map_err(|e| AppError::TemplateError(e.to_string()))?;
    instructions
        .set_column_width(2, 80)
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
