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
    AppState,
};

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

    // Auto-create employment phase for probation/training
    if matches!(
        contract.contract_type,
        crate::models::enums::ContractType::Probation
            | crate::models::enums::ContractType::Training
    ) {
        if let Some(end_date) = contract.end_date {
            let phase_type = match contract.contract_type {
                crate::models::enums::ContractType::Probation => "probation",
                crate::models::enums::ContractType::Training => "training",
                _ => unreachable!(),
            };

            let pool = state.infra.db.api();
            let definitions = crate::repositories::employment_phase_repository::EmploymentPhaseRepository::get_definitions(
                pool,
                workspace_id,
            ).await.unwrap_or_default();

            let target_def = definitions.into_iter().find(|d| d.phase_type == phase_type);

            let def_id = if let Some(def) = target_def {
                Some(def.id)
            } else {
                let name = if phase_type == "probation" {
                    "Thử việc"
                } else {
                    "Đào tạo"
                };
                crate::repositories::employment_phase_repository::EmploymentPhaseRepository::create_definition(
                    pool,
                    workspace_id,
                    name,
                    None,
                    phase_type,
                    &serde_json::json!({}),
                    1,
                ).await.map(|d| d.id).ok()
            };

            if let Some(did) = def_id {
                let _ = crate::repositories::employment_phase_repository::EmploymentPhaseRepository::assign_phase(
                    pool,
                    workspace_id,
                    contract.employee_id,
                    did,
                    contract.start_date,
                    end_date,
                    None,
                ).await;
            }
        }
    }

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

    Ok(StatusCode::NO_CONTENT)
}
