use sqlx::Error;
use crate::models::DeviceConfig;
use chrono::{NaiveDateTime, TimeZone};
use chrono_tz::Tz;

use super::Database;

impl Database {
    /// Get device timezone from database, fallback to UTC if not found or invalid
    pub async fn get_device_timezone(&self, device_id: &str) -> Result<Tz, Error> {
        let timezone_str: Option<String> = sqlx::query_scalar(
            "SELECT timezone FROM devices WHERE id = ?"
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        let tz_str = timezone_str.unwrap_or_else(|| "UTC".to_string());
        let tz = tz_str.parse::<Tz>().unwrap_or(chrono_tz::UTC);
        Ok(tz)
    }

    /// Convert local time string to UTC Unix timestamp
    /// Returns i64 timestamp in seconds since Unix epoch
    pub fn parse_local_to_utc_timestamp(local_time_str: &str, tz: Tz) -> Result<i64, String> {
        // Try parsing with different formats
        let naive_dt = NaiveDateTime::parse_from_str(local_time_str, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(local_time_str, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|e| format!("Failed to parse timestamp: {e}"))?;

        // Convert to timezone-aware datetime
        let local_dt = tz.from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| format!("Ambiguous or non-existent local time: {local_time_str}"))?;

        // Convert to UTC and get Unix timestamp
        let utc_dt = local_dt.with_timezone(&chrono::Utc);
        Ok(utc_dt.timestamp())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn upsert_device(&self, id: &str, name: &str, ip: &str, port: i32, brand: &str, timezone: &str, is_enabled: bool) -> Result<(), Error> {
        let is_active = if is_enabled { 1 } else { 0 };
        sqlx::query(
            r#"
            INSERT INTO devices (id, name, ip, port, brand, timezone, last_seen, is_active)
            VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                ip = excluded.ip,
                port = excluded.port,
                brand = excluded.brand,
                timezone = excluded.timezone,
                last_seen = CURRENT_TIMESTAMP,
                is_active = excluded.is_active
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(ip)
        .bind(port)
        .bind(brand)
        .bind(timezone)
        .bind(is_active)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), Error> {
        sqlx::query(
            r#"
            INSERT INTO agent_config (key, value)
            VALUES (?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_agent_config(&self, key: &str) -> Result<Option<String>, Error> {
        let value: Option<String> = sqlx::query_scalar(
            "SELECT value FROM agent_config WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        Ok(value)
    }

    /// Clear local config on startup to sync fresh from Server
    /// - Clears ALL agent_config entries (agent will use defaults until ConfigPush)
    /// - Deactivates all devices (wait for fresh device list from Server)
    pub async fn clear_startup_config(&self) -> Result<(), Error> {
        // Clear all config entries
        sqlx::query("DELETE FROM agent_config")
            .execute(&self.pool)
            .await?;

        // Deactivate all devices
        sqlx::query("UPDATE devices SET is_active = 0 WHERE is_active = 1")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_active_devices(&self) -> Result<Vec<DeviceConfig>, Error> {
        let devices = sqlx::query_as::<_, DeviceConfig>(
            "SELECT * FROM devices WHERE is_active = 1"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(devices)
    }

    pub async fn get_device_metadata(&self, id: &str) -> Result<Option<DeviceConfig>, Error> {
        let device = sqlx::query_as::<_, DeviceConfig>(
            "SELECT * FROM devices WHERE LOWER(id) = LOWER(?)"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(device)
    }

    pub async fn update_device_metadata(
        &self, 
        id: &str, 
        mac: &str, 
        serial: &str, 
        fw: &str, 
        plt: &str, 
        hw: &str, 
        users: i64, 
        logs: i64
    ) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE devices SET 
                mac = ?, 
                serial = ?, 
                firmware = ?, 
                platform = ?, 
                hw_name = ?, 
                users_count = ?, 
                logs_count = ?, 
                last_info_update = CURRENT_TIMESTAMP 
            WHERE id = ?
            "#
        )
        .bind(mac)
        .bind(serial)
        .bind(fw)
        .bind(plt)
        .bind(hw)
        .bind(users)
        .bind(logs)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_device_counts(&self, id: &str, users: i64, logs: i64) -> Result<(), Error> {
        sqlx::query(
            r#"
            UPDATE devices SET 
                users_count = CASE WHEN ? >= 0 THEN ? ELSE users_count END,
                logs_count = CASE WHEN ? >= 0 THEN ? ELSE logs_count END,
                last_info_update = CURRENT_TIMESTAMP 
            WHERE LOWER(id) = LOWER(?)
            "#
        )
        .bind(users)
        .bind(users)
        .bind(logs)
        .bind(logs)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn deactivate_devices_not_in(&self, active_ids: &[String]) -> Result<u64, Error> {
        if active_ids.is_empty() {
            // Deactivate ALL devices if none are in the active list
            let res = sqlx::query("UPDATE devices SET is_active = 0 WHERE is_active = 1")
                .execute(&self.pool)
                .await?;
            return Ok(res.rows_affected());
        }

        let placeholders: Vec<String> = active_ids.iter().map(|_| "?".to_string()).collect();
        let query = format!(
            "UPDATE devices SET is_active = 0 WHERE is_active = 1 AND id NOT IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for id in active_ids {
            q = q.bind(id);
        }

        let res = q.execute(&self.pool).await?;
        Ok(res.rows_affected())
    }
}
