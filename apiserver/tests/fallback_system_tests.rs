mod common;

use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;
use apiserver::repositories::roster_repository;
use apiserver::models::AssignmentRuleConditions;
use uuid::Uuid;

#[tokio::test]
async fn test_fallback_hierarchy_full_chain() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;

    // Create Shifts
    let shift_roster = factory.shift().with_workspace(ws_id).with_name("Roster Shift").create().await;
    let shift_pattern = factory.shift().with_workspace(ws_id).with_name("Pattern Shift").create().await;
    let shift_rule = factory.shift().with_workspace(ws_id).with_name("Rule Shift").create().await;
    let shift_dept_pattern = factory.shift().with_workspace(ws_id).with_name("Dept Pattern Shift").create().await;
    let shift_fixed = factory.shift().with_workspace(ws_id).with_name("Fixed Shift").create().await;
    let shift_dept_fixed = factory.shift().with_workspace(ws_id).with_name("Dept Fixed Shift").create().await;

    let test_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(); // Thursday
    let dow = 4; // 0=Sun, 1=Mon, ..., 4=Thu

    // Helper to create working pattern via SQL
    let create_pattern = |name: &str| {
        let pool = pool.clone();
        let name = name.to_string();
        async move {
            sqlx::query_scalar!(
                "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, $2) RETURNING id",
                ws_id, name
            ).fetch_one(&pool).await.unwrap()
        }
    };

    // 1. SETUP: Create Department and Assign Employee
    let dept_id = factory.department().with_workspace(ws_id).create().await;
    sqlx::query!(
        "INSERT INTO employee_department_history (id, employee_id, department_id, effective_date) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(), emp_id, dept_id, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
    ).execute(&pool).await.unwrap();

    // 2. SETUP: Department Fixed (P10)
    // Create Department Default Shift
    sqlx::query!(
        "INSERT INTO department_default_shifts (department_id, shift_id) VALUES ($1, $2)",
        dept_id, shift_dept_fixed
    ).execute(&pool).await.unwrap();

    // Verify P10 wins (only one present)
    let (sid, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();
    assert_eq!(sid, Some(shift_dept_fixed), "Should fall back to Dept Fixed");
    assert_eq!(traces.last().unwrap().priority, 10);

    // 3. SETUP: Fixed Assignment (P20)
    sqlx::query!(
        "INSERT INTO employee_shifts (id, employee_id, shift_id, start_date) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(), emp_id, shift_fixed, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
    ).execute(&pool).await.unwrap();

    // Verify P20 wins over P10
    let (sid, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();
    assert_eq!(sid, Some(shift_fixed), "Should prefer Fixed over Dept Fixed");
    assert_eq!(traces.last().unwrap().priority, 20);

    // 4. SETUP: Department Pattern (P40)
    let pat_dept = create_pattern("Dept Pattern").await;
    sqlx::query!(
        "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)",
        pat_dept, dow, shift_dept_pattern
    ).execute(&pool).await.unwrap();
    
    // Update department to use this pattern
    sqlx::query!(
        "UPDATE departments SET default_working_pattern_id = $1 WHERE id = $2",
        pat_dept, dept_id
    ).execute(&pool).await.unwrap();

    // Verify P40 wins over P20
    let (sid, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();
    assert_eq!(sid, Some(shift_dept_pattern), "Should prefer Dept Pattern over Fixed");
    assert_eq!(traces.last().unwrap().priority, 40);

    // 5. SETUP: Dynamic Rule (P60 - Arbitrary > 40)
    let pat_rule = create_pattern("Rule Pattern").await;
    sqlx::query!(
        "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)",
        pat_rule, dow, shift_rule
    ).execute(&pool).await.unwrap();

    // Assign tag to employee
    sqlx::query!("UPDATE employees SET tags = $1 WHERE id = $2", &vec!["VIP".to_string()], emp_id).execute(&pool).await.unwrap();

    // Create Rule
    factory.assignment_rule()
        .with_workspace(ws_id)
        .with_priority(60)
        .with_pattern(pat_rule)
        .with_conditions(AssignmentRuleConditions {
            department_id: None,
            employee_id_numbers: None,
            tags: Some(vec!["VIP".to_string()]),
        })
        .create().await;

    // Verify P60 wins over P40
    let (sid, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();
    assert_eq!(sid, Some(shift_rule), "Should prefer Dynamic Rule over Dept Pattern");
    
    let winner_trace = traces.iter().find(|t| t.status == "applied").unwrap();
    assert_eq!(winner_trace.priority, 60);

    // 6. SETUP: Individual Pattern (P80)
    let pat_ind = create_pattern("Individual Pattern").await;
    sqlx::query!(
        "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)",
        pat_ind, dow, shift_pattern
    ).execute(&pool).await.unwrap();

    sqlx::query!(
        "INSERT INTO employee_working_patterns (id, employee_id, pattern_id, start_date, workspace_id) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(), emp_id, pat_ind, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), ws_id
    ).execute(&pool).await.unwrap();

    // Verify P80 wins over P60
    let (sid, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();
    assert_eq!(sid, Some(shift_pattern), "Should prefer Individual Pattern over Dynamic Rule");
    let winner_trace = traces.iter().find(|t| t.status == "applied").unwrap();
    assert_eq!(winner_trace.priority, 80);

    // 7. SETUP: Manual Roster (P100)
    sqlx::query!(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'confirmed')",
        Uuid::new_v4(), emp_id, shift_roster, test_date
    ).execute(&pool).await.unwrap();

    // Verify P100 wins over P80
    let (sid, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();
    assert_eq!(sid, Some(shift_roster), "Should prefer Roster over Individual Pattern");
    let winner_trace = traces.iter().find(|t| t.status == "applied").unwrap();
    assert_eq!(winner_trace.priority, 100);
}