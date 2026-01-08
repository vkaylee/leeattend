use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration};
use agent_rust::zk_client::ZKClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Rust Agent Orchestrator...");

    // 1. Start Python Worker using venv
    println!("Starting ZK Python Worker sidecar...");
    let mut child = Command::new("../agent/venv_linux/bin/python3")
        .arg("../agent/zk_worker.py")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Wait for worker to start
    sleep(Duration::from_secs(3)).await;

    // 2. Connect to Worker
    let addr = "127.0.0.1:5000";
    println!("Connecting to Sidecar at {}...", addr);
    let mut client = match ZKClient::connect(addr).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to connect to sidecar: {}", e);
            child.kill().await?;
            return Err(e.into());
        }
    };

    // 3. Example RPC Call
    println!("Orchestrator ready. Testing sidecar connection...");
    match client.call("zk.get_users", json!({"device_id": "BOOT_TEST"})).await {
        Ok(_) => println!("✅ Sidecar communication verified."),
        Err(e) => println!("⚠️ Sidecar reachable but device not connected: {}", e),
    }

    // Keep child alive for demo or handle lifecycle
    println!("Agent running (Press Ctrl+C to stop)...");
    child.wait().await?;
    
    Ok(())
}
