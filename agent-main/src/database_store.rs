use crate::models::AttendanceData;
use crate::redb_store::{StoreError, StoredDevice, StoredLog};
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait DatabaseStore: Send + Sync {
    async fn save_raw_attendance(
        &self,
        device_id: String,
        data: AttendanceData,
    ) -> Result<i64, StoreError>;

    async fn save_raw_attendance_batch(
        &self,
        batch: Vec<(Arc<String>, AttendanceData)>,
    ) -> Result<usize, StoreError>;

    async fn get_unsynced_logs(&self, limit: usize) -> Result<Vec<StoredLog>, StoreError>;

    async fn count_unsynced_logs(&self) -> usize;

    async fn mark_in_flight(&self, ids: Vec<i64>);

    async fn mark_synced(&self, ids: Vec<i64>);

    async fn reset_to_unsynced(&self, id: i64);

    async fn mark_synced_by_key(&self, composite_key: String) -> bool;

    async fn mark_synced_by_msg_id(&self, msg_id: uuid::Uuid) -> bool;

    async fn reset_to_unsynced_by_key(&self, composite_key: String) -> bool;

    async fn reset_to_unsynced_by_msg_id(&self, msg_id: uuid::Uuid) -> bool;

    async fn evict_old_signatures(&self) -> (usize, usize, usize);

    async fn unload_device_memory(&self, device_id: String) -> Result<(), StoreError>;

    fn get_active_devices(&self) -> Result<Vec<StoredDevice>, StoreError>;

    fn get_agent_config(&self, key: &str) -> Result<Option<String>, StoreError>;

    fn get_msg_id(&self, id: i64) -> Option<uuid::Uuid>;

    fn upsert_agent_config(&self, key: &str, value: &str) -> Result<(), StoreError>;

    fn deactivate_devices_not_in(&self, device_ids: &[String]) -> Result<u64, StoreError>;

    #[allow(clippy::too_many_arguments)]
    fn upsert_device(
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
    ) -> Result<(), StoreError>;

    fn get_device_metadata(&self, device_id: &str) -> Result<Option<StoredDevice>, StoreError>;

    #[allow(clippy::too_many_arguments)]
    fn update_device_metadata(
        &self,
        device_id: &str,
        mac: &str,
        serial: &str,
        firmware: &str,
        platform: &str,
        hw_name: &str,
        users_count: i64,
        logs_count: i64,
        users_capacity: i64,
        logs_capacity: i64,
        device_time: &str,
        now_ts: i64,
    ) -> Result<(), StoreError>;

    fn update_device_checksum(
        &self,
        device_id: &str,
        use_legacy_checksum: bool,
    ) -> Result<(), StoreError>;

    fn update_device_counts(
        &self,
        device_id: &str,
        users_count: i64,
        logs_count: i64,
        now_ts: i64,
    ) -> Result<(), StoreError>;

    fn move_expired_logs_to_dlq(&self, max_retries: i64) -> Result<u64, StoreError>;

    fn reset_stale_syncing(&self, stale_timeout_secs: i64) -> Result<u64, StoreError>;

    fn cleanup_old_synced_logs(&self, retention_days: i64) -> Result<u64, StoreError>;

    fn compact(&self) -> Result<bool, StoreError>;

    fn recover_from_crash(&self) -> Result<u64, StoreError>;
}
