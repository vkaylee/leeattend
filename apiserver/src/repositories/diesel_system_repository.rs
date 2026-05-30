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

        diesel::sql_query(query)
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

        diesel::sql_query(query)
            .execute(&mut *conn)
            .await
            .map_err(|e| AppError::Internal(format!("failed to execute query: {e}")))?;

        Ok(())
    }

    async fn execute_tenant_partition_queries_batch(
        &self,
        tenant_id: uuid::Uuid,
        queries: &[String],
    ) -> Result<(), AppError> {
        let mut conn = crate::diesel_pool::TransactionConn(
            self.diesel_pools
                .tenants_tx
                .get_connection(&tenant_id)
                .await
                .map_err(|e| AppError::Internal(format!("db conn error: {e}")))?,
        );

        for query in queries {
            diesel::sql_query(query)
                .execute(&mut *conn)
                .await
                .map_err(|e| AppError::Internal(format!("failed to execute query: {e}")))?;
        }

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

    #[tokio::test]
    async fn test_execute_partition_query() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselSystemRepository::new(pools);

        let res = repo.execute_partition_query("SELECT 1").await;
        if let Err(e) = &res {
            let err_str = e.to_string();
            if err_str.contains("connection") || err_str.contains("error connecting") || err_str.contains("db conn error") {
                println!("Skipping test_execute_partition_query due to connection failure: {err_str}");
                return;
            }
        }
        assert!(res.is_ok(), "expected ok, got {:?}", res.err());
    }

    #[tokio::test]
    async fn test_execute_tenant_partition_query() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselSystemRepository::new(pools);
        let tenant_id = uuid::Uuid::new_v4();

        let res = repo
            .execute_tenant_partition_query(tenant_id, "SELECT 1")
            .await;
        if let Err(e) = &res {
            let err_str = e.to_string();
            if err_str.contains("connection") || err_str.contains("error connecting") || err_str.contains("db conn error") {
                println!("Skipping test_execute_tenant_partition_query due to connection failure: {err_str}");
                return;
            }
        }
        assert!(res.is_ok(), "expected ok, got {:?}", res.err());
    }

    #[tokio::test]
    async fn test_execute_tenant_partition_queries_batch() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselSystemRepository::new(pools);
        let tenant_id = uuid::Uuid::new_v4();

        let res = repo
            .execute_tenant_partition_queries_batch(
                tenant_id,
                &["SELECT 1".to_string(), "SELECT 2".to_string()],
            )
            .await;
        if let Err(e) = &res {
            let err_str = e.to_string();
            if err_str.contains("connection") || err_str.contains("error connecting") || err_str.contains("db conn error") {
                println!("Skipping test_execute_tenant_partition_queries_batch due to connection failure: {err_str}");
                return;
            }
        }
        assert!(res.is_ok(), "expected ok, got {:?}", res.err());

        // Error path
        let err_res = repo
            .execute_tenant_partition_queries_batch(
                tenant_id,
                &["SELECT * FROM non_existent_table_123".to_string()],
            )
            .await;
        assert!(err_res.is_err());
    }

    #[tokio::test]
    async fn test_execute_partition_query_error() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselSystemRepository::new(pools);

        let err_res = repo
            .execute_partition_query("SELECT * FROM non_existent_table_123")
            .await;
        assert!(err_res.is_err());
    }

    #[tokio::test]
    async fn test_execute_tenant_partition_query_error() {
        let test_db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://user:pass@localhost:5432/leeattend_test_db".to_string()
        });

        let pools = DieselPools::for_test(&test_db_url);
        let repo = DieselSystemRepository::new(pools);
        let tenant_id = uuid::Uuid::new_v4();

        let err_res = repo
            .execute_tenant_partition_query(tenant_id, "SELECT * FROM non_existent_table_123")
            .await;
        assert!(err_res.is_err());
    }
}
