use super::{ScanResult, VirusScanner};
use crate::errors::AppResult;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct MockScanner;

#[async_trait]
impl VirusScanner for MockScanner {
    async fn scan(&self, _data: &[u8]) -> AppResult<ScanResult> {
        // Return clean automatically for local development
        // Wait 10ms to simulate network latency
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        Ok(ScanResult::Clean)
    }
}
