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
pub trait UserRepository: super::PoolBacked + Send + Sync {
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
}

// ---------------------------------------------------------------------------
// SqlxUserRepository — production implementation backed by PgPool
// ---------------------------------------------------------------------------
pub struct SqlxUserRepository {
    pub pool: PgPool,
}

impl SqlxUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqlxUserRepository {
    async fn create_user(&self, user: &User) -> Result<(), AppError> {
        create_user(&self.pool, user).await.map_err(Into::into)
    }
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        get_user_by_email(&self.pool, email)
            .await
            .map_err(Into::into)
    }
    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        get_user_by_id(&self.pool, id).await.map_err(Into::into)
    }
    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
            .bind(password_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
    async fn get_system_roles_for_user(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let roles = sqlx::query_scalar::<_, String>(
            "SELECT sr.name FROM user_system_roles usr \
             JOIN system_roles sr ON sr.id = usr.role_id \
             WHERE usr.user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(roles)
    }
    async fn update_user_profile(
        &self,
        user_id: Uuid,
        full_name: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    ) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            "UPDATE users SET full_name = COALESCE($1, full_name), avatar_url = COALESCE($2, avatar_url), bio = COALESCE($3, bio) WHERE id = $4 RETURNING *",
        )
        .bind(full_name)
        .bind(avatar_url)
        .bind(bio)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }
    async fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u \
             JOIN workspace_members wm ON u.id = wm.user_id \
             WHERE wm.workspace_id = $1 \
             ORDER BY u.created_at DESC",
        )
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
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

impl super::PoolBacked for SqlxUserRepository {
    fn from_pool(pool: sqlx::PgPool) -> Self {
        Self::new(pool)
    }
    fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}
