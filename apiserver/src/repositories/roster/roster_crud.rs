//! Roster CRUD operations (get/upsert/bulk-upsert/delete).

use crate::models::ShiftRoster;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

/// Fetch all rosters for a single employee in a date range.
pub async fn get_employee_rosters(
    pool: &PgPool,
    employee_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ShiftRoster>, sqlx::Error> {
    sqlx::query_as::<_, ShiftRoster>(
        "SELECT * FROM shift_rosters WHERE employee_id = $1 AND date >= $2 AND date <= $3 ORDER BY date ASC"
    )
    .bind(employee_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await
}

/// Upsert a single roster entry (insert or update on conflict).
pub async fn upsert_roster(
    pool: &PgPool,
    roster: &ShiftRoster,
) -> Result<ShiftRoster, sqlx::Error> {
    sqlx::query_as::<_, ShiftRoster>(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status, notes)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (employee_id, date) 
         DO UPDATE SET shift_id = EXCLUDED.shift_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = NOW()
         RETURNING *",
    )
    .bind(roster.id)
    .bind(roster.employee_id)
    .bind(roster.shift_id)
    .bind(roster.date)
    .bind(roster.status.clone())
    .bind(roster.notes.clone())
    .fetch_one(pool)
    .await
}

/// Bulk upsert roster entries for a single employee within a transaction.
pub async fn bulk_upsert_roster(
    pool: &PgPool,
    employee_id: Uuid,
    items: Vec<crate::models::RosterItem>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    for item in items {
        sqlx::query(
            "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status, notes)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (employee_id, date) 
             DO UPDATE SET shift_id = EXCLUDED.shift_id, status = EXCLUDED.status, notes = EXCLUDED.notes, updated_at = NOW()",
        )
        .bind(crate::utils::id::new_id())
        .bind(employee_id)
        .bind(item.shift_id)
        .bind(item.date)
        .bind("confirmed")
        .bind(item.notes)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

/// Delete a single roster entry for an employee on a specific date.
pub async fn delete_roster(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM shift_rosters WHERE employee_id = $1 AND date = $2")
        .bind(employee_id)
        .bind(date)
        .execute(pool)
        .await?;

    Ok(())
}

/// Get assignment history for a workspace (union of all assignment types).
pub async fn get_assignment_history(
    pool: &PgPool,
    workspace_id: Uuid,
    limit: i64,
) -> Result<Vec<crate::models::AssignmentHistory>, sqlx::Error> {
    sqlx::query_as::<_, crate::models::AssignmentHistory>(
        r#"
        SELECT 
            id,
            employee_id,
            (SELECT full_name FROM employees WHERE id = history.employee_id) as employee_name,
            assignment_type,
            shift_name,
            date_range,
            created_at
        FROM (
            -- Fixed Assignments
            SELECT 
                es.id,
                es.employee_id,
                'fixed'::assignment_history_type as assignment_type,
                s.name as shift_name,
                TO_CHAR(es.start_date, 'YYYY-MM-DD') || 
                CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END as date_range,
                es.created_at
            FROM employee_shifts es
            JOIN shifts s ON es.shift_id = s.id
            JOIN employees e ON es.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Pattern Assignments
            SELECT 
                ewp.id,
                ewp.employee_id,
                'pattern'::assignment_history_type as assignment_type,
                wp.name as shift_name,
                TO_CHAR(ewp.start_date, 'YYYY-MM-DD') || 
                CASE WHEN ewp.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(ewp.end_date, 'YYYY-MM-DD') END as date_range,
                ewp.created_at
            FROM employee_working_patterns ewp
            JOIN working_patterns wp ON ewp.pattern_id = wp.id
            JOIN employees e ON ewp.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Roster Assignments
            SELECT 
                sr.id,
                sr.employee_id,
                'roster'::assignment_history_type as assignment_type,
                CASE WHEN sr.shift_id IS NULL THEN 'OFF' ELSE s.name END as shift_name,
                TO_CHAR(sr.date, 'YYYY-MM-DD') as date_range,
                sr.created_at
            FROM shift_rosters sr
            LEFT JOIN shifts s ON sr.shift_id = s.id
            JOIN employees e ON sr.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Department Patterns
            SELECT 
                edh.id,
                edh.employee_id,
                'department_pattern'::assignment_history_type as assignment_type,
                wp.name as shift_name,
                TO_CHAR(edh.effective_date, 'YYYY-MM-DD') || ' -> ∞' as date_range,
                edh.created_at
            FROM employee_department_history edh
            JOIN departments d ON edh.department_id = d.id
            JOIN working_patterns wp ON d.default_working_pattern_id = wp.id
            JOIN employees e ON edh.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Department Fixed
            SELECT 
                edh.id,
                edh.employee_id,
                'department_fixed'::assignment_history_type as assignment_type,
                s.name as shift_name,
                TO_CHAR(edh.effective_date, 'YYYY-MM-DD') || ' -> ∞' as date_range,
                edh.created_at
            FROM employee_department_history edh
            JOIN department_default_shifts dds ON edh.department_id = dds.department_id
            JOIN shifts s ON dds.shift_id = s.id
            JOIN employees e ON edh.employee_id = e.id
            WHERE e.workspace_id = $1
        ) as history
        ORDER BY created_at DESC
        LIMIT $2
        "#
    )
    .bind(workspace_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}
