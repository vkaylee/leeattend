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

pub async fn get_incentive_policies(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
) -> AppResult<Json<Vec<PayrollIncentivePolicyResponse>>> {
    let policies = state
        .payroll
        .repo
        .get_active_incentive_policies(workspace_id)
        .await?;

    let mut response = Vec::new();
    for policy in policies {
        let tiers = state.payroll.repo.get_policy_tiers(workspace_id, policy.id).await?;
        response.push(PayrollIncentivePolicyResponse { policy, tiers });
    }
    Ok(Json(response))
}

pub async fn create_incentive_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreatePayrollIncentivePolicyRequest>,
) -> AppResult<(StatusCode, Json<PayrollIncentivePolicy>)> {
    let policy = state
        .payroll
        .repo
        .create_incentive_policy(ctx.workspace_id, &req)
        .await?;
    Ok((StatusCode::CREATED, Json(policy)))
}

pub async fn delete_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    state
        .payroll
        .repo
        .delete_incentive_policy(ctx.workspace_id, policy_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clone_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<(StatusCode, Json<PayrollIncentivePolicy>)> {
    let policy = state
        .payroll
        .repo
        .clone_incentive_policy(ctx.workspace_id, policy_id)
        .await?;
    Ok((StatusCode::CREATED, Json(policy)))
}
