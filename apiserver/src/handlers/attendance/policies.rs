use axum::{
    extract::{Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;

use crate::{
    domain::permissions::Permission,
    errors::{AppError, AppResult},
    handlers::common::MemberContext,
    models::AttendanceRules,
    AppState,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UpdateAttendanceRulesRequest {
    pub work_unit_minutes: Option<i32>,
    pub rounding_policy: Option<String>,
    /// ISO 8601 date string for when this version takes effect.
    /// If omitted, defaults to now.
    pub valid_from: Option<String>,
}

/// Response shape for a single policy version (includes metadata)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AttendancePolicyVersionResponse {
    pub id: String,
    pub work_unit_minutes: i32,
    pub rounding_policy: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
}

pub async fn get_default_attendance_rules(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<AttendanceRules>> {
    // 1. Get default policy ID
    let policy_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true LIMIT 1"
    )
    .bind(workspace_id)
    .fetch_optional(state.infra.db.reader())
    .await?;

    let policy_id = match policy_id {
        Some(id) => id,
        None => {
            return Ok(Json(AttendanceRules {
                work_unit_minutes: 15,
                rounding_policy: "none".to_string(),
            }));
        }
    };

    let rules: Option<sqlx::types::Json<AttendanceRules>> = sqlx::query_scalar(
        r#"
        SELECT rules FROM attendance_policy_versions 
        WHERE policy_id = $1 
        ORDER BY valid_from DESC 
        LIMIT 1
        "#
    )
    .bind(policy_id)
    .fetch_optional(state.infra.db.reader())
    .await?;

    match rules {
        Some(r) => Ok(Json(r.0)),
        None => Ok(Json(AttendanceRules {
            work_unit_minutes: 15,
            rounding_policy: "none".to_string(),
        })),
    }
}

/// Returns all versions of the default attendance policy, ordered newest first.
pub async fn list_default_attendance_policy_versions(
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Path(_ws_id): Path<Uuid>,
) -> AppResult<Json<Vec<AttendancePolicyVersionResponse>>> {
    let policy_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true LIMIT 1"
    )
    .bind(workspace_id)
    .fetch_optional(state.infra.db.reader())
    .await?;

    let policy_id = match policy_id {
        Some(id) => id,
        None => return Ok(Json(vec![])),
    };

    let rows = sqlx::query_as::<_, (Uuid, sqlx::types::Json<AttendanceRules>, chrono::DateTime<Utc>, Option<chrono::DateTime<Utc>>, Option<Uuid>, chrono::DateTime<Utc>)>(
        r#"
        SELECT id, rules, valid_from, valid_to, created_by, created_at
        FROM attendance_policy_versions
        WHERE policy_id = $1
        ORDER BY valid_from DESC
        "#
    )
    .bind(policy_id)
    .fetch_all(state.infra.db.reader())
    .await?;

    let versions: Vec<AttendancePolicyVersionResponse> = rows
        .into_iter()
        .map(|(id, rules, valid_from, valid_to, created_by, created_at)| {
            AttendancePolicyVersionResponse {
                id: id.to_string(),
                work_unit_minutes: rules.0.work_unit_minutes,
                rounding_policy: rules.0.rounding_policy,
                valid_from: valid_from.to_rfc3339(),
                valid_to: valid_to.map(|v| v.to_rfc3339()),
                created_by: created_by.map(|u| u.to_string()),
                created_at: created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(versions))
}

pub async fn update_default_attendance_rules(
    State(state): State<Arc<AppState>>,
    ctx: MemberContext,
    Path(_ws_id): Path<Uuid>,
    Json(req): Json<UpdateAttendanceRulesRequest>,
) -> AppResult<Json<AttendancePolicyVersionResponse>> {
    ctx.require_permission(Permission::WorkspaceSettingsWrite)?;

    let mut tx = state.infra.db.api().begin().await?;

    // 1. Get default policy ID, if not exists create one
    let policy_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = true LIMIT 1"
    )
    .bind(ctx.workspace_id)
    .fetch_optional(&mut *tx)
    .await?;

    let policy_id = match policy_id {
        Some(id) => id,
        None => {
            let new_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO attendance_policies (id, workspace_id, name, is_default, created_at, updated_at) VALUES ($1, $2, $3, true, $4, $4)"
            )
            .bind(new_id)
            .bind(ctx.workspace_id)
            .bind("Default Policy")
            .bind(Utc::now())
            .execute(&mut *tx)
            .await?;
            new_id
        }
    };

    // 2. Fetch current rules
    let current_rules: Option<sqlx::types::Json<AttendanceRules>> = sqlx::query_scalar(
        r#"
        SELECT rules FROM attendance_policy_versions 
        WHERE policy_id = $1 
        ORDER BY valid_from DESC 
        LIMIT 1
        "#
    )
    .bind(policy_id)
    .fetch_optional(&mut *tx)
    .await?;

    let mut new_rules = match current_rules {
        Some(r) => r.0,
        None => AttendanceRules {
            work_unit_minutes: 15,
            rounding_policy: "none".to_string(),
        },
    };

    if let Some(w) = req.work_unit_minutes {
        new_rules.work_unit_minutes = w;
    }
    if let Some(r) = req.rounding_policy {
        new_rules.rounding_policy = r;
    }

    // Parse valid_from or default to now
    let valid_from = match &req.valid_from {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                // Also try parsing as date-only "2026-05-01"
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map_err(|e| e.to_string())
                    .and_then(|d| {
                        d.and_hms_opt(0, 0, 0)
                            .map(|dt| dt.and_utc())
                            .ok_or_else(|| "Invalid time".to_string())
                    })
            })
            .map_err(|_| AppError::Validation("Invalid valid_from date format. Use ISO 8601 or YYYY-MM-DD.".into()))?,
        None => Utc::now(),
    };

    // Close existing version if any
    sqlx::query(
        "UPDATE attendance_policy_versions SET valid_to = $1 WHERE policy_id = $2 AND valid_to IS NULL"
    )
    .bind(valid_from)
    .bind(policy_id)
    .execute(&mut *tx)
    .await?;

    // 3. Create a new version
    let version_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO attendance_policy_versions (id, policy_id, rules, valid_from, created_by, created_at) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(version_id)
    .bind(policy_id)
    .bind(sqlx::types::Json(new_rules.clone()))
    .bind(valid_from)
    .bind(ctx.user_id)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(AttendancePolicyVersionResponse {
        id: version_id.to_string(),
        work_unit_minutes: new_rules.work_unit_minutes,
        rounding_policy: new_rules.rounding_policy,
        valid_from: valid_from.to_rfc3339(),
        valid_to: None,
        created_by: Some(ctx.user_id.to_string()),
        created_at: now.to_rfc3339(),
    }))
}
