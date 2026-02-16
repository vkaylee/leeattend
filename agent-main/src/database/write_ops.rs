use sqlx::Error;
use crate::models::AttendanceData;
use serde_json;

use super::Database;

impl Database {
    pub async fn save_raw_attendance(
        &self,
        device_id: &str,
        data: AttendanceData,
    ) -> Result<i64, Error> {
        // Get device timezone
        let tz = self.get_device_timezone(device_id).await?;

        // Parse timestamp and convert to UTC Unix timestamp
        let timestamp = match Self::parse_local_to_utc_timestamp(&data.timestamp, tz) {
            Ok(ts) => ts,
            Err(e) => {
                let raw_json = serde_json::to_string(&data).unwrap_or_default();
                let error_msg = format!("Timestamp parse error: {e}");

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

        // Write to disk first
        let res = sqlx::query(
            r#"
            INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(device_id, user_id, timestamp, punch, status) DO NOTHING
            "#,
        )
        .bind(device_id)
        .bind(&data.user_id)
        .bind(timestamp)
        .bind(data.status)
        .bind(data.punch)
        .execute(&self.pool)
        .await?;

        let id = res.last_insert_rowid();

        // If inserted (not duplicate), also insert to memory store
        if res.rows_affected() > 0 {
            // Try in-memory insert (best effort, ignore errors for now)
            let _ = self.memory_store.insert(device_id.to_string(), data, timestamp);
        }

        Ok(id)
    }

    pub async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(String, AttendanceData)>,
    ) -> Result<usize, Error> {
        if batch.is_empty() {
            return Ok(0);
        }

        // Parse and prepare all records first
        type BatchRecord = (String, String, i64, Option<i32>, Option<i32>);
        let mut records: Vec<BatchRecord> = Vec::new();
        let mut batch_with_timestamps: Vec<(String, AttendanceData, i64)> = Vec::new();

        for (device_id, data) in batch {
            // Get device timezone
            let tz = self.get_device_timezone(&device_id).await?;

            // Parse timestamp and convert to UTC Unix timestamp
            if let Ok(timestamp) = Self::parse_local_to_utc_timestamp(&data.timestamp, tz) {
                records.push((device_id.clone(), data.user_id.clone(), timestamp, data.status, data.punch));
                batch_with_timestamps.push((device_id.clone(), data.clone(), timestamp));
            }
        }

        if records.is_empty() {
            return Ok(0);
        }

        // Write to disk first (with deduplication via CONFLICT clause)
        let mut tx = self.pool.begin().await?;
        let placeholders: Vec<&str> = records.iter().map(|_| "(?, ?, ?, ?, ?)").collect();
        let query = format!(
            "INSERT INTO zkt_raw_attendance (device_id, user_id, timestamp, status, punch) VALUES {} ON CONFLICT(device_id, user_id, timestamp, punch, status) DO NOTHING",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for (device_id, user_id, timestamp, status, punch) in &records {
            q = q.bind(device_id).bind(user_id).bind(timestamp).bind(status).bind(punch);
        }

        let result = q.execute(&mut *tx).await?;
        tx.commit().await?;

        let rows_affected = result.rows_affected() as usize;

        // Also insert to memory store (best effort)
        let _ = self.memory_store.insert_batch(batch_with_timestamps);

        Ok(rows_affected)
    }
}
