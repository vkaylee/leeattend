use std::env;
use log::{info, warn};
// use std::path::Path;

pub struct Config {
    pub agent_id: String,
    pub agent_token: String,
    pub central_server_url: String,
    pub database_url: String,
    pub poll_interval_seconds: u64,
    pub max_concurrent_workers: usize,
    pub signature_ttl_days: i64,
    pub signature_cleanup_hours: u64,
    pub inflight_timeout_secs: u64,
}

impl Config {
    pub fn load() -> Self {
        let mut fallbacks: Vec<String> = Vec::new();

        let agent_id = env::var("AGENT_ID").unwrap_or_else(|_| {
            fallbacks.push("AGENT_ID (mặc định: test-agent)".to_string());
            "test-agent".to_string()
        });

        let agent_token = env::var("AGENT_TOKEN").unwrap_or_else(|_| {
            fallbacks.push("AGENT_TOKEN (mặc định: test-token)".to_string());
            "test-token".to_string()
        });

        let central_server_url = env::var("CENTRAL_SERVER_URL").unwrap_or_else(|_| {
            fallbacks.push("CENTRAL_SERVER_URL (mặc định: ws://localhost:3001/api/ws)".to_string());
            "ws://localhost:3001/api/ws".to_string()
        });

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            let default_db = format!("sqlite:{agent_id}.db?mode=rwc");
            fallbacks.push(format!("DATABASE_URL (mặc định: {default_db})"));
            default_db
        });


        let max_concurrent_workers = env::var("MAX_CONCURRENT_WORKERS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(5);

        let poll_interval_seconds = env::var("POLL_INTERVAL_SECONDS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(300);

        let signature_ttl_days = env::var("SIGNATURE_TTL_DAYS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(90);

        let signature_cleanup_hours = env::var("SIGNATURE_CLEANUP_HOURS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(24);

        let inflight_timeout_secs = env::var("INFLIGHT_TIMEOUT_SECS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(300);

        if !fallbacks.is_empty() {
            warn!("⚠️  PHÁT HIỆN THIẾU CẤU HÌNH .ENV - SỬ DỤNG GIÁ TRỊ MẶC ĐỊNH:");
            for f in fallbacks {
                warn!("   👉 {f}");
            }
            warn!("------------------------------------------------------------");
        }

        Self {
            agent_id,
            agent_token,
            central_server_url,
            database_url,
            poll_interval_seconds,
            max_concurrent_workers,
            signature_ttl_days,
            signature_cleanup_hours,
            inflight_timeout_secs,
        }
    }

    pub fn show_summary(&self) {
        info!("============================================================");
        info!("🚀 AGENT CONFIGURATION");
        info!("============================================================");
        info!("Agent ID           : {}", self.agent_id);
        info!("Central Server URL : {}", self.central_server_url);
        info!("Database URL       : {}", self.database_url);
        info!("------------------------------------------------------------");
        info!("⚙️  RUNTIME SETTINGS");
        info!("------------------------------------------------------------");
        info!("Poll Interval      : {}s", self.poll_interval_seconds);
        info!("Max Workers        : {}", self.max_concurrent_workers);
        info!("------------------------------------------------------------");
        info!("🧹 MEMORY MANAGEMENT");
        info!("------------------------------------------------------------");
        info!("Signature TTL      : {} days", self.signature_ttl_days);
        info!("Signature Cleanup  : every {}h", self.signature_cleanup_hours);
        info!("Inflight Timeout   : {}s", self.inflight_timeout_secs);
        info!("------------------------------------------------------------");
        info!("🔄 RETRY & BACKOFF CONSTANTS");
        info!("------------------------------------------------------------");
        info!("Sync Backoff Cap   : 300s (5 min) ±10% jitter");
        info!("Sync Max Shift     : 2^8 = 256s before cap");
        info!("Sidecar Max Delay  : 60s ±25% jitter");
        info!("Sidecar Max Retry  : 10 attempts");
        info!("Sidecar Stability  : 30s");
        info!("============================================================");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to ensure env-modifying tests don't run in parallel
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_config_struct_fields() {
        let config = Config {
            agent_id: "test-agent".to_string(),
            agent_token: "test-token".to_string(),
            central_server_url: "ws://localhost:3001/api/ws".to_string(),
            database_url: "sqlite:test.db".to_string(),
            poll_interval_seconds: 300,
            max_concurrent_workers: 5,
            signature_ttl_days: 90,
            signature_cleanup_hours: 24,
            inflight_timeout_secs: 300,
        };

        assert_eq!(config.agent_id, "test-agent");
        assert_eq!(config.agent_token, "test-token");
        assert_eq!(config.central_server_url, "ws://localhost:3001/api/ws");
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.poll_interval_seconds, 300);
        assert_eq!(config.max_concurrent_workers, 5);
        assert_eq!(config.signature_ttl_days, 90);
        assert_eq!(config.signature_cleanup_hours, 24);
        assert_eq!(config.inflight_timeout_secs, 300);
    }

    #[test]
    fn test_config_load_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();

        // SAFETY: Mutex ensures no parallel env modification
        unsafe {
            std::env::set_var("AGENT_ID", "prod-agent-001");
            std::env::set_var("AGENT_TOKEN", "secret-token-123");
            std::env::set_var("CENTRAL_SERVER_URL", "wss://prod.example.com/ws");
            std::env::set_var("DATABASE_URL", "sqlite:prod.db");
            std::env::set_var("POLL_INTERVAL_SECONDS", "120");
            std::env::set_var("MAX_CONCURRENT_WORKERS", "10");
            std::env::set_var("SIGNATURE_TTL_DAYS", "30");
            std::env::set_var("SIGNATURE_CLEANUP_HOURS", "12");
            std::env::set_var("INFLIGHT_TIMEOUT_SECS", "600");
        }

        let config = Config::load();

        assert_eq!(config.agent_id, "prod-agent-001");
        assert_eq!(config.agent_token, "secret-token-123");
        assert_eq!(config.central_server_url, "wss://prod.example.com/ws");
        assert_eq!(config.database_url, "sqlite:prod.db");
        assert_eq!(config.poll_interval_seconds, 120);
        assert_eq!(config.max_concurrent_workers, 10);
        assert_eq!(config.signature_ttl_days, 30);
        assert_eq!(config.signature_cleanup_hours, 12);
        assert_eq!(config.inflight_timeout_secs, 600);

        // Cleanup
        unsafe {
            std::env::remove_var("AGENT_ID");
            std::env::remove_var("AGENT_TOKEN");
            std::env::remove_var("CENTRAL_SERVER_URL");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("POLL_INTERVAL_SECONDS");
            std::env::remove_var("MAX_CONCURRENT_WORKERS");
            std::env::remove_var("SIGNATURE_TTL_DAYS");
            std::env::remove_var("SIGNATURE_CLEANUP_HOURS");
            std::env::remove_var("INFLIGHT_TIMEOUT_SECS");
        }
    }

    #[test]
    fn test_config_invalid_numeric_uses_default() {
        let _lock = ENV_MUTEX.lock().unwrap();

        // SAFETY: Mutex ensures no parallel env modification
        unsafe {
            // Clear all config env vars first
            std::env::remove_var("AGENT_ID");
            std::env::remove_var("AGENT_TOKEN");
            std::env::remove_var("CENTRAL_SERVER_URL");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("RPC_TIMEOUT_SECONDS");
            std::env::remove_var("POLL_INTERVAL_SECONDS");
            std::env::remove_var("MAX_CONCURRENT_WORKERS");
            std::env::remove_var("SIGNATURE_TTL_DAYS");
            std::env::remove_var("SIGNATURE_CLEANUP_HOURS");
            std::env::remove_var("INFLIGHT_TIMEOUT_SECS");

            std::env::set_var("POLL_INTERVAL_SECONDS", "");
            std::env::set_var("MAX_CONCURRENT_WORKERS", "-5");
            std::env::set_var("SIGNATURE_TTL_DAYS", "abc");
            std::env::set_var("SIGNATURE_CLEANUP_HOURS", "");
            std::env::set_var("INFLIGHT_TIMEOUT_SECS", "-1");
        }

        let config = Config::load();

        // Should fall back to defaults for invalid values
        assert_eq!(config.poll_interval_seconds, 300);
        // "-5" parses as invalid for usize, falls back to default
        assert_eq!(config.max_concurrent_workers, 5);
        assert_eq!(config.signature_ttl_days, 90);
        assert_eq!(config.signature_cleanup_hours, 24);
        assert_eq!(config.inflight_timeout_secs, 300);

        // Cleanup
        unsafe {
            std::env::remove_var("RPC_TIMEOUT_SECONDS");
            std::env::remove_var("POLL_INTERVAL_SECONDS");
            std::env::remove_var("MAX_CONCURRENT_WORKERS");
            std::env::remove_var("SIGNATURE_TTL_DAYS");
            std::env::remove_var("SIGNATURE_CLEANUP_HOURS");
            std::env::remove_var("INFLIGHT_TIMEOUT_SECS");
        }
    }

    #[test]
    fn test_show_summary_does_not_panic() {
        let config = Config {
            agent_id: "test".to_string(),
            agent_token: "token".to_string(),
            central_server_url: "ws://localhost".to_string(),
            database_url: "sqlite:test.db".to_string(),
            poll_interval_seconds: 300,
            max_concurrent_workers: 5,
            signature_ttl_days: 90,
            signature_cleanup_hours: 24,
            inflight_timeout_secs: 300,
        };

        // Should not panic
        config.show_summary();
    }
}
