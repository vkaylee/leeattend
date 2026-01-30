use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::{
    AuthResponse, ChangePasswordRequest, LoginRequest, RefreshRequest, RegisterRequest,
    UpdateUserRequest, User, UserResponse,
};
use crate::repositories::user_repository;
use crate::services::auth_service::AuthService;
use crate::{AppState, UserId};

// --- Auth Handlers ---

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // 1. Check if email exists
    let existing_user = match user_repository::get_user_by_email(&state.db, &req.email).await {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error checking existing user: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    if existing_user.is_some() {
        return (StatusCode::CONFLICT, "Email already exists").into_response();
    }

    // 2. Hash password
    let password_hash = match AuthService::hash_password(&req.password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 3. Begin transaction for atomicity
    let mut tx = match state.db.begin().await {
        Ok(tx) => tx,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

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
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // 5. Create session within transaction
    let refresh_token = match AuthService::create_session_tx(&mut *tx, user.id).await {
        Ok(t) => t,
        Err(e) => {
            let _ = tx.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
        }
    };

    // 6. Commit transaction
    if let Err(e) = tx.commit().await {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
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

    // 7. Generate access token (stateless, no DB write needed)
    let access_token = match AuthService::generate_token(user.id, &state.jwt_secret, 60) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })
    .into_response()
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    let user = match user_repository::get_user_by_email(&state.db, &req.email).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response(),
        Err(e) => {
            eprintln!("Error fetching user: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    if !AuthService::verify_password(&req.password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Invalid email or password").into_response();
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

    let access_token = match AuthService::generate_token(user.id, &state.jwt_secret, 60) {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    let refresh_token = match AuthService::create_session(&state.db, user.id).await {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    Json(AuthResponse {
        access_token,
        refresh_token,
        user: user_response,
    })
    .into_response()
}

pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    match AuthService::refresh_session(&state.db, &req.refresh_token, &state.jwt_secret).await {
        Ok(res) => Json(res).into_response(),
        Err(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
    }
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    // 1. Get current user to verify old password
    let user = match user_repository::get_user_by_id(&state.db, user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => return (StatusCode::NOT_FOUND, "User not found").into_response(),
        Err(e) => {
            eprintln!("Error fetching user for password change: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    // 2. Verify old password
    if !AuthService::verify_password(&req.old_password, &user.password_hash) {
        return (StatusCode::UNAUTHORIZED, "Mật khẩu cũ không chính xác").into_response();
    }

    // 3. Hash new password
    let new_hash = match AuthService::hash_password(&req.new_password) {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    };

    // 4. Update DB
    if let Err(e) = sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await
    {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    // 5. Revoke all sessions (Force re-login)
    let _ = AuthService::revoke_all_sessions_for_user(&state.db, user_id).await;

    (
        StatusCode::OK,
        "Mật khẩu đã được thay đổi. Vui lòng đăng nhập lại.",
    )
        .into_response()
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> impl IntoResponse {
    match AuthService::revoke_session(&state.db, &req.refresh_token).await {
        Ok(_) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

pub async fn get_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> impl IntoResponse {
    tracing::debug!("Fetching profile for user: {}", user_id);
    match user_repository::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            preferred_language: user.preferred_language,
            created_at: user.created_at,
        })
        .into_response(),
        Ok(None) => {
            tracing::warn!("User {} not found in database", user_id);
            (StatusCode::NOT_FOUND, "User not found".to_string()).into_response()
        }
        Err(e) => {
            tracing::error!("Database error fetching user {}: {}", user_id, e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn update_me(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, User>(
        "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio) WHERE id = $4 RETURNING *"
    )
    .bind(req.full_name)
    .bind(req.avatar_url)
    .bind(req.bio)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            preferred_language: user.preferred_language,
            created_at: user.created_at,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_user_profile(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
) -> impl IntoResponse {
    tracing::debug!("Fetching profile for user: {}", user_id);
    match user_repository::get_user_by_id(&state.db, user_id).await {
        Ok(Some(user)) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            preferred_language: user.preferred_language,
            created_at: user.created_at,
        })
        .into_response(),
        Ok(None) => {
            tracing::warn!("User {} not found in database", user_id);
            (StatusCode::NOT_FOUND, "User not found".to_string()).into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_users(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await
    {
        Ok(users) => Json(users).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn update_user_profile(
    State(state): State<Arc<AppState>>,
    UserId(user_id): UserId,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, User>(
        "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio), preferred_language = COALESCE($4, preferred_language) WHERE id = $5 RETURNING *"
    )
    .bind(req.full_name)
    .bind(req.avatar_url)
    .bind(req.bio)
    .bind(req.preferred_language)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => Json(UserResponse {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            avatar_url: user.avatar_url,
            bio: user.bio,
            preferred_language: user.preferred_language,
            created_at: user.created_at,
        })
        .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[cfg(debug_assertions)]
pub async fn get_dev_super_user(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    use serde_json::json;
    tracing::info!("Dev fast login requested");

    // 1. Thử lấy user admin mặc định (từ seed)
    let user_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users WHERE email = 'admin@leeattend.com' LIMIT 1",
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu không thấy admin@, lấy đại bất kỳ user nào
            match sqlx::query_scalar::<_, Uuid>("SELECT id FROM users LIMIT 1")
                .fetch_one(&state.db)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    tracing::error!("Dev login failed: No users found in DB: {}", e);
                    return (
                        StatusCode::NOT_FOUND,
                        "No users found in database. Run migrations or seed your DB.",
                    )
                        .into_response();
                }
            }
        }
    };

    // 2. Lấy workspace đầu tiên mà user này là thành viên
    let workspace_id: Uuid = match sqlx::query_scalar::<_, Uuid>(
        "SELECT workspace_id FROM workspace_members WHERE user_id = $1 LIMIT 1",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(id) => id,
        Err(_) => {
            // Nếu user đầu tiên không có workspace, thử tìm bất kỳ workspace nào
            match sqlx::query_scalar::<_, Uuid>(
                "SELECT workspace_id FROM workspace_members LIMIT 1",
            )
            .fetch_one(&state.db)
            .await
            {
                Ok(ws_id) => ws_id,
                Err(e) => {
                    tracing::error!("Dev login failed: No workspace memberships found: {}", e);
                    return (
                        StatusCode::NOT_FOUND,
                        "No workspace memberships found. Please invite user to a workspace.",
                    )
                        .into_response();
                }
            }
        }
    };

    tracing::info!(
        "Dev login successful: user={}, workspace={}",
        user_id,
        workspace_id
    );

    // Sinh JWT cho dev mode
    let access_token =
        AuthService::generate_token(user_id, &state.jwt_secret, 60 * 24 * 7).unwrap_or_default();

    Json(json!({
        "user_id": user_id,
        "workspace_id": workspace_id,
        "access_token": access_token,
        "refresh_token": "dev_refresh_token"
    }))
    .into_response()
}
