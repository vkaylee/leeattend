mod write_ops;
mod read_ops;
mod sync_ops;
mod device_management;
mod maintenance;

use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
use crate::models::RawAttendanceLog;
use crate::memory_store::InMemoryStore;
use std::sync::Arc;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
    memory_store: Arc<InMemoryStore>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        // Optimize pool settings based on database type
        // For in-memory SQLite (tests), use single connection to avoid lock contention
        // and prevent connection drops that would lose in-memory data
        let is_in_memory = database_url.contains(":memory:") || database_url.contains("mode=memory");

        let pool = if is_in_memory {
            SqlitePoolOptions::new()
                .max_connections(1)  // Single connection for in-memory to avoid lock issues
                .idle_timeout(None)  // Never drop connection (in-memory DB loses data on disconnect)
                .connect(database_url)
                .await?
        } else {
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?
        };

        sqlx::migrate!("./migrations").run(&pool).await?;

        // Enable WAL mode with extended busy timeout (30s) to handle high write load
        // Skip WAL for in-memory databases (not supported)
        if !is_in_memory {
            sqlx::query("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=30000; PRAGMA cache_size=-64000; PRAGMA temp_store=MEMORY;")
                .execute(&pool)
                .await?;
        } else {
            // In-memory optimizations: temp_store and busy_timeout only
            sqlx::query("PRAGMA busy_timeout=30000; PRAGMA temp_store=MEMORY;")
                .execute(&pool)
                .await?;
        }

        // Initialize in-memory store with 10K capacity (as per plan)
        let memory_store = Arc::new(InMemoryStore::new(10_000));

        // Load existing unsynced/in-flight logs from disk into memory (startup recovery)
        let existing_logs = sqlx::query_as::<_, RawAttendanceLog>(
            "SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
             FROM zkt_raw_attendance
             WHERE is_synced IN (0, 2)
             ORDER BY id ASC"
        )
        .fetch_all(&pool)
        .await?;

        memory_store.load_from_disk(existing_logs);

        Ok(Self { pool, memory_store })
    }

    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }
}
