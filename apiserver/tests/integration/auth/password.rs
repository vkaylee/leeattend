//! Authentication password change integration tests

use crate::common::app::spawn_app;
use crate::common::db::TestDatabase;
use apiserver::models::{AuthResponse, ChangePasswordRequest, LoginRequest, RegisterRequest};

#[tokio::test]
async fn test_change_password_flow() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = reqwest::Client::new();

    // 1. Register
    let reg_req = RegisterRequest {
        email: "test_change@example.com".to_string(),
        password: "old_password_123".to_string(),
        full_name: "Test User".to_string(),
    };
    let res = client
        .post(format!("{}/api/auth/register", address))
        .json(&reg_req)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 2. Login
    let login_req = LoginRequest {
        email: "test_change@example.com".to_string(),
        password: "old_password_123".to_string(),
    };
    let res = client
        .post(format!("{}/api/auth/login", address))
        .json(&login_req)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let auth_res: AuthResponse = res.json().await.unwrap();
    let token = auth_res.access_token;

    // 3. Change Password
    let change_req = ChangePasswordRequest {
        old_password: "old_password_123".to_string(),
        new_password: "new_password_456".to_string(),
    };
    let res = client
        .post(format!("{}/api/auth/change-password", address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&change_req)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 4. Try login with OLD password (should fail)
    let login_req_old = LoginRequest {
        email: "test_change@example.com".to_string(),
        password: "old_password_123".to_string(),
    };
    let res = client
        .post(format!("{}/api/auth/login", address))
        .json(&login_req_old)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 401);

    // 5. Try login with NEW password (should succeed)
    let login_req_new = LoginRequest {
        email: "test_change@example.com".to_string(),
        password: "new_password_456".to_string(),
    };
    let res = client
        .post(format!("{}/api/auth/login", address))
        .json(&login_req_new)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}
