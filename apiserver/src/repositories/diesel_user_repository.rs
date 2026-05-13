use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::core::UserEntity;
use crate::models::User;
use crate::repositories::user_repository::UserRepository;
use crate::schema::{system_roles, user_system_roles, users, workspace_members};
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DieselUserRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselUserRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl UserRepository for DieselUserRepository {
    async fn create_user(&self, user: &User) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = UserEntity {
            id: user.id,
            email: user.email.clone(),
            password_hash: user.password_hash.clone(),
            created_at: Some(user.created_at),
            full_name: user.full_name.clone(),
            avatar_url: user.avatar_url.clone(),
            bio: user.bio.clone(),
            preferred_language: user.preferred_language.clone(),
        };

        diesel::insert_into(users::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = users::table
            .filter(users::email.eq(email))
            .select(UserEntity::as_select())
            .first::<UserEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = users::table
            .find(id)
            .select(UserEntity::as_select())
            .first::<UserEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.map(Into::into))
    }

    async fn update_password_hash(
        &self,
        user_id: Uuid,
        password_hash: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(users::table.find(user_id))
            .set(users::password_hash.eq(password_hash))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_system_roles_for_user(&self, user_id: Uuid) -> Result<Vec<String>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let roles = user_system_roles::table
            .inner_join(system_roles::table.on(system_roles::id.eq(user_system_roles::role_id)))
            .filter(user_system_roles::user_id.eq(user_id))
            .select(system_roles::name)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(roles)
    }

    async fn update_user_profile(
        &self,
        user_id: Uuid,
        full_name: Option<String>,
        avatar_url: Option<String>,
        bio: Option<String>,
    ) -> Result<User, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = diesel::update(users::table.find(user_id))
            .set((
                full_name.map(|f| users::full_name.eq(f)),
                avatar_url.map(|a| users::avatar_url.eq(a)),
                bio.map(|b| users::bio.eq(b)),
            ))
            .get_result::<UserEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entity.into())
    }

    async fn get_workspace_users(&self, workspace_id: Uuid) -> Result<Vec<User>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entities = users::table
            .inner_join(workspace_members::table)
            .filter(workspace_members::workspace_id.eq(workspace_id))
            .select(users::all_columns)
            .order(users::created_at.desc())
            .load::<UserEntity>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(entities.into_iter().map(Into::into).collect())
    }
}
