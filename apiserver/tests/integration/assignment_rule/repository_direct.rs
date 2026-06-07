//! Repository-level integration tests for assignment rule repository.

use crate::common::db::TestDatabase;
use crate::common::factory::TestFactory;
use apiserver::models::employee::{
    AssignmentRuleConditions, CreateAssignmentRuleRequest, UpdateAssignmentRuleRequest,
};
use apiserver::repositories::{
    assignment_rule_repository::AssignmentRuleRepository,
    diesel_assignment_rule_repository::DieselAssignmentRuleRepository,
};
use uuid::Uuid;

async fn setup_db() -> (TestDatabase, TestFactory) {
    let test_db = TestDatabase::new().await;
    let factory = TestFactory::new(test_db.diesel_pools.clone());
    (test_db, factory)
}

#[tokio::test]
async fn test_assignment_rule_crud() {
    let (test_db, factory) = setup_db().await;

    let user_id = factory.user().create().await;
    let ws_id = factory.workspace().create_with_owner(user_id).await;
    let auth_token = apiserver::diesel_pool::TenantAuthToken::system_override(ws_id);

    let repo = DieselAssignmentRuleRepository::new(test_db.diesel_pools.clone());

    // Create
    let pattern_id = Uuid::new_v4();
    crate::test_execute!(
        &test_db.diesel_pools.api,
        "INSERT INTO working_patterns (id, workspace_id, name, pattern_type) VALUES ($1, $2, 'Test Pattern', 'weekly')",
        diesel::sql_types::Uuid, pattern_id,
        diesel::sql_types::Uuid, ws_id
    );
    let create_req = CreateAssignmentRuleRequest {
        pattern_id,
        name: "Test Rule".to_string(),
        priority: 10,
        conditions: AssignmentRuleConditions::new(
            None,
            Some(vec!["E123".to_string()]),
            None,
            None,
            None,
        ),
        start_date: None,
        end_date: None,
    };

    let rule = repo
        .create_rule(&auth_token, &create_req)
        .await
        .expect("Failed to create rule");
    assert_eq!(rule.name(), "Test Rule");
    assert_eq!(rule.priority(), 10);
    assert_eq!(rule.pattern_id(), pattern_id);
    assert!(rule.is_active());

    // Read - get_rules
    let rules = repo
        .get_rules(&auth_token)
        .await
        .expect("Failed to get rules");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id(), rule.id());

    // Read - get_active_rules
    let active_rules = repo
        .get_active_rules(&auth_token)
        .await
        .expect("Failed to get active rules");
    assert_eq!(active_rules.len(), 1);

    // Update
    let update_req = UpdateAssignmentRuleRequest {
        pattern_id: None,
        name: Some("Updated Rule".to_string()),
        priority: Some(20),
        conditions: None,
        is_active: Some(false),
        max_concurrent_workers: None,
        start_date: None,
        end_date: None,
    };
    let updated_rule = repo
        .update_rule(&auth_token, rule.id(), &update_req)
        .await
        .expect("Failed to update rule");
    assert_eq!(updated_rule.name(), "Updated Rule");
    assert_eq!(updated_rule.priority(), 20);
    assert!(!updated_rule.is_active());

    // Read active again
    let active_rules_after = repo
        .get_active_rules(&auth_token)
        .await
        .expect("Failed to get active rules");
    assert_eq!(active_rules_after.len(), 0);

    // Delete
    repo.delete_rule(&auth_token, rule.id())
        .await
        .expect("Failed to delete rule");
    let rules_after_delete = repo
        .get_rules(&auth_token)
        .await
        .expect("Failed to get rules");
    assert_eq!(rules_after_delete.len(), 0);
}
