use redis::{aio::ConnectionManager, Client, RedisError};

/// Clone-friendly Redis connection pool wrapper using ConnectionManager
/// for automatic reconnection.
#[derive(Clone)]
pub struct RedisPool {
    conn_manager: ConnectionManager,
    client: Client,
}

impl RedisPool {
    /// Create a new Redis pool from a connection URL.
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection string (e.g., "redis://localhost:6379")
    ///
    /// # Errors
    /// Returns `RedisError` if connection fails.
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;
        let conn_manager = ConnectionManager::new(client.clone()).await?;
        Ok(Self {
            conn_manager,
            client,
        })
    }

    /// Get a clone of the connection manager for executing commands.
    /// ConnectionManager handles automatic reconnection on failures.
    pub fn conn(&self) -> ConnectionManager {
        self.conn_manager.clone()
    }

    /// Get an async connection for Pub/Sub operations.
    /// Unlike ConnectionManager, this connection supports `into_pubsub()`.
    pub async fn get_async_connection(
        &self,
    ) -> Result<redis::aio::MultiplexedConnection, RedisError> {
        self.client.get_multiplexed_async_connection().await
    }

    /// Get a dedicated connection for Pub/Sub subscriptions.
    pub async fn get_pubsub_connection(&self) -> Result<redis::aio::PubSub, RedisError> {
        self.client.get_async_pubsub().await
    }

    /// Check if Redis is reachable by sending a PING command.
    pub async fn health_check(&self) -> Result<(), RedisError> {
        let mut conn = self.conn_manager.clone();
        redis::cmd("PING").query_async::<String>(&mut conn).await?;
        Ok(())
    }
}
