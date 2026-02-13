use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

use crate::handlers::common::MemberContext;
use crate::models::{
    CreatePayrollIncentivePolicyRequest, PayrollIncentivePolicy, PayrollIncentivePolicyResponse,
};
use crate::AppState;

pub async fn get_incentive_policies(
    Path(workspace_id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { .. }: MemberContext,
) -> impl IntoResponse {
    match crate::repositories::payroll_repository::get_active_incentive_policies(&state.db, workspace_id)
        .await
    {
        Ok(policies) => {
            let mut response = Vec::new();
            for policy in policies {
                let tiers = crate::repositories::payroll_repository::get_policy_tiers(&state.db, policy.id)
                    .await
                    .unwrap_or_default();
                response.push(PayrollIncentivePolicyResponse {
                    policy,
                    tiers,
                });
            }
            Json(response).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn create_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, workspace_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<CreatePayrollIncentivePolicyRequest>,
) -> impl IntoResponse {
    let policy_id = Uuid::new_v4();
    let res: Result<sqlx::postgres::PgRow, sqlx::Error> = sqlx::query(
        r#"INSERT INTO payroll_incentive_policies (id, workspace_id, name, base_amount, payout_type, is_active)
           VALUES ($1, $2, $3, $4, $5, TRUE)
           RETURNING id, workspace_id, name, base_amount, payout_type, is_active, created_at"#,
    )
    .bind(policy_id)
    .bind(workspace_id)
    .bind(&req.name)
    .bind(req.base_amount)
    .bind(&req.payout_type)
    .fetch_one(&state.db)
    .await;

    let row = match res {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    for tier in &req.tiers {
        let tier_id = Uuid::new_v4();
        let tier_res = sqlx::query(
            r#"INSERT INTO payroll_incentive_tiers (id, policy_id, priority, max_late_early_count, max_unpaid_leave_days, max_sick_leave_days, max_total_leave_days, payout_amount)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        )
        .bind(tier_id)
        .bind(policy_id)
        .bind(tier.priority)
        .bind(tier.max_late_early_count)
        .bind(tier.max_unpaid_leave_days)
        .bind(tier.max_sick_leave_days)
        .bind(tier.max_total_leave_days)
        .bind(tier.payout_amount)
        .execute(&state.db)
        .await;

        if let Err(e) = tier_res {
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    }



    (StatusCode::CREATED, Json(PayrollIncentivePolicy {
        id: row.get("id"),
        workspace_id: row.get("workspace_id"),
        name: row.get("name"),
        base_amount: row.get("base_amount"),
        payout_type: row.get("payout_type"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
    })).into_response()
}

pub async fn delete_incentive_policy(
    State(state): State<Arc<AppState>>,
    Path((_ws_id, policy_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    match sqlx::query("DELETE FROM payroll_incentive_policies WHERE id = $1").bind(policy_id).execute(&state.db).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
