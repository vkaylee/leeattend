use axum::{
    extract::State,
    http::StatusCode,
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

// --- Auth Handlers ---

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    // 1. Validation
    if req.full_name.trim().is_empty() {
        return Err(AppError::Validation("Full name is required".to_string()));
    }
    validate_email(&req.email)?;
    validate_password(&req.password)?;

    // 2. Check if email exists
    let existing_user = user_repository::get_user_by_email(state.db.api(), &req.email).await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Email already exists".to_string()));
    }

    // 3. Hash password
    let password_hash = AuthService::hash_password(&req.password).map_err(AppError::from)?;

    // 3. Begin transaction for atomicity
    let mut tx = state.db.api().begin().await?;

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
        AuthService::generate_token(user.id, &state.jwt_secret, 60).map_err(AppError::from)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    }))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let user = user_repository::get_user_by_email(state.db.api(), &req.email)
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
        AuthService::generate_token(user.id, &state.jwt_secret, 60).map_err(AppError::from)?;

    let refresh_token = AuthService::create_session(state.db.api(), user.id)
        .await
        .map_err(AppError::from)?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    }))
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<AuthResponse>> {
    AuthService::refresh_session(state.db.api(), &req.refresh_token, &state.jwt_secret)
        .await
        .map(Json)
        .map_err(AppError::Auth)
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    // 1. Get current user to verify old password
    let user = user_repository::get_user_by_id(state.db.api(), user_id)
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
        .execute(state.db.api())
        .await?;

    // 6. Revoke all sessions (Force re-login)
    if let Err(e) = AuthService::revoke_all_sessions_for_user(state.db.api(), user_id).await {
        tracing::error!(user_id = %user_id, error = %e, "Failed to revoke sessions after password change");
    }

    Ok((
        StatusCode::OK,
        "Password changed successfully. Please log in again.",
    ))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<StatusCode> {
    AuthService::revoke_session(state.db.api(), &req.refresh_token)
        .await
        .map(|_| StatusCode::OK)
        .map_err(AppError::from)
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> AppResult<Json<UserResponse>> {
    tracing::debug!("Fetching profile for user: {}", user_id);
    let user = user_repository::get_user_by_id(state.db.api(), user_id)
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
    .fetch_one(state.db.api())
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
    .fetch_all(state.db.api())
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
) -> AppResult<Json<serde_json::Value>> {
    use serde_json::json;
    tracing::info!("Dev fast login requested");

    // 1. Thử lấy user admin mặc định (từ seed)
    let user_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = 'admin@leeattend.com' LIMIT 1",
    )
    .fetch_one(state.db.api())
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu không thấy admin@, lấy đại bất kỳ user nào
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
                .fetch_one(state.db.api())
                .await
                .map_err(|e| AppError::NotFound(format!("No users found in database: {e}")))?
        }
    };

    // 2. Lấy workspace đầu tiên mà user này là thành viên
    let workspace_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT workspace_id FROM workspace_members WHERE user_id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(state.db.api())
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu user đầu tiên không có workspace, thử tìm bất kỳ workspace nào
            sqlx::query_scalar::<_, Uuid>("SELECT workspace_id FROM workspace_members LIMIT 1")
                .fetch_one(state.db.api())
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
    let access_token = AuthService::generate_token(user_id, &state.jwt_secret, 60 * 24 * 7)
        .map_err(|e| AppError::Internal(format!("Dev token generation failed: {e}")))?;

    Ok(Json(json!({
        "user_id": user_id,
        "workspace_id": workspace_id,
        "access_token": access_token,
        "refresh_token": "dev_refresh_token"
    })))
}
