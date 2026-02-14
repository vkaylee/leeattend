use sqlx::Error;
use crate::models::{RawAttendanceLog, SignatureRow};

use super::Database;

impl Database {
    /// Get unsynced logs for transmission. Returns logs WITHOUT marking them in-flight.
    /// Caller must call mark_in_flight() immediately after successful fetch.
    /// FIFO order: Logs are returned in insertion order (id ASC).
    pub async fn get_unsynced_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        // For Phase 2, read from disk to maintain test compatibility
        // Later phases will optimize to read from memory only
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            ORDER BY id ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Get all logs for memory store initialization (startup only)
    /// Returns all unsynced and in-flight logs to populate memory on startup
    pub async fn get_all_logs(&self) -> Result<Vec<RawAttendanceLog>, Error> {
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch,
                   is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced IN (0, 2)  -- Only load pending/in-flight
            ORDER BY id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Load all signatures for dedup index (startup only - Phase 3)
    /// Returns minimal data: device_id, user_id, timestamp, punch, status for all logs
    /// Only loads signatures within TTL window to bound memory usage
    pub async fn get_all_signatures(&self, ttl_days: i64) -> Result<Vec<SignatureRow>, Error> {
        let cutoff = chrono::Utc::now().timestamp() - (ttl_days * 86400);
        // Optimization: Only load signatures for ACTIVE devices to save memory.
        let rows = sqlx::query_as::<_, SignatureRow>(
            r#"
            SELECT r.device_id, r.user_id, r.timestamp, r.punch, r.status
            FROM zkt_raw_attendance r
            JOIN devices d ON r.device_id = d.id
            WHERE d.is_active = 1 AND r.timestamp > ?
            ORDER BY r.id ASC
            "#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Get signatures for a specific device (for runtime loading when enabling device)
    /// Only loads signatures within TTL window to bound memory usage
    pub async fn get_signatures_for_device(&self, device_id: &str, ttl_days: i64) -> Result<Vec<SignatureRow>, Error> {
        let cutoff = chrono::Utc::now().timestamp() - (ttl_days * 86400);
        let rows = sqlx::query_as::<_, SignatureRow>(
            r#"
            SELECT device_id, user_id, timestamp, punch, status
            FROM zkt_raw_attendance
            WHERE device_id = ? AND timestamp > ?
            ORDER BY id ASC
            "#
        )
        .bind(device_id)
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    /// Count total unsynced logs for sync coordination
    pub async fn count_unsynced_logs(&self) -> Result<i64, Error> {
        // For Phase 2, read from disk to maintain test compatibility
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Count unsynced logs for a specific device
    pub async fn count_unsynced_logs_for_device(&self, device_id: &str) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE is_synced = 0 AND device_id = ?"
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    /// Get unsynced logs for a specific device (for runtime loading when enabling device)
    pub async fn get_unsynced_logs_for_device(&self, device_id: &str) -> Result<Vec<RawAttendanceLog>, Error> {
        let rows = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch,
                   is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0 AND device_id = ?
            ORDER BY id ASC
            "#
        )
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    /// Count total logs for a specific device (for polling coordination)
    pub async fn count_device_logs(&self, device_id: &str) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance WHERE device_id = ?"
        )
        .bind(device_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    /// Get logs that failed to sync (attempted but failed).
    /// These have is_synced=0 but updated_at is not null (marked in-flight at least once).
    pub async fn get_failed_sync_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        // For Phase 2, read from disk to maintain test compatibility
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at, updated_at, retry_count
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            AND updated_at IS NOT NULL
            ORDER BY updated_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    /// Count logs that failed to sync (attempted but failed).
    pub async fn count_failed_sync_logs(&self) -> Result<i64, Error> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM zkt_raw_attendance
             WHERE is_synced = 0 AND updated_at IS NOT NULL"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}
