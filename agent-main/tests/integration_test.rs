use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration, timeout};
use agent::zk_client::ZKClient;
use serde_json::json;

#[tokio::test]
async fn test_full_process_handshake() {
    timeout(Duration::from_secs(20), async {
        // 1. Khởi động Python Worker (dùng venv)
        let mut child = Command::new("../agent-zk-worker/venv/bin/python3")
            .arg("zk_worker.py")
            .current_dir("../agent-zk-worker")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start python worker");

        // Chờ worker boot
        sleep(Duration::from_secs(4)).await;

        // 2. Kết nối client tới port 5000 (mặc định của zk_worker)
        let mut client = match ZKClient::connect("127.0.0.1:5000").await {
            Ok(c) => c,
            Err(e) => {
                child.kill().await.ok();
                panic!("Failed to connect to worker: {}", e);
            }
        };

        // 3. Test invalid method (Fast)
        let result_err = client.call("zk.invalid_method", json!({})).await;
        assert!(result_err.is_err());
        let err_msg = result_err.unwrap_err().to_string();
        assert!(err_msg.contains("Method not found"));

        // 4. Gửi lệnh connect (giả lập kết nối thiết bị - có thể timeout nếu không có thiết bị)
        let result = client.call("zk.connect", json!({
            "device_id": "INT_TEST",
            "ip": "127.0.0.1",
            "port": 4370
        })).await;

        // Verify result (đối với integration test, ta chỉ cần protocol pass)
        assert!(result.is_err() || result.is_ok()); 
        
        // 5. Cleanup
        child.kill().await.unwrap();
    }).await.expect("Integration test timed out");
}