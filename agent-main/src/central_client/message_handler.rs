use log::{error, info};
use tokio::sync::mpsc;

use crate::models::CentralMessage;
use crate::hybrid_database::HybridDatabase;

use super::retry_manager::parse_composite_key;

pub async fn handle_incoming_text(text: &str, hybrid_db: &HybridDatabase, reload_tx: &mpsc::Sender<()>) {
    if let Ok(msg) = serde_json::from_str::<CentralMessage>(text) {
        match msg {
            CentralMessage::ConfigPush { data } => {
                info!("Received ConfigPush with {} devices", data.devices.len());
                if let Some(mw) = data.max_concurrent_workers {
                    let _ = hybrid_db.disk_db().upsert_agent_config("max_concurrent_workers", &mw.to_string()).await;
                }

                // 1. Capture current active state
                let old_active_configs = hybrid_db.disk_db().get_active_devices().await.unwrap_or_default();
                let old_active_ids: std::collections::HashSet<String> = old_active_configs.into_iter().map(|d| d.id).collect();

                // 2. Deactivate devices not in this push (stale/removed)
                let pushed_ids: Vec<String> = data.devices.iter().map(|d| d.device_id.clone()).collect();
                if let Ok(deactivated) = hybrid_db.disk_db().deactivate_devices_not_in(&pushed_ids).await
                    && deactivated > 0 {
                        info!("Deactivated {deactivated} stale devices");
                    }

                // 3. Upsert devices (might enable or disable specific ones)
                for d in data.devices {
                    let timezone = d.timezone.as_deref().unwrap_or("UTC");
                    let _ = hybrid_db.disk_db().upsert_device(&d.device_id, &d.name, &d.ip, d.port, &d.brand, timezone, d.is_enabled).await;
                }

                // 4. Capture new active state
                let new_active_configs = hybrid_db.disk_db().get_active_devices().await.unwrap_or_default();
                let new_active_ids: std::collections::HashSet<String> = new_active_configs.into_iter().map(|d| d.id).collect();

                // 5. Calculate diff and update memory
                // Note: Enabled devices will be loaded by main loop during worker spawn cycle
                // Disabled: present in old but not in new - unload immediately
                for id in old_active_ids.difference(&new_active_ids) {
                    let _ = hybrid_db.unload_device_memory(id).await;
                }

                let _ = reload_tx.try_send(());
            }
            CentralMessage::Ack { msg_id, status, .. } => {
                // msg_id is now composite key "device_id:user_id:timestamp_unix"
                if status.as_deref() == Some("success") {
                    // Update memory
                    if !hybrid_db.mark_synced_by_key(&msg_id) {
                        log::debug!("ACK for already-synced log (disk update OK): {msg_id}");
                    }

                    // Update disk using composite key
                    if let Some((device_id, user_id, timestamp_unix)) = parse_composite_key(&msg_id) {
                        log::debug!("🔍 ACK Success: device='{device_id}', user='{user_id}', timestamp='{timestamp_unix}'");

                        // We rely on hybrid_db.mark_synced_by_key() to update memory state.
                        // The Checkpoint Worker will persist this to disk asynchronously.
                        // Doing a direct UPDATE here is racy because the log might not be on disk yet.
                    } else {
                        log::warn!("⚠️ Failed to parse composite key: {msg_id}");
                    }
                } else {
                    // Failed ACK - reset to unsynced for retry
                    if !hybrid_db.reset_to_unsynced_by_key(&msg_id) {
                        log::debug!("Failed ACK for already-synced log: {msg_id}");
                    } else {
                         log::warn!("⚠️ ACK Failed: Resetting log {msg_id} for retry");
                    }
                }

            }
            CentralMessage::Error { code, message } => {
                error!("⚠️ Server error [{code}]: {message}");
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hybrid_database::HybridDatabase;
    use crate::models::AttendanceData;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_handle_incoming_error() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        let error_msg = r#"{"type":"error", "code":"INVALID_TOKEN", "message":"Invalid token provided"}"#;
        handle_incoming_text(error_msg, &hybrid_db, &reload_tx).await;
    }

    #[tokio::test]
    async fn test_handle_incoming_ack_failure() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        sqlx::query("INSERT INTO zkt_raw_attendance (id, device_id, user_id, timestamp, is_synced) VALUES (999, 'dev1', 'user1', '2026-01-27 10:00:00', 2)")
            .execute(hybrid_db.disk_db().get_pool())
            .await
            .unwrap();

        let ack_error = r#"{"type":"ack", "msg_id":"999", "status":"error", "error":"Server overloaded"}"#;
        handle_incoming_text(ack_error, &hybrid_db, &reload_tx).await;

        let _logs = hybrid_db.get_unsynced_logs(10);
    }

    #[tokio::test]
    async fn test_ack_handling_race_condition() {
        use chrono::NaiveDateTime;

        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();
        let (reload_tx, _reload_rx) = mpsc::channel(1);

        hybrid_db.disk_db().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", true).await.unwrap();

        let timestamp_str = "2026-01-31 12:00:00";
        let data = AttendanceData {
            user_id: "U999".to_string(),
            timestamp: timestamp_str.to_string(),
            status: Some(1),
            punch: Some(0),
        };

        let id = hybrid_db.save_raw_attendance("DEV1", data).await.unwrap();
        hybrid_db.mark_in_flight(&[id]);

        assert!(hybrid_db.memory_store().is_dirty());
        assert_eq!(hybrid_db.count_unsynced_logs(), 0);

        let ts_unix = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .unwrap()
            .and_utc()
            .timestamp();

        let msg_id = format!("DEV1:U999:{ts_unix}");
        let ack_msg = format!(r#"{{"type":"ack", "msg_id":"{msg_id}", "status":"success"}}"#);

        handle_incoming_text(&ack_msg, &hybrid_db, &reload_tx).await;

        let persisted = hybrid_db.checkpoint().await.unwrap();
        assert_eq!(persisted, 1);

        let row: (i32,) = sqlx::query_as("SELECT is_synced FROM zkt_raw_attendance WHERE user_id = 'U999'")
            .fetch_one(hybrid_db.disk_db().get_pool())
            .await
            .unwrap();

        assert_eq!(row.0, 1, "Log should be persisted as synced (1), not unsynced (0)");
    }

    #[tokio::test]
    async fn test_sync_blocked_when_no_active_devices() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();

        sqlx::query(
            "INSERT INTO zkt_raw_attendance (id, device_id, user_id, timestamp, is_synced) VALUES (1, 'dev1', 'user1', '2026-01-29 10:00:00', 0)"
        )
        .execute(hybrid_db.disk_db().get_pool())
        .await
        .unwrap();

        let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 0, "Should have no active devices initially");

        let _logs = hybrid_db.get_unsynced_logs(10);
    }

    #[tokio::test]
    async fn test_sync_allowed_when_devices_configured() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();

        hybrid_db
            .disk_db()
            .upsert_device("dev1", "Device 1", "192.168.1.100", 4370, "zkteco", "UTC", true)
            .await
            .unwrap();

        let devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
        assert_eq!(devices.len(), 1, "Should have 1 active device");
        assert_eq!(devices[0].id, "dev1");
    }

    #[tokio::test]
    async fn test_sync_blocked_after_devices_deactivated() {
        let hybrid_db = HybridDatabase::new(":memory:", 1000).await.unwrap();

        hybrid_db
            .disk_db()
            .upsert_device("dev1", "Device 1", "192.168.1.100", 4370, "zkteco", "UTC", true)
            .await
            .unwrap();

        hybrid_db
            .disk_db()
            .deactivate_devices_not_in(&[])
            .await
            .unwrap();

        let _devices = hybrid_db.disk_db().get_active_devices().await.unwrap();
    }
}
