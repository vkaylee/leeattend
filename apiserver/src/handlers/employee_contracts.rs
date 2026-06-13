use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest},
    queue::{Job, JobPayload, SyncHireDatePayload, TaskType},
    repositories::employee_contract_repository::EmployeeContractRepository,
    utils::i18n::I18nKey,
    AppState,
};

async fn dispatch_sync_hire_date(
    state: &Arc<AppState>,
    workspace_id: Uuid,
    employee_id: Uuid,
    contract_id: Uuid,
) {
    let Ok(queue) = state.infra().job_queue() else {
        tracing::warn!("Redis not available, skipping SyncEmployeeHireDate job");
        return;
    };

    let payload = SyncHireDatePayload {
        employee_id,
        triggered_by_contract_id: Some(contract_id),
    };

    let job = Job::new(
        Uuid::new_v4(),
        0,
        TaskType::SyncEmployeeHireDate,
        workspace_id,
        JobPayload::SyncEmployeeHireDate(payload),
    );

    if let Err(e) = queue.enqueue_jobs(vec![job]).await {
        tracing::error!("Failed to enqueue SyncEmployeeHireDate job: {}", e);
    }
}

/// Lấy danh sách hợp đồng của một nhân viên
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    responses(
        (status = 200, description = "Danh sách hợp đồng", body = [EmployeeContract])
    )
)]
pub async fn list_employee_contracts(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<EmployeeContract>>> {
    let (_ws_id, employee_id) = params;

    let employments = state
        .employees()
        .employment_repo()
        .list_employee_employments(&ctx.auth_token, employee_id)
        .await?;

    let employment_ids: Vec<Uuid> = employments.into_iter().map(|e| e.id()).collect();

    let mut contracts = Vec::new();
    if !employment_ids.is_empty() {
        let mut batch_map = state
            .employees()
            .contract_repo()
            .list_contracts_batch(&ctx.auth_token, &employment_ids)
            .await?;
        for id in employment_ids {
            if let Some(mut list) = batch_map.remove(&id) {
                contracts.append(&mut list);
            }
        }
    }

    Ok(Json(contracts))
}

/// Lấy thông tin chi tiết của một hợp đồng
#[utoipa::path(
    get,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 200, description = "Thông tin hợp đồng", body = EmployeeContract)
    )
)]
pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<EmployeeContract>> {
    let (_ws_id, _employee_id, contract_id) = params;
    let _workspace_id = ctx.workspace_id;

    let contract = state
        .employees()
        .contract_repo()
        .get_contract_by_id(&ctx.auth_token, contract_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::ContractNotFound))?;

    Ok(Json(contract))
}

/// Tạo mới một hợp đồng
#[utoipa::path(
    post,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID")
    ),
    request_body = CreateContractRequest,
    responses(
        (status = 201, description = "Đã tạo hợp đồng", body = EmployeeContract)
    )
)]
pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateContractRequest>,
) -> AppResult<(StatusCode, Json<EmployeeContract>)> {
    let (_ws_id, _employee_id) = params;
    let workspace_id = ctx.workspace_id;

    let contract = state
        .employees()
        .contract_repo()
        .create_contract(&ctx.auth_token, &payload)
        .await?;

    let employee_id = state
        .employees()
        .employment_repo()
        .get_employment_by_id(&ctx.auth_token, contract.employment_id())
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?
        .employee_id();

    // Assign employment phase if requested
    if let Some(phase_id) = payload.assign_phase_id {
        let end_date = contract.end_date().unwrap_or_else(|| {
            contract
                .start_date()
                .checked_add_months(chrono::Months::new(12))
                .unwrap_or_else(|| contract.start_date())
        });
        let _ = state
            .employees()
            .employment_phase_repo()
            .assign_phase(
                workspace_id,
                contract.employment_id(),
                phase_id,
                contract.start_date(),
                end_date,
                None,
            )
            .await;
    }

    dispatch_sync_hire_date(&state, workspace_id, employee_id, contract.id()).await;

    Ok((StatusCode::CREATED, Json(contract)))
}

/// Cập nhật một hợp đồng
#[utoipa::path(
    patch,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    request_body = UpdateContractRequest,
    responses(
        (status = 200, description = "Đã cập nhật hợp đồng", body = EmployeeContract)
    )
)]
pub async fn update_contract(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateContractRequest>,
) -> AppResult<Json<EmployeeContract>> {
    let (_ws_id, _employee_id, contract_id) = params;
    let workspace_id = ctx.workspace_id;

    let contract = state
        .employees()
        .contract_repo()
        .update_contract(&ctx.auth_token, contract_id, &payload)
        .await?;

    let employee_id = state
        .employees()
        .employment_repo()
        .get_employment_by_id(&ctx.auth_token, contract.employment_id())
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::NotFound))?
        .employee_id();

    dispatch_sync_hire_date(&state, workspace_id, employee_id, contract.id()).await;

    Ok(Json(contract))
}

/// Xóa một hợp đồng
#[utoipa::path(
    delete,
    path = "/api/v1/workspaces/{ws_id}/employees/{employee_id}/contracts/{contract_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("employee_id" = Uuid, Path, description = "Employee ID"),
        ("contract_id" = Uuid, Path, description = "Contract ID")
    ),
    responses(
        (status = 204, description = "Đã xóa hợp đồng")
    )
)]
pub async fn delete_contract(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(params): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    let (_ws_id, employee_id, contract_id) = params;
    let workspace_id = ctx.workspace_id;

    state
        .employees()
        .contract_repo()
        .delete_contract(&ctx.auth_token, contract_id)
        .await?;

    dispatch_sync_hire_date(&state, workspace_id, employee_id, contract_id).await;

    Ok(StatusCode::NO_CONTENT)
}
