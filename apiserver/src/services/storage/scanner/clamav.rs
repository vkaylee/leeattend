use super::{ScanResult, VirusScanner};
use crate::errors::{AppError, AppResult};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ClamAvScanner {
    host: String,
    port: u16,
}

impl ClamAvScanner {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait]
impl VirusScanner for ClamAvScanner {
    async fn scan(&self, data: &[u8]) -> AppResult<ScanResult> {
        let addr = format!("{}:{}", self.host, self.port);
        let data_vec = data.to_vec();

        let response = tokio::task::spawn_blocking(move || {
            let connection = clamav_client::Tcp { host_address: addr };
            clamav_client::scan_buffer(
                &data_vec,
                connection,
                Some(10000), // 10 seconds timeout in milliseconds
            )
        })
        .await
        .map_err(|e| AppError::Internal(format!("ClamAV scan join error: {e}")))?
        .map_err(|e| AppError::Internal(format!("ClamAV scan connection error: {e}")))?;

        if clamav_client::clean(&response).unwrap_or(false) {
            Ok(ScanResult::Clean)
        } else {
            let response_str = String::from_utf8_lossy(&response);
            Ok(ScanResult::Infected(response_str.trim().to_string()))
        }
    }
}
