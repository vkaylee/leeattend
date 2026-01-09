use crate::models::{ShiftRoster, EmployeeSchedule, EffectiveSchedule};
use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use std::collections::HashMap;

// ... (existing imports)

pub async fn get_workspace_schedules(
    pool: &PgPool,
    workspace_id: Uuid,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<EmployeeSchedule>, sqlx::Error> {
    // 1. Fetch Employees
    let employees = sqlx::query!(
        "SELECT id, first_name, last_name FROM employees WHERE workspace_id = $1 AND is_active = true",
        workspace_id
    )
    .fetch_all(pool)
    .await?;

    // 2. Fetch all shifts (for details)
    let shifts_map: HashMap<Uuid, (String, chrono::NaiveTime, chrono::NaiveTime, i32, Option<chrono::NaiveTime>, Option<chrono::NaiveTime>, bool)> = sqlx::query!(
        "SELECT id, name, start_time, end_time, break_duration_minutes, break_start, break_end, is_break_auto_deduct FROM shifts WHERE workspace_id = $1",
        workspace_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|s| (s.id, (s.name, s.start_time, s.end_time, s.break_duration_minutes, s.break_start, s.break_end, s.is_break_auto_deduct)))
    .collect();

    // 3. Fetch Rosters in range
    let rosters = sqlx::query_as::<_, ShiftRoster>(
        r#"
        SELECT sr.* 
        FROM shift_rosters sr
        JOIN employees e ON sr.employee_id = e.id
        WHERE e.workspace_id = $1 
          AND sr.date >= $2 
          AND sr.date <= $3
        "#
    )
    .bind(workspace_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    // Group rosters by (employee_id, date)
    let mut roster_map: HashMap<(Uuid, NaiveDate), Option<Uuid>> = HashMap::new();
    for r in rosters {
        roster_map.insert((r.employee_id, r.date), r.shift_id);
    }

    // 4. Fetch Fixed Assignments (overlapping)
    // We fetch all assignments that *start* before end_date and *end* after start_date (or are null)
    let fixed_assignments = sqlx::query!(
        r#"
        SELECT employee_id, shift_id, start_date, end_date, created_at
        FROM employee_shifts
        WHERE employee_id IN (SELECT id FROM employees WHERE workspace_id = $1)
          AND start_date <= $2
          AND (end_date IS NULL OR end_date >= $3)
        ORDER BY created_at DESC
        "#,
        workspace_id,
        end_date,
        start_date
    )
    .fetch_all(pool)
    .await?;

    // 5. Fetch Department Defaults & History
    // (Simplification: For now, we assume if Dept Default exists, we fetch it. 
    // Implementing full Dept History temporal logic in bulk is complex, 
    // we can do a simplified version or iterate.)
    
    // Let's iterate employees to build result
    let mut result = Vec::new();

    for emp in employees {
        let mut daily_schedules = Vec::new();
        let mut curr_date = start_date;

        while curr_date <= end_date {
            let mut effective_shift_id = None;
            let source: String;

            // P1: Roster
            if let Some(shift_opt) = roster_map.get(&(emp.id, curr_date)) {
                effective_shift_id = *shift_opt; // Can be None (Day Off)
                source = "roster".to_string();
            } else {
                // P2: Fixed (Find latest created valid assignment)
                // Since we sorted by created_at DESC in SQL, the first match is the effective one
                let fixed = fixed_assignments.iter().find(|a| {
                    a.employee_id == emp.id &&
                    a.start_date <= curr_date &&
                    (a.end_date.is_none() || a.end_date.unwrap() >= curr_date)
                });

                if let Some(f) = fixed {
                    effective_shift_id = Some(f.shift_id);
                    source = "fixed".to_string();
                } else {
                    // P3: Dept Default (Fallback - simplified query per day/employee if needed, 
                    // or just leave as 'none' if we want to save optimization effort for now.
                    // Given the requirement is "display fixed + roster", Dept is less critical for this specific view 
                    // but ideally should be there. Let's call the single helper if needed, 
                    // OR just skip for bulk performance now and add later.)
                    
                    // For massive performance, we should preload Dept data. 
                    // For now, let's leave Dept fallback as a TODO or do a quick check if we want perfection.
                    // The user specifically asked for "Fixed + Roster".
                    source = "none".to_string();
                }
            }

            let mut sched = EffectiveSchedule {
                date: curr_date,
                shift_id: effective_shift_id,
                source,
                shift_name: None,
                start_time: None,
                end_time: None,
                break_duration_minutes: None,
                break_start: None,
                break_end: None,
                is_break_auto_deduct: None,
            };

            if let Some(sid) = effective_shift_id {
                if let Some((name, start, end, duration, b_start, b_end, auto_deduct)) = shifts_map.get(&sid) {
                    sched.shift_name = Some(name.clone());
                    sched.start_time = Some(*start);
                    sched.end_time = Some(*end);
                    sched.break_duration_minutes = Some(*duration);
                    sched.break_start = *b_start;
                    sched.break_end = *b_end;
                    sched.is_break_auto_deduct = Some(*auto_deduct);
                }
            }

            daily_schedules.push(sched);
            curr_date = curr_date.succ_opt().unwrap();
        }

        result.push(EmployeeSchedule {
            employee_id: emp.id,
            first_name: emp.first_name,
            last_name: emp.last_name,
            schedules: daily_schedules,
        });
    }

    Ok(result)
}

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
            (SELECT first_name || ' ' || last_name FROM employees WHERE id = history.employee_id) as employee_name,
            assignment_type,
            shift_name,
            date_range,
            created_at
        FROM (
            -- Fixed Assignments
            SELECT 
                es.id,
                es.employee_id,
                'fixed' as assignment_type,
                s.name as shift_name,
                TO_CHAR(es.start_date, 'YYYY-MM-DD') || 
                CASE WHEN es.end_date IS NULL THEN ' -> ∞' ELSE ' -> ' || TO_CHAR(es.end_date, 'YYYY-MM-DD') END as date_range,
                es.created_at
            FROM employee_shifts es
            JOIN shifts s ON es.shift_id = s.id
            JOIN employees e ON es.employee_id = e.id
            WHERE e.workspace_id = $1

            UNION ALL

            -- Roster Assignments
            SELECT 
                sr.id,
                sr.employee_id,
                'roster' as assignment_type,
                CASE WHEN sr.shift_id IS NULL THEN 'OFF' ELSE s.name END as shift_name,
                TO_CHAR(sr.date, 'YYYY-MM-DD') as date_range,
                sr.created_at
            FROM shift_rosters sr
            LEFT JOIN shifts s ON sr.shift_id = s.id
            JOIN employees e ON sr.employee_id = e.id
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
        dotenvy::dotenv().ok();
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

        // 5. Test History
        let history = get_assignment_history(&pool, ws_id, 10).await.unwrap();
        assert!(history.len() >= 2);
        assert!(history.iter().any(|h| h.assignment_type == "fixed"));
        assert!(history.iter().any(|h| h.assignment_type == "roster"));
    }

    #[tokio::test]
    async fn test_get_workspace_schedules_join() {
        dotenvy::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url).await.unwrap();

        // Setup test data
        let ws_id: Uuid = sqlx::query_scalar("INSERT INTO workspaces (name) VALUES ('Join Test') RETURNING id").fetch_one(&pool).await.unwrap();
        let user_id: Uuid = sqlx::query_scalar("INSERT INTO users (email, full_name, password_hash) VALUES ($1, 'Join Test User', 'hash') RETURNING id")
            .bind(format!("join_test_{}@test.com", Uuid::new_v4()))
            .fetch_one(&pool).await.unwrap();
        let emp_id: Uuid = sqlx::query_scalar("INSERT INTO employees (workspace_id, user_id, first_name, last_name, employee_id_number, hire_date) VALUES ($1, $2, 'Join', 'Employee', $3, '2020-01-01') RETURNING id")
            .bind(ws_id).bind(user_id).bind(format!("EMP_JOIN_{}", Uuid::new_v4())).fetch_one(&pool).await.unwrap();

        let shift_id: Uuid = sqlx::query_scalar("INSERT INTO shifts (workspace_id, name, start_time, end_time) VALUES ($1, 'Join Shift', '09:00:00', '18:00:00') RETURNING id").bind(ws_id).fetch_one(&pool).await.unwrap();
        
        let date = NaiveDate::from_ymd_opt(2026, 1, 25).unwrap();

        // Insert a roster entry to verify the JOIN works
        sqlx::query("INSERT INTO shift_rosters (id, employee_id, shift_id, date) VALUES ($1, $2, $3, $4)")
            .bind(Uuid::new_v4())
            .bind(emp_id)
            .bind(shift_id)
            .bind(date)
            .execute(&pool)
            .await.unwrap();

        // Call the function that had the error
        let schedules = get_workspace_schedules(&pool, ws_id, date, date).await.expect("Should not fail with column error");

        assert_eq!(schedules.len(), 1);
        let emp_sched = &schedules[0];
        assert_eq!(emp_sched.employee_id, emp_id);
        assert_eq!(emp_sched.schedules.len(), 1);
        
        let daily = &emp_sched.schedules[0];
        assert_eq!(daily.date, date);
        assert_eq!(daily.shift_id, Some(shift_id));
        assert_eq!(daily.source, "roster");
    }
}
