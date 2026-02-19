use serde_json::Value;
use std::time::Duration;
use chrono::Local;

pub struct MockWorker;

impl Default for MockWorker {
    fn default() -> Self {
        Self::new()
    }
}

impl MockWorker {
    pub fn new() -> Self {
        Self
    }

    pub async fn connect(&self, _ip: &str, _port: u16) -> Result<bool, String> {
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(true)
    }

    pub async fn get_attendance_logs(&self) -> Result<Vec<Value>, String> {
        // Return a static mock log similar to the Python version
        let log = serde_json::json!({
            "user_id": "999",
            "timestamp": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            "status": 1,
            "punch": 1
        });
        Ok(vec![log])
    }

    pub async fn get_info(&self) -> Result<Value, String> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_worker_capacity() {
        let worker = MockWorker::new();
        let info = worker.get_info().await.expect("Failed to get info");
        
        assert_eq!(info["users_count"], 100);
        assert_eq!(info["users_capacity"], 1000);
        assert_eq!(info["logs_capacity"], 10000);
        assert!(info["device_time"].is_string());
        assert_ne!(info["device_time"], "N/A");
    }
}
