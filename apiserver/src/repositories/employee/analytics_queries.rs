//! Employee analytics and dashboard queries.
//!
//! Contains: get_turnover_analytics (tenure distribution, by source, by dept).

use chrono::NaiveDate;
use sqlx::{query_scalar, PgPool};
use uuid::Uuid;

use crate::models::{
    TurnoverAnalytics, TurnoverByDepartment, TurnoverBySource, TurnoverTenureBucket,
};

/// Dashboard chart: turnover analytics for terminated employees within a date range
pub async fn get_turnover_analytics(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<TurnoverAnalytics, sqlx::Error> {
    // 1. Total terminated count
    let total: i64 = query_scalar(
        r#"SELECT COUNT(*) FROM employees e
           JOIN employment_periods ep ON ep.employee_id = e.id
           WHERE e.workspace_id = $1 AND e.is_active = FALSE
             AND ep.termination_date IS NOT NULL
             AND ep.termination_date >= $2 AND ep.termination_date <= $3"#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(pool)
    .await?;

    // 2. Tenure distribution
    #[derive(Debug, sqlx::FromRow)]
    struct TenureRow {
        bucket: String,
        count: i64,
    }

    let tenure_rows = sqlx::query_as::<_, TenureRow>(
        r#"
        SELECT bucket, COUNT(*) AS count FROM (
            SELECT CASE
                WHEN ep.termination_date - ep.hire_date < 90 THEN '< 3 months'
                WHEN ep.termination_date - ep.hire_date < 180 THEN '3-6 months'
                WHEN ep.termination_date - ep.hire_date < 365 THEN '6-12 months'
                WHEN ep.termination_date - ep.hire_date < 730 THEN '1-2 years'
                WHEN ep.termination_date - ep.hire_date < 1825 THEN '2-5 years'
                ELSE '> 5 years'
            END AS bucket
            FROM employees e
            JOIN employment_periods ep ON ep.employee_id = e.id
            WHERE e.workspace_id = $1 AND e.is_active = FALSE
              AND ep.termination_date IS NOT NULL
              AND ep.termination_date >= $2 AND ep.termination_date <= $3
        ) sub
        GROUP BY bucket
        ORDER BY CASE bucket
            WHEN '< 3 months' THEN 1
            WHEN '3-6 months' THEN 2
            WHEN '6-12 months' THEN 3
            WHEN '1-2 years' THEN 4
            WHEN '2-5 years' THEN 5
            WHEN '> 5 years' THEN 6
        END
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    // 3. By recruitment source
    #[derive(Debug, sqlx::FromRow)]
    struct SourceRow {
        source_type: String,
        source_name: String,
        count: i64,
    }

    let source_rows = sqlx::query_as::<_, SourceRow>(
        r#"
        SELECT
            COALESCE(rs.source_type::text, 'direct') AS source_type,
            COALESCE(rs.name, 'Direct Hire') AS source_name,
            COUNT(*) AS count
        FROM employees e
        JOIN employment_periods ep ON ep.employee_id = e.id
        LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
        WHERE e.workspace_id = $1 AND e.is_active = FALSE
          AND ep.termination_date IS NOT NULL
          AND ep.termination_date >= $2 AND ep.termination_date <= $3
        GROUP BY rs.source_type, rs.name
        ORDER BY count DESC
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    // 4. By department (latest department assignment before termination)
    #[derive(Debug, sqlx::FromRow)]
    struct DeptRow {
        department_name: String,
        count: i64,
    }

    let dept_rows = sqlx::query_as::<_, DeptRow>(
        r#"
        SELECT
            COALESCE(d.name, 'Unassigned') AS department_name,
            COUNT(*) AS count
        FROM employees e
        JOIN employment_periods ep ON ep.employee_id = e.id
        LEFT JOIN LATERAL (
            SELECT edh.department_id
            FROM employee_department_history edh
            WHERE edh.employee_id = e.id
            ORDER BY edh.effective_date DESC
            LIMIT 1
        ) latest_dept ON TRUE
        LEFT JOIN departments d ON d.id = latest_dept.department_id
        WHERE e.workspace_id = $1 AND e.is_active = FALSE
          AND ep.termination_date IS NOT NULL
          AND ep.termination_date >= $2 AND ep.termination_date <= $3
        GROUP BY d.name
        ORDER BY count DESC
        "#,
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    Ok(TurnoverAnalytics {
        tenure_distribution: tenure_rows
            .into_iter()
            .map(|r| TurnoverTenureBucket {
                bucket: r.bucket,
                count: r.count,
            })
            .collect(),
        by_source: source_rows
            .into_iter()
            .map(|r| TurnoverBySource {
                source_type: r.source_type,
                source_name: r.source_name,
                count: r.count,
            })
            .collect(),
        by_department: dept_rows
            .into_iter()
            .map(|r| TurnoverByDepartment {
                department_name: r.department_name,
                count: r.count,
            })
            .collect(),
        total_terminated: total,
        period_start: start_date.to_string(),
        period_end: end_date.to_string(),
    })
}
