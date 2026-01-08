use sqlx::PgPool;
use uuid::Uuid;
use chrono::NaiveDate;
use crate::models::ShiftRoster;

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

pub async fn upsert_roster(
    pool: &PgPool,
    roster: &ShiftRoster,
) -> Result<ShiftRoster, sqlx::Error> {
    sqlx::query_as::<_, ShiftRoster>(
        r#"
        INSERT INTO shift_rosters (id, employee_id, shift_id, date, status, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (employee_id, date) DO UPDATE SET
            shift_id = EXCLUDED.shift_id,
            status = EXCLUDED.status,
            notes = EXCLUDED.notes,
            updated_at = CURRENT_TIMESTAMP
        RETURNING *
        "#
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

pub async fn get_effective_shift(
    pool: &PgPool,
    employee_id: Uuid,
    date: NaiveDate,
) -> Result<Option<Uuid>, sqlx::Error> {
    // Priority 1: Shift Roster
    let roster_shift = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT shift_id FROM shift_rosters WHERE employee_id = $1 AND date = $2"
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    if let Some(shift) = roster_shift {
        return Ok(shift); // Returns the shift ID or None (day off)
    }

    // Priority 2: Employee Default Shift (effective_date <= current date)
    let employee_shift = sqlx::query_scalar::<_, Uuid>(
        "SELECT shift_id FROM employee_shifts WHERE employee_id = $1 AND effective_date <= $2 ORDER BY effective_date DESC LIMIT 1"
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    if employee_shift.is_some() {
        return Ok(employee_shift);
    }

    // Priority 3: Department Default Shift
    let dept_shift = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT dds.shift_id 
        FROM department_default_shifts dds
        JOIN employee_department_history edh ON edh.department_id = dds.department_id
        WHERE edh.employee_id = $1 AND edh.effective_date <= $2
        ORDER BY edh.effective_date DESC LIMIT 1
        "#
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    Ok(dept_shift)
}
