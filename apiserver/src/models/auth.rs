use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, utoipa::ToSchema)]
pub struct User {
    id: Uuid,
    email: String,
    #[serde(skip_serializing)]
    password_hash: String,
    created_at: DateTime<Utc>,
    full_name: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
    preferred_language: Option<String>,
}

impl User {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: Uuid,
        email: String,
        password_hash: String,
        created_at: DateTime<Utc>,
        full_name: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
        preferred_language: Option<String>,
    ) -> Self {
        Self {
            id,
            email,
            password_hash,
            created_at,
            full_name,
            avatar_url,
            bio,
            preferred_language,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn full_name(&self) -> Option<&str> {
        self.full_name.as_deref()
    }

    pub fn avatar_url(&self) -> Option<&str> {
        self.avatar_url.as_deref()
    }

    pub fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }

    pub fn preferred_language(&self) -> Option<&str> {
        self.preferred_language.as_deref()
    }
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
        let user = User::new(
            Uuid::new_v4(),
            "john.doe@example.com".to_string(),
            "secret_hash".to_string(),
            Utc::now(),
            Some("John Doe".to_string()),
            None,
            None,
            Some("en".to_string()),
        );

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

    #[test]
    fn test_user_getters() {
        let id = Uuid::new_v4();
        let created_at = Utc::now();
        let user = User::new(
            id,
            "user@example.com".to_string(),
            "hashed".to_string(),
            created_at,
            Some("Full Name".to_string()),
            Some("http://avatar.url".to_string()),
            Some("My Bio".to_string()),
            Some("vi".to_string()),
        );

        assert_eq!(user.id(), id);
        assert_eq!(user.email(), "user@example.com");
        assert_eq!(user.password_hash(), "hashed");
        assert_eq!(user.created_at(), created_at);
        assert_eq!(user.full_name(), Some("Full Name"));
        assert_eq!(user.avatar_url(), Some("http://avatar.url"));
        assert_eq!(user.bio(), Some("My Bio"));
        assert_eq!(user.preferred_language(), Some("vi"));
    }

    #[test]
    fn test_user_response_default_roles() {
        let json_data = r#"{
            "id": "a0a0a0a0-b0b0-c0c0-d0d0-e0e0e0e0e0e0",
            "email": "test@example.com",
            "full_name": "Test User",
            "avatar_url": null,
            "bio": null,
            "preferred_language": "en",
            "created_at": "2026-06-06T12:00:00Z"
        }"#;

        let response: UserResponse = serde_json::from_str(json_data).unwrap();
        assert_eq!(response.email, "test@example.com");
        assert!(response.system_roles.is_empty()); // default system_roles as empty vector
    }
}
