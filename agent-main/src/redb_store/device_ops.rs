use super::{AGENT_CONFIG, DEVICES, RedbStore, StoreError, StoredDevice};
use redb::{ReadableDatabase, ReadableTable};

impl RedbStore {
    pub fn get_device_timezone(&self, device_id: &str) -> Result<chrono_tz::Tz, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(DEVICES)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let Some(g) = table
            .get(device_id)
            .map_err(|e| StoreError::DbError(e.to_string()))?
        else {
            return Err(StoreError::InvalidTimezone);
        };

        let d = StoredDevice::from_bytes(g.value())?;
        d.timezone
            .parse::<chrono_tz::Tz>()
            .map_err(|_| StoreError::InvalidTimezone)
    }

    /// Parse a device timestamp into a UTC Unix timestamp.
    ///
    /// Explicit offsets are authoritative: RFC3339/offset timestamps are
    /// converted directly and never interpreted through the device timezone.
    /// Only offset-free wall-clock values use `tz` for local-to-UTC conversion.
    pub fn parse_local_to_utc_timestamp(
        timestamp_str: &str,
        tz: chrono_tz::Tz,
    ) -> Result<i64, String> {
        use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};

        let timestamp_str = timestamp_str.trim();
        if timestamp_str.is_empty() {
            return Err("Timestamp is empty".to_string());
        }

        if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp_str) {
            return Ok(dt.with_timezone(&Utc).timestamp());
        }

        if let Ok(dt) = DateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S %z") {
            return Ok(dt.with_timezone(&Utc).timestamp());
        }

        let naive_dt = NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%dT%H:%M:%S"))
            .map_err(|e| format!("Failed to parse timestamp: {e}"))?;

        // Unified DST strategy (matches apiserver/src/utils/tz.rs):
        // - Single → direct conversion
        // - Ambiguous → pick first (pre-transition / standard-time) instant
        // - None (gap) → shift forward by 1 hour and retry
        let local_dt = match tz.from_local_datetime(&naive_dt) {
            chrono::LocalResult::Single(dt) | chrono::LocalResult::Ambiguous(dt, _) => dt,
            chrono::LocalResult::None => {
                let shifted = naive_dt + Duration::hours(1);
                tz.from_local_datetime(&shifted)
                    .single()
                    .ok_or_else(|| format!("Cannot resolve DST gap: {timestamp_str}"))?
            }
        };
        Ok(local_dt.with_timezone(&Utc).timestamp())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_device(
        &self,
        id: &str,
        name: &str,
        ip: &str,
        port: i32,
        brand: &str,
        timezone: &str,
        password: Option<&str>,
        is_enabled: bool,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        let now = now_ts;

        // Read existing device data first (separate read txn)
        let existing = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .get(id)
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        let device = StoredDevice {
            id: id.into(),
            name: name.into(),
            ip: ip.into(),
            port: port as i64,
            brand: brand.into(),
            timezone: timezone.into(),
            last_seen: Some(now),
            is_active: is_enabled,
            mac: existing.as_ref().and_then(|d| d.mac.clone()),
            serial: existing.as_ref().and_then(|d| d.serial.clone()),
            firmware: existing.as_ref().and_then(|d| d.firmware.clone()),
            platform: existing.as_ref().and_then(|d| d.platform.clone()),
            hw_name: existing.as_ref().and_then(|d| d.hw_name.clone()),
            users_count: existing.as_ref().and_then(|d| d.users_count),
            logs_count: existing.as_ref().and_then(|d| d.logs_count),
            users_capacity: existing.as_ref().and_then(|d| d.users_capacity),
            logs_capacity: existing.as_ref().and_then(|d| d.logs_capacity),
            password: password.map(|p| p.to_string()),
            device_time: existing.as_ref().and_then(|d| d.device_time.clone()),
            last_info_update: existing.as_ref().and_then(|d| d.last_info_update),
            use_legacy_checksum: existing.as_ref().and_then(|d| d.use_legacy_checksum),
        };

        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .insert(id, device.to_bytes().as_slice())
                .map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn get_active_devices(&self) -> Result<Vec<StoredDevice>, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(DEVICES)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut devices = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| StoreError::DbError(e.to_string()))?
        {
            let (_, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            let d = StoredDevice::from_bytes(g.value())?;
            if d.is_active {
                devices.push(d);
            }
        }
        Ok(devices)
    }

    /// Persist the detected checksum mode for a device (smart checksum memory).
    /// Called after successful connect to remember the auto-detected mode.
    pub fn update_device_checksum(&self, id: &str, use_legacy: bool) -> Result<(), StoreError> {
        let key = self.find_device_key_ci(id)?;
        let k = match key {
            Some(k) => k,
            None => return Ok(()),
        };

        let existing = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .get(k.as_str())
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        if let Some(mut d) = existing {
            d.use_legacy_checksum = Some(use_legacy);

            let txn = self
                .db
                .read()
                .begin_write()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut table = txn
                    .open_table(DEVICES)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                table
                    .insert(k.as_str(), d.to_bytes().as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn get_device_metadata(&self, id: &str) -> Result<Option<StoredDevice>, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(DEVICES)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        if let Some(g) = table
            .get(id)
            .map_err(|e| StoreError::DbError(e.to_string()))?
        {
            return Ok(Some(StoredDevice::from_bytes(g.value())?));
        }
        // Case-insensitive fallback
        let id_lower = id.to_lowercase();
        for entry in table
            .iter()
            .map_err(|e| StoreError::DbError(e.to_string()))?
        {
            let (k, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            if k.value().to_lowercase() == id_lower {
                return Ok(Some(StoredDevice::from_bytes(g.value())?));
            }
        }
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_device_metadata(
        &self,
        id: &str,
        mac: &str,
        serial: &str,
        fw: &str,
        plt: &str,
        hw: &str,
        users: i64,
        logs: i64,
        users_cap: i64,
        logs_cap: i64,
        time: &str,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        let now = now_ts;

        // Read existing device (separate read txn to avoid borrow conflict)
        let existing = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .get(id)
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        if let Some(mut d) = existing {
            d.mac = Some(mac.into());
            d.serial = Some(serial.into());
            d.firmware = Some(fw.into());
            d.platform = Some(plt.into());
            d.hw_name = Some(hw.into());
            d.users_count = Some(users);
            d.logs_count = Some(logs);
            d.users_capacity = Some(users_cap);
            d.logs_capacity = Some(logs_cap);
            d.device_time = Some(time.into());
            d.last_info_update = Some(now);

            let txn = self
                .db
                .read()
                .begin_write()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut table = txn
                    .open_table(DEVICES)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                table
                    .insert(id, d.to_bytes().as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn update_device_counts(
        &self,
        id: &str,
        users: i64,
        logs: i64,
        now_ts: i64,
    ) -> Result<(), StoreError> {
        // Read existing device
        let key = self.find_device_key_ci(id)?;
        let k = match key {
            Some(k) => k,
            None => return Ok(()),
        };

        let existing = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .get(k.as_str())
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        if let Some(mut d) = existing {
            if users >= 0 {
                d.users_count = Some(users);
            }
            if logs >= 0 {
                d.logs_count = Some(logs);
            }
            d.last_info_update = Some(now_ts);

            let txn = self
                .db
                .read()
                .begin_write()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut table = txn
                    .open_table(DEVICES)
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
                table
                    .insert(k.as_str(), d.to_bytes().as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    fn find_device_key_ci(&self, id: &str) -> Result<Option<String>, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(DEVICES)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        if table
            .get(id)
            .map_err(|e| StoreError::DbError(e.to_string()))?
            .is_some()
        {
            return Ok(Some(id.to_string()));
        }
        let id_lower = id.to_lowercase();
        for entry in table
            .iter()
            .map_err(|e| StoreError::DbError(e.to_string()))?
        {
            let (k, _) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            if k.value().to_lowercase() == id_lower {
                return Ok(Some(k.value().to_string()));
            }
        }
        Ok(None)
    }

    pub fn deactivate_devices_not_in(&self, active_ids: &[String]) -> Result<u64, StoreError> {
        // Phase 1: Read all devices to find which ones to deactivate
        let updates: Vec<(String, StoredDevice)> = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in table
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
            {
                let (k, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                let mut d = StoredDevice::from_bytes(g.value())?;
                if d.is_active && !active_ids.iter().any(|aid| aid == k.value()) {
                    d.is_active = false;
                    result.push((k.value().to_string(), d));
                }
            }
            result
        };

        let count = updates.len() as u64;
        if count == 0 {
            return Ok(0);
        }

        // Phase 2: Write deactivated devices
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, d) in &updates {
                table
                    .insert(id.as_str(), d.to_bytes().as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    pub fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn
                .open_table(AGENT_CONFIG)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .insert(key, value)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn get_agent_config(&self, key: &str) -> Result<Option<String>, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(AGENT_CONFIG)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(table
            .get(key)
            .map_err(|e| StoreError::DbError(e.to_string()))?
            .map(|g| g.value().to_string()))
    }

    pub fn clear_startup_config(&self) -> Result<(), StoreError> {
        // Phase 1: Read all keys and device data
        let config_keys: Vec<String>;
        let device_updates: Vec<(String, StoredDevice)>;
        {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;

            let ct = txn
                .open_table(AGENT_CONFIG)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            config_keys = ct
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| e.ok().map(|(k, _)| k.value().to_string()))
                .collect();

            let dt = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            device_updates = dt
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| {
                    let (k, g) = e.ok()?;
                    let mut d = StoredDevice::from_bytes(g.value()).ok()?;
                    d.is_active = false;
                    Some((k.value().to_string(), d))
                })
                .collect();
        }

        // Phase 2: Write changes
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut ct = txn
                .open_table(AGENT_CONFIG)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for k in &config_keys {
                ct.remove(k.as_str())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        {
            let mut dt = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, d) in &device_updates {
                dt.insert(id.as_str(), d.to_bytes().as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn get_all_devices(&self) -> Result<Vec<StoredDevice>, StoreError> {
        let txn = self
            .db
            .read()
            .begin_read()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn
            .open_table(DEVICES)
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut devices = Vec::new();
        for entry in table
            .iter()
            .map_err(|e| StoreError::DbError(e.to_string()))?
        {
            let (_, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            devices.push(StoredDevice::from_bytes(g.value())?);
        }
        Ok(devices)
    }

    /// Clear all agent configuration entries.
    pub fn clear_all_config(&self) -> Result<(), StoreError> {
        // Phase 1: Read all keys
        let keys: Vec<String> = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(AGENT_CONFIG)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            table
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| e.ok().map(|(k, _)| k.value().to_string()))
                .collect()
        };
        if keys.is_empty() {
            return Ok(());
        }

        // Phase 2: Remove all
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn
                .open_table(AGENT_CONFIG)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for key in &keys {
                let _ = table.remove(key.as_str());
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Deactivate all devices (set is_enabled = false).
    pub fn deactivate_all_devices(&self) -> Result<(), StoreError> {
        // Phase 1: Read all devices
        let updates: Vec<(String, Vec<u8>)> = {
            let txn = self
                .db
                .read()
                .begin_read()
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in table
                .iter()
                .map_err(|e| StoreError::DbError(e.to_string()))?
            {
                let (k, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                let mut d = StoredDevice::from_bytes(g.value())?;
                if d.is_active {
                    d.is_active = false;
                    result.push((k.value().to_string(), d.to_bytes()));
                }
            }
            result
        };
        if updates.is_empty() {
            return Ok(());
        }

        // Phase 2: Write updates
        let txn = self
            .db
            .read()
            .begin_write()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn
                .open_table(DEVICES)
                .map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                table
                    .insert(id.as_str(), bytes.as_slice())
                    .map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit()
            .map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    fn make_store() -> RedbStore {
        RedbStore::new(":memory:", 90).expect("in-memory store")
    }

    fn add_device(store: &RedbStore, id: &str) {
        store
            .upsert_device(
                id,
                "Test Device",
                "192.168.1.1",
                4370,
                "zkteco",
                "UTC",
                None,
                true,
                100000,
            )
            .unwrap();
    }

    #[test]
    fn test_upsert_device_new() {
        let store = make_store();
        add_device(&store, "DEV1");
        let devices = store.get_active_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "DEV1");
        assert_eq!(devices[0].ip, "192.168.1.1");
        assert!(devices[0].is_active);
    }

    #[test]
    fn test_upsert_device_update_preserves_metadata() {
        let store = make_store();
        add_device(&store, "DEV1");
        store
            .update_device_metadata(
                "DEV1",
                "AA:BB",
                "SN123",
                "1.0",
                "Linux",
                "ZK400",
                50,
                1000,
                100,
                50000,
                "2026-01-01 12:00:00",
                200000,
            )
            .unwrap();

        // Re-upsert with different IP
        store
            .upsert_device(
                "DEV1", "Updated", "10.0.0.1", 4370, "zkteco", "UTC", None, true, 300000,
            )
            .unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.ip, "10.0.0.1");
        assert_eq!(dev.mac, Some("AA:BB".into())); // Preserved
        assert_eq!(dev.serial, Some("SN123".into())); // Preserved
    }

    #[test]
    fn test_get_active_devices_filters_inactive() {
        let store = make_store();
        add_device(&store, "DEV1");
        store
            .upsert_device(
                "DEV2", "Inactive", "10.0.0.2", 4370, "zkteco", "UTC", None, false, 100000,
            )
            .unwrap();

        let active = store.get_active_devices().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "DEV1");
    }

    #[test]
    fn test_get_device_metadata_not_found() {
        let store = make_store();
        let dev = store.get_device_metadata("NONEXIST").unwrap();
        assert!(dev.is_none());
    }

    #[test]
    fn test_get_device_metadata_case_insensitive() {
        let store = make_store();
        add_device(&store, "DEV1");
        let dev = store.get_device_metadata("dev1").unwrap();
        assert!(dev.is_some());
    }

    #[test]
    fn test_update_device_metadata() {
        let store = make_store();
        add_device(&store, "DEV1");

        store
            .update_device_metadata(
                "DEV1",
                "AA:BB:CC",
                "SN456",
                "2.0",
                "Linux",
                "ZK800",
                100,
                5000,
                200,
                100000,
                "2026-01-01 15:00:00",
                200000,
            )
            .unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.mac, Some("AA:BB:CC".into()));
        assert_eq!(dev.serial, Some("SN456".into()));
        assert_eq!(dev.firmware, Some("2.0".into()));
        assert_eq!(dev.users_count, Some(100));
        assert_eq!(dev.logs_count, Some(5000));
    }

    #[test]
    fn test_update_device_metadata_nonexistent() {
        let store = make_store();
        // Should not panic on nonexistent device
        store
            .update_device_metadata("NOPE", "AA", "SN", "1.0", "L", "Z", 0, 0, 0, 0, "time", 100)
            .unwrap();
    }

    #[test]
    fn test_update_device_counts() {
        let store = make_store();
        add_device(&store, "DEV1");

        store
            .update_device_counts("DEV1", 42, 1000, 200000)
            .unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.users_count, Some(42));
        assert_eq!(dev.logs_count, Some(1000));
    }

    #[test]
    fn test_update_device_counts_nonexistent() {
        let store = make_store();
        store.update_device_counts("NOPE", 10, 20, 100).unwrap(); // Should not panic
    }

    #[test]
    fn test_deactivate_devices_not_in() {
        let store = make_store();
        add_device(&store, "DEV1");
        add_device(&store, "DEV2");
        add_device(&store, "DEV3");

        let deactivated = store
            .deactivate_devices_not_in(&["DEV1".into(), "DEV3".into()])
            .unwrap();
        assert_eq!(deactivated, 1);

        let active = store.get_active_devices().unwrap();
        assert_eq!(active.len(), 2);
        let ids: Vec<&str> = active.iter().map(|d| d.id.as_str()).collect();
        assert!(ids.contains(&"DEV1"));
        assert!(ids.contains(&"DEV3"));
    }

    #[test]
    fn test_deactivate_devices_not_in_all_active() {
        let store = make_store();
        add_device(&store, "DEV1");
        let deactivated = store.deactivate_devices_not_in(&["DEV1".into()]).unwrap();
        assert_eq!(deactivated, 0);
    }

    #[test]
    fn test_agent_config_crud() {
        let store = make_store();

        // Get nonexistent key
        assert!(store.get_agent_config("key1").unwrap().is_none());

        // Set and get
        store.upsert_agent_config("key1", "value1").unwrap();
        assert_eq!(
            store.get_agent_config("key1").unwrap(),
            Some("value1".into())
        );

        // Update
        store.upsert_agent_config("key1", "updated").unwrap();
        assert_eq!(
            store.get_agent_config("key1").unwrap(),
            Some("updated".into())
        );
    }

    #[test]
    fn test_clear_all_config() {
        let store = make_store();
        store.upsert_agent_config("k1", "v1").unwrap();
        store.upsert_agent_config("k2", "v2").unwrap();

        store.clear_all_config().unwrap();

        assert!(store.get_agent_config("k1").unwrap().is_none());
        assert!(store.get_agent_config("k2").unwrap().is_none());
    }

    #[test]
    fn test_clear_all_config_empty() {
        let store = make_store();
        store.clear_all_config().unwrap(); // Should not panic
    }

    #[test]
    fn test_clear_startup_config() {
        let store = make_store();
        add_device(&store, "DEV1");
        store
            .upsert_agent_config("central_url", "ws://server:3000")
            .unwrap();

        store.clear_startup_config().unwrap();

        // Config should be cleared
        assert!(store.get_agent_config("central_url").unwrap().is_none());

        // Devices should be deactivated
        let active = store.get_active_devices().unwrap();
        assert!(active.is_empty());

        // But device should still exist (just inactive)
        let all = store.get_all_devices().unwrap();
        assert_eq!(all.len(), 1);
        assert!(!all[0].is_active);
    }

    #[test]
    fn test_get_all_devices() {
        let store = make_store();
        add_device(&store, "DEV1");
        store
            .upsert_device(
                "DEV2", "Inactive", "10.0.0.2", 4370, "zkteco", "UTC", None, false, 100,
            )
            .unwrap();

        let all = store.get_all_devices().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_deactivate_all_devices() {
        let store = make_store();
        add_device(&store, "DEV1");
        add_device(&store, "DEV2");

        store.deactivate_all_devices().unwrap();

        let active = store.get_active_devices().unwrap();
        assert!(active.is_empty());

        let all = store.get_all_devices().unwrap();
        assert_eq!(all.len(), 2);
        assert!(all.iter().all(|d| !d.is_active));
    }

    #[test]
    fn test_deactivate_all_devices_already_inactive() {
        let store = make_store();
        store
            .upsert_device("DEV1", "D", "1.1.1.1", 4370, "z", "UTC", None, false, 100)
            .unwrap();
        store.deactivate_all_devices().unwrap(); // Should not panic
    }

    #[test]
    fn test_get_device_timezone_missing_is_an_error() {
        let store = make_store();
        assert!(matches!(
            store.get_device_timezone("NOPE"),
            Err(StoreError::InvalidTimezone)
        ));
    }

    #[test]
    fn test_get_device_timezone_custom() {
        let store = make_store();
        store
            .upsert_device(
                "DEV1",
                "D",
                "1.1.1.1",
                4370,
                "z",
                "Asia/Ho_Chi_Minh",
                None,
                true,
                100,
            )
            .unwrap();
        let tz = store.get_device_timezone("DEV1").unwrap();
        assert_eq!(tz, chrono_tz::Asia::Ho_Chi_Minh);
    }

    #[test]
    fn test_parse_local_to_utc_timestamp() {
        let ts = RedbStore::parse_local_to_utc_timestamp(
            "2026-01-01 08:00:00",
            chrono_tz::Asia::Ho_Chi_Minh,
        )
        .unwrap();
        // 8:00 AM in HCM (UTC+7) = 1:00 AM UTC
        let expected_ts =
            RedbStore::parse_local_to_utc_timestamp("2026-01-01 01:00:00", chrono_tz::UTC).unwrap();
        assert_eq!(ts, expected_ts);
    }

    #[test]
    fn test_parse_explicit_utc_offset_without_device_reinterpretation() {
        let utc = RedbStore::parse_local_to_utc_timestamp(
            "2026-01-01T08:00:00Z",
            chrono_tz::Asia::Ho_Chi_Minh,
        )
        .unwrap();
        let expected =
            RedbStore::parse_local_to_utc_timestamp("2026-01-01 08:00:00", chrono_tz::UTC).unwrap();
        assert_eq!(utc, expected);
    }

    #[test]
    fn test_parse_explicit_numeric_offset() {
        let with_offset = RedbStore::parse_local_to_utc_timestamp(
            "2026-01-01T08:00:00+07:00",
            chrono_tz::America::New_York,
        )
        .unwrap();
        let expected = RedbStore::parse_local_to_utc_timestamp(
            "2026-01-01 08:00:00",
            chrono_tz::Asia::Ho_Chi_Minh,
        )
        .unwrap();
        assert_eq!(with_offset, expected);
    }

    #[test]
    fn test_parse_local_to_utc_timestamp_invalid() {
        let result = RedbStore::parse_local_to_utc_timestamp("not-a-date", chrono_tz::UTC);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_local_to_utc_timestamp_rejects_offsetless_z_like_suffix() {
        let result =
            RedbStore::parse_local_to_utc_timestamp("2026-01-01T08:00:00Z-extra", chrono_tz::UTC);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_local_to_utc_dst_ambiguous() {
        // America/New_York DST ends 2026-11-01 at 02:00 → clocks fall back to 01:00
        // 01:30 exists twice — should pick first (EDT, UTC-4)
        let result = RedbStore::parse_local_to_utc_timestamp(
            "2026-11-01 01:30:00",
            chrono_tz::America::New_York,
        );
        assert!(
            result.is_ok(),
            "Ambiguous time should be resolved, not rejected"
        );
        let ts = result.unwrap();
        // First occurrence: EDT (UTC-4) → 01:30 + 4 = 05:30 UTC
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(utc_dt.hour(), 5);
        assert_eq!(utc_dt.minute(), 30);
    }

    #[test]
    fn test_parse_local_to_utc_dst_gap() {
        // America/New_York DST starts 2026-03-08 at 02:00 → clocks jump to 03:00
        // 02:30 does not exist — should shift forward to 03:30 EDT (UTC-4)
        let result = RedbStore::parse_local_to_utc_timestamp(
            "2026-03-08 02:30:00",
            chrono_tz::America::New_York,
        );
        assert!(result.is_ok(), "DST gap should be resolved by shifting");
        let ts = result.unwrap();
        // After shift: 03:30 EDT (UTC-4) = 07:30 UTC
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(utc_dt.hour(), 7);
        assert_eq!(utc_dt.minute(), 30);
    }

    #[test]
    fn test_parse_local_to_utc_hcm_common_case() {
        // Vietnam (UTC+7) — no DST, most common timezone for this system
        let ts = RedbStore::parse_local_to_utc_timestamp(
            "2026-06-15 08:00:00",
            chrono_tz::Asia::Ho_Chi_Minh,
        )
        .unwrap();
        // 08:00 ICT = 01:00 UTC
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(utc_dt.hour(), 1);
        assert_eq!(utc_dt.minute(), 0);
    }

    #[test]
    fn test_parse_local_to_utc_kolkata_half_hour_offset() {
        // India (UTC+5:30) — half-hour offset
        let ts = RedbStore::parse_local_to_utc_timestamp(
            "2026-03-15 12:00:00",
            chrono_tz::Asia::Kolkata,
        )
        .unwrap();
        // 12:00 IST = 06:30 UTC
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(utc_dt.hour(), 6);
        assert_eq!(utc_dt.minute(), 30);
    }

    #[test]
    fn test_parse_local_to_utc_european_dst_gap() {
        // Europe/Berlin DST starts 2026-03-29 at 02:00 → clocks jump to 03:00
        let result = RedbStore::parse_local_to_utc_timestamp(
            "2026-03-29 02:30:00",
            chrono_tz::Europe::Berlin,
        );
        assert!(result.is_ok(), "European DST gap should be resolved");
        let ts = result.unwrap();
        // After shift: 03:30 CEST (UTC+2) = 01:30 UTC
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(utc_dt.hour(), 1);
        assert_eq!(utc_dt.minute(), 30);
    }

    #[test]
    fn test_parse_local_to_utc_negative_offset_date_boundary() {
        // America/Los_Angeles (UTC-8 in winter)
        // 23:00 PT on Jan 15 = 07:00 UTC on Jan 16 (next day)
        let ts = RedbStore::parse_local_to_utc_timestamp(
            "2026-01-15 23:00:00",
            chrono_tz::America::Los_Angeles,
        )
        .unwrap();
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(
            utc_dt.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 1, 16).unwrap()
        );
        assert_eq!(utc_dt.hour(), 7);
    }

    #[test]
    fn test_parse_local_to_utc_iso_format_fallback() {
        // Test the ISO T-format fallback
        let ts = RedbStore::parse_local_to_utc_timestamp("2026-01-16T08:00:00Z", chrono_tz::UTC)
            .unwrap();
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        assert_eq!(utc_dt.hour(), 8);
    }

    #[test]
    fn test_parse_local_to_utc_consistency_across_methods() {
        // Verify agent produces same UTC instant for same input as a manual conversion
        let tz = chrono_tz::Asia::Ho_Chi_Minh;
        let ts = RedbStore::parse_local_to_utc_timestamp("2026-06-15 14:30:00", tz).unwrap();
        // Manual: 14:30 ICT (UTC+7) = 07:30 UTC
        let expected = chrono::NaiveDate::from_ymd_opt(2026, 6, 15)
            .unwrap()
            .and_hms_opt(7, 30, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        assert_eq!(ts, expected);
    }

    #[test]
    fn test_parse_local_to_utc_australian_dst_gap() {
        // Australia/Sydney DST starts first Sunday of October
        // 2026-10-04 at 02:00 AEST → clocks jump to 03:00 AEDT
        let result = RedbStore::parse_local_to_utc_timestamp(
            "2026-10-04 02:30:00",
            chrono_tz::Australia::Sydney,
        );
        assert!(result.is_ok(), "Australian DST gap should be resolved");
    }

    #[test]
    fn test_parse_local_to_utc_dst_fall_back_picks_first() {
        // Europe/Berlin DST ends 2026-10-25 at 03:00 CEST → falls back to 02:00 CET
        // 02:30 exists twice — should pick first (CEST, UTC+2)
        let ts = RedbStore::parse_local_to_utc_timestamp(
            "2026-10-25 02:30:00",
            chrono_tz::Europe::Berlin,
        )
        .unwrap();
        let utc_dt = chrono::DateTime::from_timestamp(ts, 0).unwrap();
        // First occurrence: CEST (UTC+2) → 02:30 - 2 = 00:30 UTC
        assert_eq!(utc_dt.hour(), 0);
        assert_eq!(utc_dt.minute(), 30);
    }

    #[test]
    fn test_upsert_device_with_password() {
        let store = make_store();
        store
            .upsert_device(
                "DEV1",
                "D",
                "1.1.1.1",
                4370,
                "z",
                "UTC",
                Some("secret"),
                true,
                100,
            )
            .unwrap();
        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.password, Some("secret".into()));
    }

    // === Smart Checksum Memory Tests ===

    #[test]
    fn test_new_device_has_no_checksum() {
        let store = make_store();
        add_device(&store, "DEV1");
        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(
            dev.use_legacy_checksum, None,
            "New device should have no checksum preference"
        );
    }

    #[test]
    fn test_update_device_checksum_legacy() {
        let store = make_store();
        add_device(&store, "DEV1");

        store.update_device_checksum("DEV1", true).unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.use_legacy_checksum, Some(true));
    }

    #[test]
    fn test_update_device_checksum_modern() {
        let store = make_store();
        add_device(&store, "DEV1");

        store.update_device_checksum("DEV1", false).unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.use_legacy_checksum, Some(false));
    }

    #[test]
    fn test_update_device_checksum_overwrite() {
        let store = make_store();
        add_device(&store, "DEV1");

        // First detection: legacy
        store.update_device_checksum("DEV1", true).unwrap();
        assert_eq!(
            store
                .get_device_metadata("DEV1")
                .unwrap()
                .unwrap()
                .use_legacy_checksum,
            Some(true)
        );

        // Second detection: modern (firmware upgrade scenario)
        store.update_device_checksum("DEV1", false).unwrap();
        assert_eq!(
            store
                .get_device_metadata("DEV1")
                .unwrap()
                .unwrap()
                .use_legacy_checksum,
            Some(false)
        );
    }

    #[test]
    fn test_update_device_checksum_nonexistent() {
        let store = make_store();
        // Should not panic on nonexistent device
        store.update_device_checksum("NOPE", true).unwrap();
    }

    #[test]
    fn test_update_device_checksum_case_insensitive() {
        let store = make_store();
        add_device(&store, "DEV1");

        store.update_device_checksum("dev1", true).unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.use_legacy_checksum, Some(true));
    }

    #[test]
    fn test_upsert_preserves_checksum() {
        let store = make_store();
        add_device(&store, "DEV1");

        // Set checksum to legacy
        store.update_device_checksum("DEV1", true).unwrap();

        // Re-upsert (simulating ConfigPush) — checksum should be preserved
        store
            .upsert_device(
                "DEV1", "Updated", "10.0.0.1", 4370, "zkteco", "UTC", None, true, 200000,
            )
            .unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.ip, "10.0.0.1"); // IP updated
        assert_eq!(dev.use_legacy_checksum, Some(true)); // Checksum preserved!
    }

    #[test]
    fn test_checksum_survives_deactivation_reactivation() {
        let store = make_store();
        add_device(&store, "DEV1");
        store.update_device_checksum("DEV1", false).unwrap();

        // Deactivate
        store.deactivate_devices_not_in(&[]).unwrap();
        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert!(!dev.is_active);
        assert_eq!(dev.use_legacy_checksum, Some(false)); // Still preserved

        // Reactivate via upsert
        store
            .upsert_device(
                "DEV1", "D", "1.1.1.1", 4370, "zkteco", "UTC", None, true, 300000,
            )
            .unwrap();
        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert!(dev.is_active);
        assert_eq!(dev.use_legacy_checksum, Some(false)); // Still preserved!
    }

    #[test]
    fn test_checksum_does_not_affect_other_metadata() {
        let store = make_store();
        add_device(&store, "DEV1");
        store
            .update_device_metadata(
                "DEV1",
                "AA:BB",
                "SN1",
                "1.0",
                "Linux",
                "ZK400",
                50,
                1000,
                100,
                50000,
                "2026-01-01 12:00:00",
                200000,
            )
            .unwrap();

        store.update_device_checksum("DEV1", true).unwrap();

        let dev = store.get_device_metadata("DEV1").unwrap().unwrap();
        assert_eq!(dev.mac, Some("AA:BB".into())); // Preserved
        assert_eq!(dev.serial, Some("SN1".into())); // Preserved
        assert_eq!(dev.users_count, Some(50)); // Preserved
        assert_eq!(dev.use_legacy_checksum, Some(true)); // Updated
    }
}
