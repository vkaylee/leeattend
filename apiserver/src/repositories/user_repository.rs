use crate::errors::AppError;
use crate::models::User;
use async_trait::async_trait;
use sqlx::{query, query_as, Executor, PgPool, Postgres};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Trait: UserRepository — DIP abstraction
// (create_user_tx excluded — takes generic Executor)
// ---------------------------------------------------------------------------
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: &User) -> Result<(), AppError>;
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
}

pub async fn create_user(pool: &PgPool, user: &User) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO users (id, email, password_hash, created_at, full_name, avatar_url, bio, preferred_language) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        user.id,
        user.email,
        user.password_hash,
        user.created_at,
        user.full_name,
        user.avatar_url,
        user.bio,
        user.preferred_language
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn create_user_tx<'e, E>(executor: E, user: &User) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query(
        "INSERT INTO users (id, email, password_hash, created_at, full_name, avatar_url, bio, preferred_language) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind(user.id)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(user.created_at)
    .bind(&user.full_name)
    .bind(&user.avatar_url)
    .bind(&user.bio)
    .bind(&user.preferred_language)
    .execute(executor)
    .await?;
    Ok(())
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    query_as!(
        User,
        "SELECT id, email, password_hash, created_at as \"created_at!\", full_name, avatar_url, bio, preferred_language FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    query_as!(
        User,
        "SELECT id, email, password_hash, created_at as \"created_at!\", full_name, avatar_url, bio, preferred_language FROM users WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn user_count_tx<'e, E>(executor: E) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(executor)
        .await
}

pub async fn assign_platform_admin_tx<'e, E>(executor: E, user_id: Uuid) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Postgres>,
{
    sqlx::query(
        "INSERT INTO user_system_roles (user_id, role_id) \
         SELECT $1, id FROM system_roles WHERE name = 'platform_admin'",
    )
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(())
}
