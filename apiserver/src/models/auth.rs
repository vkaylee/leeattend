use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub preferred_language: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_user_debug_masks_pii() {
        let user = User {
            id: Uuid::new_v4(),
            email: "john.doe@example.com".to_string(),
            password_hash: "secret_hash".to_string(),
            created_at: Utc::now(),
            full_name: Some("John Doe".to_string()),
            avatar_url: None,
            bio: None,
            preferred_language: Some("en".to_string()),
        };

        let debug_str = format!("{user:?}");
        assert!(!debug_str.contains("john.doe@example.com"));
        assert!(
            debug_str.contains("jo***@example.com")
                || debug_str.contains("j***e@example.com")
                || debug_str.contains("***@example.com")
        );
        assert!(!debug_str.contains("secret_hash"));
        assert!(debug_str.contains("John Doe"));
    }

    #[test]
    fn test_login_request_debug_masks_pii() {
        let req = LoginRequest {
            email: "jane.doe@example.com".to_string(),
            password: "super_secret_password".to_string(),
        };

        let debug_str = format!("{req:?}");
        assert!(!debug_str.contains("jane.doe@example.com"));
        assert!(!debug_str.contains("super_secret_password"));
    }

    #[test]
    fn test_register_request_debug_masks_pii() {
        let req = RegisterRequest {
            email: "bob@example.com".to_string(),
            password: "my_password123".to_string(),
            full_name: "Bob Smith".to_string(),
        };

        let debug_str = format!("{req:?}");
        assert!(!debug_str.contains("bob@example.com"));
        assert!(!debug_str.contains("my_password123"));
        assert!(debug_str.contains("Bob Smith"));
    }

    #[test]
    fn test_change_password_request_debug_masks_pii() {
        let req = ChangePasswordRequest {
            old_password: "old_password_123".to_string(),
            new_password: "new_password_456".to_string(),
        };

        let debug_str = format!("{req:?}");
        assert!(!debug_str.contains("old_password_123"));
        assert!(!debug_str.contains("new_password_456"));
    }
}
