use crate::common::db::TestDatabase;
use apiserver::diesel_pool::TenantAuthToken;
use apiserver::repositories::diesel_holiday_repository::DieselHolidayRepository;
use apiserver::repositories::holiday_repository::HolidayRepository;
use uuid::Uuid;

#[tokio::test]
async fn test_holiday_repo_list_empty_for_new_tenant() {
    let test_db = TestDatabase::new().await;
    let repo = DieselHolidayRepository::new(test_db.diesel_pools.clone());

    let mock_workspace_id = Uuid::new_v4();
    let token = TenantAuthToken::system_override(mock_workspace_id);

    let (holidays, _total) = repo
        .list(&token, Some(2026), None, 1, 50)
        .await
        .expect("Query failed");

    assert!(
        holidays.is_empty(),
        "Expected empty holidays for a mock workspace"
    );
}
