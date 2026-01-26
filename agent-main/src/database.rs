use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
// use log::info;
use crate::models::{RawAttendanceLog, DeviceConfig, AttendanceData};
use chrono::NaiveDateTime;

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
        let timestamp = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
            .unwrap_or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ").unwrap_or_default());

        let res = sqlx::query(
            r#"
            INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch, updated_at)
            VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
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
        let res = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2 AND (strftime(%s, now) - strftime(%s, updated_at)) > ?")
            .bind(timeout_secs)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
    }

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
            // SQLite doesnt easily bind arrays, so we use a loop or format string.
            // For simplicity and safety with small batches, we can do individual updates or a structured query.
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
    async fn test_device_upsert() {
        let db = setup_db().await;
        db.upsert_device("D1", Some("Device 1"), "1.1.1.1", 4370).await.unwrap();
        let devices = db.get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "D1");
    }
}
