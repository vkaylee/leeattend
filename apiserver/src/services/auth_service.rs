use crate::models::{AuthResponse, UserResponse};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: i64,
    pub iat: i64,
}

use crate::repositories::session_repository::SessionRepository;
use crate::repositories::user_repository::UserRepository;
use std::sync::Arc;

pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    session_repo: Arc<dyn SessionRepository>,
}

impl AuthService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        session_repo: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            user_repo,
            session_repo,
        }
    }

    // --- Pure functions (no pool needed) — kept as associated functions ---

    #[allow(clippy::expect_used)]
    pub fn generate_token(
        user_id: Uuid,
        secret: &str,
        duration_mins: i64,
    ) -> Result<String, String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::minutes(duration_mins))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            iat: Utc::now().timestamp(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| e.to_string())
    }

    pub fn verify_token(token: &str, secret: &str) -> Result<Uuid, String> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| e.to_string())?;

        Uuid::parse_str(&token_data.claims.sub).map_err(|e| e.to_string())
    }

    pub fn hash_password(password: &str) -> Result<String, String> {
        #[cfg(debug_assertions)]
        let cost = 4;
        #[cfg(not(debug_assertions))]
        let cost = bcrypt::DEFAULT_COST;

        hash(password, cost).map_err(|e| e.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }

    // --- DB methods (use self.pool) ---

    #[tracing::instrument(skip_all, err)]
    pub async fn create_session(&self, user_id: Uuid) -> Result<String, String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(7);

        self.session_repo
            .create_session(user_id, &token, expires_at)
            .await
            .map_err(|e| e.to_string())?;

        Ok(token)
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn refresh_session(
        &self,
        refresh_token: &str,
        secret: &str,
    ) -> Result<AuthResponse, String> {
        let session = self
            .session_repo
            .get_session_by_token(refresh_token)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Invalid or expired refresh token".to_string())?;

        let user = self
            .user_repo
            .get_user_by_id(session.user_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "User not found".to_string())?;

        let access_token = Self::generate_token(user.id, secret, 60)?; // 1 hour access

        // Rotate refresh token
        let new_refresh_token = Uuid::new_v4().to_string();
        self.session_repo
            .rotate_refresh_token(session.id, &new_refresh_token)
            .await
            .map_err(|e| e.to_string())?;

        // Fetch system-level roles for this user
        let system_roles = self
            .user_repo
            .get_system_roles_for_user(user.id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh_token,
            user: UserResponse {
                id: user.id,
                email: user.email,
                full_name: user.full_name,
                avatar_url: user.avatar_url,
                bio: user.bio,
                preferred_language: user.preferred_language,
                created_at: user.created_at,
                system_roles,
            },
        })
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn revoke_session(&self, refresh_token: &str) -> Result<(), String> {
        self.session_repo
            .revoke_session(refresh_token)
            .await
            .map_err(|e| e.to_string())
    }

    #[tracing::instrument(skip_all, err)]
    pub async fn revoke_all_sessions_for_user(&self, user_id: Uuid) -> Result<(), String> {
        self.session_repo
            .revoke_all_sessions_for_user(user_id)
            .await
            .map_err(|e| e.to_string())
    }
}
