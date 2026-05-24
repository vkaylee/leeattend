use async_trait::async_trait;
use diesel_async::RunQueryDsl;

use crate::errors::AppError;
use crate::repositories::system_repository::SystemRepository;

pub struct DieselSystemRepository {
    pub diesel_pools: crate::diesel_pool::DieselPools,
}

impl DieselSystemRepository {
    pub fn new(diesel_pools: crate::diesel_pool::DieselPools) -> Self {
        Self { diesel_pools }
    }
}

#[async_trait]
impl SystemRepository for DieselSystemRepository {
    async fn execute_partition_query(&self, query: &str) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .api
                .get()
                .await
                .map_err(|e| AppError::Internal(format!("db conn error: {e}")))?,
        );

        crate::system_crud_query!(&mut conn, query)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("failed to execute query: {e}")))?;

        Ok(())
    }

    async fn execute_tenant_partition_query(
        &self,
        tenant_id: uuid::Uuid,
        query: &str,
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&tenant_id)
                .await
                .map_err(|e| AppError::Internal(format!("db conn error: {e}")))?,
        );

        crate::crud_sql_query!(&mut conn, query)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("failed to execute query: {e}")))?;

        Ok(())
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
        let _repo = DieselSystemRepository::new(pools);
        // DI connection test passes if it compiles
    }
}
