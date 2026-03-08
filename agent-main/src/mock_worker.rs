use crate::worker::Worker;
use async_trait::async_trait;
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
}

#[async_trait]
impl Worker for MockWorker {
    async fn connect(&mut self, _ip: &str, _port: u16) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        // Simulate connection delay
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(true)
    }

    async fn get_attendance_logs(&mut self) -> Result<Vec<crate::models::AttendanceData>, Box<dyn std::error::Error + Send + Sync>> {
        // Return a static mock log similar to the Python version
        let log = crate::models::AttendanceData {
            user_id: "999".to_string(),
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            status: Some(1),
            punch: Some(1),
        };
        Ok(vec![log])
    }

    async fn get_info(&mut self, _time_sync: &crate::time_sync::TimeSync) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
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

    async fn disconnect(&mut self) {
        // No-op for mock workers — no real connection to release
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_worker_capacity() {
        let mut worker = MockWorker::new();
        let info = worker.get_info(&crate::time_sync::TimeSync::new()).await.expect("Failed to get info");
        
        assert_eq!(info["users_count"], 100);
        assert_eq!(info["users_capacity"], 1000);
        assert_eq!(info["logs_capacity"], 10000);
        assert!(info["device_time"].is_string());
        assert_ne!(info["device_time"], "N/A");
    }
}
