use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    domain::permissions::Permission,
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::AttendanceRules,
    AppState,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PolicyAssignmentResponse {
    pub id: String,
    pub target_type: String,
    pub target_id: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub rules: AttendanceRules,
}

#[derive(Debug, serde::Deserialize)]
pub struct UpsertPolicyAssignmentRequest {
    pub target_type: String, // "USER" or "GROUP"
    pub target_id: Uuid,
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    pub early_arrival_strategy: crate::models::enums::EarlyArrivalStrategy,
    pub valid_from: Option<String>,
}

pub async fn list_policy_assignments(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<PolicyAssignmentResponse>>> {
    ctx.require_permission(Permission::WorkspaceSettingsRead)?;

    let rows = sqlx::query!(
        r#"
        SELECT 
            apa.id,
            apa.target_type,
            apa.target_id,
            apa.valid_from,
            apa.valid_to,
            apv.rules as "rules: sqlx::types::Json<AttendanceRules>"
        FROM attendance_policy_assignments apa
        JOIN attendance_policies ap ON ap.id = apa.policy_id
        JOIN attendance_policy_versions apv ON apv.policy_id = ap.id
        WHERE ap.workspace_id = $1
          AND apv.valid_from <= NOW()
          AND (apv.valid_to IS NULL OR apv.valid_to >= NOW())
          AND apa.valid_from <= NOW()
          AND (apa.valid_to IS NULL OR apa.valid_to >= NOW())
        "#,
        ctx.workspace_id
    )
    .fetch_all(state.infra.db.reader())
    .await?;

    let assignments = rows
        .into_iter()
        .map(|row| PolicyAssignmentResponse {
            id: row.id.to_string(),
            target_type: row.target_type,
            target_id: row.target_id.to_string(),
            valid_from: row.valid_from.to_rfc3339(),
            valid_to: row.valid_to.map(|d| d.to_rfc3339()),
            rules: row.rules.0,
        })
        .collect();

    Ok(Json(assignments))
}

pub async fn upsert_policy_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(req): Json<UpsertPolicyAssignmentRequest>,
) -> AppResult<Json<PolicyAssignmentResponse>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    let mut tx = state.infra.db.api().begin().await?;

    let valid_from = match &req.valid_from {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| e.to_string())
                    .and_then(|d| {
                        d.and_hms_opt(0, 0, 0)
                            .map(|dt| dt.and_utc())
                            .ok_or_else(|| "Invalid time".to_string())
                    })
            })
            .map_err(|_e| AppError::Validation("Invalid valid_from date format.".into()))?,
        None => Utc::now(),
    };

    // Find if target already has an active assignment
    let existing: Option<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, policy_id FROM attendance_policy_assignments WHERE target_type = $1 AND target_id = $2 AND valid_to IS NULL"
    )
    .bind(&req.target_type)
    .bind(req.target_id)
    .fetch_optional(&mut *tx)
    .await?;

    let policy_id;
    let assignment_id;

    if let Some((ext_assignment_id, ext_policy_id)) = existing {
        policy_id = ext_policy_id;
        assignment_id = ext_assignment_id;

        // Close existing version
        sqlx::query(
            "UPDATE attendance_policy_versions SET valid_to = $1 WHERE policy_id = $2 AND valid_to IS NULL"
        )
        .bind(valid_from)
        .bind(policy_id)
        .execute(&mut *tx)
        .await?;
    } else {
        // Create new policy
        policy_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO attendance_policies (id, workspace_id, name, is_default, created_at, updated_at) VALUES ($1, $2, $3, false, $4, $4)"
        )
        .bind(policy_id)
        .bind(ctx.workspace_id)
        .bind(format!("{} {} Override", req.target_type, req.target_id))
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;

        assignment_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO attendance_policy_assignments (id, policy_id, target_type, target_id, valid_from, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $6)"
        )
        .bind(assignment_id)
        .bind(policy_id)
        .bind(&req.target_type)
        .bind(req.target_id)
        .bind(valid_from)
        .bind(Utc::now())
        .execute(&mut *tx)
        .await?;
    }

    // Insert new version
    let version_id = Uuid::new_v4();
    let rules = AttendanceRules {
        work_unit_minutes: req.work_unit_minutes,
        rounding_policy: req.rounding_policy.clone(),
        early_arrival_strategy: req.early_arrival_strategy,
    };

    sqlx::query(
        "INSERT INTO attendance_policy_versions (id, policy_id, rules, valid_from, created_by, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(version_id)
    .bind(policy_id)
    .bind(sqlx::types::Json(rules.clone()))
    .bind(valid_from)
    .bind(ctx.user_id)
    .bind(Utc::now())
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(PolicyAssignmentResponse {
        id: assignment_id.to_string(),
        target_type: req.target_type,
        target_id: req.target_id.to_string(),
        valid_from: valid_from.to_rfc3339(),
        valid_to: None,
        rules,
    }))
}

pub async fn delete_policy_assignment(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path((_ws_id, assignment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<axum::http::StatusCode> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    // Just set valid_to = now
    sqlx::query(
        "UPDATE attendance_policy_assignments SET valid_to = NOW(), updated_at = NOW() WHERE id = $1 AND target_id IN (SELECT target_id FROM attendance_policy_assignments apa JOIN attendance_policies ap ON ap.id = apa.policy_id WHERE ap.workspace_id = $2)"
    )
    .bind(assignment_id)
    .bind(ctx.workspace_id)
    .execute(state.infra.db.api())
    .await?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
