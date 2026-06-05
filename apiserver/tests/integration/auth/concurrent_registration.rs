//! Integration tests for concurrent user registration to test race condition handling.

use crate::common::app::spawn_app;
use crate::common::db::TestDatabase;
use apiserver::models::User;
use apiserver::repositories::user_repository::UserRepository;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_registration_platform_admin_assignment() {
    let (_, test_db): (String, TestDatabase) = spawn_app().await;
    let pool = test_db.diesel_pools.clone();

    // Verify DB starts with zero users
    let user_count: i64 =
        crate::test_query_scalar_i64!(&pool.api, "SELECT count(*) AS count FROM users");
    assert_eq!(user_count, 0, "DB should start empty");

    let repo = Arc::new(
        apiserver::repositories::diesel_user_repository::DieselUserRepository::new(pool.clone()),
    );

    let num_tasks = 10;
    let mut join_handles = vec![];

    for i in 0..num_tasks {
        let repo_clone = Arc::clone(&repo);
        let handle = tokio::spawn(async move {
            let password_hash = "fake_hash".to_string();
            let user = User::new(
                Uuid::new_v4(),
                format!("concurrent_user_{i}@test.com"),
                password_hash,
                Utc::now(),
                Some(format!("User {i}")),
                None,
                None,
                None,
            );

            // Call the atomic transaction method
            repo_clone.create_user_with_admin_check(&user).await
        });
        join_handles.push(handle);
    }

    let mut success_count = 0;
    let mut admin_assignments = 0;

    for handle in join_handles {
        let result = handle.await.expect("Task panicked");
        if let Ok(is_first) = result {
            success_count += 1;
            if is_first {
                admin_assignments += 1;
            }
        }
    }

    assert_eq!(
        success_count, num_tasks,
        "All users should be created successfully"
    );
    assert_eq!(
        admin_assignments, 1,
        "Exactly one task should report being the first user"
    );

    // Verify total system role assignments: only 1 platform_admin exists
    let total_admins: i64 = crate::test_query_scalar_i64!(
        &pool.api,
        "SELECT count(*) AS count FROM user_system_roles usr \
         JOIN system_roles sr ON usr.role_id = sr.id \
         WHERE sr.name = 'platform_admin'"
    );
    assert_eq!(
        total_admins, 1,
        "Exactly one platform_admin should exist in the system despite concurrent registrations"
    );
}
