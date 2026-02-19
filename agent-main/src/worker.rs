use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Worker: Send + Sync {
    async fn connect(&mut self, ip: &str, port: u16) -> Result<bool, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_attendance_logs(&mut self) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_info(&mut self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>>;
}
