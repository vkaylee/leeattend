use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Json},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    AuthResponse, ChangePasswordRequest, LoginRequest, RefreshRequest, RegisterRequest,
    UpdateUserRequest, User, UserResponse,
};
use crate::repositories::user_repository;
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
        _ => "".to_string(),
    };
    
    // access_token cookie
    let access_cookie = format!(
        "access_token={}; {}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=3600",
        access_token, domain_attr
    );
    // refresh_token cookie
    let refresh_cookie = format!(
        "refresh_token={}; {}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=2592000",
        refresh_token, domain_attr
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
        _ => "".to_string(),
    };
    
    let access_cookie = format!(
        "access_token=; {}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0",
        domain_attr
    );
    let refresh_cookie = format!(
        "refresh_token=; {}HttpOnly; SameSite=None; Secure; Path=/; Max-Age=0",
        domain_attr
    );

    if let Ok(val) = HeaderValue::from_str(&access_cookie) {
        headers.append(SET_COOKIE, val);
    }
    if let Ok(val) = HeaderValue::from_str(&refresh_cookie) {
        headers.append(SET_COOKIE, val);
    }
    
    headers
}

// --- Auth Handlers ---

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

    // 2. Check if email exists
    let existing_user = state.core.user_repo.get_user_by_email(&req.email).await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // 3. Hash password
    let password_hash = AuthService::hash_password(&req.password).map_err(AppError::from)?;

    // 3. Begin transaction for atomicity
    let mut tx = state.infra.db.api().begin().await?;

    // 4. Create User within transaction
    let user_id = Uuid::new_v4();
    let created_at = Utc::now();
    let user = User {
        id: user_id,
        email: req.email.clone(),
        created_at,
        full_name: Some(req.full_name.clone()),
        avatar_url: None,
        bio: None,
        preferred_language: None,
        password_hash,
    };

    if let Err(e) = user_repository::create_user_tx(&mut *tx, &user).await {
        let _ = tx.rollback().await;
        return Err(e.into());
    }

    // 5. Create session within transaction
    let refresh_token = match AuthService::create_session_tx(&mut *tx, user.id).await {
        Ok(t) => t,
        Err(e) => {
            let _ = tx.rollback().await;
            return Err(AppError::from(e));
        }
    };

    // 6. Commit transaction
    tx.commit().await?;

    // Convert to UserResponse for the client
    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        preferred_language: user.preferred_language,
        created_at: user.created_at,
    };

    // 7. Generate access token (stateless, no DB write needed)
    let access_token =
        AuthService::generate_token(user.id, &state.core.jwt_secret, 60).map_err(AppError::from)?;

    let headers = create_auth_cookies(&access_token, &refresh_token);

    Ok((headers, Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let user = state.core.user_repo.get_user_by_email(&req.email)
        .await?
        .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

    if !AuthService::verify_password(&req.password, &user.password_hash) {
        return Err(AppError::Auth("Invalid email or password".to_string()));
    }

    // Convert to UserResponse for the client
    let user_response = UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        preferred_language: user.preferred_language,
        created_at: user.created_at,
    };

    let access_token =
        AuthService::generate_token(user.id, &state.core.jwt_secret, 60).map_err(AppError::from)?;

    let refresh_token = state.core.auth_service.create_session(user.id)
        .await
        .map_err(AppError::from)?;

    let headers = create_auth_cookies(&access_token, &refresh_token);

    Ok((headers, Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })))
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req_opt: Option<Json<RefreshRequest>>,
) -> AppResult<impl IntoResponse> {
    // Try to get refresh_token from body, fallback to cookie
    let mut token_to_refresh = None;
    
    if let Some(Json(req)) = req_opt {
        token_to_refresh = Some(req.refresh_token.clone());
    }
    
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

    let token = token_to_refresh.ok_or_else(|| AppError::Auth("No refresh token provided".to_string()))?;

    let auth_response = state.core.auth_service.refresh_session(&token, &state.core.jwt_secret)
        .await
        .map_err(AppError::Auth)?;

    let resp_headers = create_auth_cookies(&auth_response.access_token, &auth_response.refresh_token);
    
    Ok((resp_headers, Json(auth_response)))
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    // 1. Get current user to verify old password
    let user = state.core.user_repo.get_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // 2. Verify old password
    if !AuthService::verify_password(&req.old_password, &user.password_hash) {
        return Err(AppError::Auth("Current password is incorrect".to_string()));
    }

    // 3. Validate new password strength
    validate_password(&req.new_password)?;

    // 4. Hash new password
    let new_hash = AuthService::hash_password(&req.new_password).map_err(AppError::from)?;

    // 5. Update DB
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(state.infra.db.api())
        .await?;

    // 6. Revoke all sessions (Force re-login)
    if let Err(e) = state.core.auth_service.revoke_all_sessions_for_user(user_id).await {
        tracing::error!(user_id = %user_id, error = %e, "Failed to revoke sessions after password change");
    }

    Ok((
        StatusCode::OK,
        "Password changed successfully. Please log in again.",
    ))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    req_opt: Option<Json<RefreshRequest>>,
) -> AppResult<impl IntoResponse> {
    // Try to get refresh_token from body, fallback to cookie
    let mut token_to_revoke = None;
    
    if let Some(Json(req)) = req_opt {
        token_to_revoke = Some(req.refresh_token.clone());
    }
    
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
        let _ = state.core.auth_service.revoke_session(&token).await;
    }

    Ok((clear_auth_cookies(), StatusCode::OK))
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<UserResponse>> {
    tracing::debug!("Fetching profile for user: {}", user_id);
    let user = state.core.user_repo.get_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        preferred_language: user.preferred_language,
        created_at: user.created_at,
    }))
}

/// GET /me/permissions
/// Returns the calling user's effective permissions and roles for the current workspace.
pub async fn get_my_permissions(
    ctx: crate::handlers::common::MemberContext,
) -> AppResult<Json<serde_json::Value>> {
    let permissions: Vec<&str> = ctx
        .permissions
        .iter()
        .map(|p| p.to_str_permission())
        .collect();

    let roles: Vec<String> = ctx.roles.iter().map(|r| format!("{r:?}")).collect();

    Ok(Json(serde_json::json!({
        "permissions": permissions,
        "roles": roles,
        "managed_department_ids": ctx.managed_department_ids,
    })))
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    let user = sqlx::query_as::<_, User>(
        "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio) WHERE id = $4 RETURNING *"
    )
    .bind(req.full_name)
    .bind(req.avatar_url)
    .bind(req.bio)
    .bind(user_id)
    .fetch_one(state.infra.db.api())
    .await?;

    Ok(Json(UserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        avatar_url: user.avatar_url,
        bio: user.bio,
        preferred_language: user.preferred_language,
        created_at: user.created_at,
    }))
}

pub async fn get_users(
    State(state): State<Arc<AppState>>,
    ctx: crate::handlers::common::MemberContext,
) -> AppResult<Json<Vec<UserResponse>>> {
    // Scope to workspace members only to prevent cross-tenant data exposure
    let users = sqlx::query_as::<_, User>(
        "SELECT u.* FROM users u \
         JOIN workspace_members wm ON u.id = wm.user_id \
         WHERE wm.workspace_id = $1 \
         ORDER BY u.created_at DESC",
    )
    .bind(ctx.workspace_id)
    .fetch_all(state.infra.db.api())
    .await?;

    let user_responses: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            avatar_url: u.avatar_url,
            bio: u.bio,
            preferred_language: u.preferred_language,
            created_at: u.created_at,
        })
        .collect();

    Ok(Json(user_responses))
}


#[cfg(debug_assertions)]
pub async fn get_dev_super_user(
    State(state): State<Arc<AppState>>,
) -> AppResult<impl IntoResponse> {
    use serde_json::json;
    tracing::info!("Dev fast login requested");

    // 1. Thử lấy user admin mặc định (từ seed)
    let user_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = 'admin@leeattend.com' LIMIT 1",
    )
    .fetch_one(state.infra.db.api())
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu không thấy admin@, lấy đại bất kỳ user nào
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
                .fetch_one(state.infra.db.api())
                .await
                .map_err(|e| AppError::NotFound(format!("No users found in database: {e}")))?
        }
    };

    // 2. Lấy workspace đầu tiên mà user này là thành viên
    let workspace_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT workspace_id FROM workspace_members WHERE user_id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(state.infra.db.api())
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu user đầu tiên không có workspace, thử tìm bất kỳ workspace nào
            sqlx::query_scalar::<_, Uuid>("SELECT workspace_id FROM workspace_members LIMIT 1")
                .fetch_one(state.infra.db.api())
                .await
                .map_err(|e| AppError::NotFound(format!("No workspace memberships found: {e}")))?
        }
    };

    tracing::info!(
        "Dev login successful: user={}, workspace={}",
        user_id,
        workspace_id
    );

    // Generate JWT for dev mode
    let access_token = AuthService::generate_token(user_id, &state.core.jwt_secret, 60 * 24 * 7)
        .map_err(|e| AppError::Internal(format!("Dev token generation failed: {e}")))?;

    let refresh_token = "dev_refresh_token".to_string();
    let headers = create_auth_cookies(&access_token, &refresh_token);

    Ok((headers, Json(json!({
        "user_id": user_id,
        "workspace_id": workspace_id,
        "access_token": access_token,
        "refresh_token": refresh_token
    }))))
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
}
