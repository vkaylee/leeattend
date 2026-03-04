use redb::{ReadableTable, ReadableDatabase};
use super::{RedbStore, StoredDevice, StoreError, DEVICES, AGENT_CONFIG};

impl RedbStore {
    pub fn get_device_timezone(&self, device_id: &str) -> Result<chrono_tz::Tz, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
        if let Some(g) = table.get(device_id).map_err(|e| StoreError::DbError(e.to_string()))? {
            let d = StoredDevice::from_bytes(g.value())?;
            Ok(d.timezone.parse::<chrono_tz::Tz>().unwrap_or(chrono_tz::UTC))
        } else {
            Ok(chrono_tz::UTC)
        }
    }

    pub fn parse_local_to_utc_timestamp(local_time_str: &str, tz: chrono_tz::Tz) -> Result<i64, String> {
        use chrono::{NaiveDateTime, TimeZone};
        let naive_dt = NaiveDateTime::parse_from_str(local_time_str, "%Y-%m-%d %H:%M:%S")
            .or_else(|_| NaiveDateTime::parse_from_str(local_time_str, "%Y-%m-%dT%H:%M:%SZ"))
            .map_err(|e| format!("Failed to parse timestamp: {e}"))?;
        let local_dt = tz.from_local_datetime(&naive_dt).single()
            .ok_or_else(|| format!("Ambiguous local time: {local_time_str}"))?;
        Ok(local_dt.with_timezone(&chrono::Utc).timestamp())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_device(&self, id: &str, name: &str, ip: &str, port: i32, brand: &str, timezone: &str, password: Option<&str>, is_enabled: bool) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();

        // Read existing device data first (separate read txn)
        let existing = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.get(id).map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        let device = StoredDevice {
            id: id.into(), name: name.into(), ip: ip.into(), port: port as i64,
            brand: brand.into(), timezone: timezone.into(), last_seen: Some(now), is_active: is_enabled,
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
        };

        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.insert(id, device.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn get_active_devices(&self) -> Result<Vec<StoredDevice>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut devices = Vec::new();
        for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
            let (_, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            let d = StoredDevice::from_bytes(g.value())?;
            if d.is_active { devices.push(d); }
        }
        Ok(devices)
    }

    pub fn get_device_metadata(&self, id: &str) -> Result<Option<StoredDevice>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
        if let Some(g) = table.get(id).map_err(|e| StoreError::DbError(e.to_string()))? {
            return Ok(Some(StoredDevice::from_bytes(g.value())?));
        }
        // Case-insensitive fallback
        let id_lower = id.to_lowercase();
        for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
            let (k, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            if k.value().to_lowercase() == id_lower {
                return Ok(Some(StoredDevice::from_bytes(g.value())?));
            }
        }
        Ok(None)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_device_metadata(&self, id: &str, mac: &str, serial: &str, fw: &str, plt: &str, hw: &str, users: i64, logs: i64, users_cap: i64, logs_cap: i64, time: &str) -> Result<(), StoreError> {
        let now = chrono::Utc::now().timestamp();

        // Read existing device (separate read txn to avoid borrow conflict)
        let existing = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.get(id).map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        if let Some(mut d) = existing {
            d.mac = Some(mac.into()); d.serial = Some(serial.into());
            d.firmware = Some(fw.into()); d.platform = Some(plt.into());
            d.hw_name = Some(hw.into()); d.users_count = Some(users);
            d.logs_count = Some(logs); d.users_capacity = Some(users_cap);
            d.logs_capacity = Some(logs_cap); d.device_time = Some(time.into());
            d.last_info_update = Some(now);

            let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
                table.insert(id, d.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    pub fn update_device_counts(&self, id: &str, users: i64, logs: i64) -> Result<(), StoreError> {
        // Read existing device
        let key = self.find_device_key_ci(id)?;
        let k = match key { Some(k) => k, None => return Ok(()) };

        let existing = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.get(k.as_str()).map_err(|e| StoreError::DbError(e.to_string()))?
                .and_then(|g| StoredDevice::from_bytes(g.value()).ok())
        };

        if let Some(mut d) = existing {
            if users >= 0 { d.users_count = Some(users); }
            if logs >= 0 { d.logs_count = Some(logs); }
            d.last_info_update = Some(chrono::Utc::now().timestamp());

            let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
            {
                let mut table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
                table.insert(k.as_str(), d.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
            txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        Ok(())
    }

    fn find_device_key_ci(&self, id: &str) -> Result<Option<String>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
        if table.get(id).map_err(|e| StoreError::DbError(e.to_string()))?.is_some() {
            return Ok(Some(id.to_string()));
        }
        let id_lower = id.to_lowercase();
        for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
            let (k, _) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            if k.value().to_lowercase() == id_lower { return Ok(Some(k.value().to_string())); }
        }
        Ok(None)
    }

    pub fn deactivate_devices_not_in(&self, active_ids: &[String]) -> Result<u64, StoreError> {
        // Phase 1: Read all devices to find which ones to deactivate
        let updates: Vec<(String, StoredDevice)> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
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
        if count == 0 { return Ok(0); }

        // Phase 2: Write deactivated devices
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, d) in &updates {
                table.insert(id.as_str(), d.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(count)
    }

    pub fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), StoreError> {
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn.open_table(AGENT_CONFIG).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.insert(key, value).map_err(|e| StoreError::DbError(e.to_string()))?;
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn get_agent_config(&self, key: &str) -> Result<Option<String>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(AGENT_CONFIG).map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(table.get(key).map_err(|e| StoreError::DbError(e.to_string()))?.map(|g| g.value().to_string()))
    }

    pub fn clear_startup_config(&self) -> Result<(), StoreError> {
        // Phase 1: Read all keys and device data
        let config_keys: Vec<String>;
        let device_updates: Vec<(String, StoredDevice)>;
        {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;

            let ct = txn.open_table(AGENT_CONFIG).map_err(|e| StoreError::DbError(e.to_string()))?;
            config_keys = ct.iter().map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| e.ok().map(|(k, _)| k.value().to_string())).collect();

            let dt = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            device_updates = dt.iter().map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| {
                    let (k, g) = e.ok()?;
                    let mut d = StoredDevice::from_bytes(g.value()).ok()?;
                    d.is_active = false;
                    Some((k.value().to_string(), d))
                }).collect();
        }

        // Phase 2: Write changes
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut ct = txn.open_table(AGENT_CONFIG).map_err(|e| StoreError::DbError(e.to_string()))?;
            for k in &config_keys {
                ct.remove(k.as_str()).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        {
            let mut dt = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, d) in &device_updates {
                dt.insert(id.as_str(), d.to_bytes().as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    pub fn get_all_devices(&self) -> Result<Vec<StoredDevice>, StoreError> {
        let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
        let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
        let mut devices = Vec::new();
        for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
            let (_, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
            devices.push(StoredDevice::from_bytes(g.value())?);
        }
        Ok(devices)
    }

    /// Clear all agent configuration entries.
    pub fn clear_all_config(&self) -> Result<(), StoreError> {
        // Phase 1: Read all keys
        let keys: Vec<String> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(AGENT_CONFIG).map_err(|e| StoreError::DbError(e.to_string()))?;
            table.iter().map_err(|e| StoreError::DbError(e.to_string()))?
                .filter_map(|e| e.ok().map(|(k, _)| k.value().to_string()))
                .collect()
        };
        if keys.is_empty() { return Ok(()); }

        // Phase 2: Remove all
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn.open_table(AGENT_CONFIG).map_err(|e| StoreError::DbError(e.to_string()))?;
            for key in &keys {
                let _ = table.remove(key.as_str());
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }

    /// Deactivate all devices (set is_enabled = false).
    pub fn deactivate_all_devices(&self) -> Result<(), StoreError> {
        // Phase 1: Read all devices
        let updates: Vec<(String, Vec<u8>)> = {
            let txn = self.db.begin_read().map_err(|e| StoreError::DbError(e.to_string()))?;
            let table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            let mut result = Vec::new();
            for entry in table.iter().map_err(|e| StoreError::DbError(e.to_string()))? {
                let (k, g) = entry.map_err(|e| StoreError::DbError(e.to_string()))?;
                let mut d = StoredDevice::from_bytes(g.value())?;
                if d.is_active {
                    d.is_active = false;
                    result.push((k.value().to_string(), d.to_bytes()));
                }
            }
            result
        };
        if updates.is_empty() { return Ok(()); }

        // Phase 2: Write updates
        let txn = self.db.begin_write().map_err(|e| StoreError::DbError(e.to_string()))?;
        {
            let mut table = txn.open_table(DEVICES).map_err(|e| StoreError::DbError(e.to_string()))?;
            for (id, bytes) in &updates {
                table.insert(id.as_str(), bytes.as_slice()).map_err(|e| StoreError::DbError(e.to_string()))?;
            }
        }
        txn.commit().map_err(|e| StoreError::DbError(e.to_string()))?;
        Ok(())
    }
}
