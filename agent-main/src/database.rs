use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Error};
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
        
        sqlx::query("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA cache_size=-64000; PRAGMA temp_store=MEMORY;")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

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

    pub async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(String, AttendanceData)>,
    ) -> Result<usize, Error> {
        if batch.is_empty() {
            return Ok(0);
        }

        let mut tx = self.pool.begin().await?;
        let mut count = 0;

        for (device_id, data) in batch {
            let timestamp_res = NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&data.timestamp, "%Y-%m-%dT%H:%M:%SZ"));

            let timestamp = match timestamp_res {
                Ok(t) => t,
                Err(_) => continue, 
            };

            sqlx::query(
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
            .execute(&mut *tx)
            .await?;
            count += 1;
        }

        tx.commit().await?;
        Ok(count)
    }

    pub async fn reset_stale_syncing(&self, timeout_secs: i64) -> Result<u64, Error> {
        let res = sqlx::query("UPDATE zkt_raw_attendance SET is_synced = 0 WHERE is_synced = 2 AND (strftime(\"%s\", \"now\") - strftime(\"%s\", updated_at)) > ?")
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
    async fn test_save_attendance_batch() {
        let db = setup_db().await;
        let batch = vec![
            ("D1".to_string(), AttendanceData { user_id: "U1".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) }),
            ("D1".to_string(), AttendanceData { user_id: "U2".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) }),
            ("D1".to_string(), AttendanceData { user_id: "U1".into(), timestamp: "2026-01-01 09:00:00".into(), status: Some(1), punch: Some(1) }), // Duplicate
        ];

        let count = db.save_raw_attendance_batch(batch).await.unwrap();
        assert_eq!(count, 3); // 3 attempted, but DB logic handles "DO NOTHING"

        let logs = db.get_unsynced_logs(10).await.unwrap();
        assert_eq!(logs.len(), 2, "Duplicate in batch should be ignored by DB constraint");
    }
}
