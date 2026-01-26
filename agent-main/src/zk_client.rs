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
    pub fn new(stream: TcpStream) -> Self {
        Self { 
            stream, 
            next_id: 1,
            timeout: Duration::from_secs(30)
        }
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
        request_bytes.push(b'\n');

        self.stream.write_all(&request_bytes).await?;

        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        
        let read_future = reader.read_line(&mut line);
        match timeout(self.timeout, read_future).await {
            Ok(result) => {
                if result? == 0 {
                    return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Sidecar connection closed"));
                }
            },
            Err(_) => return Err(io::Error::new(io::ErrorKind::TimedOut, "RPC Call Timed Out")),
        };

        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        
        if let Some(error) = response.error {
            return Err(io::Error::other(error.message));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }

    pub async fn read_next_message(&mut self) -> io::Result<String> {
        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            return Err(io::Error::new(io::ErrorKind::ConnectionAborted, "Connection closed"));
        }
        Ok(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use serde_json::json;

    #[tokio::test]
    async fn test_zk_client_call_server_mode() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            let mut stream = TcpStream::connect(addr).await.unwrap();
            let mut reader = BufReader::new(&mut stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            
            let req: JsonRpcRequest = serde_json::from_str(&line).unwrap();
            let res = json!({
                "jsonrpc": "2.0",
                "result": "pong",
                "id": req.id
            });
            let mut res_bytes = serde_json::to_vec(&res).unwrap();
            res_bytes.push(b'\n');
            stream.write_all(&res_bytes).await.unwrap();
        });

        let (stream, _) = listener.accept().await.unwrap();
        let mut client = ZKClient::new(stream);
        let result = client.call("zk.ping", json!({})).await.unwrap();

        assert_eq!(result, "pong");
    }
}
