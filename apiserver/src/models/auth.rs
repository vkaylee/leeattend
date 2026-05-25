use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("email", &crate::utils::masking::mask_email(&self.email))
            .field("password_hash", &"***")
            .field("created_at", &self.created_at)
            .field("full_name", &self.full_name)
            .field("avatar_url", &self.avatar_url)
            .field("bio", &self.bio)
            .field("preferred_language", &self.preferred_language)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl std::fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginRequest")
            .field("email", &crate::utils::masking::mask_email(&self.email))
            .field("password", &"***")
            .finish()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
}

impl std::fmt::Debug for RegisterRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisterRequest")
            .field("email", &crate::utils::masking::mask_email(&self.email))
            .field("password", &"***")
            .field("full_name", &self.full_name)
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub system_roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

impl std::fmt::Debug for ChangePasswordRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangePasswordRequest")
            .field("old_password", &"***")
            .field("new_password", &"***")
            .finish()
    }
}
