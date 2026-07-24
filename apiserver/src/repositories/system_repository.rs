use crate::errors::AppError;
use async_trait::async_trait;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SystemRepository: Send + Sync {
    async fn execute_partition_query(&self, query: &str) -> Result<(), AppError>;
    async fn execute_tenant_partition_query(
        &self,
        tenant_id: uuid::Uuid,
        query: &str,
    ) -> Result<(), AppError>;
    async fn execute_tenant_partition_queries_batch(
        &self,
        tenant_id: uuid::Uuid,
        queries: &[String],
    ) -> Result<(), AppError>;
}
