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
        let mut conn = self
            .diesel_pools
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("db conn error: {e}")))?;

        diesel::sql_query(query)
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("failed to execute query: {e}")))?;

        Ok(())
    }
}
