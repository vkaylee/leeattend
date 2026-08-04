use crate::errors::{AppError, AppResult};
use crate::models::{AuthResponse, UserResponse};
use crate::utils::i18n::I18nKey;
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
        .map_err(|e| {
            AppError::auth_args(
                I18nKey::InvalidToken,
                vec![("details", format!("Token generation failed: {e}"))],
            )
        })
    }

    pub fn verify_token_claims(token: &str, secret: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| {
            AppError::auth_args(
                I18nKey::InvalidToken,
                vec![("details", format!("Invalid token: {e}"))],
            )
        })?;

        Ok(token_data.claims)
    }

    pub fn verify_token(token: &str, secret: &str) -> AppResult<Uuid> {
        let claims = Self::verify_token_claims(token, secret)?;
        Uuid::parse_str(&claims.sub).map_err(|e| {
            AppError::auth_args(
                I18nKey::InvalidToken,
                vec![("details", format!("Invalid token subject: {e}"))],
            )
        })
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
        // Fallback: legacy bcrypt is disabled
        if hash.starts_with("$2b$") || hash.starts_with("$2a$") || hash.starts_with("$2y$") {
            tracing::warn!("Attempted login with disabled legacy bcrypt hash");
        }
        (false, false)
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
                        return Err(AppError::auth_args(
                            I18nKey::InvalidToken,
                            vec![(
                                "details",
                                "Token theft detected. All sessions revoked for security."
                                    .to_string(),
                            )],
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

        // Fetch all response data before mutating the refresh session. A role
        // lookup failure must not invalidate the only refresh cookie.
        let system_roles = self.user_repo.get_system_roles_for_user(user.id()).await?;

        // Store the old refresh token in Redis for reuse detection before rotating
        if let Some(redis_pool) = &self.redis_pool {
            let key = format!("jwt:rotated:{refresh_token}");
            let mut conn = redis_pool.conn();
            use redis::AsyncCommands;
            let _: Result<(), _> = conn
                .set_ex(&key, user.id().to_string(), 7 * 24 * 3600)
                .await;
        }

        // Rotate refresh token only after all fallible reads succeed.
        let new_refresh_token = Uuid::new_v4().to_string();
        self.session_repo
            .rotate_refresh_token(session.id, &new_refresh_token)
            .await?;

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
                avatar_file_id: user.avatar_file_id(),
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

    #[tracing::instrument(skip(self), err)]
    pub async fn create_password_reset_token(&self, user_id: Uuid) -> AppResult<String> {
        let token = Uuid::new_v4().to_string();
        let key = format!("pwd_reset:{token}");
        if let Some(redis_pool) = &self.redis_pool {
            let mut conn = redis_pool.conn();
            use redis::AsyncCommands;
            // set token with 15 mins (900s) TTL
            conn.set_ex::<_, _, ()>(&key, user_id.to_string(), 900)
                .await
                .map_err(|e| {
                    AppError::Redis(format!("Failed to save reset token to Redis: {e}"))
                })?;
            Ok(token)
        } else {
            Err(AppError::Redis("Redis pool not initialized".to_string()))
        }
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn verify_and_consume_reset_token(&self, token: &str) -> AppResult<Uuid> {
        let key = format!("pwd_reset:{token}");
        if let Some(redis_pool) = &self.redis_pool {
            let mut conn = redis_pool.conn();
            use redis::AsyncCommands;
            let user_id_str: Option<String> = conn.get(&key).await.map_err(|e| {
                AppError::Redis(format!("Failed to retrieve reset token from Redis: {e}"))
            })?;

            let Some(user_id_str) = user_id_str else {
                return Err(AppError::validation(I18nKey::ResetTokenInvalid));
            };

            // Delete the token immediately (single-use)
            let _: Result<(), _> = conn.del(&key).await;

            let user_id = Uuid::parse_str(&user_id_str).map_err(|e| {
                tracing::warn!("Invalid reset token UUID format: {e}");
                AppError::validation(I18nKey::ResetTokenInvalid)
            })?;
            Ok(user_id)
        } else {
            Err(AppError::Redis("Redis pool not initialized".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;
    use crate::models::diesel::core::SessionEntity;
    use crate::models::{NewUserParams, User};
    use crate::repositories::session_repository::MockSessionRepository;
    use crate::repositories::user_repository::MockUserRepository;
    use chrono::{Duration, Utc};
    use std::sync::Arc;

    const TEST_SECRET: &str = "test-secret-key-for-jwt-testing-32chars";

    // --- JWT Token Tests ---

    #[test]
    fn test_generate_and_verify_token_roundtrip() {
        let user_id = Uuid::new_v4();
        let token = AuthService::generate_token(user_id, TEST_SECRET, 60).unwrap();
        let verified = AuthService::verify_token(&token, TEST_SECRET).unwrap();
        assert_eq!(verified, user_id);
    }

    #[test]
    fn test_verify_token_invalid_signature() {
        let user_id = Uuid::new_v4();
        let token = AuthService::generate_token(user_id, TEST_SECRET, 60).unwrap();
        let result = AuthService::verify_token(&token, "wrong-secret-key-32chars-long!");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_malformed() {
        let result = AuthService::verify_token("not-a-valid-jwt", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_claims_contains_sub_and_exp() {
        let user_id = Uuid::new_v4();
        let token = AuthService::generate_token(user_id, TEST_SECRET, 60).unwrap();
        let claims = AuthService::verify_token_claims(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, user_id.to_string());
        assert!(claims.exp > claims.iat);
        assert!(claims.exp <= claims.iat + 3600); // 60 min buffer
    }

    #[test]
    fn test_generate_token_different_duration() {
        let user_id = Uuid::new_v4();
        let token = AuthService::generate_token(user_id, TEST_SECRET, 10080).unwrap(); // 7 days
        let claims = AuthService::verify_token_claims(&token, TEST_SECRET).unwrap();
        // 7 days in seconds
        assert!(claims.exp > claims.iat + 600_000); // at least ~7 days
    }

    #[test]
    fn test_verify_token_expired() {
        let user_id = Uuid::new_v4();
        // Generate a token that expired 10 minutes ago
        let token = AuthService::generate_token(user_id, TEST_SECRET, -10).unwrap();
        let result = AuthService::verify_token(&token, TEST_SECRET);
        assert!(result.is_err(), "Expired token should be rejected");
    }

    // --- Password Hashing Tests ---

    #[test]
    fn test_hash_password_produces_argon2id() {
        let hash = AuthService::hash_password("my-secure-password").unwrap();
        assert!(
            hash.starts_with("$argon2"),
            "Hash should start with $argon2"
        );
    }

    #[test]
    fn test_hash_password_different_salts() {
        let hash1 = AuthService::hash_password("same-password").unwrap();
        let hash2 = AuthService::hash_password("same-password").unwrap();
        assert_ne!(
            hash1, hash2,
            "Same password should produce different hashes (unique salt)"
        );
    }

    #[test]
    fn test_verify_password_valid() {
        let hash = AuthService::hash_password("correct-password").unwrap();
        let (valid, needs_rehash) = AuthService::verify_password("correct-password", &hash);
        assert!(valid);
        assert!(!needs_rehash, "argon2id hash should not need rehash");
    }

    #[test]
    fn test_verify_password_wrong() {
        let hash = AuthService::hash_password("correct-password").unwrap();
        let (valid, needs_rehash) = AuthService::verify_password("wrong-password", &hash);
        assert!(!valid);
        assert!(!needs_rehash);
    }

    #[test]
    fn test_verify_password_empty_password() {
        let hash = AuthService::hash_password("").unwrap();
        let (valid, _) = AuthService::verify_password("", &hash);
        assert!(valid, "Empty password should verify against its own hash");
    }

    #[test]
    fn test_verify_password_unicode() {
        let password = "mật-khẩu-🏠-secure";
        let hash = AuthService::hash_password(password).unwrap();
        let (valid, _) = AuthService::verify_password(password, &hash);
        assert!(valid, "Unicode password should verify correctly");
    }

    #[test]
    fn test_verify_password_legacy_bcrypt_disabled() {
        // Legacy bcrypt hashes should be rejected (feature disabled)
        let legacy_hash = "$2b$12$LJ3m4ys3YOlDkOmMrLJ3eO1fHkD9X1Yq";
        let (valid, needs_rehash) = AuthService::verify_password("anything", legacy_hash);
        assert!(!valid);
        assert!(!needs_rehash);
    }

    #[tokio::test]
    async fn test_refresh_does_not_rotate_when_system_role_lookup_fails() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let refresh_token = "refresh-token-for-role-error";
        let session = SessionEntity {
            id: session_id,
            user_id,
            token: refresh_token.to_string(),
            expires_at: Utc::now() + Duration::hours(1),
            created_at: Some(Utc::now()),
            revoked_at: None,
        };
        let user = User::new(NewUserParams {
            id: user_id,
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: Utc::now(),
            full_name: Some("Test User".to_string()),
            avatar_url: None,
            bio: None,
            preferred_language: Some("vi".to_string()),
            avatar_file_id: None,
        });

        let session = Arc::new(std::sync::Mutex::new(Some(session)));
        let mut session_repo = MockSessionRepository::new();
        session_repo
            .expect_get_session_by_token()
            .returning(move |_| {
                let mut guard = session.lock().map_err(|error| {
                    AppError::Internal(format!("session fixture lock poisoned: {error}"))
                })?;
                Ok(guard.take())
            });
        session_repo.expect_rotate_refresh_token().times(0);

        let user = Arc::new(std::sync::Mutex::new(Some(user)));
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_get_user_by_id().returning(move |_| {
            let mut guard = user.lock().map_err(|error| {
                AppError::Internal(format!("user fixture lock poisoned: {error}"))
            })?;
            Ok(guard.take())
        });
        user_repo
            .expect_get_system_roles_for_user()
            .returning(|_| Err(AppError::Internal("role lookup failed".to_string())));

        let service = AuthService::new(Arc::new(user_repo), Arc::new(session_repo), None);
        let result = service.refresh_session(refresh_token, TEST_SECRET).await;

        assert!(result.is_err());
    }
}
