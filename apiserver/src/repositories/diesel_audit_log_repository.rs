use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools, domain::audit::AuditLog, models::diesel::audit_log::AuditLogEntity,
    schema::audit_logs,
};

use super::AuditLogRepository;

pub struct DieselAuditLogRepository {
    pub sqlx_pool: PgPool,
    pub diesel_pools: DieselPools,
}

impl DieselAuditLogRepository {
    pub fn new(sqlx_pool: PgPool, diesel_pools: DieselPools) -> Self {
        Self {
            sqlx_pool,
            diesel_pools,
        }
    }
}

#[async_trait]
impl AuditLogRepository for DieselAuditLogRepository {
    async fn create_log(&self, log: &AuditLog) -> Result<AuditLog, sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let entity: AuditLogEntity = log.clone().into();

        let inserted: AuditLogEntity = diesel::insert_into(audit_logs::table)
            .values(&entity)
            .get_result(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok(inserted.into())
    }

    async fn find_by_workspace(
        &self,
        workspace_id_val: Uuid,
        limit: i64,
        offset: i64,
        action_val: Option<String>,
        start_date_val: Option<chrono::DateTime<chrono::Utc>>,
        end_date_val: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(Vec<AuditLog>, i64), sqlx::Error> {
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let mut query = audit_logs::table
            .filter(audit_logs::workspace_id.eq(workspace_id_val))
            .into_boxed();

        let mut count_query = audit_logs::table
            .filter(audit_logs::workspace_id.eq(workspace_id_val))
            .into_boxed();

        if let Some(a) = action_val {
            query = query.filter(audit_logs::action.eq(a.clone()));
            count_query = count_query.filter(audit_logs::action.eq(a));
        }

        if let Some(sd) = start_date_val {
            query = query.filter(audit_logs::created_at.ge(sd));
            count_query = count_query.filter(audit_logs::created_at.ge(sd));
        }

        if let Some(ed) = end_date_val {
            query = query.filter(audit_logs::created_at.le(ed));
            count_query = count_query.filter(audit_logs::created_at.le(ed));
        }

        let count: i64 = count_query
            .count()
            .get_result(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        let logs_result = query
            .order(audit_logs::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<AuditLogEntity>(&mut conn)
            .await
            .map_err(|e| sqlx::Error::Configuration(Box::new(e)))?;

        Ok((logs_result.into_iter().map(Into::into).collect(), count))
    }
}

impl crate::repositories::PoolBacked for DieselAuditLogRepository {
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    #[allow(clippy::panic)]
    fn from_pool(_pool: PgPool) -> Self {
        panic!("DieselAuditLogRepository requires diesel_pools to be manually injected via new()");
    }

    fn pool(&self) -> &PgPool {
        &self.sqlx_pool
    }
}
