mod common;

use apiserver::models::AssignmentRuleConditions;
use apiserver::repositories::roster_repository;
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_schedule_traceability_logic() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let ws_id = factory.workspace().create().await;
    
    // 1. Create two patterns
    let shift_a = factory.shift().with_workspace(ws_id).with_name("Shift A").create().await;
    let shift_b = factory.shift().with_workspace(ws_id).with_name("Shift B").create().await;

    let pattern_a = sqlx::query_scalar!(
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, 'Pattern A') RETURNING id",
        ws_id
    ).fetch_one(&pool).await.unwrap();
    
    let pattern_b = sqlx::query_scalar!(
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, 'Pattern B') RETURNING id",
        ws_id
    ).fetch_one(&pool).await.unwrap();

    for i in 0..7 {
        sqlx::query!(
            "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)",
            pattern_a, i, shift_a
        ).execute(&pool).await.unwrap();
        
        sqlx::query!(
            "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)",
            pattern_b, i, shift_b
        ).execute(&pool).await.unwrap();
    }

    // 2. Create Employee with Tags
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    sqlx::query!(
        "UPDATE employees SET tags = $1 WHERE id = $2",
        &vec!["Special".to_string()],
        emp_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // 3. Create two matching rules with different priorities
    // Rule 1: Priority 10 (Winner)
    factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_name("High Priority Rule")
        .with_pattern(pattern_a)
        .with_priority(100)
        .with_conditions(AssignmentRuleConditions {
            department_id: None,
            employee_id_numbers: None,
            tags: Some(vec!["Special".to_string()]),
        })
        .create()
        .await;

    // Rule 2: Priority 5 (Loser)
    factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_name("Low Priority Rule")
        .with_pattern(pattern_b)
        .with_priority(50)
        .with_conditions(AssignmentRuleConditions {
            department_id: None,
            employee_id_numbers: None,
            tags: Some(vec!["Special".to_string()]),
        })
        .create()
        .await;

    // 4. Resolve schedule
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let (shift_id, traces) = roster_repository::get_effective_shift(&pool, emp_id, test_date).await.unwrap();

    // 5. Verify
    assert_eq!(shift_id, Some(shift_a));
    
    // We expect 2 traces: one applied, one overridden
    assert_eq!(traces.len(), 2);
    
    let applied = traces.iter().find(|t| t.status == "applied").unwrap();
    assert_eq!(applied.rule_name, "High Priority Rule");
    
    let overridden = traces.iter().find(|t| t.status == "overridden").unwrap();
    assert_eq!(overridden.rule_name, "Low Priority Rule");
    assert!(overridden.reason.as_ref().unwrap().contains("Overridden"));
}

#[tokio::test]
async fn test_workspace_schedules_with_traces() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let ws_id = factory.workspace().create().await;
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    let shift_id = factory.shift().with_workspace(ws_id).create().await;

    // Assign via manual roster (highest priority)
    let test_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    sqlx::query!(
        "INSERT INTO shift_rosters (id, employee_id, shift_id, date, status) VALUES ($1, $2, $3, $4, 'confirmed')",
        uuid::Uuid::new_v4(), emp_id, shift_id, test_date
    ).execute(&pool).await.unwrap();

    // Fetch workspace schedules
    let schedules = roster_repository::get_workspace_schedules(&pool, ws_id, test_date, test_date).await.unwrap();
    
    let emp_sched = schedules.iter().find(|s| s.employee_id == emp_id).unwrap();
    let day_sched = &emp_sched.schedules[0];
    
    assert_eq!(day_sched.shift_id, Some(shift_id));
    assert_eq!(day_sched.source, "roster");
    assert_eq!(day_sched.traces.len(), 1);
    assert_eq!(day_sched.traces[0].rule_name, "Manual Roster");
    assert_eq!(day_sched.traces[0].status, "applied");
}
