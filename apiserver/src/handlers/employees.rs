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

        truncated: None,
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
                .list_definitions(&ctx.auth_token, Some("employee".to_string()))
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
                            .find_entity_ids_by_value(&ctx.auth_token, def_id, val)
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
            )
            .with_status(query.status.clone()),
        )
        .await?;

    if !data.is_empty() {
        let entity_ids: Vec<Uuid> = data.iter().map(Employee::id).collect();
        let mut custom_fields_map = state
            .core()
            .custom_field_service()
            .get_values_for_entities(&ctx.auth_token, &entity_ids)
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

        truncated: None,
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
        .get_values_for_entity(&ctx.auth_token, id)
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

    let mapped_rows = crate::utils::file_parser::extract_mapped_rows(
        &file_bytes,
        &["employee_code", "termination_date", "termination_reason"],
    )?;

    let service = crate::services::TerminationService::new(
        Arc::clone(state.employees().repo()),
        Arc::clone(state.employees().employment_repo()),
        Arc::clone(state.leaves().balance_repo()),
    );

    let preview = service.build_preview(&ctx.auth_token, mapped_rows).await?;

    Ok(Json(preview))
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
