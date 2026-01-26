use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
// use log::info;
use crate::models::{RawAttendanceLog, DeviceConfig, AttendanceData};
use chrono::NaiveDateTime;
use serde_json;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        sqlx::query("PRAGMA journal_mode=WAL;").execute(&pool).await?;
        Ok(Self { pool })
    }

    #[allow(dead_code)]
    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    #[allow(dead_code)]
    pub async fn save_raw_attendance(
        &self,
        device_id: &str,
        data: AttendanceData,
    ) -> Result<i64, Error> {
        let timestamp_res = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"));

        let timestamp = match timestamp_res {
            Ok(t) => t,
            Err(e) => {
                let raw_json = serde_json::to_string(&data).unwrap_or_default();
                let error_msg = format!("Timestamp parse error: {}", e);
                
                sqlx::query(
                    r#"
                    INSERT INTO parsing_errors (device_id, raw_data, error_msg, created_at)
                    VALUES (?, ?, ?, CURRENT_TIMESTAMP)
                    "#
                )
                .bind(device_id)
                .bind(raw_json)
                .bind(error_msg)
                .execute(&self.pool)
                .await?;
                
                return Ok(0);
            }
        };

        let res = sqlx::query(
            r#"
            INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch, updated_at)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(device_id, user_id, timestamp) DO NOTHING
            "#,
        )
        .bind(device_id)
        .bind(data.user_id)
        .bind(timestamp)
        .bind(data.status)
        .bind(data.punch)
        .execute(&self.pool)
        .await?;

        Ok(res.last_insert_rowid())
    }

    #[allow(dead_code)]
    pub async fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, Error> {
        let res = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2 AND (strftime(\"%s\", \"now\") - strftime(\"%s\", updated_at)) > ?")
            .bind(timeout_secs)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

    #[allow(dead_code)]
    pub async fn reset_syncing_status(&self) -> Result<(), Error> {
        sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_unsynced_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        let mut tx = self.pool.begin().await?;
        
        let logs = sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT id, device_id, user_id, timestamp, status, punch, is_synced, created_at
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&mut *tx)
        .await?;

        if !logs.is_empty() {
            let ids: Vec<i64> = logs.iter().map(|l| l.id).collect();
            for id in ids {
                sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 2, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
                    .bind(id)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(logs)
    }

    #[allow(dead_code)]
    pub async fn mark_synced(&self, ids: &[i64]) -> Result<(), Error> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut tx = self.pool.begin().await?;
        for id in ids {
            sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 1 WHERE id = ?")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn upsert_device(&self, id: &str, name: Option<&str>, ip: &str, port: i32) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO devices (id, name, ip, port, last_seen, is_active)
            VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP, 1)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                ip = excluded.ip,
                port = excluded.port,
                last_seen = CURRENT_TIMESTAMP,
                is_active = 1
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(ip)
        .bind(port)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_active_devices(&self) -> Result<Vec<DeviceConfig>, Error> {
        sqlx::query_as::<_, DeviceConfig>(
            r#"
            SELECT id, name, ip, port, last_seen, is_active
            FROM devices
            WHERE is_active = 1
            "#
        )
        .fetch_all(&self.pool)
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> Database {
        let db = Database::new("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(db.get_pool()).await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_save_and_retrieve_attendance() {
        let db = setup_db().await;
        let data = AttendanceData {
            user_id: "U1".into(),
            timestamp: "2026-01-01 08:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        db.save_raw_attendance("D1", data).await.unwrap();
        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].user_id, "U1");
    }

    #[tokio::test]
    async fn test_save_duplicate_attendance() {
        let db = setup_db().await;
        let data = AttendanceData {
            user_id: "U_DUP".into(),
            timestamp: "2026-01-26 12:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };

        db.save_raw_attendance("D_DUP", data.clone()).await.unwrap();
        db.save_raw_attendance("D_DUP", data).await.unwrap();

        let logs = sqlx::query_as::<_, RawAttendanceLog>("SELECT * FROM zkt_raw_attendance WHERE user_id = 'U_DUP'")
            .fetch_all(db.get_pool())
            .await
            .unwrap();
            
        assert_eq!(logs.len(), 1, "Should only have 1 log entry after duplicate save");
    }

    #[tokio::test]
    async fn test_save_malformed_attendance() {
        let db = setup_db().await;
        let data = AttendanceData {
            user_id: "U_BAD".into(),
            timestamp: "INVALID-TIME-FORMAT".into(),
            status: Some(1),
            punch: Some(1),
        };

        let id = db.save_raw_attendance("D_BAD", data).await.unwrap();
        assert_eq!(id, 0);

        let logs = sqlx::query_as::<_, RawAttendanceLog>("SELECT * FROM zkt_raw_attendance WHERE user_id = 'U_BAD'")
            .fetch_all(db.get_pool())
            .await
            .unwrap();
        assert_eq!(logs.len(), 0);

        let error_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM parsing_errors WHERE device_id = 'D_BAD'")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert_eq!(error_count, 1);
    }

    #[tokio::test]
    async fn test_mark_synced() {
        let db = setup_db().await;
        let data = AttendanceData {
            user_id: "U1".into(),
            timestamp: "2026-01-01 08:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        let id = db.save_raw_attendance("D1", data).await.unwrap();
        db.mark_synced(&[id]).await.unwrap();
        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 0);
    }

    #[tokio::test]
    async fn test_syncing_state_and_stale_reset() {
        let db = setup_db().await;
        let data = AttendanceData {
            user_id: "U1".into(),
            timestamp: "2026-01-01 08:00:00".into(),
            status: Some(1),
            punch: Some(1),
        };
        let id = db.save_raw_attendance("D1", data).await.unwrap();
        
        // 1. Fetching should mark as Syncing (is_synced = 2)
        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 1);
        
        // 2. Fetching again immediately should return nothing (already syncing)
        let logs2 = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs2.len(), 0);
        
        // 3. Resetting syncing status should make it available again
        db.reset_syncing_status().await.unwrap();
        let logs3 = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs3.len(), 1);
        
        // 4. Test stale reset (force an old updated_at)
        sqlx::query("UPDATE zkt_raw_attendance SET updated_at = \"2000-01-01 00:00:00\" WHERE id = ?")
            .bind(id)
            .execute(db.get_pool())
            .await.unwrap();
            
        // Reset logs older than 5 mins (300s)
        let count = db.reset_stale_syncing(300).await.unwrap();
        assert_eq!(count, 1);
        
        let logs4 = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs4.len(), 1);
    }

    #[tokio::test]
    async fn test_wal_mode() {
        let db = setup_db().await;
        let mode: String = sqlx::query_scalar("PRAGMA journal_mode;")
            .fetch_one(db.get_pool())
            .await
            .unwrap();
        assert!(mode == "wal" || mode == "memory", "Journal mode should be wal or memory (actual: {})", mode);
    }

    #[tokio::test]
    async fn test_device_upsert() {
        let db = setup_db().await;
        db.upsert_device("D1", Some("Device 1"), "1.1.1.1", 4370).await.unwrap();
        let devices = db.get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "D1");
    }
}
