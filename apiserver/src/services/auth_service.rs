use crate::errors::{AppError, AppResult};
use crate::utils::i18n::I18nKey;
use crate::models::{AuthResponse, UserResponse};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,                // user_id
    pub workspace_id: Option<Uuid>, // per access-control.md §3
    pub exp: i64,
    pub iat: i64,
}

use crate::queue::RedisPool;
use crate::repositories::session_repository::SessionRepository;
use crate::repositories::user_repository::UserRepository;
use std::sync::Arc;

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
    redis_pool: Option<RedisPool>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
        redis_pool: Option<RedisPool>,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
            redis_pool,
        }
    }

    // --- Pure functions (no pool needed) — kept as associated functions ---

    pub fn generate_token(user_id: Uuid, secret: &str, duration_mins: i64) -> AppResult<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(duration_mins))
            .ok_or_else(|| AppError::auth(I18nKey::OverflowCalculation))?
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            workspace_id: None,
            exp: expiration,
            iat: Utc::now().timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| AppError::auth_raw(format!("Token generation failed: {e}")))
    }

    pub fn verify_token_claims(token: &str, secret: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| AppError::auth_raw(format!("Invalid token: {e}")))?;

        Ok(token_data.claims)
    }

    pub fn verify_token(token: &str, secret: &str) -> AppResult<Uuid> {
        let claims = Self::verify_token_claims(token, secret)?;
        Uuid::parse_str(&claims.sub)
            .map_err(|e| AppError::auth_raw(format!("Invalid token subject: {e}")))
    }

    /// Hash a password using argon2id (per access-control.md §3).
    pub fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default(); // argon2id v19, standard OWASP params
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))
    }

    /// Verify a password against any supported hash format.
    /// Returns `(is_valid, needs_rehash)`.
    /// - argon2id hashes: verified natively, `needs_rehash = false`.
    /// - bcrypt hashes (legacy): verified via bcrypt, `needs_rehash = true` if valid.
    pub fn verify_password(password: &str, hash: &str) -> (bool, bool) {
        // Try argon2id first (new format starts with "$argon2")
        if hash.starts_with("$argon2") {
            let valid = PasswordHash::new(hash)
                .ok()
                .map(|parsed| {
                    Argon2::default()
                        .verify_password(password.as_bytes(), &parsed)
                        .is_ok()
                })
                .unwrap_or(false);
            return (valid, false);
        }
        // Fallback: bcrypt (legacy — starts with "$2b$" or "$2a$")
        let valid = bcrypt::verify(password, hash).unwrap_or(false);
        (valid, valid) // If valid bcrypt, needs rehash to argon2id
    }

    // --- DB methods (use self.pool) ---

    #[tracing::instrument(skip_all, err)]
    pub async fn create_session(&self, user_id: Uuid) -> AppResult<String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(7);

        self.session_repo
            .create_session(user_id, &token, expires_at)
            .await?;

        Ok(token)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn refresh_session(
        &self,
        refresh_token: &str,
        secret: &str,
    ) -> AppResult<AuthResponse> {
        let Some(session) = self
            .session_repo
            .get_session_by_token(refresh_token)
            .await?
        else {
            if let Some(redis_pool) = &self.redis_pool {
                let key = format!("jwt:rotated:{refresh_token}");
                let mut conn = redis_pool.conn();
                use redis::AsyncCommands;
                if let Ok(Some(user_id_str)) = conn.get::<_, Option<String>>(&key).await {
                    if let Ok(user_id) = Uuid::parse_str(&user_id_str) {
                        tracing::error!(
                            user_id = %user_id,
                            "CRITICAL: Refresh token reuse detected (potential session theft)! Revoking all sessions for user."
                        );
                        let _ = self.revoke_all_sessions_for_user(user_id).await;
                        return Err(AppError::auth_raw(
                            "Token theft detected. All sessions revoked for security.",
                        ));
                    }
                }
            }
            return Err(AppError::auth(I18nKey::InvalidToken));
        };

        let user = self
            .user_repo
            .get_user_by_id(session.user_id)
            .await?
            .ok_or_else(|| AppError::not_found(I18nKey::UserNotFound))?;

        let access_token = Self::generate_token(user.id(), secret, 15)?; // 15 minutes access (per access-control.md §3)

        // Store the old refresh token in Redis for reuse detection before rotating
        if let Some(redis_pool) = &self.redis_pool {
            let key = format!("jwt:rotated:{refresh_token}");
            let mut conn = redis_pool.conn();
            use redis::AsyncCommands;
            let _: Result<(), _> = conn
                .set_ex(&key, user.id().to_string(), 7 * 24 * 3600)
                .await;
        }

        // Rotate refresh token
        let new_refresh_token = Uuid::new_v4().to_string();
        self.session_repo
            .rotate_refresh_token(session.id, &new_refresh_token)
            .await?;

        // Fetch system-level roles for this user
        let system_roles = self.user_repo.get_system_roles_for_user(user.id()).await?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            user: UserResponse {
                id: user.id(),
                email: user.email().to_string(),
                full_name: user.full_name().map(String::from),
                avatar_url: user.avatar_url().map(String::from),
                bio: user.bio().map(String::from),
                preferred_language: user.preferred_language().map(String::from),
                created_at: user.created_at(),
                system_roles,
            },
        })
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn revoke_session(&self, refresh_token: &str) -> AppResult<()> {
        self.session_repo.revoke_session(refresh_token).await
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn revoke_all_sessions_for_user(&self, user_id: Uuid) -> AppResult<()> {
        self.session_repo
            .revoke_all_sessions_for_user(user_id)
            .await?;

        if let Some(redis_pool) = &self.redis_pool {
            let key = format!("user:revoked_at:{user_id}");
            let now_ts = Utc::now().timestamp().to_string();
            let mut conn = redis_pool.conn();
            use redis::AsyncCommands;
            let _: () = conn
                .set_ex(&key, now_ts, 900)
                .await
                .map_err(|e| AppError::Redis(format!("Failed to set user revocation: {e}")))?;
        }

        Ok(())
    }
}
