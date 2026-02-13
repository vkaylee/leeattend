use std::process::Stdio;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{Duration, timeout, sleep};
use agent::zk_client::ZKClient;
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_full_process_handshake_new_architecture() {
    timeout(Duration::from_secs(20), async {
        // Determine worker directory (supports both host and container paths)
        let worker_dir = if PathBuf::from("/workspace/agent-zk-worker").exists() {
            // Running in Docker container
            PathBuf::from("/workspace/agent-zk-worker")
        } else {
            // Running on host
            PathBuf::from("../agent-zk-worker")
        };

        // Prefer venv Python on host, system Python in container (to avoid GLIBC mismatch)
        let python_bin = if PathBuf::from("/workspace").exists() {
            // Docker container - use system Python (dependencies installed via pip)
            PathBuf::from("/usr/bin/python3")
        } else {
            // Host - use venv
            let venv_python = worker_dir.join("venv/bin/python3");
            if venv_python.exists() {
                venv_python
            } else {
                PathBuf::from("python3")
            }
        };

        // Skip test if worker script doesn't exist
        let worker_script = worker_dir.join("zk_worker.py");
        if !worker_script.exists() {
            eprintln!("⚠️  Skipping test: Worker script not found at {:?}", worker_script);
            return;
        }

        // Bind to port 0 for dynamic port assignment
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind dynamic port");
        let actual_port = listener.local_addr().unwrap().port();

        let mut child = Command::new(&python_bin)
            .arg("zk_worker.py")
            .arg(actual_port.to_string())
            .current_dir(&worker_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start python worker");

        // Give worker time to start
        sleep(Duration::from_millis(500)).await;

        let (stream, _) = listener.accept().await.expect("Failed to accept connection");
        let mut client = ZKClient::new(stream);

        let result = client.call("common.ping", json!({})).await.expect("RPC ping failed");
        assert_eq!(result, "pong");

        let result_err = client.call("common.invalid_method", json!({})).await;
        assert!(result_err.is_err());

        child.kill().await.unwrap();
    }).await.expect("Integration test timed out");
}
