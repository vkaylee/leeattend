use std::env;
use log::{info, warn};
use std::path::Path;

pub struct Config {
    pub agent_id: String,
    pub agent_token: String,
    pub central_server_url: String,
    pub database_url: String,
    pub python_path: String,
    pub rpc_timeout_seconds: u64,
    pub poll_interval_seconds: u64,
    pub max_concurrent_workers: usize,
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
            let default_db = format!("sqlite:{}.db?mode=rwc", agent_id);
            fallbacks.push(format!("DATABASE_URL (mặc định: {})", default_db));
            default_db
        });

        let python_path = env::var("PYTHON_PATH").unwrap_or_else(|_| {
            let (rel_path, fallback_path) = if cfg!(target_os = "windows") {
                ("../agent-zk-worker/venv/Scripts/python.exe", "../agent-zk-worker/.venv/Scripts/python.exe")
            } else {
                ("../agent-zk-worker/venv/bin/python3", "../agent-zk-worker/venv_linux/bin/python3")
            };

            if Path::new(rel_path).exists() {
                rel_path.to_string()
            } else {
                fallbacks.push(format!("PYTHON_PATH (mặc định: {})", fallback_path));
                fallback_path.to_string()
            }
        });

        let rpc_timeout_seconds = env::var("RPC_TIMEOUT_SECONDS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(30);

        let max_concurrent_workers = env::var("MAX_CONCURRENT_WORKERS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(5);

        let poll_interval_seconds = env::var("POLL_INTERVAL_SECONDS")
            .unwrap_or_default()
            .parse()
            .unwrap_or(60);

        if !fallbacks.is_empty() {
            warn!("⚠️  PHÁT HIỆN THIẾU CẤU HÌNH .ENV - SỬ DỤNG GIÁ TRỊ MẶC ĐỊNH:");
            for f in fallbacks {
                warn!("   👉 {}", f);
            }
            warn!("------------------------------------------------------------");
        }

        Self {
            agent_id,
            agent_token,
            central_server_url,
            database_url,
            python_path,
            rpc_timeout_seconds,
            poll_interval_seconds,
            max_concurrent_workers,
        }
    }

    pub fn show_summary(&self) {
        info!("============================================================");
        info!("🚀 AGENT CONFIGURATION");
        info!("============================================================");
        info!("Agent ID           : {}", self.agent_id);
        info!("Central Server URL : {}", self.central_server_url);
        info!("Database URL       : {}", self.database_url);
        info!("Python Path        : {}", self.python_path);
        info!("------------------------------------------------------------");
        info!("⚙️  RUNTIME SETTINGS");
        info!("------------------------------------------------------------");
        info!("RPC Timeout        : {}s", self.rpc_timeout_seconds);
        info!("Poll Interval      : {}s", self.poll_interval_seconds);
        info!("Max Workers        : {}", self.max_concurrent_workers);
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

    #[test]
    fn test_config_struct_fields() {
        let config = Config {
            agent_id: "test-agent".to_string(),
            agent_token: "test-token".to_string(),
            central_server_url: "ws://localhost:3001/api/ws".to_string(),
            database_url: "sqlite:test.db".to_string(),
            python_path: "/usr/bin/python3".to_string(),
            rpc_timeout_seconds: 30,
            poll_interval_seconds: 60,
            max_concurrent_workers: 5,
        };

        assert_eq!(config.agent_id, "test-agent");
        assert_eq!(config.agent_token, "test-token");
        assert_eq!(config.central_server_url, "ws://localhost:3001/api/ws");
        assert_eq!(config.database_url, "sqlite:test.db");
        assert_eq!(config.python_path, "/usr/bin/python3");
        assert_eq!(config.rpc_timeout_seconds, 30);
        assert_eq!(config.poll_interval_seconds, 60);
        assert_eq!(config.max_concurrent_workers, 5);
    }

    #[test]
    fn test_config_load_from_env() {
        // SAFETY: Test runs single-threaded, env modification is isolated
        unsafe {
            std::env::set_var("AGENT_ID", "prod-agent-001");
            std::env::set_var("AGENT_TOKEN", "secret-token-123");
            std::env::set_var("CENTRAL_SERVER_URL", "wss://prod.example.com/ws");
            std::env::set_var("DATABASE_URL", "sqlite:prod.db");
            std::env::set_var("RPC_TIMEOUT_SECONDS", "45");
            std::env::set_var("POLL_INTERVAL_SECONDS", "120");
            std::env::set_var("MAX_CONCURRENT_WORKERS", "10");
        }

        let config = Config::load();

        assert_eq!(config.agent_id, "prod-agent-001");
        assert_eq!(config.agent_token, "secret-token-123");
        assert_eq!(config.central_server_url, "wss://prod.example.com/ws");
        assert_eq!(config.database_url, "sqlite:prod.db");
        assert_eq!(config.rpc_timeout_seconds, 45);
        assert_eq!(config.poll_interval_seconds, 120);
        assert_eq!(config.max_concurrent_workers, 10);

        // Cleanup
        unsafe {
            std::env::remove_var("AGENT_ID");
            std::env::remove_var("AGENT_TOKEN");
            std::env::remove_var("CENTRAL_SERVER_URL");
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("RPC_TIMEOUT_SECONDS");
            std::env::remove_var("POLL_INTERVAL_SECONDS");
            std::env::remove_var("MAX_CONCURRENT_WORKERS");
        }
    }

    #[test]
    fn test_config_invalid_numeric_uses_default() {
        // SAFETY: Test runs single-threaded
        unsafe {
            std::env::set_var("RPC_TIMEOUT_SECONDS", "not-a-number");
            std::env::set_var("POLL_INTERVAL_SECONDS", "");
            std::env::set_var("MAX_CONCURRENT_WORKERS", "-5");
        }

        let config = Config::load();

        // Should fall back to defaults for invalid values
        assert_eq!(config.rpc_timeout_seconds, 30);
        assert_eq!(config.poll_interval_seconds, 60);
        // "-5" parses as invalid for usize, falls back to default
        assert_eq!(config.max_concurrent_workers, 5);

        // Cleanup
        unsafe {
            std::env::remove_var("RPC_TIMEOUT_SECONDS");
            std::env::remove_var("POLL_INTERVAL_SECONDS");
            std::env::remove_var("MAX_CONCURRENT_WORKERS");
        }
    }

    #[test]
    fn test_show_summary_does_not_panic() {
        let config = Config {
            agent_id: "test".to_string(),
            agent_token: "token".to_string(),
            central_server_url: "ws://localhost".to_string(),
            database_url: "sqlite:test.db".to_string(),
            python_path: "/usr/bin/python3".to_string(),
            rpc_timeout_seconds: 30,
            poll_interval_seconds: 60,
            max_concurrent_workers: 5,
        };

        // Should not panic
        config.show_summary();
    }
}
