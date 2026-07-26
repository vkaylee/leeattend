pub mod clamav;
pub mod mock;

use crate::errors::AppResult;
use async_trait::async_trait;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScanResult {
    Clean,
    Infected(String),
    Skipped(String),
}

#[async_trait]
pub trait VirusScanner: Send + Sync + Debug {
    async fn scan(&self, data: &[u8]) -> AppResult<ScanResult>;
}

pub struct ScanProvider;

impl ScanProvider {
    /// Build a virus scanner from the centralized env registry.
    pub fn build_from_registry(
        registry: &crate::utils::env_registry::EnvRegistry,
    ) -> std::sync::Arc<dyn VirusScanner> {
        if let Some(host) = registry.get("CLAMD_HOST") {
            if !host.is_empty() {
                let port = registry.get_parsed::<u16>("CLAMD_PORT").unwrap_or(3310);
                tracing::info!("Initializing ClamAvScanner at {}:{}", host, port);
                return std::sync::Arc::new(clamav::ClamAvScanner::new(host.to_string(), port));
            }
        }

        tracing::info!("Initializing MockScanner (CLAMD_HOST not set)");
        std::sync::Arc::new(mock::MockScanner)
    }

    #[allow(clippy::disallowed_methods)]
    pub fn build() -> std::sync::Arc<dyn VirusScanner> {
        if let Ok(host) = std::env::var("CLAMD_HOST") {
            if !host.is_empty() {
                let port = std::env::var("CLAMD_PORT")
                    .unwrap_or_else(|_| "3310".to_string())
                    .parse::<u16>()
                    .unwrap_or(3310);
                tracing::info!("Initializing ClamAvScanner at {}:{}", host, port);
                return std::sync::Arc::new(clamav::ClamAvScanner::new(host, port));
            }
        }

        tracing::info!("Initializing MockScanner (CLAMD_HOST not set)");
        std::sync::Arc::new(mock::MockScanner)
    }
}
