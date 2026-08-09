//! Aggregate qualifying-day overcount diagnostics.

use chrono::NaiveDate;
use diesel::sql_types::{BigInt, Date, Text, Uuid as UuidSql};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::str::FromStr;
use uuid::Uuid;

use crate::diesel_pool::{DieselPools, DieselPoolsTenantTxExt, TenantAuthToken};
use crate::errors::AppError;

const MAX_RANGE_DAYS: i64 = 366;
const REPORT_QUERY: &str = r"
WITH params AS (
    SELECT $1::uuid AS workspace_id,
           $2::date AS start_date,
           $3::date AS end_date,
           $4::text AS timezone
),
scoped_employments AS (
    SELECT DISTINCT ON (em.employee_id)
           em.employee_id,
           em.id AS employment_id
    FROM employments em
    JOIN employees e ON e.id = em.employee_id
    CROSS JOIN params p
    WHERE em.workspace_id = p.workspace_id
      AND e.workspace_id = p.workspace_id
      AND em.deleted_at IS NULL
      AND e.deleted_at IS NULL
      AND e.is_active = TRUE
    ORDER BY em.employee_id,
             (em.status = 'active') DESC,
             em.start_date DESC,
             em.id DESC
),
work_rows AS (
    SELECT se.employee_id,
           se.employment_id,
           (te.punch_in_time AT TIME ZONE p.timezone)::date AS local_date
    FROM time_entries te
    JOIN scoped_employments se ON se.employment_id = te.employment_id
    CROSS JOIN params p
    WHERE te.workspace_id = p.workspace_id
      AND te.entry_type = 'work'
      AND te.punch_in_time >= (p.start_date::timestamp AT TIME ZONE p.timezone)
      AND te.punch_in_time < ((p.end_date + 1)::timestamp AT TIME ZONE p.timezone)
),
leave_days AS (
    SELECT DISTINCT se.employee_id,
                    se.employment_id,
                    generated.local_date::date AS local_date
    FROM leave_requests lr
    JOIN scoped_employments se ON se.employment_id = lr.employment_id
    CROSS JOIN params p
    CROSS JOIN LATERAL generate_series(
        GREATEST(lr.start_date, p.start_date),
        LEAST(lr.end_date, p.end_date),
        interval '1 day'
    ) AS generated(local_date)
    WHERE lr.status IN ('approved', 'pre_approved')
      AND lr.start_date <= p.end_date
      AND lr.end_date >= p.start_date
),
work_months AS (
    SELECT employee_id,
           employment_id,
           date_trunc('month', local_date)::date AS month_start,
           COUNT(*)::bigint AS raw_work_rows
    FROM work_rows
    GROUP BY employee_id, employment_id, date_trunc('month', local_date)::date
),
leave_months AS (
    SELECT employee_id,
           employment_id,
           date_trunc('month', local_date)::date AS month_start,
           COUNT(DISTINCT local_date)::bigint AS leave_days
    FROM leave_days
    GROUP BY employee_id, employment_id, date_trunc('month', local_date)::date
),
qualifying_months AS (
    SELECT employee_id,
           employment_id,
           date_trunc('month', local_date)::date AS month_start,
           COUNT(*)::bigint AS corrected_qualifying_days
    FROM (
        SELECT employee_id, employment_id, local_date FROM work_rows
        UNION
        SELECT employee_id, employment_id, local_date FROM leave_days
    ) qualifying_days
    GROUP BY employee_id, employment_id, date_trunc('month', local_date)::date
),
comparison AS (
    SELECT COALESCE(w.employee_id, l.employee_id, q.employee_id) AS employee_id,
           COALESCE(w.employment_id, l.employment_id, q.employment_id) AS employment_id,
           COALESCE(w.month_start, l.month_start, q.month_start) AS month_start,
           COALESCE(w.raw_work_rows, 0) + COALESCE(l.leave_days, 0) AS current_batch_days,
           COALESCE(q.corrected_qualifying_days, 0) AS corrected_qualifying_days
    FROM work_months w
    FULL OUTER JOIN leave_months l
      ON l.employee_id = w.employee_id
     AND l.employment_id = w.employment_id
     AND l.month_start = w.month_start
    FULL OUTER JOIN qualifying_months q
      ON q.employee_id = COALESCE(w.employee_id, l.employee_id)
     AND q.employment_id = COALESCE(w.employment_id, l.employment_id)
     AND q.month_start = COALESCE(w.month_start, l.month_start)
)
SELECT COUNT(*) FILTER (WHERE current_batch_days > corrected_qualifying_days)::bigint
           AS affected_employee_months,
       COALESCE(SUM(current_batch_days - corrected_qualifying_days), 0)::bigint
           AS total_inflated_days,
       COALESCE(SUM(current_batch_days), 0)::bigint AS current_batch_days,
       COALESCE(SUM(corrected_qualifying_days), 0)::bigint
           AS corrected_qualifying_days
FROM comparison
";

#[derive(Debug, Clone, Serialize)]
pub struct QualifyingDaysBlastRadius {
    pub workspace_id: Uuid,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    pub affected_employee_months: i64,
    pub total_inflated_days: i64,
    pub current_batch_days: i64,
    pub corrected_qualifying_days: i64,
}

#[derive(diesel::QueryableByName)]
struct ReportRow {
    #[diesel(sql_type = BigInt)]
    affected_employee_months: i64,
    #[diesel(sql_type = BigInt)]
    total_inflated_days: i64,
    #[diesel(sql_type = BigInt)]
    current_batch_days: i64,
    #[diesel(sql_type = BigInt)]
    corrected_qualifying_days: i64,
}

pub fn requested() -> bool {
    std::env::args().any(|arg| arg == "report-qualifying-days-blast-radius")
}

pub fn parse_process_args() -> Result<(Uuid, String, NaiveDate, NaiveDate), AppError> {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    if command.as_deref() != Some("report-qualifying-days-blast-radius") {
        return Err(AppError::Internal(
            "missing report-qualifying-days-blast-radius command".to_string(),
        ));
    }

    let mut workspace_id = None;
    let mut timezone = None;
    let mut from_date = None;
    let mut to_date = None;

    while let Some(flag) = args.next() {
        let value = args.next().ok_or_else(|| {
            AppError::Internal(format!("missing value for diagnostic argument {flag}"))
        })?;
        match flag.as_str() {
            "--workspace-id" => {
                workspace_id = Some(value.parse::<Uuid>().map_err(|_err| {
                    AppError::Internal("invalid --workspace-id; expected UUID".to_string())
                })?);
            }
            "--timezone" => timezone = Some(value),
            "--from-date" => {
                from_date = Some(NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(
                    |_err| {
                        AppError::Internal("invalid --from-date; expected YYYY-MM-DD".to_string())
                    },
                )?);
            }
            "--to-date" => {
                to_date = Some(
                    NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_err| {
                        AppError::Internal("invalid --to-date; expected YYYY-MM-DD".to_string())
                    })?,
                );
            }
            _ => {
                return Err(AppError::Internal(format!(
                    "unknown diagnostic argument {flag}"
                )));
            }
        }
    }

    Ok((
        workspace_id.ok_or_else(|| AppError::Internal("--workspace-id is required".to_string()))?,
        timezone.ok_or_else(|| AppError::Internal("--timezone is required".to_string()))?,
        from_date.ok_or_else(|| AppError::Internal("--from-date is required".to_string()))?,
        to_date.ok_or_else(|| AppError::Internal("--to-date is required".to_string()))?,
    ))
}

pub async fn run(
    pools: &DieselPools,
    workspace_id: Uuid,
    timezone: &str,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<QualifyingDaysBlastRadius, AppError> {
    if timezone.trim().is_empty() {
        return Err(AppError::Internal("timezone must not be empty".to_string()));
    }
    chrono_tz::Tz::from_str(timezone)
        .map_err(|_err| AppError::Internal(format!("invalid timezone: {timezone}")))?;
    if to_date < from_date {
        return Err(AppError::Internal(
            "to_date must be on or after from_date".to_string(),
        ));
    }
    let range_days = (to_date - from_date).num_days();
    if range_days > MAX_RANGE_DAYS {
        return Err(AppError::Internal(format!(
            "date range must not exceed {MAX_RANGE_DAYS} days"
        )));
    }

    let token = TenantAuthToken::system_override(workspace_id);
    let timezone = timezone.to_string();
    let row = pools
        .with_tenant_tx(&token, |conn| {
            Box::pin(async move {
                diesel::sql_query("SET TRANSACTION READ ONLY")
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("failed to set read-only transaction: {e}"))
                    })?;
                diesel::sql_query("SET LOCAL statement_timeout = '120s'")
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("failed to set statement timeout: {e}"))
                    })?;

                diesel::sql_query(REPORT_QUERY)
                    .bind::<UuidSql, _>(workspace_id)
                    .bind::<Date, _>(from_date)
                    .bind::<Date, _>(to_date)
                    .bind::<Text, _>(timezone)
                    .get_result::<ReportRow>(&mut *conn)
                    .await
                    .map_err(|e| {
                        AppError::Internal(format!("qualifying-day report query failed: {e}"))
                    })
            })
        })
        .await?;

    Ok(QualifyingDaysBlastRadius {
        workspace_id,
        from_date,
        to_date,
        affected_employee_months: row.affected_employee_months,
        total_inflated_days: row.total_inflated_days,
        current_batch_days: row.current_batch_days,
        corrected_qualifying_days: row.corrected_qualifying_days,
    })
}
