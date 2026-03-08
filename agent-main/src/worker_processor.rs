
#[derive(Clone, Debug)]
pub struct WorkerConfig {
    pub id: String,
    pub name: String,
    pub brand: String,
    pub ip: String,
    pub port: u32,
    pub password: Option<u32>,
}

impl WorkerConfig {
    /// Returns true if connection-relevant fields changed (needs reconnect).
    /// Name/timezone changes do NOT require reconnect.
    pub fn connection_changed(&self, other: &WorkerConfig) -> bool {
        self.ip != other.ip || self.port != other.port
            || self.brand != other.brand || self.password != other.password
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_changed() {
        let base = WorkerConfig {
            id: "dev1".into(), name: "Office".into(), brand: "zkteco".into(),
            ip: "192.168.1.1".into(), port: 4370, password: None,
        };

        // Same config → no change
        assert!(!base.connection_changed(&base));

        // Name change only → no reconnect needed
        let mut name_changed = base.clone();
        name_changed.name = "Office Renamed".into();
        assert!(!base.connection_changed(&name_changed));

        // IP changed → reconnect
        let mut ip_changed = base.clone();
        ip_changed.ip = "192.168.1.2".into();
        assert!(base.connection_changed(&ip_changed));

        // Port changed → reconnect
        let mut port_changed = base.clone();
        port_changed.port = 4371;
        assert!(base.connection_changed(&port_changed));

        // Password changed → reconnect
        let mut pwd_changed = base.clone();
        pwd_changed.password = Some(12345);
        assert!(base.connection_changed(&pwd_changed));

        // Brand changed → reconnect
        let mut brand_changed = base.clone();
        brand_changed.brand = "mock".into();
        assert!(base.connection_changed(&brand_changed));
    }
}
