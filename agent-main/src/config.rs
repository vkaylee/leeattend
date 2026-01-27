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
        info!("🚀 CẤU HÌNH HIỆN TẠI:");
        info!("   • Agent ID     : {}", self.agent_id);
        info!("   • Central URL  : {}", self.central_server_url);
        info!("   • Database     : {}", self.database_url);
        info!("   • Python Path  : {}", self.python_path);
        info!("   • RPC Timeout  : {}s", self.rpc_timeout_seconds);
        info!("   • Poll Interval: {}s", self.poll_interval_seconds);
        info!("   • Max Workers   : {}", self.max_concurrent_workers);
        info!("============================================================");
    }
}
