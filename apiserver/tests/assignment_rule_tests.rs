mod common;

use apiserver::models::AssignmentRuleConditions;
use apiserver::repositories::roster_repository;
use chrono::NaiveDate;
use common::db::TestDatabase;
use common::factory::TestFactory;

#[tokio::test]
async fn test_rule_based_pattern_resolution() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());

    let ws_id = factory.workspace().create().await;
    let _dept_id = factory.department().with_workspace(ws_id).create().await;

    // 1. Create a Pattern
    let shift_id = factory.shift().with_workspace(ws_id).create().await;
    let pattern_id = sqlx::query_scalar!(
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, 'Rule Pattern') RETURNING id",
        ws_id
    ).fetch_one(&pool).await.unwrap();

    for i in 0..7 {
        sqlx::query!(
            "INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)",
            pattern_id, i, shift_id
        ).execute(&pool).await.unwrap();
    }

    // 2. Create Employee with Tags
    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    sqlx::query!(
        "UPDATE employees SET tags = $1 WHERE id = $2",
        &vec!["NightShift".to_string()],
        emp_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // 3. Create Rule: if tag = 'NightShift' -> use Rule Pattern
    factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_pattern(pattern_id)
        .with_priority(10)
        .with_conditions(AssignmentRuleConditions {
            department_id: None,
            employee_id_numbers: None,
            tags: Some(vec!["NightShift".to_string()]),
        })
        .create()
        .await;

    // 4. Verify Resolution
    let date = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(); // Sunday
    let (resolved_shift, _) = roster_repository::get_effective_shift(&pool, emp_id, date)
        .await
        .unwrap();

    assert_eq!(resolved_shift, Some(shift_id));
}

#[tokio::test]
async fn test_rule_priority_and_overrides() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    let ws_id = factory.workspace().create().await;

    let shift_a = factory
        .shift()
        .with_name("Shift A")
        .with_workspace(ws_id)
        .create()
        .await;
    let shift_b = factory
        .shift()
        .with_name("Shift B")
        .with_workspace(ws_id)
        .create()
        .await;

    let pattern_a = sqlx::query_scalar!(
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, 'Pattern A') RETURNING id",
        ws_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let pattern_b = sqlx::query_scalar!(
        "INSERT INTO working_patterns (workspace_id, name) VALUES ($1, 'Pattern B') RETURNING id",
        ws_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    for i in 0..7 {
        sqlx::query!("INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)", pattern_a, i, shift_a).execute(&pool).await.unwrap();
        sqlx::query!("INSERT INTO working_pattern_details (pattern_id, day_of_week, shift_id) VALUES ($1, $2, $3)", pattern_b, i, shift_b).execute(&pool).await.unwrap();
    }

    let emp_id = factory.employee().with_workspace(ws_id).create().await;
    sqlx::query!(
        "UPDATE employees SET tags = $1 WHERE id = $2",
        &vec!["TagA".to_string()],
        emp_id
    )
    .execute(&pool)
    .await
    .unwrap();

    // Low priority rule (Match All)
    factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_pattern(pattern_b)
        .with_priority(1)
        .create()
        .await;

    // High priority rule (Tag A)
    factory
        .assignment_rule()
        .with_workspace(ws_id)
        .with_pattern(pattern_a)
        .with_priority(100)
        .with_conditions(AssignmentRuleConditions {
            department_id: None,
            employee_id_numbers: None,
            tags: Some(vec!["TagA".to_string()]),
        })
        .create()
        .await;

    let date = NaiveDate::from_ymd_opt(2026, 1, 18).unwrap();
    let (resolved, _) = roster_repository::get_effective_shift(&pool, emp_id, date)
        .await
        .unwrap();

    assert_eq!(resolved, Some(shift_a)); // Shift A wins because priority 100 > 1
}
