use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
use log::info;
use crate::models::{RawAttendanceLog, DeviceConfig};

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

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;

        info!("Database connected and migrations applied.");
        Ok(Self { pool })
    }

    #[allow(dead_code)]
    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    pub async fn save_raw_attendance(
        &self,
        device_id: &str,
        user_id: &str,
        timestamp: chrono::NaiveDateTime,
        status: Option<i32>,
        punch: Option<i32>,
    ) -> Result<i64, Error> {
        let res = sqlx::query(
            r#"
            INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(device_id)
        .bind(user_id)
        .bind(timestamp)
        .bind(status)
        .bind(punch)
        .execute(&self.pool)
        .await?;

        Ok(res.last_insert_rowid())
    }

    pub async fn get_unsynced_logs(&self, limit: i64) -> Result<Vec<RawAttendanceLog>, Error> {
        sqlx::query_as::<_, RawAttendanceLog>(
            r#"
            SELECT 
                id, 
                device_id, 
                user_id, 
                timestamp, 
                status, 
                punch, 
                is_synced, 
                created_at
            FROM zkt_raw_attendance
            WHERE is_synced = 0
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_synced(&self, ids: &[i64]) -> Result<(), Error> {
        if ids.is_empty() {
            return Ok(());
        }

        // SQLite doesn't support array parameters directly like Postgres, so we build the query dynamically or iterate.
        // For simplicity and safety with small batches, we can loop (transaction) or build a generic query string.
        // Let's use a transaction.
        let mut tx = self.pool.begin().await?;
        
        for id in ids {
            sqlx::query(
                "UPDATE zkt_raw_attendance SET is_synced = 1 WHERE id = ?",
            )
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
    use chrono::NaiveDate;

    async fn setup_db() -> Database {
        // Use in-memory database for testing
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory pool");

        // Run migrations (assuming ./migrations exists relative to cargo workspace root)
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Database { pool }
    }

    #[tokio::test]
    async fn test_save_and_retrieve_attendance() {
        let db = setup_db().await;
        
        let timestamp = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        let id = db.save_raw_attendance(
            "DEV001", 
            "USER123", 
            timestamp, 
            Some(1), 
            Some(0)
        ).await.expect("Failed to save attendance");

        assert!(id > 0);

        let logs = db.get_unsynced_logs(10).await.expect("Failed to get logs");
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].device_id, "DEV001");
        assert_eq!(logs[0].user_id, "USER123");
        assert_eq!(logs[0].timestamp, timestamp);
    }

    #[tokio::test]
    async fn test_mark_synced() {
        let db = setup_db().await;
        let timestamp = chrono::Utc::now().naive_utc();
        
        let id = db.save_raw_attendance("DEV001", "U1", timestamp, None, None)
            .await
            .unwrap();

        let logs_before = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs_before.len(), 1);

        db.mark_synced(&[id]).await.unwrap();

        let logs_after = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs_after.len(), 0);
    }

    #[tokio::test]
    async fn test_device_upsert() {
        let db = setup_db().await;

        // Insert new
        db.upsert_device("DEV001", Some("Entrance"), "192.168.1.10", 4370)
            .await
            .unwrap();

        let devices = db.get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "DEV001");
        assert_eq!(devices[0].name.as_deref(), Some("Entrance"));

        // Update existing
        db.upsert_device("DEV001", Some("Main Gate"), "192.168.1.20", 4370)
            .await
            .unwrap();

        let devices = db.get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name.as_deref(), Some("Main Gate"));
        assert_eq!(devices[0].ip, "192.168.1.20");
    }
}
