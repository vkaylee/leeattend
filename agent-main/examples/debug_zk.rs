use log::{debug, error, info, warn};
use rustzk::{ZK, ZKProtocol};
use serde_json::Value;
use std::time::Duration;

#[derive(Debug)]
pub struct ZkError {
    pub message: String,
}

impl std::fmt::Display for ZkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ZkError {}

pub struct ZkWorker {
    zk: Option<ZK>,
    ip: String,
}

impl ZkWorker {
    pub fn new() -> Self {
        Self {
            zk: None,
            ip: String::new(),
        }
    }

    pub async fn connect(
        &mut self,
        ip: &str,
        port: u16,
        protocol: ZKProtocol,
    ) -> Result<bool, ZkError> {
        info!("[{ip}] Connecting via {:?}...", protocol);
        self.ip = ip.to_string();
        let ip_clone = self.ip.clone();
        let mut zk = ZK::new(&ip_clone, port);
        let join_handle = tokio::task::spawn_blocking(move || {
            let res = zk.connect(protocol);
            (zk, res)
        });
        match tokio::time::timeout(Duration::from_secs(10), join_handle).await {
            Ok(Ok((zk_instance, Ok(_)))) => {
                self.zk = Some(zk_instance);
                info!("✅ Connected");
                Ok(true)
            }
            Ok(Ok((_, Err(e)))) => Err(ZkError {
                message: format!("Failed: {e}"),
            }),
            _ => Err(ZkError {
                message: "Timeout".to_string(),
            }),
        }
    }

    pub async fn run_test(&mut self) {
        if self.zk.is_none() {
            return;
        }

        // 1. Firmware
        info!("Step 1: Get Firmware");
        let mut zk = self.zk.take().unwrap();
        let (zk, res) = tokio::task::spawn_blocking(move || {
            let res = zk.get_firmware_version();
            (zk, res)
        })
        .await
        .unwrap();
        match res {
            Ok(fw) => info!("✅ Firmware: {}", fw),
            Err(e) => error!("❌ Firmware failed: {}", e),
        }
        self.zk = Some(zk);

        // 2. Users
        info!("Step 2: Get Users");
        let mut zk = self.zk.take().unwrap();
        let (zk, res) = tokio::task::spawn_blocking(move || {
            let res = zk.get_users();
            (zk, res)
        })
        .await
        .unwrap();
        match res {
            Ok(users) => info!("✅ Users: {}", users.len()),
            Err(e) => error!("❌ Users failed: {}", e),
        }
        self.zk = Some(zk);

        // 3. Attendance
        info!("Step 3: Get Attendance");
        let mut zk = self.zk.take().unwrap();
        let join_handle = tokio::task::spawn_blocking(move || {
            let res = zk.get_attendance();
            (zk, res)
        });
        match tokio::time::timeout(Duration::from_secs(60), join_handle).await {
            Ok(Ok((_, Ok(logs)))) => info!("✅ Attendance: {}", logs.len()),
            Ok(Ok((_, Err(e)))) => error!("❌ Attendance failed: {}", e),
            Err(_) => error!("❌ Attendance timed out (60s)"),
            _ => error!("❌ Task error"),
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let target_ip = "192.168.12.13";
    let mut worker = ZkWorker::new();
    if let Ok(_) = worker.connect(target_ip, 4370, ZKProtocol::TCP).await {
        worker.run_test().await;
    }
}
