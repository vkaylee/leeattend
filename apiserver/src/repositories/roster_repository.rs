use crate::models::ShiftRoster;
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;

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
        "#,
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
        "SELECT shift_id FROM shift_rosters WHERE employee_id = $1 AND date = $2",
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    if let Some(shift) = roster_shift {
        return Ok(shift); // Returns the shift ID or None (day off)
    }

    // Priority 2: Employee Default Shift (Temporal Overriding Logic)
    // Find the latest created assignment that is valid for the given date
    let employee_shift = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT shift_id 
        FROM employee_shifts 
        WHERE employee_id = $1 
          AND start_date <= $2 
          AND (end_date IS NULL OR end_date >= $2)
        ORDER BY created_at DESC 
        LIMIT 1
        "#,
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
        "#,
    )
    .bind(employee_id)
    .bind(date)
    .fetch_optional(pool)
    .await?;

    Ok(dept_shift)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Duration};

    #[tokio::test]
    async fn test_get_effective_shift_priority() {
        // This test requires a running database, which spawn_app provides in integration tests.
        // For unit tests in repository, we assume the DB is available via DATABASE_URL.
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Setup test data
        let ws_id: Uuid = sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ('Prio Test') RETURNING id").fetch_one(&pool).await.unwrap();
        let user_id: Uuid = sqlx::query_scalar("INSERT INTO users (email, full_name, password_hash) VALUES ($1, 'Test', 'hash') RETURNING id")
            .bind(format!("prio_{}@test.com", Uuid::new_v4()))
            .fetch_one(&pool).await.unwrap();
        let emp_id: Uuid = sqlx::query_scalar("INSERT INTO employees (workspace_id, user_id, first_name, last_name, employee_id_number, hire_date) VALUES ($1, $2, 'Prio', 'User', $3, '2020-01-01') RETURNING id")
            .bind(ws_id).bind(user_id).bind(format!("EMP_{}", Uuid::new_v4())).fetch_one(&pool).await.unwrap();

        let s_dept: Uuid = sqlx::query_scalar("INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Dept', '09:00:00', '18:00:00') RETURNING id").bind(ws_id).fetch_one(&pool).await.unwrap();
        let s_fixed: Uuid = sqlx::query_scalar("INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Fixed', '08:00:00', '17:00:00') RETURNING id").bind(ws_id).fetch_one(&pool).await.unwrap();
        let s_roster: Uuid = sqlx::query_scalar("INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Roster', '14:00:00', '22:00:00') RETURNING id").bind(ws_id).fetch_one(&pool).await.unwrap();

        let date = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();

        // 1. Dept Priority
        let dept_id: Uuid = sqlx::query_scalar("INSERT INTO departments (workspace_id, name) VALUES ($1, 'Dev') RETURNING id").bind(ws_id).fetch_one(&pool).await.unwrap();
        sqlx::query("INSERT INTO employee_department_history (id, employee_id, department_id, effective_date) VALUES ($1, $2, $3, $4)").bind(Uuid::new_v4()).bind(emp_id).bind(dept_id).bind(date - Duration::days(1)).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO department_default_shifts (department_id, shift_id) VALUES ($1, $2)").bind(dept_id).bind(s_dept).execute(&pool).await.unwrap();
        
        let res = get_effective_shift(&pool, emp_id, date).await.unwrap();
        assert_eq!(res, Some(s_dept));

        // 2. Fixed Overrides Dept
        sqlx::query("INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)").bind(Uuid::new_v4()).bind(emp_id).bind(s_fixed).bind(date).execute(&pool).await.unwrap();
        let res = get_effective_shift(&pool, emp_id, date).await.unwrap();
        assert_eq!(res, Some(s_fixed));

        // 3. Roster Overrides Fixed
        sqlx::query("INSERT INTO shift_rosters (id, employee_id, shift_id, date) VALUES ($1, $2, $3, $4)").bind(Uuid::new_v4()).bind(emp_id).bind(s_roster).bind(date).execute(&pool).await.unwrap();
        let res = get_effective_shift(&pool, emp_id, date).await.unwrap();
        assert_eq!(res, Some(s_roster));

        // 4. Roster NULL Overrides Fixed
        sqlx::query("UPDATE shift_rosters SET shift_id = NULL WHERE employee_id = $1 AND date = $2").bind(emp_id).bind(date).execute(&pool).await.unwrap();
        let res = get_effective_shift(&pool, emp_id, date).await.unwrap();
        assert_eq!(res, None);
    }
}
