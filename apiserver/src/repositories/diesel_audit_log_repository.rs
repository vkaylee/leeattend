use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    diesel_pool::DieselPools, domain::audit::AuditLog, errors::AppError,
    models::diesel::audit_log::AuditLogEntity, schema::audit_logs,
};

use super::AuditLogRepository;

pub struct DieselAuditLogRepository {
    diesel_pools: DieselPools,
}

impl DieselAuditLogRepository {
    pub fn new(diesel_pools: DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl AuditLogRepository for DieselAuditLogRepository {
    async fn create_log(&self, log: &AuditLog) -> Result<AuditLog, AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&log.workspace_id())
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

        let entity: AuditLogEntity = log.clone().into();

        let inserted: AuditLogEntity = diesel::insert_into(audit_logs::table)
            .values(&entity)
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

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
    ) -> Result<(Vec<AuditLog>, i64), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&workspace_id_val)
                .await
                .map_err(|e| AppError::Internal(format!("db: {e}")))?,
        );

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
            .get_result(&mut *conn)
            .await
            .map_err(AppError::from)?;

        let logs_result = query
            .order(audit_logs::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<AuditLogEntity>(&mut *conn)
            .await
            .map_err(AppError::from)?;

        Ok((logs_result.into_iter().map(Into::into).collect(), count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diesel_pool::DieselPools;

    #[tokio::test]
    async fn test_repo_initialization() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let _repo = DieselAuditLogRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
