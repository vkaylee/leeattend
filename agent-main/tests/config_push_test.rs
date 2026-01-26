use agent::{database::Database, models::{CentralMessage, ConfigData, DeviceConfigPush}};
use tempfile::tempdir;

#[tokio::test]
async fn test_handle_config_push_enum() {
    // Use in-memory SQLite for reliable testing
    let db = Database::new("sqlite::memory:").await.unwrap();

    let config_msg = CentralMessage::ConfigPush {
        data: ConfigData {
            devices: vec![
                DeviceConfigPush {
                    device_id: "dev-1".to_string(),
                    name: "Office Machine".to_string(),
                    ip: "192.168.1.100".to_string(),
                    port: 4370,
                }
            ],
        },
    };

    if let CentralMessage::ConfigPush { data } = config_msg {
        for d in data.devices {
            db.upsert_device(&d.device_id, &d.name, &d.ip, d.port).await.unwrap();
        }
    }

    let devices = db.get_active_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].id, "dev-1");
    assert_eq!(devices[0].name, "Office Machine");
}
