use crate::errors::AppError;
use crate::models::system_db_cluster::{NewSystemDbCluster, SystemDbCluster};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait SystemDbClusterRepository: Send + Sync {
    /// Gets the active cluster with the fewest active tenants
    async fn get_least_loaded_cluster(&self) -> Result<Option<SystemDbCluster>, AppError>;

    /// Increments the active_tenants count for a specific cluster by 1
    async fn increment_active_tenants(&self, cluster_id: Uuid) -> Result<(), AppError>;

    /// Creates a new cluster record
    async fn create_cluster(
        &self,
        cluster: &NewSystemDbCluster,
    ) -> Result<SystemDbCluster, AppError>;

    /// Lists all database clusters
    async fn list_clusters(&self) -> Result<Vec<SystemDbCluster>, AppError>;

    /// Deletes a database cluster by ID
    async fn delete_cluster(&self, cluster_id: Uuid) -> Result<(), AppError>;

    /// Updates the active status of a cluster
    async fn update_cluster_status(
        &self,
        cluster_id: Uuid,
        is_active: bool,
    ) -> Result<SystemDbCluster, AppError>;
}
