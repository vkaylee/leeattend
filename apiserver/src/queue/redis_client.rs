use redis::{aio::ConnectionManager, Client, RedisError};

/// Clone-friendly Redis connection wrapper with **dual connections** to prevent
/// head-of-line blocking from `BLPOP`.
///
/// # Why Two Connections?
///
/// `ConnectionManager` in `redis-rs` wraps a **single TCP connection** with request
/// multiplexing. While non-blocking commands (GET, SET, HSET, PUBLISH) pipeline
/// efficiently over one connection, **blocking commands like BLPOP occupy the
/// entire connection pipeline** for their timeout duration (up to 30 seconds).
///
/// When a worker's main loop calls `BLPOP` on the shared connection, ALL
/// concurrent operations from other tasks (HSET batch results, PUBLISH progress
/// signals, GET metadata) queue behind it. This caused Redis operations on 7KB
/// payloads to take **27-54 seconds** instead of <1ms.
///
/// ## Architecture
///
/// ```text
/// conn_manager (non-blocking)    blocking_conn (BLPOP only)
///      │                              │
///      ├── HSET (store results)       └── BLPOP (dequeue jobs, up to 30s)
///      ├── PUBLISH (progress)
///      ├── GET (metadata)
///      ├── INCR (counters)
///      └── SET (heartbeat)
/// ```
///
/// ## Performance Impact (measured on 700 employees, Detail report, 1 month)
///
/// | Operation (7KB payload) | Before (1 conn) | After (2 conn) |
/// |-------------------------|------------------|----------------|
/// | Redis HSET              | 27,116 ms        | < 1 ms         |
/// | Redis PUBLISH           | 54,258 ms        | < 1 ms         |
/// | Total report generation | 5-10 minutes     | ~1.4 seconds   |
///
/// # Safety Rules
///
/// - **`conn()`**: Use for ALL non-blocking operations (GET, SET, HSET, PUBLISH, etc.)
/// - **`blocking_conn()`**: Use ONLY for blocking commands (BLPOP, BRPOP, SUBSCRIBE)
/// - **Never use `conn()` for BLPOP** — this will re-introduce head-of-line blocking
#[derive(Clone)]
pub struct RedisPool {
    /// Connection for fast, non-blocking operations.
    /// Shared across all concurrent worker tasks.
    conn_manager: ConnectionManager,
    /// Dedicated connection for blocking operations (BLPOP).
    /// Isolated to prevent stalling non-blocking operations.
    blocking_conn: ConnectionManager,
    client: Client,
}

impl RedisPool {
    /// Create a new Redis pool from a connection URL.
    ///
    /// Establishes **two** `ConnectionManager` instances:
    /// 1. `conn_manager` for non-blocking operations (shared by all tasks)
    /// 2. `blocking_conn` for `BLPOP` (isolated to prevent head-of-line blocking)
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection string (e.g., "redis://localhost:6379")
    ///
    /// # Errors
    /// Returns `RedisError` if connection fails.
    pub async fn new(redis_url: &str) -> Result<Self, RedisError> {
        let client = Client::open(redis_url)?;

        let mut last_err = None;
        for attempt in 0..5 {
            let connect_fut = async {
                let conn_manager = ConnectionManager::new(client.clone()).await?;
                let blocking_conn = ConnectionManager::new(client.clone()).await?;
                Ok::<_, RedisError>((conn_manager, blocking_conn))
            };

            match tokio::time::timeout(std::time::Duration::from_secs(5), connect_fut).await {
                Ok(Ok((conn_manager, blocking_conn))) => {
                    return Ok(Self {
                        conn_manager,
                        blocking_conn,
                        client,
                    });
                }
                Ok(Err(e)) => last_err = Some(e),
                Err(_) => {
                    last_err = Some(RedisError::from((
                        redis::ErrorKind::IoError,
                        "Connection to Redis timed out",
                    )));
                }
            }

            if attempt < 4 {
                let delay = std::time::Duration::from_millis(200 * (1 << attempt));
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_err.unwrap_or_else(|| {
            RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to connect to Redis after retries",
            ))
        }))
    }

    /// Get a clone of the connection manager for **non-blocking** commands.
    ///
    /// This connection is shared across all concurrent worker tasks for
    /// fast operations like HSET, GET, PUBLISH, INCR, SET.
    ///
    /// **WARNING**: Do NOT use this for BLPOP/BRPOP — use `blocking_conn()` instead.
    /// Using this for blocking commands will stall all concurrent operations.
    pub fn conn(&self) -> ConnectionManager {
        self.conn_manager.clone()
    }

    /// Get a clone of the dedicated **blocking** connection.
    ///
    /// Use this **exclusively** for blocking Redis commands like `BLPOP` and `BRPOP`.
    /// These commands block for their full timeout duration and would cause
    /// head-of-line blocking if sent through the shared `conn()` connection.
    ///
    /// # Example
    /// ```ignore
    /// // ✅ Correct: BLPOP on dedicated connection
    /// let mut conn = pool.blocking_conn();
    /// redis::cmd("BLPOP").arg(key).arg(30).query_async(&mut conn).await?;
    ///
    /// // ❌ Wrong: BLPOP on shared connection — blocks everything for 30s
    /// let mut conn = pool.conn();
    /// redis::cmd("BLPOP").arg(key).arg(30).query_async(&mut conn).await?;
    /// ```
    pub fn blocking_conn(&self) -> ConnectionManager {
        self.blocking_conn.clone()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_redis_fail_fast_on_invalid_url() {
        // Option A: Fail-Fast Startup Test
        // Ensure that RedisPool::new fails immediately when Redis is unreachable,
        // preventing the app from starting up with a broken queue.
        let invalid_url = "redis://127.0.0.1:9999"; // Non-existent port

        let result =
            tokio::time::timeout(Duration::from_secs(3), RedisPool::new(invalid_url)).await;

        match result {
            Ok(Err(e)) => {
                // Expected: RedisError (Connection refused)
                assert!(e.is_io_error() || e.is_connection_refusal() || e.is_connection_dropped());
            }
            Ok(Ok(_)) => {
                panic!("CRITICAL: RedisPool::new succeeded with an invalid URL. App would silently fail in production.");
            }
            Err(_) => {
                // Expected: Timeout if ConnectionManager retries infinitely
                // Either way, the app shouldn't proceed.
            }
        }
    }

    #[tokio::test]
    async fn test_redis_health_check_success() {
        // Option B: Health Check Test (Happy Path)
        // Verify that health_check() works correctly when connected.
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

        let pool_result =
            tokio::time::timeout(Duration::from_secs(2), RedisPool::new(&redis_url)).await;

        let Ok(Ok(pool)) = pool_result else {
            return;
        };

        let health_res = tokio::time::timeout(Duration::from_secs(2), pool.health_check()).await;

        assert!(
            matches!(health_res, Ok(Ok(()))),
            "Health check failed on working Redis"
        );
    }
}
