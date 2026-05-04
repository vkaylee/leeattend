use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use sqlx::Row;
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
        let tiers = state.payroll.repo.get_policy_tiers(policy.id).await?;
        response.push(PayrollIncentivePolicyResponse { policy, tiers });
    }
    Ok(Json(response))
}

pub async fn create_incentive_policy(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Json(req): Json<CreatePayrollIncentivePolicyRequest>,
) -> AppResult<(StatusCode, Json<PayrollIncentivePolicy>)> {
    let policy_id = Uuid::new_v4();
    let row: sqlx::postgres::PgRow = sqlx::query(
        r#"INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active, effective_from)
           VALUES ($1, $2, $3, $4, $5, TRUE, $6)
           RETURNING id, workspace_id, name, base_amount, payout_type, is_active, created_at, effective_from, effective_to"#,
    )
    .bind(policy_id)
    .bind(ctx.workspace_id)
    .bind(&req.name)
    .bind(req.base_amount)
    .bind(&req.payout_type)
    .bind(req.effective_from)
    .fetch_one(state.infra.db.api())
    .await?;

    for tier in &req.tiers {
        let tier_id = Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO payroll_incentive_tiers (id, policy_id, priority, rule_config, payout_amount)
               VALUES ($1, $2, $3, $4, $5)"#,
        )
        .bind(tier_id)
        .bind(policy_id)
        .bind(tier.priority)
        .bind(sqlx::types::Json(&tier.rule_config))
        .bind(tier.payout_amount)
        .execute(state.infra.db.api())
        .await?;
    }

    Ok((
        StatusCode::CREATED,
        Json(PayrollIncentivePolicy {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            name: row.get("name"),
            base_amount: row.get("base_amount"),
            payout_type: row.get("payout_type"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
        }),
    ))
}

pub async fn delete_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<StatusCode> {
    sqlx::query("DELETE FROM payroll_incentive_policies WHERE id = $1 AND workspace_id = $2")
        .bind(policy_id)
        .bind(ctx.workspace_id)
        .execute(state.infra.db.api())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn clone_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_, policy_id)): Path<(Uuid, Uuid)>,
    ctx: MemberContext,
) -> AppResult<(StatusCode, Json<PayrollIncentivePolicy>)> {
    // 1. Fetch original policy
    let original_policy: sqlx::postgres::PgRow = sqlx::query(
        r#"SELECT name, base_amount, payout_type FROM payroll_incentive_policies WHERE id = $1 AND workspace_id = $2"#
    )
    .bind(policy_id)
    .bind(ctx.workspace_id)
    .fetch_one(state.infra.db.api())
    .await?;

    let new_policy_id = Uuid::new_v4();
    let new_name = format!("{} (Copy)", original_policy.get::<String, _>("name"));

    // 2. Insert new policy
    let row: sqlx::postgres::PgRow = sqlx::query(
        r#"INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active, effective_from)
           VALUES ($1, $2, $3, $4, $5, FALSE, CURRENT_DATE)
           RETURNING id, workspace_id, name, base_amount, payout_type, is_active, created_at, effective_from, effective_to"#
    )
    .bind(new_policy_id)
    .bind(ctx.workspace_id)
    .bind(new_name)
    .bind(original_policy.get::<f64, _>("base_amount"))
    .bind(original_policy.get::<String, _>("payout_type"))
    .fetch_one(state.infra.db.api())
    .await?;

    // 3. Clone tiers
    sqlx::query(
        r#"INSERT INTO payroll_incentive_tiers (id, policy_id, priority, rule_config, payout_amount)
           SELECT gen_random_uuid(), $1, priority, rule_config, payout_amount
           FROM payroll_incentive_tiers
           WHERE policy_id = $2"#,
    )
    .bind(new_policy_id)
    .bind(policy_id)
    .execute(state.infra.db.api())
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(PayrollIncentivePolicy {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            name: row.get("name"),
            base_amount: row.get("base_amount"),
            payout_type: row.get("payout_type"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            effective_from: row.get("effective_from"),
            effective_to: row.get("effective_to"),
        }),
    ))
}
