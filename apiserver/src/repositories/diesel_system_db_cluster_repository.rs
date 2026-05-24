use crate::diesel_pool::DieselPools;
use crate::errors::AppError;
use crate::models::system_db_cluster::{NewSystemDbCluster, SystemDbCluster};
use crate::repositories::system_db_cluster_repository::SystemDbClusterRepository;
use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

pub struct DieselSystemDbClusterRepository {
    pool: DieselPools,
}

impl DieselSystemDbClusterRepository {
    pub fn new(pool: DieselPools) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SystemDbClusterRepository for DieselSystemDbClusterRepository {
    async fn get_least_loaded_cluster(&self) -> Result<Option<SystemDbCluster>, AppError> {
        use crate::schema::system_db_clusters::dsl::*;

        let mut conn = self
            .pool
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {}", e)))?;

        let cluster = system_db_clusters
            .filter(is_active.eq(true))
            .filter(active_tenants.lt(max_tenants))
            .order(active_tenants.asc())
            .first::<SystemDbCluster>(&mut conn)
            .await
            .optional()
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(cluster)
    }

    async fn increment_active_tenants(&self, cluster_id: Uuid) -> Result<(), AppError> {
        use crate::schema::system_db_clusters::dsl::*;

        let mut conn = self
            .pool
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {}", e)))?;

        diesel::update(system_db_clusters.filter(id.eq(cluster_id)))
            .set((
                active_tenants.eq(active_tenants + 1),
                updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(())
    }

    async fn create_cluster(
        &self,
        cluster: &NewSystemDbCluster,
    ) -> Result<SystemDbCluster, AppError> {
        use crate::schema::system_db_clusters::dsl::*;

        let mut conn = self
            .pool
            .api
            .get()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get connection: {}", e)))?;

        let new_cluster = diesel::insert_into(system_db_clusters)
            .values(cluster)
            .get_result::<SystemDbCluster>(&mut conn)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?;

        Ok(new_cluster)
    }
}
