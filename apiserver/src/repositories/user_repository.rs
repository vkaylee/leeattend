use crate::errors::AppError;
use crate::models::User;
use async_trait::async_trait;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: UserRepository — DIP abstraction
// (create_user_tx excluded — takes generic Executor)
// ---------------------------------------------------------------------------
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), AppError>;
    async fn get_user_count(&self) -> Result<i64, AppError>;
    async fn assign_platform_admin(&self, user_id: Uuid) -> Result<(), AppError>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError>;
    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), AppError>;
    async fn get_system_roles_for_user(&self, user_id: Uuid) -> Result<Vec<String>, AppError>;
    async fn update_user_profile(
        &self,
        user_id: Uuid,
        full_name: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    ) -> Result<User, AppError>;
    async fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<User>, AppError>;
    async fn get_dashboard_preferences(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<Option<crate::handlers::dashboard_preferences::DashboardPreferences>, AppError>;
    async fn update_dashboard_preferences(
        &self,
        user_id: Uuid,
        workspace_id: Uuid,
        req: crate::handlers::dashboard_preferences::UpdateDashboardPreferences,
    ) -> Result<crate::handlers::dashboard_preferences::DashboardPreferences, AppError>;
    async fn get_notifications(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<crate::models::Notification>, AppError>;
    async fn get_unread_notification_count(&self, user_id: Uuid) -> Result<i64, AppError>;
    async fn mark_notification_as_read(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<(), AppError>;
    async fn mark_all_notifications_as_read(&self, user_id: Uuid) -> Result<(), AppError>;
    async fn check_recent_notification_exists(
        &self,
        user_id: Uuid,
        notification_type: &str,
        link: &str,
        since_days: i32,
    ) -> Result<bool, AppError>;
    async fn create_notification(
        &self,
        notification: &crate::models::Notification,
    ) -> Result<(), AppError>;
}
