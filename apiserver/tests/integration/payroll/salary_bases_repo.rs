use crate::common::db::TestDatabase;
use apiserver::repositories::diesel_salary_base_repository::DieselSalaryBaseRepository;
use apiserver::repositories::salary_base_repository::SalaryBaseRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_isolation_assign_component() {
    let test_db = TestDatabase::new().await;
    let repo = DieselSalaryBaseRepository::new(test_db.diesel_pools.clone());

    let workspace_id = Uuid::new_v4();
    let component_id = Uuid::new_v4();
    let base_group_id = Uuid::new_v4();

    let result = repo
        .assign_component_to_group(workspace_id, component_id, base_group_id)
        .await;

    // Since the base_group_id does not exist, the workspace validation must fail
    assert!(
        matches!(result, Err(apiserver::errors::AppError::Forbidden(_))),
        "Expected Forbidden, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_tenant_isolation_unassign_component() {
    let test_db = TestDatabase::new().await;
    let repo = DieselSalaryBaseRepository::new(test_db.diesel_pools.clone());

    let workspace_id = Uuid::new_v4();
    let component_id = Uuid::new_v4();
    let base_group_id = Uuid::new_v4();

    // This should not fail with Forbidden since the DB just deletes 0 rows safely if validation fails.
    let result = repo
        .unassign_component_from_group(workspace_id, component_id, base_group_id)
        .await;
    assert!(result.is_ok());
}
