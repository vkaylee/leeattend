use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use std::sync::Arc;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AuthResponse, ChangePasswordRequest, LoginRequest, RefreshRequest, RegisterRequest,
    UpdateUserRequest, User, UserResponse,
};

use crate::services::auth_service::AuthService;
use crate::{AppState, UserId};

// --- Password Validation ---

const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 128;

fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "Password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "Password must not exceed {MAX_PASSWORD_LEN} characters"
        )));
    }

    // Simple strength check: must have at least one digit or special char
    if !password
        .chars()
        .any(|c| c.is_ascii_digit() || c.is_ascii_punctuation())
    {
        return Err(AppError::Validation(
            "Password must contain at least one digit or special character".to_string(),
        ));
    }

    Ok(())
}

fn validate_email(email: &str) -> AppResult<()> {
    // Basic structural validation: local@domain.tld
    let trimmed = email.trim();
    if trimmed.is_empty() || trimmed.len() > 254 {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }

    let parts: Vec<&str> = trimmed.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }

    let (local, domain) = (parts[0], parts[1]);

    // Local part must not be empty
    if local.is_empty() || local.len() > 64 {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }

    // Domain must have at least one dot, no consecutive dots, no leading/trailing dots
    if domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
    {
        return Err(AppError::Validation("Invalid email format".to_string()));
    }

    Ok(())
}

// --- Cookie Helpers ---

fn create_auth_cookies(access_token: &str, refresh_token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let domain_attr = match std::env::var("COOKIE_DOMAIN") {
        Ok(d) if !d.is_empty() => format!("Domain={d}; "),
        _ => String::new(),
    };

    // access_token cookie
    let access_cookie = format!(
        "access_token={access_token}; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=900"
    );
    // refresh_token cookie
    let refresh_cookie = format!(
        "refresh_token={refresh_token}; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=2592000"
    );

    if let Ok(val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, val);
    }
    if let Ok(val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, val);
    }

    headers
}

fn clear_auth_cookies() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let domain_attr = match std::env::var("COOKIE_DOMAIN") {
        Ok(d) if !d.is_empty() => format!("Domain={d}; "),
        _ => String::new(),
    };

    let access_cookie =
        format!("access_token=; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0");
    let refresh_cookie =
        format!("refresh_token=; {domain_attr}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0");

    if let Ok(val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, val);
    }
    if let Ok(val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, val);
    }

    headers
}

// --- Auth Handlers ---

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "Successfully registered", body = AuthResponse)
    )
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    // 1. Validation
    if req.full_name.trim().is_empty() {
        return Err(AppError::Validation("Full name is required".to_string()));
    }
    validate_email(&req.email)?;
    validate_password(&req.password)?;

    tracing::info!(email = %crate::utils::masking::mask_email(&req.email), "User attempting to register");

    // 2. Check if email exists
    let existing_user = state
        .core()
        .user_repo()
        .get_user_by_email(&req.email)
        .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // 3. Hash password
    let password = req.password.clone();
    let password_hash = tokio::task::spawn_blocking(move || AuthService::hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("Failed to spawn blocking task: {e}")))??;

    // 4. Create User
    let user_id = crate::utils::id::new_id();
    let created_at = Utc::now();
    let user = User::new(
        user_id,
        req.email.clone(),
        password_hash,
        created_at,
        Some(req.full_name.clone()),
        None,
        None,
        None,
    );

    // 4. Create User and auto-assign platform_admin atomically
    let is_first = state
        .core()
        .user_repo()
        .create_user_with_admin_check(&user)
        .await?;

    if is_first {
        tracing::info!(
            user_id = %user.id(),
            "First user registered — auto-assigned platform_admin role"
        );
    }

    // 6. Create session (post-commit — session is ephemeral, safe outside tx)
    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user.id())
        .await?;

    // Convert to UserResponse for the client
    let user_response = UserResponse {
        id: user.id(),
        email: user.email().to_string(),
        full_name: user.full_name().map(String::from),
        avatar_url: user.avatar_url().map(String::from),
        bio: user.bio().map(String::from),
        preferred_language: user.preferred_language().map(String::from),
        created_at: user.created_at(),
        system_roles: vec![],
    };

    // 7. Generate access token (stateless, no DB write needed)
    let access_token = AuthService::generate_token(user.id(), state.core().jwt_secret(), 15)?;

    let headers = create_auth_cookies(&access_token, &refresh_token);

    Ok((
        headers,
        Json(AuthResponse {
            access_token,
            refresh_token,
            user: user_response,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Successfully logged in", body = AuthResponse)
    )
)]
#[allow(clippy::too_many_lines)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Response> {
    let email = req.email.trim().to_lowercase();
    let masked_email = crate::utils::masking::mask_email(&email);
    tracing::info!(email = %masked_email, "User attempting to login");

    let redis_pool = state.infra().redis();

    // 1. Check if the account is currently locked out
    if let Some(pool) = redis_pool {
        let lockout_key = format!("login:lockout:{email}");
        let mut conn = pool.conn();
        use redis::AsyncCommands;
        if let Ok(Some(_attempts_str)) = conn.get::<_, Option<String>>(&lockout_key).await {
            tracing::warn!(email = %masked_email, "Login blocked: Account is locked");
            return Ok((
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": {
                        "code": "TOO_MANY_REQUESTS",
                        "message": crate::utils::i18n::t("error.login_locked")
                    }
                })),
            )
                .into_response());
        }
    }

    let user_opt = state.core().user_repo().get_user_by_email(&email).await?;

    let Some(user) = user_opt else {
        tracing::warn!(email = %masked_email, "Login failed: User not found");
        if let Some(pool) = redis_pool {
            let attempts_key = format!("login:attempts:{email}");
            let lockout_key = format!("login:lockout:{email}");
            let mut conn = pool.conn();
            use redis::AsyncCommands;
            let attempts: u32 = conn.incr(&attempts_key, 1).await.unwrap_or(0);
            if attempts == 1 {
                let _: Result<(), _> = conn.expire(&attempts_key, 900).await;
            }
            if attempts >= 5 {
                let exponent = attempts.saturating_sub(5);
                let multiplier = 2_u32.pow(exponent.min(6));
                let lock_secs = 900 * multiplier;
                let _: Result<(), _> = conn
                    .set_ex(&lockout_key, attempts.to_string(), u64::from(lock_secs))
                    .await;
            }
        }
        return Err(AppError::Auth("Invalid email or password".to_string()));
    };

    let password = req.password.clone();
    let hash = user.password_hash().to_string();
    let (is_valid, needs_rehash) =
        tokio::task::spawn_blocking(move || AuthService::verify_password(&password, &hash))
            .await
            .map_err(|e| AppError::Internal(format!("Failed to spawn blocking task: {e}")))?;

    if !is_valid {
        tracing::warn!(user_id = %user.id(), "Login failed: Invalid password");
        if let Some(pool) = redis_pool {
            let attempts_key = format!("login:attempts:{email}");
            let lockout_key = format!("login:lockout:{email}");
            let mut conn = pool.conn();
            use redis::AsyncCommands;
            let attempts: u32 = conn.incr(&attempts_key, 1).await.unwrap_or(0);
            if attempts == 1 {
                let _: Result<(), _> = conn.expire(&attempts_key, 900).await;
            }
            if attempts >= 5 {
                let exponent = attempts.saturating_sub(5);
                let multiplier = 2_u32.pow(exponent.min(6));
                let lock_secs = 900 * multiplier;
                let _: Result<(), _> = conn
                    .set_ex(&lockout_key, attempts.to_string(), u64::from(lock_secs))
                    .await;
            }
        }
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    // Success -> Clear Redis attempts and lockout keys
    if let Some(pool) = redis_pool {
        let attempts_key = format!("login:attempts:{email}");
        let lockout_key = format!("login:lockout:{email}");
        let mut conn = pool.conn();
        use redis::AsyncCommands;
        let _: Result<(), _> = conn.del(&[attempts_key, lockout_key]).await;
    }

    // Transparent rehash: migrate legacy bcrypt hash to argon2id
    if needs_rehash {
        let pw_for_rehash = req.password.clone();
        let new_hash =
            tokio::task::spawn_blocking(move || AuthService::hash_password(&pw_for_rehash))
                .await
                .map_err(|e| AppError::Internal(format!("Failed to spawn blocking task: {e}")))?;
        if let Ok(h) = new_hash {
            if let Err(e) = state
                .core()
                .user_repo()
                .update_password_hash(user.id(), &h)
                .await
            {
                tracing::warn!(user_id = %user.id(), error = %e, "Failed to rehash password");
            } else {
                tracing::info!(user_id = %user.id(), "Password rehashed from bcrypt to argon2id");
            }
        }
    }

    tracing::info!(user_id = %user.id(), "User logged in successfully");

    // Convert to UserResponse for the client
    let user_response = UserResponse {
        id: user.id(),
        email: user.email().to_string(),
        full_name: user.full_name().map(String::from),
        avatar_url: user.avatar_url().map(String::from),
        bio: user.bio().map(String::from),
        preferred_language: user.preferred_language().map(String::from),
        created_at: user.created_at(),
        system_roles: vec![],
    };

    let access_token = AuthService::generate_token(user.id(), state.core().jwt_secret(), 15)?;

    let refresh_token = state
        .core()
        .auth_service()
        .create_session(user.id())
        .await?;

    let headers = create_auth_cookies(&access_token, &refresh_token);

    Ok((
        headers,
        Json(AuthResponse {
            access_token,
            refresh_token,
            user: user_response,
        }),
    )
        .into_response())
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Successfully refreshed session", body = AuthResponse)
    )
)]
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req_opt: Option<Json<RefreshRequest>>,
) -> AppResult<impl IntoResponse> {
    // Try to get refresh_token from body, fallback to cookie
    let mut token_to_refresh = req_opt.map(|Json(req)| req.refresh_token);

    if token_to_refresh.is_none() {
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(token) = cookie.strip_prefix("refresh_token=") {
                        token_to_refresh = Some(token.to_string());
                    }
                }
            }
        }
    }

    let token =
        token_to_refresh.ok_or_else(|| AppError::Auth("No refresh token provided".to_string()))?;

    let auth_response = state
        .core()
        .auth_service()
        .refresh_session(&token, state.core().jwt_secret())
        .await?;

    let resp_headers =
        create_auth_cookies(&auth_response.access_token, &auth_response.refresh_token);

    Ok((resp_headers, Json(auth_response)))
}

#[utoipa::path(
    post,
    path = "/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Successfully changed password")
    )
)]
pub async fn change_password(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    // 1. Get current user to verify old password
    let user = state
        .core()
        .user_repo()
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // 2. Verify old password
    let old_password = req.old_password.clone();
    let old_hash = user.password_hash().to_string();
    let (is_valid, _needs_rehash) =
        tokio::task::spawn_blocking(move || AuthService::verify_password(&old_password, &old_hash))
            .await
            .map_err(|e| AppError::Internal(format!("Failed to spawn blocking task: {e}")))?;

    if !is_valid {
        return Err(AppError::Auth("Current password is incorrect".to_string()));
    }

    // 3. Validate new password strength
    validate_password(&req.new_password)?;

    // 4. Hash new password
    let new_password = req.new_password.clone();
    let new_hash = tokio::task::spawn_blocking(move || AuthService::hash_password(&new_password))
        .await
        .map_err(|e| AppError::Internal(format!("Failed to spawn blocking task: {e}")))??;

    // 5. Update DB via repository
    state
        .core()
        .user_repo()
        .update_password_hash(user_id, &new_hash)
        .await?;

    // 6. Revoke all sessions (Force re-login)
    if let Err(e) = state
        .core()
        .auth_service()
        .revoke_all_sessions_for_user(user_id)
        .await
    {
        tracing::error!(user_id = %user_id, error = %e, "Failed to revoke sessions after password change");
    }

    Ok((
        StatusCode::OK,
        "Password changed successfully. Please log in again.",
    ))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Successfully logged out")
    )
)]
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req_opt: Option<Json<RefreshRequest>>,
) -> AppResult<impl IntoResponse> {
    // Try to get refresh_token from body, fallback to cookie
    let mut token_to_revoke = req_opt.map(|Json(req)| req.refresh_token);

    if token_to_revoke.is_none() {
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(token) = cookie.strip_prefix("refresh_token=") {
                        token_to_revoke = Some(token.to_string());
                    }
                }
            }
        }
    }

    if let Some(token) = token_to_revoke {
        let _ = state.core().auth_service().revoke_session(&token).await;
    }

    // Extract access token to blocklist it immediately
    let mut access_token_to_block = None;
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if let Some(token) = auth_str.strip_prefix("Bearer ") {
                access_token_to_block = Some(token.to_string());
            }
        }
    }
    if access_token_to_block.is_none() {
        if let Some(cookie_header) = headers.get("cookie") {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(token) = cookie.strip_prefix("access_token=") {
                        access_token_to_block = Some(token.to_string());
                    }
                }
            }
        }
    }

    if let Some(token) = access_token_to_block {
        if let Some(redis_pool) = state.infra().redis() {
            if let Some(sig) = token.rsplit('.').next() {
                let key = format!("jwt:revoked:{sig}");
                let mut conn = redis_pool.conn();
                use redis::AsyncCommands;
                let mut ttl: u64 = 900;
                if let Ok(claims) =
                    AuthService::verify_token_claims(&token, state.core().jwt_secret())
                {
                    let now = Utc::now().timestamp();
                    if claims.exp > now {
                        ttl = (claims.exp - now) as u64;
                    }
                }
                let _: Result<(), _> = conn.set_ex(&key, "1", ttl).await;
            }
        }
    }

    Ok((clear_auth_cookies(), StatusCode::OK))
}

#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "Get current user info", body = UserResponse)
    )
)]
pub async fn get_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<UserResponse>> {
    tracing::debug!("Fetching profile for user: {}", user_id);
    let user = state
        .core()
        .user_repo()
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // Fetch system-level roles (platform_admin, support, etc.)
    let system_roles = state
        .core()
        .user_repo()
        .get_system_roles_for_user(user_id)
        .await?;

    Ok(Json(UserResponse {
        id: user.id(),
        email: user.email().to_string(),
        full_name: user.full_name().map(String::from),
        avatar_url: user.avatar_url().map(String::from),
        bio: user.bio().map(String::from),
        preferred_language: user.preferred_language().map(String::from),
        created_at: user.created_at(),
        system_roles,
    }))
}

/// GET /me/permissions
/// Returns the calling user's effective permissions and roles for the current workspace.
#[utoipa::path(
    get,
    path = "/me/permissions",
    responses(
        (status = 200, description = "Get calling user permissions and roles")
    )
)]
pub async fn get_my_permissions(
    ctx: crate::handlers::common::MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    let permissions: Vec<&str> = ctx
        .permissions
        .iter()
        .map(super::super::domain::permissions::Permission::to_str_permission)
        .collect();

    let roles: Vec<String> = ctx.roles.iter().map(|r| format!("{r:?}")).collect();

    Ok(Json(serde_json::json!({
        "permissions": permissions,
        "roles": roles,
        "managed_department_ids": ctx.managed_department_ids,
    })))
}

#[utoipa::path(
    patch,
    path = "/me",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Update user profile info", body = UserResponse)
    )
)]
pub async fn update_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = state
        .core()
        .user_repo()
        .update_user_profile(user_id, req.full_name, req.avatar_url, req.bio)
        .await?;

    Ok(Json(UserResponse {
        id: user.id(),
        email: user.email().to_string(),
        full_name: user.full_name().map(String::from),
        avatar_url: user.avatar_url().map(String::from),
        bio: user.bio().map(String::from),
        preferred_language: user.preferred_language().map(String::from),
        created_at: user.created_at(),
        system_roles: vec![],
    }))
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "Get all workspace users", body = Vec<UserResponse>)
    )
)]
pub async fn get_users(
    State(state): State<Arc<AppState>>,
    ctx: crate::handlers::common::MemberContext,
) -> AppResult<Json<Vec<UserResponse>>> {
    // Scope to workspace members only to prevent cross-tenant data exposure
    let users = state
        .core()
        .user_repo()
        .get_workspace_users(ctx.workspace_id)
        .await?;

    let user_responses: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id(),
            email: u.email().to_string(),
            full_name: u.full_name().map(String::from),
            avatar_url: u.avatar_url().map(String::from),
            bio: u.bio().map(String::from),
            preferred_language: u.preferred_language().map(String::from),
            created_at: u.created_at(),
            system_roles: vec![],
        })
        .collect();

    Ok(Json(user_responses))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- validate_password tests ---

    #[test]
    fn test_password_valid() {
        assert!(validate_password("secure1pass").is_ok());
        assert!(validate_password("my!secret8").is_ok());
        assert!(validate_password("12345678").is_ok());
    }

    #[test]
    fn test_password_too_short() {
        assert!(validate_password("abc1").is_err());
        assert!(validate_password("1234567").is_err());
    }

    #[test]
    fn test_password_too_long() {
        let long = "a1".repeat(65); // 130 chars > 128
        assert!(validate_password(&long).is_err());
    }

    #[test]
    fn test_password_no_digit_or_special() {
        assert!(validate_password("abcdefghi").is_err());
        assert!(validate_password("NoDigitsHere").is_err());
    }

    #[test]
    fn test_password_special_char_accepted() {
        assert!(validate_password("password!").is_ok());
        assert!(validate_password("pass@wrd").is_ok());
    }

    #[test]
    fn test_password_exactly_min_length() {
        assert!(validate_password("abcdef1!").is_ok()); // 8 chars
    }

    // --- validate_email tests ---

    #[test]
    fn test_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("a@b.co").is_ok());
        assert!(validate_email("user+tag@sub.domain.com").is_ok());
    }

    #[test]
    fn test_email_empty() {
        assert!(validate_email("").is_err());
        assert!(validate_email("   ").is_err());
    }

    #[test]
    fn test_email_no_at() {
        assert!(validate_email("userexample.com").is_err());
    }

    #[test]
    fn test_email_no_dot_in_domain() {
        assert!(validate_email("user@localhost").is_err());
    }

    #[test]
    fn test_email_empty_local_part() {
        assert!(validate_email("@example.com").is_err());
    }

    #[test]
    fn test_email_domain_starts_with_dot() {
        assert!(validate_email("user@.example.com").is_err());
    }

    #[test]
    fn test_email_domain_ends_with_dot() {
        assert!(validate_email("user@example.com.").is_err());
    }

    #[test]
    fn test_email_consecutive_dots() {
        assert!(validate_email("user@example..com").is_err());
    }

    #[test]
    fn test_email_too_long_local() {
        let local = "a".repeat(65);
        assert!(validate_email(&format!("{local}@example.com")).is_err());
    }

    #[test]
    fn test_email_too_long_total() {
        let long_email = format!("{}@{}.com", "a".repeat(64), "b".repeat(200));
        assert!(validate_email(&long_email).is_err());
    }

    // --- cookie tests ---

    #[test]
    fn test_create_auth_cookies_contains_secure_and_samesite_none() {
        // Temporarily clear COOKIE_DOMAIN if it was set
        std::env::remove_var("COOKIE_DOMAIN");

        let headers = create_auth_cookies("my_access", "my_refresh");

        let mut found_access = false;
        let mut found_refresh = false;

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            if cookie_str.starts_with("access_token=") {
                assert!(cookie_str.contains("SameSite=None"));
                assert!(cookie_str.contains("Secure"));
                assert!(cookie_str.contains("HttpOnly"));
                found_access = true;
            } else if cookie_str.starts_with("refresh_token=") {
                assert!(cookie_str.contains("SameSite=None"));
                assert!(cookie_str.contains("Secure"));
                assert!(cookie_str.contains("HttpOnly"));
                found_refresh = true;
            }
        }

        assert!(found_access, "access_token cookie not found");
        assert!(found_refresh, "refresh_token cookie not found");
    }

    #[test]
    fn test_clear_auth_cookies_contains_secure_and_samesite_none() {
        std::env::remove_var("COOKIE_DOMAIN");

        let headers = clear_auth_cookies();

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            assert!(cookie_str.contains("SameSite=None"));
            assert!(cookie_str.contains("Secure"));
            assert!(cookie_str.contains("Max-Age=0"));
        }
    }

    #[test]
    fn test_create_auth_cookies_with_domain() {
        std::env::set_var("COOKIE_DOMAIN", ".vlee.dev");

        let headers = create_auth_cookies("tok_access", "tok_refresh");

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            assert!(
                cookie_str.contains("Domain=.vlee.dev"),
                "Cookie missing Domain attribute: {cookie_str}"
            );
        }

        // Cleanup
        std::env::remove_var("COOKIE_DOMAIN");
    }

    #[test]
    fn test_create_auth_cookies_without_domain_env() {
        std::env::remove_var("COOKIE_DOMAIN");

        let headers = create_auth_cookies("tok_access", "tok_refresh");

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            assert!(
                !cookie_str.contains("Domain="),
                "Cookie should NOT have Domain when COOKIE_DOMAIN is unset: {cookie_str}"
            );
        }
    }

    #[test]
    fn test_clear_auth_cookies_with_domain() {
        std::env::set_var("COOKIE_DOMAIN", ".vlee.dev");

        let headers = clear_auth_cookies();

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            assert!(cookie_str.contains("Domain=.vlee.dev"));
            assert!(cookie_str.contains("Max-Age=0"));
        }

        std::env::remove_var("COOKIE_DOMAIN");
    }

    #[test]
    fn test_create_auth_cookies_token_values() {
        std::env::remove_var("COOKIE_DOMAIN");

        let headers = create_auth_cookies("my_access_123", "my_refresh_456");

        let mut found_access = false;
        let mut found_refresh = false;

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            if cookie_str.starts_with("access_token=my_access_123;") {
                found_access = true;
                assert!(cookie_str.contains("Max-Age=900"));
            } else if cookie_str.starts_with("refresh_token=my_refresh_456;") {
                found_refresh = true;
                assert!(cookie_str.contains("Max-Age=2592000"));
            }
        }

        assert!(found_access, "access_token with correct value not found");
        assert!(found_refresh, "refresh_token with correct value not found");
    }

    #[test]
    fn test_cookie_domain_empty_string_ignored() {
        std::env::set_var("COOKIE_DOMAIN", "");

        let headers = create_auth_cookies("tok", "ref");

        for cookie in headers.get_all(SET_COOKIE) {
            let cookie_str = cookie.to_str().expect("valid cookie string");
            assert!(
                !cookie_str.contains("Domain="),
                "Empty COOKIE_DOMAIN should not produce Domain attr: {cookie_str}"
            );
        }

        std::env::remove_var("COOKIE_DOMAIN");
    }
}
