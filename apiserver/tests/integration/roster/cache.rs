//! Roster cache build data tests

use crate::integration::common::db::TestDatabase;
use crate::integration::common::factory::TestFactory;
use apiserver::cache::build_roster_data;


#[tokio::test]
async fn test_empty_workspace_all_maps_empty() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    let workspace_id = factory.workspace().create().await;
    
    let data = build_roster_data(&pool, workspace_id).await.unwrap();
    
    assert!(data.shifts.is_empty());
    assert!(data.fixed_assignments.is_empty());
    assert!(data.pattern_assignments.is_empty());
}

#[tokio::test]
async fn test_shifts_populated() {
    let test_db = TestDatabase::new().await;
    let pool = test_db.pool();
    let factory = TestFactory::new(pool.clone());
    
    let workspace_id = factory.workspace().create().await;
    let s1 = factory.shift().with_workspace(workspace_id).create().await;
    let s2 = factory.shift().with_workspace(workspace_id).create().await;

    let data = build_roster_data(&pool, workspace_id).await.unwrap();
    
    assert_eq!(data.shifts.len(), 2);
    assert!(data.shifts.contains_key(&s1));
    assert!(data.shifts.contains_key(&s2));
}
