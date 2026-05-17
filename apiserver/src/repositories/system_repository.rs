use crate::errors::AppError;
use async_trait::async_trait;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait SystemRepository: Send + Sync {
    async fn execute_partition_query(&self, query: &str) -> Result<(), AppError>;
}
