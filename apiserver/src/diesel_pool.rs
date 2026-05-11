use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::time::Duration;

use crate::db_pool::DbPoolConfig;

pub type DieselPool = Pool<AsyncPgConnection>;
pub type DieselConnection = Object<AsyncPgConnection>;

#[derive(Clone)]
pub struct DieselPools {
    pub api: DieselPool,
    pub worker: DieselPool,
}

impl DieselPools {
    #[allow(clippy::unused_async, clippy::expect_used)]
    pub async fn connect(config: &DbPoolConfig) -> Self {
        tracing::info!(
            api_max = config.api_max_conn,
            worker_max = config.worker_max_conn,
            "Initializing Diesel pools"
        );

        let (api_conn_url, worker_conn_url, _) = config.resolve_pool_urls();

        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(api_conn_url);
        let api_pool = Pool::builder(api_manager)
            .max_size(config.api_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(config.api_acquire_timeout_secs)))
            .build()
            .expect("Failed to create API Diesel pool");

        let worker_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(worker_conn_url);
        let worker_pool = Pool::builder(worker_manager)
            .max_size(config.worker_max_conn as usize)
            .wait_timeout(Some(Duration::from_secs(
                config.worker_acquire_timeout_secs,
            )))
            .build()
            .expect("Failed to create Worker Diesel pool");

        tracing::info!("Diesel pools connected successfully");

        Self {
            api: api_pool,
            worker: worker_pool,
        }
    }

    #[allow(clippy::expect_used)]
    pub fn for_test(url: &str) -> Self {
        let api_manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url.to_string());
        let api_pool = Pool::builder(api_manager)
            .max_size(5)
            .build()
            .expect("Failed to create test API Diesel pool");

        let worker_manager =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(url.to_string());
        let worker_pool = Pool::builder(worker_manager)
            .max_size(5)
            .build()
            .expect("Failed to create test Worker Diesel pool");

        Self {
            api: api_pool,
            worker: worker_pool,
        }
    }
}
