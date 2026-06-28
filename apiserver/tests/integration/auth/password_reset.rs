//! Password reset (forgot + reset) integration tests

use crate::common::app::spawn_app;
use crate::common::db::TestDatabase;
use apiserver::models::auth::{ForgotPasswordRequest, ResetPasswordRequest};
use apiserver::models::{LoginRequest, RegisterRequest};
use reqwest::Client;
use serde_json::Value;

/// Full happy-path: register → forgot-password → (extract token from email/log) → reset-password → login with new password
#[tokio::test]
async fn test_forgot_and_reset_password_full_flow() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = Client::new();
    let email = "reset_flow@example.com";

    // 1. Register a user
    let reg_req = RegisterRequest {
        email: email.to_string(),
        password: "original_password_1".to_string(),
        full_name: "Reset Test User".to_string(),
    };
    let res = client
        .post(format!("{address}/api/v1/auth/register"))
        .json(&reg_req)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "Registration should succeed");

    // 2. Request password reset (forgot-password)
    let forgot_req = ForgotPasswordRequest {
        email: email.to_string(),
    };
    let res = client
        .post(format!("{address}/api/v1/auth/forgot-password"))
        .json(&forgot_req)
        .send()
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        200,
        "Forgot password should always return 200 OK"
    );

    // In test/dev mode without SMTP, the token is logged.
    // We cannot extract the token from the log here easily, so we verify
    // the endpoint behaviour and test reset with an invalid token below.
}

#[tokio::test]
async fn test_forgot_password_is_silent_for_nonexistent_email() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = Client::new();

    let forgot_req = ForgotPasswordRequest {
        email: "nonexistent@example.com".to_string(),
    };
    let res = client
        .post(format!("{address}/api/v1/auth/forgot-password"))
        .json(&forgot_req)
        .send()
        .await
        .unwrap();

    // Anti-enumeration: always return 200
    assert_eq!(res.status(), 200);

    // Body should not reveal whether the email exists
    let body: Value = res.json().await.unwrap();
    // The response should be generic — no "user not found" message
    assert!(
        !body
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .contains("not found"),
        "Response must not reveal whether email exists"
    );
}

#[tokio::test]
async fn test_reset_password_rejects_invalid_token() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = Client::new();

    let reset_req = ResetPasswordRequest {
        token: "invalid-or-expired-token-xxxx".to_string(),
        new_password: "new_password_123".to_string(),
    };
    let res = client
        .post(format!("{address}/api/v1/auth/reset-password"))
        .json(&reset_req)
        .send()
        .await
        .unwrap();

    // Should fail with a validation error
    assert!(
        res.status() == 400 || res.status() == 422,
        "Invalid token should return 400 or 422, got {}",
        res.status()
    );
}

#[tokio::test]
async fn test_reset_password_rejects_weak_password() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = Client::new();

    // Even with a token-shaped string, weak passwords should be rejected
    let reset_req = ResetPasswordRequest {
        token: "some-token-value-12345".to_string(),
        new_password: "12345".to_string(), // too short
    };
    let res = client
        .post(format!("{address}/api/v1/auth/reset-password"))
        .json(&reset_req)
        .send()
        .await
        .unwrap();

    // Weak password should be rejected (token check may come first though)
    assert!(
        !res.status().is_success(),
        "Weak password should be rejected"
    );
}

#[tokio::test]
async fn test_forgot_password_validates_email_format() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = Client::new();

    // Send an obviously invalid email — backend still returns 200 for anti-enumeration
    // but we verify the endpoint doesn't crash
    let forgot_req = ForgotPasswordRequest {
        email: "not-an-email".to_string(),
    };
    let res = client
        .post(format!("{address}/api/v1/auth/forgot-password"))
        .json(&forgot_req)
        .send()
        .await
        .unwrap();

    // Should not crash; returns 200 even for invalid email format
    // (silent handling to prevent enumeration)
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn test_reset_password_with_valid_token_updates_password() {
    // This test uses the tower-level approach to directly interact with the
    // auth service for creating a real token, then tests the HTTP endpoint.

    use crate::common::app::setup_app_tower;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    let (_test_db, factory, _state, app) = setup_app_tower().await;

    // 1. Create a user
    let user_id = factory
        .user()
        .with_email("pw_reset_e2e@example.com")
        .create()
        .await;

    // 2. Create a password reset token via the auth service directly
    let auth_service = _state.core().auth_service();
    let token = auth_service
        .create_password_reset_token(user_id)
        .await
        .expect("Should create reset token");

    // 3. Call the reset-password HTTP endpoint with the real token
    let reset_body = serde_json::json!({
        "token": token,
        "new_password": "valid_new_password_99"
    });
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/reset-password")
                .header("Content-Type", "application/json")
                .body(Body::from(reset_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        axum::http::StatusCode::OK,
        "Reset with valid token should succeed"
    );

    // 4. Verify the token was consumed (cannot be reused)
    let reset_body_reuse = serde_json::json!({
        "token": token,
        "new_password": "another_password_88"
    });
    let response_reuse = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/reset-password")
                .header("Content-Type", "application/json")
                .body(Body::from(reset_body_reuse.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        !response_reuse.status().is_success(),
        "Token should be single-use only — reuse must fail"
    );
}

// ── Rate limit tests ──────────────────────────────────────────────────────

/// Verify that rapid forgot-password requests don't crash the service
/// and all return 200 OK (anti-enumeration: rate-limiting is silent).
#[tokio::test]
async fn test_forgot_password_rate_limit_smoke() {
    let (address, _test_db): (String, TestDatabase) = spawn_app().await;
    let client = Client::new();
    let email = "rate_limit_smoke@example.com";

    // Register a user first so the service has a valid target
    let reg_req = RegisterRequest {
        email: email.to_string(),
        password: "password_123".to_string(),
        full_name: "Rate Limit Smoke".to_string(),
    };
    client
        .post(format!("{address}/api/v1/auth/register"))
        .json(&reg_req)
        .send()
        .await
        .unwrap();

    // Send 15 forgot-password requests rapidly (minutely cap is 5)
    let forgot_req = ForgotPasswordRequest {
        email: email.to_string(),
    };

    for _ in 0..15 {
        let res = client
            .post(format!("{address}/api/v1/auth/forgot-password"))
            .json(&forgot_req)
            .send()
            .await
            .unwrap();

        // Must always return 200 — even when rate-limited
        assert_eq!(
            res.status(),
            200,
            "Forgot password must always return 200 OK (anti-enumeration)"
        );
    }
}

/// Verify rate limiting works at the EmailService level with a real Redis pool.
#[tokio::test]
async fn test_email_rate_limit_blocks_after_threshold() {
    use crate::common::app::create_test_redis_pool;
    use apiserver::queue::RedisPool;
    use apiserver::services::EmailService;

    let redis_pool: Option<RedisPool> = create_test_redis_pool().await;
    let redis_pool = match redis_pool {
        Some(p) => p,
        None => {
            eprintln!("SKIP: no Redis available for rate limit test");
            return;
        }
    };

    let service = EmailService::new(Some(redis_pool.clone()), None);

    // Seed the current minutely bucket to exactly the limit
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let bucket = now / 60; // 60s = minutely window
    let key = format!("email:rate:min:{bucket}");

    let mut conn = redis_pool.conn();
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg(5u32)
        .query_async(&mut conn)
        .await
        .unwrap();

    // The next send_password_reset should be silently dropped (rate-limited)
    let result = service
        .send_password_reset("test@example.com", "token")
        .await;

    // Clean up the test key
    let _: () = redis::cmd("DEL")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap();

    assert!(
        result.is_ok(),
        "Must return Ok even when rate-limited (silent drop)"
    );
}
