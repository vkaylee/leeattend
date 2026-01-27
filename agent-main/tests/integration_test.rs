use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{Duration, timeout};
use agent::zk_client::ZKClient;
use serde_json::json;
use tokio::net::TcpListener;

#[tokio::test]
async fn test_full_process_handshake_new_architecture() {
    timeout(Duration::from_secs(20), async {
        // Bind to port 0 for dynamic port assignment
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("Failed to bind dynamic port");
        let actual_port = listener.local_addr().unwrap().port();

        let mut child = Command::new("../agent-zk-worker/venv/bin/python3")
            .arg("zk_worker.py")
            .arg(actual_port.to_string())
            .current_dir("../agent-zk-worker")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start python worker");

        let (stream, _) = listener.accept().await.expect("Failed to accept connection");
        let mut client = ZKClient::new(stream);

        let result = client.call("common.ping", json!({})).await.expect("RPC ping failed");
        assert_eq!(result, "pong");

        let result_err = client.call("common.invalid_method", json!({})).await;
        assert!(result_err.is_err());
        
        child.kill().await.unwrap();
    }).await.expect("Integration test timed out");
}
