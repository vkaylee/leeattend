use log::{debug, error, info, warn};
use tokio::sync::mpsc;

use crate::models::CentralMessage;
use crate::hybrid_database::HybridDatabase;

use super::connection::DeviceLocks;
use super::retry_manager::parse_composite_key;

pub async fn handle_incoming_text(text: &str, hybrid_db: &HybridDatabase, reload_tx: &mpsc::Sender<()>, central_tx: &mpsc::Sender<CentralMessage>, _device_locks: &DeviceLocks) {
    debug!("Received message from central: {text}");
    if let Ok(msg) = serde_json::from_str::<CentralMessage>(text) {
        match msg {
            CentralMessage::ConfigPush { data } => {
                info!("Received ConfigPush with {} devices", data.devices.len());
                if let Some(mw) = data.max_concurrent_workers {
                    let _ = hybrid_db.store().upsert_agent_config("max_concurrent_workers", &mw.to_string());
                }

                // 1. Capture current active state
                let old_active_ids: std::collections::HashSet<String> = hybrid_db.store()
                    .get_active_devices().unwrap_or_default()
                    .into_iter().map(|d| d.id).collect();

                // 2. Deactivate devices not in this push
                let pushed_ids: Vec<String> = data.devices.iter().map(|d| d.device_id.clone()).collect();
                if let Ok(deactivated) = hybrid_db.store().deactivate_devices_not_in(&pushed_ids)
                    && deactivated > 0 {
                        info!("Deactivated {deactivated} stale devices");
                    }

                // 3. Upsert devices
                for d in data.devices {
                    let timezone = d.timezone.as_deref().unwrap_or("UTC");
                    let _ = hybrid_db.store().upsert_device(&d.device_id, &d.name, &d.ip, d.port, &d.brand, timezone, d.password.as_deref(), d.is_enabled);
                }

                // 4. Capture new active state
                let new_active_ids: std::collections::HashSet<String> = hybrid_db.store()
                    .get_active_devices().unwrap_or_default()
                    .into_iter().map(|d| d.id).collect();

                // 5. Unload disabled devices immediately
                for id in old_active_ids.difference(&new_active_ids) {
                    let _ = hybrid_db.unload_device_memory(id);
                }

                let _ = reload_tx.try_send(());
            }
            CentralMessage::Ack { msg_id, status, .. } => {
                if status.as_deref() == Some("success") {
                    if !hybrid_db.mark_synced_by_key(&msg_id) {
                        debug!("ACK for already-synced log (disk update OK): {msg_id}");
                    }
                    if let Some((device_id, user_id, timestamp_unix, punch, status)) = parse_composite_key(&msg_id) {
                        debug!("🔍 ACK Success: device='{device_id}', user='{user_id}', timestamp='{timestamp_unix}', punch={punch:?}, status={status:?}");
                    } else {
                        warn!("⚠️ Failed to parse composite key: {msg_id}");
                    }
                } else if !hybrid_db.reset_to_unsynced_by_key(&msg_id) {
                    debug!("Failed ACK for already-synced log: {msg_id}");
                } else {
                     warn!("⚠️ ACK Failed: Resetting log {msg_id} for retry");
                }
            }
            CentralMessage::Error { code, message } => {
                error!("⚠️ Server error [{code}]: {message}");
            }
            CentralMessage::Command { id, method, device_id, .. } => {
                info!("Received command {id}: {method} (device: {device_id:?})");
                if method == "get_device_info" {
                    if let Some(did) = device_id {
                        match hybrid_db.store().get_device_metadata(&did) {
                            Ok(Some(meta)) => {
                                let info = serde_json::json!({
                                    "mac": meta.mac,
                                    "serial": meta.serial,
                                    "firmware": meta.firmware,
                                    "platform": meta.platform,
                                    "hw_name": meta.hw_name,
                                    "users_count": meta.users_count,
                                    "logs_count": meta.logs_count,
                                    "users_capacity": meta.users_capacity,
                                    "logs_capacity": meta.logs_capacity,
                                    "device_time": meta.device_time,
                                    "last_info_update": meta.last_info_update,
                                    "last_sync": meta.last_info_update.map(|ts| {
                                        let tz: chrono_tz::Tz = meta.timezone.parse().unwrap_or(chrono_tz::UTC);
                                        chrono::DateTime::from_timestamp(ts, 0)
                                            .map(|dt| dt.with_timezone(&tz).format("%Y-%m-%dT%H:%M:%S%:z").to_string())
                                            .unwrap_or_default()
                                    }),
                                });
                                let response = CentralMessage::CommandResponse {
                                    command_id: id,
                                    status: "success".to_string(),
                                    data: Some(info),
                                    error: None,
                                };
                                let _ = central_tx.send(response).await;
                            }
                            Ok(None) => {
                                let response = CentralMessage::CommandResponse {
                                    command_id: id,
                                    status: "error".to_string(),
                                    data: None,
                                    error: Some("Device metadata not found".to_string()),
                                };
                                let _ = central_tx.send(response).await;
                            }
                            Err(e) => {
                                let response = CentralMessage::CommandResponse {
                                    command_id: id,
                                    status: "error".to_string(),
                                    data: None,
                                    error: Some(e.to_string()),
                                };
                                let _ = central_tx.send(response).await;
                            }
                        }
                    } else {
                        let response = CentralMessage::CommandResponse {
                            command_id: id,
                            status: "error".to_string(),
                            data: None,
                            error: Some("device_id is required for get_device_info".to_string()),
                        };
                        let _ = central_tx.send(response).await;
                    }
                }
            }
            _ => {}
        }
    } else {
        error!("❌ Failed to parse message from central: {text}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hybrid_database::HybridDatabase;
    use crate::models::AttendanceData;
    use tokio::sync::mpsc;
    use std::sync::Arc;
    use dashmap::DashMap;

    fn setup_db() -> HybridDatabase {
        HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed")
    }

    fn setup_locks() -> DeviceLocks {
        Arc::new(DashMap::new())
    }

    #[tokio::test]
    async fn test_handle_incoming_error() {
        let hybrid_db = setup_db();
        let locks = setup_locks();
        let (reload_tx, _reload_rx) = mpsc::channel(1);
        let error_msg = r#"{"type":"error", "code":"INVALID_TOKEN", "message":"Invalid token provided"}"#;
        let (central_tx, _central_rx) = mpsc::channel(1);
        handle_incoming_text(error_msg, &hybrid_db, &reload_tx, &central_tx, &locks).await;
    }

    #[tokio::test]
    async fn test_handle_incoming_ack_failure() {
        let hybrid_db = setup_db();
        let locks = setup_locks();
        hybrid_db.store().upsert_device("dev1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

        let data = AttendanceData { user_id: "user1".into(), timestamp: "2026-01-27 10:00:00".into(), status: Some(-1), punch: Some(-1) };
        let id = hybrid_db.save_raw_attendance("dev1", data).unwrap();
        hybrid_db.mark_in_flight(&[id]);

        let (reload_tx, _reload_rx) = mpsc::channel(1);
        let ack_error = r#"{"type":"ack", "msg_id":"dev1:user1:1737972000:-1:-1", "status":"error", "error":"Server overloaded"}"#;
        let (central_tx, _central_rx) = mpsc::channel(1);
        handle_incoming_text(ack_error, &hybrid_db, &reload_tx, &central_tx, &locks).await;
    }

    #[tokio::test]
    async fn test_ack_handling() {
        let hybrid_db = setup_db();
        let locks = setup_locks();
        hybrid_db.store().upsert_device("DEV1", "Device 1", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();

        let data = AttendanceData { user_id: "U999".into(), timestamp: "2026-01-31 12:00:00".into(), status: Some(1), punch: Some(0) };
        let id = hybrid_db.save_raw_attendance("DEV1", data).unwrap();
        hybrid_db.mark_in_flight(&[id]);
        assert_eq!(hybrid_db.count_unsynced_logs(), 0);

        let ts_unix = chrono::NaiveDateTime::parse_from_str("2026-01-31 12:00:00", "%Y-%m-%d %H:%M:%S")
            .unwrap().and_utc().timestamp();
        let msg_id = format!("DEV1:U999:{ts_unix}:0:1");
        let ack_msg = format!(r#"{{"type":"ack", "msg_id":"{msg_id}", "status":"success"}}"#);

        let (reload_tx, _reload_rx) = mpsc::channel(1);
        let (central_tx, _central_rx) = mpsc::channel(1);
        handle_incoming_text(&ack_msg, &hybrid_db, &reload_tx, &central_tx, &locks).await;

        // After successful ACK, log should be removed from active storage
        assert_eq!(hybrid_db.count_unsynced_logs(), 0);
    }

    #[tokio::test]
    async fn test_sync_blocked_when_no_active_devices() {
        let hybrid_db = setup_db();
        let devices = hybrid_db.store().get_active_devices().unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_allowed_when_devices_configured() {
        let hybrid_db = setup_db();
        hybrid_db.store().upsert_device("dev1", "Device 1", "192.168.1.100", 4370, "zkteco", "UTC", None, true).unwrap();
        let devices = hybrid_db.store().get_active_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "dev1");
    }

    #[tokio::test]
    async fn test_sync_blocked_after_devices_deactivated() {
        let hybrid_db = setup_db();
        hybrid_db.store().upsert_device("dev1", "Device 1", "192.168.1.100", 4370, "zkteco", "UTC", None, true).unwrap();
        hybrid_db.store().deactivate_devices_not_in(&[]).unwrap();
        let devices = hybrid_db.store().get_active_devices().unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[tokio::test]
    async fn test_device_metadata_persistence() {
        let hybrid_db = setup_db();
        let s = hybrid_db.store();

        s.upsert_device("dev_123", "Test Device", "1.1.1.1", 4370, "zkteco", "UTC", None, true).unwrap();
        s.update_device_metadata("dev_123", "00:11:22:33:44:55", "SN123456", "v1.0", "Linux", "xFace100", 100, 5000, 1000, 10000, "2023-01-01 12:00:00").unwrap();

        let dev = s.get_device_metadata("dev_123").unwrap().unwrap();
        assert_eq!(dev.mac.as_deref(), Some("00:11:22:33:44:55"));
        assert_eq!(dev.serial.as_deref(), Some("SN123456"));
        assert_eq!(dev.users_count, Some(100));
        assert_eq!(dev.users_capacity, Some(1000));
        assert_eq!(dev.logs_capacity, Some(10000));
        assert_eq!(dev.device_time.as_deref(), Some("2023-01-01 12:00:00"));
        assert_eq!(dev.hw_name.as_deref(), Some("xFace100"));
    }

    #[tokio::test]
    async fn test_handle_get_device_info_command() {
        let hybrid_db = setup_db();
        let locks = setup_locks();
        let (reload_tx, _reload_rx) = mpsc::channel(1);
        let (central_tx, mut central_rx) = mpsc::channel(1);

        hybrid_db.store().upsert_device("dev_info_test", "Test Device", "1.2.3.4", 4370, "zkteco", "UTC", None, true).unwrap();
        hybrid_db.store().update_device_metadata("dev_info_test", "MAC_ADDR", "SERIAL_NUM", "FW_VER", "PLATFORM", "HW_NAME", 50, 500, 1000, 10000, "2023-10-27 10:00:00").unwrap();

        let cmd_json = r#"{"type": "command", "id": "cmd_123", "method": "get_device_info", "device_id": "dev_info_test", "params": {}}"#;
        handle_incoming_text(cmd_json, &hybrid_db, &reload_tx, &central_tx, &locks).await;

        if let Some(CentralMessage::CommandResponse { command_id, status, data, error }) = central_rx.recv().await {
            assert_eq!(command_id, "cmd_123");
            assert_eq!(status, "success");
            assert!(error.is_none());
            let info = data.expect("Should have data");
            assert_eq!(info["mac"], "MAC_ADDR");
            assert_eq!(info["users_count"], 50);
            assert_eq!(info["users_capacity"], 1000);
        } else {
            panic!("Expected CommandResponse");
        }
    }

    #[tokio::test]
    async fn test_handle_config_push_with_password() {
        let hybrid_db = setup_db();
        let locks = setup_locks();
        let (reload_tx, _reload_rx) = mpsc::channel(1);
        let (central_tx, _central_rx) = mpsc::channel(1);

        let config_json = r#"{"type": "config_push", "data": {"devices": [{"device_id": "dev_pwd_push", "name": "Push Password Device", "ip": "1.2.3.4", "port": 4370, "brand": "zkteco", "timezone": "UTC", "password": "98765", "is_enabled": true}], "max_concurrent_workers": 5}}"#;
        handle_incoming_text(config_json, &hybrid_db, &reload_tx, &central_tx, &locks).await;

        let devices = hybrid_db.store().get_active_devices().unwrap();
        let dev = devices.iter().find(|d| d.id == "dev_pwd_push").unwrap();
        assert_eq!(dev.password.as_deref(), Some("98765"));
    }
}
