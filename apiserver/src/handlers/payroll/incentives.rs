use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

use std::sync::Arc;
use uuid::Uuid;

use crate::errors::AppResult;
use crate::handlers::common::MemberContext;
use crate::models::{
    CreatePayrollIncentivePolicyRequest, PayrollIncentivePolicy, PayrollIncentivePolicyResponse,
};
use crate::AppState;

/// GET /workspaces/{ws_id}/payroll/incentive-policies
#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/payroll/incentive-policies",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    responses(
        (status = 200, description = "Workspace payroll incentive policies list", body = [PayrollIncentivePolicyResponse])
    )
)]
pub async fn get_incentive_policies(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
) -> AppResult<Json<Vec<PayrollIncentivePolicyResponse>>> {
    let _workspace_id = ctx.workspace_id;

    let policies = state
        .payroll()
        .repo()
        .get_active_incentive_policies(ctx.workspace_id)
        .await?;

    let mut response = Vec::new();
    for policy in policies {
        let tiers = state
            .payroll()
            .repo()
            .get_policy_tiers(ctx.workspace_id, policy.id)
            .await?;
        response.push(PayrollIncentivePolicyResponse { policy, tiers });
    }
    Ok(Json(response))
}

/// POST /workspaces/{ws_id}/payroll/incentive-policies
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/payroll/incentive-policies",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    ),
    request_body = CreatePayrollIncentivePolicyRequest,
    responses(
        (status = 201, description = "Created incentive policy", body = PayrollIncentivePolicy)
    )
)]
pub async fn create_incentive_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreatePayrollIncentivePolicyRequest>,
) -> AppResult<(StatusCode, Json<PayrollIncentivePolicy>)> {
    let policy = state
        .payroll()
        .repo()
        .create_incentive_policy(ctx.workspace_id, &req)
        .await?;
    Ok((StatusCode::CREATED, Json(policy)))
}

/// DELETE /workspaces/{ws_id}/payroll/incentive-policies/{policy_id}
#[utoipa::path(
    delete,
    path = "/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("policy_id" = Uuid, Path, description = "Policy ID")
    ),
    responses(
        (status = 204, description = "Deleted incentive policy")
    )
)]
pub async fn delete_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    let (_, policy_id) = params;
    state
        .payroll()
        .repo()
        .delete_incentive_policy(ctx.workspace_id, policy_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone
#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/payroll/incentive-policies/{policy_id}/clone",
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("policy_id" = Uuid, Path, description = "Policy ID")
    ),
    responses(
        (status = 201, description = "Cloned incentive policy", body = PayrollIncentivePolicy)
    )
)]
pub async fn clone_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<(StatusCode, Json<PayrollIncentivePolicy>)> {
    let (_, policy_id) = params;
    let policy = state
        .payroll()
        .repo()
        .clone_incentive_policy(ctx.workspace_id, policy_id)
        .await?;
    Ok((StatusCode::CREATED, Json(policy)))
}
