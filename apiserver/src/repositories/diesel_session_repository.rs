use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::diesel::core::SessionEntity;
use crate::repositories::session_repository::SessionRepository;
use crate::schema::sessions;
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

pub struct DieselSessionRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselSessionRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

impl crate::repositories::PoolBacked for DieselSessionRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!("DieselSessionRepository requires diesel_pools to be manually injected via new()");
    }
    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}

#[async_trait]
impl SessionRepository for DieselSessionRepository {
    async fn create_session(
        &self,
        user_id: Uuid,
        token: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let entity = SessionEntity {
            id: Uuid::new_v4(),
            user_id,
            token: token.to_string(),
            expires_at,
            created_at: Some(Utc::now()),
            revoked_at: None,
        };

        diesel::insert_into(sessions::table)
            .values(&entity)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn get_session_by_token(
        &self,
        token: &str,
    ) -> Result<Option<SessionEntity>, AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        let session = sessions::table
            .filter(sessions::token.eq(token))
            .filter(sessions::expires_at.gt(Utc::now()))
            .filter(sessions::revoked_at.is_null())
            .select(SessionEntity::as_select())
            .first::<SessionEntity>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(session)
    }

    async fn rotate_refresh_token(
        &self,
        session_id: Uuid,
        new_token: &str,
    ) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(sessions::table.find(session_id))
            .set(sessions::token.eq(new_token))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn revoke_session(&self, token: &str) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(sessions::table.filter(sessions::token.eq(token)))
            .set(sessions::revoked_at.eq(Utc::now()))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }

    async fn revoke_all_sessions_for_user(&self, user_id: Uuid) -> Result<(), AppError> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        diesel::update(sessions::table.filter(sessions::user_id.eq(user_id)))
            .set(sessions::revoked_at.eq(Utc::now()))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("db: {e}")))?;

        Ok(())
    }
}
