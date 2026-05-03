//! Integration tests for auto-assign `platform_admin` to the first registered user.

use crate::common::app::spawn_app;
use crate::common::db::TestDatabase;
use apiserver::models::{AuthResponse, RegisterRequest};

/// The very first user to register on a fresh database should
/// automatically receive the `platform_admin` system role.
#[tokio::test]
async fn test_first_user_gets_platform_admin() {
    let (address, test_db): (String, TestDatabase) = spawn_app().await;
    let client = reqwest::Client::new();
    let pool = test_db.pool();

    // Verify DB starts with zero users (seed data removed)
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_count, 0, "DB should start empty (no seeded users)");

    // 1. Register the first user
    let reg_req = RegisterRequest {
        email: "founder@company.com".to_string(),
        password: "secure_pass_1!".to_string(),
        full_name: "Platform Founder".to_string(),
    };
    let res = client
        .post(format!("{address}/api/auth/register"))
        .json(&reg_req)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "First registration should succeed");

    let auth_res: AuthResponse = res.json().await.unwrap();
    let first_user_id = auth_res.user.id;

    // 2. Verify the first user has the platform_admin system role
    let admin_role_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_system_roles usr \
         JOIN system_roles sr ON usr.role_id = sr.id \
         WHERE usr.user_id = $1 AND sr.name = 'platform_admin'",
    )
    .bind(first_user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        admin_role_count, 1,
        "First user must be auto-assigned platform_admin"
    );

    // 3. Register a second user
    let reg_req2 = RegisterRequest {
        email: "employee@company.com".to_string(),
        password: "secure_pass_2!".to_string(),
        full_name: "Regular Employee".to_string(),
    };
    let res2 = client
        .post(format!("{address}/api/auth/register"))
        .json(&reg_req2)
        .send()
        .await
        .unwrap();
    assert_eq!(res2.status(), 200, "Second registration should succeed");

    let auth_res2: AuthResponse = res2.json().await.unwrap();
    let second_user_id = auth_res2.user.id;

    // 4. Verify the second user does NOT have platform_admin
    let admin_role_count2: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_system_roles usr \
         JOIN system_roles sr ON usr.role_id = sr.id \
         WHERE usr.user_id = $1 AND sr.name = 'platform_admin'",
    )
    .bind(second_user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        admin_role_count2, 0,
        "Second user must NOT get platform_admin"
    );

    // 5. Verify total system role assignments: only 1 platform_admin exists
    let total_admins: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_system_roles usr \
         JOIN system_roles sr ON usr.role_id = sr.id \
         WHERE sr.name = 'platform_admin'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        total_admins, 1,
        "Exactly one platform_admin should exist in the system"
    );
}
