use crate::worker::Worker;
use async_trait::async_trait;
use chrono::Local;
use serde_json::Value;
use std::sync::Mutex;
use std::time::Duration;

static MOCK_DETECTED_CHECKSUM: Mutex<Option<bool>> = Mutex::new(None);

pub struct MockWorker;

impl Default for MockWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl MockWorker {
    pub const fn new() -> Self {
        Self
    }

    pub fn set_mock_detected_checksum(val: Option<bool>) {
        if let Ok(mut guard) = MOCK_DETECTED_CHECKSUM.lock() {
            *guard = val;
        }
    }
}

#[async_trait]
impl Worker for MockWorker {
    async fn connect(&mut self, _ip: &str, _port: u16) -> Result<bool, crate::errors::AgentError> {
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(true)
    }

    async fn get_attendance_logs(
        &mut self,
    ) -> Result<Vec<crate::models::AttendanceData>, crate::errors::AgentError> {
        // Return a static mock log similar to the Python version
        let log = crate::models::AttendanceData {
            user_id: "999".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            status: Some(1),
            punch: Some(1),
        };
        Ok(vec![log])
    }

    async fn get_info(
        &mut self,
        _time_sync: &crate::time_sync::TimeSync,
    ) -> Result<Value, crate::errors::AgentError> {
        Ok(serde_json::json!({
            "users_count": 100,
            "users_capacity": 1000,
            "logs_count": 500,
            "logs_capacity": 10000,
            "device_time": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "firmware": "MOCK_FW_v1.0",
            "mac": "00:00:00:00:00:00",
            "serial": "MOCK_SN_12345",
            "platform": "MOCK_PLATFORM",
            "hw_name": "MOCK_DEVICE",
            "last_sync": Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
        }))
    }

    async fn ping(&mut self) -> bool {
        true // Mock is always "connected"
    }

    async fn disconnect(&mut self) {
        // No-op for mock workers — no real connection to release
    }

    fn detected_checksum_mode(&self) -> Option<bool> {
        MOCK_DETECTED_CHECKSUM.lock().ok().and_then(|g| *g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_worker_capacity() {
        let mut worker = MockWorker::new();
        let info = worker
            .get_info(&crate::time_sync::TimeSync::new())
            .await
            .expect("Failed to get info");

        assert_eq!(info["users_count"], 100);
        assert_eq!(info["users_capacity"], 1000);
        assert_eq!(info["logs_capacity"], 10000);
        assert!(info["device_time"].is_string());
        assert_ne!(info["device_time"], "N/A");
    }

    #[tokio::test]
    async fn test_mock_worker_ping_always_true() {
        let mut worker = MockWorker::new();
        assert!(
            worker.ping().await,
            "MockWorker ping should always return true"
        );
    }

    #[tokio::test]
    async fn test_mock_worker_disconnect_no_panic() {
        let mut worker = MockWorker::new();
        worker.disconnect().await; // Should be a no-op, no panic
    }
}
