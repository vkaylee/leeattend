//! Recruitment source CRUD operations.
//!
//! Contains: create/get/update/delete recruitment sources,
//! get_recruitment_source_breakdown, set_employee_recruitment_source.

use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{RecruitmentSource, RecruitmentSourceBreakdown};

pub async fn create_recruitment_source(
    pool: &PgPool,
    source: &RecruitmentSource,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO recruitment_sources
           (id, workspace_id, source_type, name,
            referrer_employee_id, referral_bonus_amount, referral_bonus_paid,
            agency_contact_name, agency_contact_email, agency_fee_amount, agency_fee_paid,
            notes, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#,
    )
    .bind(source.id)
    .bind(source.workspace_id)
    .bind(source.source_type)
    .bind(&source.name)
    .bind(source.referrer_employee_id)
    .bind(source.referral_bonus_amount)
    .bind(source.referral_bonus_paid)
    .bind(&source.agency_contact_name)
    .bind(&source.agency_contact_email)
    .bind(source.agency_fee_amount)
    .bind(source.agency_fee_paid)
    .bind(&source.notes)
    .bind(source.created_at)
    .bind(source.updated_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_recruitment_sources(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<RecruitmentSource>, sqlx::Error> {
    sqlx::query_as::<_, RecruitmentSource>(
        "SELECT * FROM recruitment_sources WHERE workspace_id = $1 ORDER BY created_at DESC",
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
}

pub async fn get_recruitment_source_by_id(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<Option<RecruitmentSource>, sqlx::Error> {
    sqlx::query_as::<_, RecruitmentSource>(
        "SELECT * FROM recruitment_sources WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_optional(pool)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn update_recruitment_source(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
    name: Option<String>,
    referrer_employee_id: Option<Uuid>,
    referral_bonus_amount: Option<Decimal>,
    referral_bonus_paid: Option<bool>,
    agency_contact_name: Option<String>,
    agency_contact_email: Option<String>,
    agency_fee_amount: Option<Decimal>,
    agency_fee_paid: Option<bool>,
    notes: Option<String>,
) -> Result<Option<RecruitmentSource>, sqlx::Error> {
    sqlx::query_as::<_, RecruitmentSource>(
        r#"UPDATE recruitment_sources SET
            name = COALESCE($3, name),
            referrer_employee_id = COALESCE($4, referrer_employee_id),
            referral_bonus_amount = COALESCE($5, referral_bonus_amount),
            referral_bonus_paid = COALESCE($6, referral_bonus_paid),
            agency_contact_name = COALESCE($7, agency_contact_name),
            agency_contact_email = COALESCE($8, agency_contact_email),
            agency_fee_amount = COALESCE($9, agency_fee_amount),
            agency_fee_paid = COALESCE($10, agency_fee_paid),
            notes = COALESCE($11, notes),
            updated_at = NOW()
        WHERE id = $1 AND workspace_id = $2
        RETURNING *"#,
    )
    .bind(id)
    .bind(workspace_id)
    .bind(name)
    .bind(referrer_employee_id)
    .bind(referral_bonus_amount)
    .bind(referral_bonus_paid)
    .bind(agency_contact_name)
    .bind(agency_contact_email)
    .bind(agency_fee_amount)
    .bind(agency_fee_paid)
    .bind(notes)
    .fetch_optional(pool)
    .await
}

pub async fn delete_recruitment_source(
    pool: &PgPool,
    id: Uuid,
    workspace_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM recruitment_sources WHERE id = $1 AND workspace_id = $2")
        .bind(id)
        .bind(workspace_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Dashboard chart: count employees by recruitment source type/name
pub async fn get_recruitment_source_breakdown(
    pool: &PgPool,
    workspace_id: Uuid,
) -> Result<Vec<RecruitmentSourceBreakdown>, sqlx::Error> {
    #[derive(Debug, sqlx::FromRow)]
    struct Row {
        source_type: String,
        source_name: String,
        count: i64,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT
            COALESCE(rs.source_type::text, 'direct') AS source_type,
            COALESCE(rs.name, 'Direct Hire') AS source_name,
            COUNT(*) AS count
        FROM employees e
        LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
        WHERE e.workspace_id = $1 AND e.is_active = TRUE
        GROUP BY rs.source_type, rs.name
        ORDER BY count DESC
        "#,
    )
    .bind(workspace_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Ok(rows
        .into_iter()
        .map(|r| RecruitmentSourceBreakdown {
            source_type: r.source_type,
            source_name: r.source_name,
            count: r.count,
        })
        .collect())
}

pub async fn set_employee_recruitment_source(
    pool: &PgPool,
    employee_id: Uuid,
    recruitment_source_id: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE employees SET recruitment_source_id = $2, updated_at = NOW() WHERE id = $1",
    )
    .bind(employee_id)
    .bind(recruitment_source_id)
    .execute(pool)
    .await?;
    Ok(())
}
