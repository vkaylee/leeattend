use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    handlers::common::MemberContext,
    models::contract::{CreateContractRequest, EmployeeContract, UpdateContractRequest},
    queue::{Job, JobPayload, JobQueue, SyncHireDatePayload, TaskType},
    AppState,
};

async fn dispatch_sync_hire_date(
    state: &Arc<AppState>,
    workspace_id: Uuid,
    employee_id: Uuid,
    contract_id: Uuid,
) {
    if let Some(redis) = state.infra.redis.as_ref() {
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

        let queue = JobQueue::new(redis.clone());
        if let Err(e) = queue.enqueue_jobs(vec![job]).await {
            tracing::error!("Failed to enqueue SyncEmployeeHireDate job: {}", e);
        }
    } else {
        tracing::warn!("Redis not available, skipping SyncEmployeeHireDate job");
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
        (status = 200, description = "Danh sách hợp đồng")
    )
)]
pub async fn list_employee_contracts(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Path((_ws_id, employee_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<Vec<EmployeeContract>>> {
    let contracts = state
        .employees
        .contract_repo
        .list_employee_contracts(workspace_id, employee_id)
        .await?;

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
        (status = 200, description = "Thông tin hợp đồng")
    )
)]
pub async fn get_contract(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Path((_ws_id, _employee_id, contract_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<Json<EmployeeContract>> {
    let contract = state
        .employees
        .contract_repo
        .get_contract_by_id(workspace_id, contract_id)
        .await?
        .ok_or_else(|| crate::errors::AppError::NotFound("Contract not found".to_string()))?;

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
        (status = 201, description = "Đã tạo hợp đồng")
    )
)]
pub async fn create_contract(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Path((_ws_id, _employee_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<CreateContractRequest>,
) -> AppResult<(StatusCode, Json<EmployeeContract>)> {
    let contract = state
        .employees
        .contract_repo
        .create_contract(workspace_id, &payload)
        .await?;

    // Assign employment phase if requested
    if let Some(phase_id) = payload.assign_phase_id {
        let end_date = contract.end_date.unwrap_or_else(|| {
            contract
                .start_date
                .checked_add_months(chrono::Months::new(12))
                .unwrap_or(contract.start_date)
        });
        let _ = state
            .employees
            .employment_phase_repo
            .assign_phase(
                workspace_id,
                contract.employee_id,
                phase_id,
                contract.start_date,
                end_date,
                None,
            )
            .await;
    }

    dispatch_sync_hire_date(&state, workspace_id, contract.employee_id, contract.id).await;

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
        (status = 200, description = "Đã cập nhật hợp đồng")
    )
)]
pub async fn update_contract(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Path((_ws_id, _employee_id, contract_id)): Path<(Uuid, Uuid, Uuid)>,
    Json(payload): Json<UpdateContractRequest>,
) -> AppResult<Json<EmployeeContract>> {
    let contract = state
        .employees
        .contract_repo
        .update_contract(workspace_id, contract_id, &payload)
        .await?;

    dispatch_sync_hire_date(&state, workspace_id, contract.employee_id, contract.id).await;

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
    MemberContext { workspace_id, .. }: MemberContext,
    Path((_ws_id, _employee_id, contract_id)): Path<(Uuid, Uuid, Uuid)>,
) -> AppResult<StatusCode> {
    state
        .employees
        .contract_repo
        .delete_contract(workspace_id, contract_id)
        .await?;

    // Since we don't return the contract, we use the employee_id from path
    let employee_id = _employee_id;
    dispatch_sync_hire_date(&state, workspace_id, employee_id, contract_id).await;

    Ok(StatusCode::NO_CONTENT)
}
