use sqlx::{PgPool, query, query_as};
use uuid::Uuid;
use crate::models::User;

pub async fn create_user(pool: &PgPool, user: &User) -> Result<(), sqlx::Error> {
    query!(
        "INSERT INTO users (id, email, password_hash, created_at, full_name, avatar_url, bio) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        user.id,
        user.email,
        user.password_hash,
        user.created_at,
        user.full_name,
        user.avatar_url,
        user.bio
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    query_as!(
        User,
        "SELECT id, email, password_hash, created_at as \"created_at!\", full_name, avatar_url, bio FROM users WHERE email = $1",
        email
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_user_by_id(pool: &PgPool, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    query_as!(
        User,
        "SELECT id, email, password_hash, created_at as \"created_at!\", full_name, avatar_url, bio FROM users WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}