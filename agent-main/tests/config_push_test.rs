use agent::hybrid_database::HybridDatabase;
use agent::models::{CentralMessage, ConfigData, DeviceConfigPush};

#[test]
fn test_handle_config_push_with_brand() {
    let hybrid = HybridDatabase::new(":memory:", 1000, 90).expect("DB setup failed");

    let config_msg = CentralMessage::ConfigPush {
        data: ConfigData {
            devices: vec![DeviceConfigPush {
                device_id: "dev-1".to_string(),
                name: "Office Machine".to_string(),
                ip: "192.168.1.100".to_string(),
                port: 4370,
                brand: "zkteco".to_string(),
                timezone: Some("UTC".to_string()),
                password: None,
                is_enabled: true,
            }],
            max_concurrent_workers: None,
            success_cooldown_secs: None,
            retention_days: None,
        },
    };

    if let CentralMessage::ConfigPush { data } = config_msg {
        for d in data.devices {
            let tz = d.timezone.as_deref().unwrap_or("UTC");
            hybrid
                .store()
                .upsert_device(
                    &d.device_id,
                    &d.name,
                    &d.ip,
                    d.port,
                    &d.brand,
                    tz,
                    None,
                    d.is_enabled,
                    123456,
                )
                .unwrap();
        }
    }

    let devices = hybrid.store().get_active_devices().unwrap();
    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].brand, "zkteco");
}
