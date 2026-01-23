use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

pub struct ZKClient {
    stream: TcpStream,
    next_id: u64,
    timeout: Duration,
}

impl ZKClient {
    pub async fn connect(addr: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self { 
            stream, 
            next_id: 1,
            timeout: Duration::from_secs(30) // Default
        })
    }

    pub fn set_timeout(&mut self, seconds: u64) {
        self.timeout = Duration::from_secs(seconds);
    }

    pub async fn call(&mut self, method: &str, params: Value) -> io::Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };

        let mut request_bytes = serde_json::to_vec(&request)?;
        request_bytes.push(b'\n'); // just 

        self.stream.write_all(&request_bytes).await?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        
        // Use the configured timeout for reading response
        let read_future = reader.read_line(&mut line);
        match timeout(self.timeout, read_future).await {
            Ok(result) => result?, // IO Result
            Err(_) => return Err(io::Error::new(io::ErrorKind::TimedOut, "RPC Call Timed Out")),
        };

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        
        if let Some(error) = response.error {
            return Err(io::Error::other(error.message));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use serde_json::json;
    use tokio::time::{timeout, sleep, Duration};

    #[tokio::test]
    async fn test_zk_client_call() {
        timeout(Duration::from_secs(5), async {
            // 1. Mock Server
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut reader = BufReader::new(&mut socket);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                
                let req: JsonRpcRequest = serde_json::from_str(&line).unwrap();

                // Gửi response giả lập kèm ký tự xuống dòng
                let res = json!({
                    "jsonrpc": "2.0",
                    "result": "ok",
                    "id": req.id
                });
                let mut response_bytes = serde_json::to_vec(&res).unwrap();
                response_bytes.push(b'\n');
                socket.write_all(&response_bytes).await.unwrap();
            });

            // 2. Client call
            let mut client = ZKClient::connect(&addr.to_string()).await.unwrap();
            let result = client.call("test.method", json!({})).await.unwrap();

            assert_eq!(result, "ok");
        }).await.expect("Test timed out");
    }

    #[tokio::test]
    async fn test_zk_client_error_response() {
        timeout(Duration::from_secs(5), async {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut reader = BufReader::new(&mut socket);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                
                let req: JsonRpcRequest = serde_json::from_str(&line).unwrap();

                // Trả về lỗi JSON-RPC
                let res = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": "Hardware Error"
                    },
                    "id": req.id
                });
                let mut response_bytes = serde_json::to_vec(&res).unwrap();
                response_bytes.push(b'\n');
                socket.write_all(&response_bytes).await.unwrap();
            });

            let mut client = ZKClient::connect(&addr.to_string()).await.unwrap();
            let result = client.call("test.method", json!({})).await;

            assert!(result.is_err());
            assert_eq!(result.unwrap_err().to_string(), "Hardware Error");
        }).await.expect("Test timed out");
    }

    #[tokio::test]
    async fn test_zk_client_timeout() {
        timeout(Duration::from_secs(5), async {
            // Test that the client times out if server doesn't respond
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();

            let _server = tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut reader = BufReader::new(&mut socket);
                let mut line = String::new();
                // Server reads request but never responds
                let _ = reader.read_line(&mut line).await;
                // Just sleep effectively forever (longer than client timeout)
                sleep(Duration::from_secs(10)).await;
                // keep socket alive
                let _ = socket.write_all(b" ").await;
            });

            let mut client = ZKClient::connect(&addr.to_string()).await.unwrap();
            client.set_timeout(1); // Set short timeout for test
            
            let result = client.call("test.timeout", json!({})).await;

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert_eq!(err.kind(), io::ErrorKind::TimedOut);
        }).await.expect("Test failed to timeout correctly or other error");
    }
}
