use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Worker: Send + Sync {
    async fn connect(&mut self, ip: &str, port: u16) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_attendance_logs(&mut self) -> Result<Vec<crate::models::AttendanceData>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_info(&mut self, time_sync: &crate::time_sync::TimeSync) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
    /// Lightweight connection probe — returns `true` if device is still reachable.
    /// Used as heartbeat to detect silent UDP session loss between poll cycles.
    /// Implementations should use a cheap operation (e.g. `get_time()`) with a short timeout.
    async fn ping(&mut self) -> bool;
    /// Explicitly disconnect from the device to release connection resources.
    /// Implementations should send protocol-level disconnect commands.
    /// Must be idempotent — safe to call multiple times.
    async fn disconnect(&mut self);
}
