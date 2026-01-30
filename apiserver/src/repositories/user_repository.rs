use crate::models::User;
use sqlx::{query, query_as, Executor, PgPool, Postgres};
use uuid::Uuid;

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
