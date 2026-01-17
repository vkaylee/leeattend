use crate::models::{AuthResponse, User, UserResponse};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: i64,
    pub iat: i64,
}

pub struct AuthService;

impl AuthService {
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

    pub async fn create_session(db: &PgPool, user_id: Uuid) -> Result<String, String> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::days(7);

        sqlx::query("INSERT INTO sessions (user_id, token, expires_at) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(&token)
            .bind(expires_at)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;

        Ok(token)
    }

    pub async fn refresh_session(
        db: &PgPool,
        refresh_token: &str,
        secret: &str,
    ) -> Result<AuthResponse, String> {
        let session: (Uuid, Uuid) = sqlx::query_as("SELECT id, user_id FROM sessions WHERE token = $1 AND expires_at > NOW() AND revoked_at IS NULL")
            .bind(refresh_token)
            .fetch_one(db)
            .await
            .map_err(|_| "Invalid or expired refresh token".to_string())?;

        let user = sqlx::query_as::<_, User>("SELECT id, email, password_hash, created_at, full_name, avatar_url, bio, preferred_language FROM users WHERE id = $1")
            .bind(session.1)
            .fetch_one(db)
            .await
            .map_err(|e| e.to_string())?;

        let access_token = Self::generate_token(user.id, secret, 60)?; // 1 hour access

        // Rotate refresh token
        let new_refresh_token = Uuid::new_v4().to_string();
        sqlx::query("UPDATE sessions SET token = $1 WHERE id = $2")
            .bind(&new_refresh_token)
            .bind(session.0)
            .execute(db)
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
            },
        })
    }

    pub async fn revoke_session(db: &PgPool, refresh_token: &str) -> Result<(), String> {
        sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE token = $1")
            .bind(refresh_token)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn revoke_all_sessions_for_user(db: &PgPool, user_id: Uuid) -> Result<(), String> {
        sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE user_id = $1")
            .bind(user_id)
            .execute(db)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn hash_password(password: &str) -> Result<String, String> {
        hash(password, DEFAULT_COST).map_err(|e| e.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }
}
