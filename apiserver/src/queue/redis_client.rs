use redis::{aio::ConnectionManager, Client, RedisError};

/// Clone-friendly Redis connection pool wrapper using ConnectionManager
/// for automatic reconnection.
#[derive(Clone)]
pub struct RedisPool {
    conn_manager: ConnectionManager,
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
        let conn_manager = ConnectionManager::new(client).await?;
        Ok(Self { conn_manager })
    }

    /// Get a clone of the connection manager for executing commands.
    /// ConnectionManager handles automatic reconnection on failures.
    pub fn conn(&self) -> ConnectionManager {
        self.conn_manager.clone()
    }

    /// Check if Redis is reachable by sending a PING command.
    pub async fn health_check(&self) -> Result<(), RedisError> {
        let mut conn = self.conn_manager.clone();
        redis::cmd("PING").query_async::<String>(&mut conn).await?;
        Ok(())
    }
}
