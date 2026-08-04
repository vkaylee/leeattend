use crate::{
    domain::permissions::Permission,
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::{NewOvertimeRuleParams, OvertimePolicy, OvertimeRule},
    utils::i18n::I18nKey,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateRuleRequest {
    pub priority: i32,
    pub multiplier: rust_decimal::Decimal,
    pub time_start: Option<NaiveTime>,
    pub time_end: Option<NaiveTime>,
    pub days_of_week: i32,
    pub is_holiday: bool,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListPoliciesQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PaginatedOvertimePolicies {
    pub data: Vec<OvertimePolicy>,
    pub total: usize,
    pub page: i64,
    pub limit: i64,
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/overtime/policies",
    responses(
        (status = 200, description = "List overtime policies", body = PaginatedOvertimePolicies)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ListPoliciesQuery
    )
)]
pub async fn list_policies(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Query(query): Query<ListPoliciesQuery>,
) -> AppResult<Json<crate::models::PaginatedResponse<OvertimePolicy>>> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 3000);

    let (policies, total) = state
        .payroll()
        .overtime_policy_repo()
        .get_policy_definitions(ctx.workspace_id, page, limit)
        .await?;
    Ok(Json(crate::models::PaginatedResponse {
        data: policies,
        total: usize::try_from(total).unwrap_or(0),
        page,
        limit,

        truncated: None,
    }))
}

#[utoipa::path(
    get,
    path = "/workspaces/{ws_id}/overtime/policies/{policy_id}",
    responses(
        (status = 200, description = "Overtime policy details with rules", body = [serde_json::Value])
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("policy_id" = Uuid, Path, description = "Policy ID")
    )
)]
pub async fn get_policy(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<Json<(OvertimePolicy, Vec<OvertimeRule>)>> {
    let (_, policy_id) = params;
    let result = state
        .payroll()
        .overtime_policy_repo()
        .get_policy_with_rules(ctx.workspace_id, policy_id)
        .await?
        .ok_or_else(|| AppError::not_found(I18nKey::PolicyNotFound))?;
    Ok(Json(result))
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/overtime/policies",
    request_body = CreatePolicyRequest,
    responses(
        (status = 200, description = "Overtime policy created", body = OvertimePolicy)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreatePolicyRequest>,
) -> AppResult<Json<OvertimePolicy>> {
    ctx.require_permission(Permission::OvertimePoliciesManage)?;
    let policy = state
        .payroll()
        .overtime_policy_repo()
        .create_policy_definition(ctx.workspace_id, &req.name, req.description.as_deref())
        .await?;
    Ok(Json(policy))
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/overtime/policies/{policy_id}/rules",
    request_body = CreateRuleRequest,
    responses(
        (status = 200, description = "Overtime rule added to policy", body = OvertimeRule)
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID"),
        ("policy_id" = Uuid, Path, description = "Policy ID")
    )
)]
pub async fn add_rule(
    State(state): State<Arc<AppState>>,
    Path(params): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
    Json(req): Json<CreateRuleRequest>,
) -> AppResult<Json<OvertimeRule>> {
    let (_workspace_id, policy_id) = params;
    ctx.require_permission(Permission::OvertimePoliciesManage)?;
    let rule = OvertimeRule::new(NewOvertimeRuleParams {
        id: crate::utils::id::new_id(),
        policy_id,
        priority: req.priority,
        multiplier: req.multiplier,
        time_start: req.time_start,
        time_end: req.time_end,
        days_of_week: req.days_of_week,
        is_holiday: req.is_holiday,
        created_at: None,
        updated_at: None,
    });

    let created = state
        .payroll()
        .overtime_policy_repo()
        .add_rule(ctx.workspace_id, &rule)
        .await?;
    Ok(Json(created))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct AssignPolicyRequest {
    pub assignee_type: String,
    pub assignee_id: Option<Uuid>,
    pub policy_id: Uuid,
    pub effective_from: NaiveDate,
}

#[utoipa::path(
    post,
    path = "/workspaces/{ws_id}/overtime/policies/assign",
    request_body = AssignPolicyRequest,
    responses(
        (status = 200, description = "Overtime policy assigned")
    ),
    params(
        ("ws_id" = Uuid, Path, description = "Workspace ID")
    )
)]
pub async fn assign_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<AssignPolicyRequest>,
) -> AppResult<StatusCode> {
    ctx.require_permission(Permission::OvertimePoliciesManage)?;
    state
        .payroll()
        .overtime_policy_repo()
        .create_policy_assignment(
            ctx.workspace_id,
            &req.assignee_type,
            req.assignee_id,
            req.policy_id,
            req.effective_from,
        )
        .await?;
    Ok(StatusCode::OK)
}
