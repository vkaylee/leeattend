use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::time::Duration;

/// Default read timeout for RPC calls (5 minutes for slow device connections)
const DEFAULT_READ_TIMEOUT_SECS: u64 = 300;
/// Default write timeout for RPC requests
const DEFAULT_WRITE_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<Value>,
    pub id: u64,
}

pub struct WorkerClient {
    stream: TcpStream,
    next_id: u64,
}

impl WorkerClient {
    /// Create a new client by connecting to the given address
    pub fn new(addr: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr)?;
        stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS)))?;
        stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_WRITE_TIMEOUT_SECS)))?;
        Ok(Self { stream, next_id: 1 })
    }

    /// Create a client from an existing TCP stream (for accepted connections)
    ///
    /// Note: If the stream comes from tokio::net::TcpStream.into_std(),
    /// it may still be in non-blocking mode. This method sets it to blocking.
    pub fn from_stream(stream: TcpStream) -> std::io::Result<Self> {
        // Ensure socket is in blocking mode (tokio sockets are non-blocking)
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS)))?;
        stream.set_write_timeout(Some(Duration::from_secs(DEFAULT_WRITE_TIMEOUT_SECS)))?;
        Ok(Self { stream, next_id: 1 })
    }

    pub fn call(&mut self, method: &str, params: Value) -> std::io::Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id,
        };

        let mut request_str = serde_json::to_string(&request)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        request_str.push('\n');
        self.stream.write_all(request_str.as_bytes())?;
        self.stream.flush()?;

        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.is_empty() {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Sidecar closed connection"));
        }

        let response: JsonRpcResponse = serde_json::from_str(&line)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if let Some(error) = response.error {
            return Err(std::io::Error::other(format!("RPC Error: {error}")));
        }

        Ok(response.result.unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn test_from_stream_creates_client() {
        // Start a mock server
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn server thread
        let server_handle = thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream // Return the accepted stream
        });

        // Connect as client
        let client_stream = TcpStream::connect(addr).unwrap();

        // Create WorkerClient from the client stream
        let client = WorkerClient::from_stream(client_stream);
        assert!(client.is_ok(), "from_stream should succeed");

        let client = client.unwrap();
        assert_eq!(client.next_id, 1, "Initial next_id should be 1");

        // Cleanup
        let _ = server_handle.join();
    }

    #[test]
    fn test_from_stream_sets_timeouts() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let _ = listener.accept();
        });

        let stream = TcpStream::connect(addr).unwrap();
        let client = WorkerClient::from_stream(stream).unwrap();

        // Verify timeouts are set by checking the stream's timeout
        let read_timeout = client.stream.read_timeout().unwrap();
        let write_timeout = client.stream.write_timeout().unwrap();

        assert_eq!(read_timeout, Some(Duration::from_secs(DEFAULT_READ_TIMEOUT_SECS)));
        assert_eq!(write_timeout, Some(Duration::from_secs(DEFAULT_WRITE_TIMEOUT_SECS)));
    }

    #[test]
    fn test_call_sends_jsonrpc_request() {
        use std::io::{BufRead, BufReader, Write};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Server echoes back a valid JSON-RPC response
        let server_handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            // Parse request to get ID
            let req: JsonRpcRequest = serde_json::from_str(&line).unwrap();

            // Send response
            let response = format!(
                r#"{{"jsonrpc":"2.0","result":"ok","id":{}}}"#,
                req.id
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(b"\n").unwrap();
            stream.flush().unwrap();
        });

        let stream = TcpStream::connect(addr).unwrap();
        let mut client = WorkerClient::from_stream(stream).unwrap();

        let result = client.call("test.method", serde_json::json!({"key": "value"}));
        assert!(result.is_ok(), "call should succeed");
        assert_eq!(result.unwrap(), serde_json::json!("ok"));

        server_handle.join().unwrap();
    }

    #[test]
    fn test_from_stream_converts_nonblocking_to_blocking() {
        // This test verifies that from_stream() correctly converts
        // a non-blocking socket (like from tokio) to blocking mode

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        // Server that responds to RPC
        let server_handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            let req: JsonRpcRequest = serde_json::from_str(&line).unwrap();
            let response = format!(
                r#"{{"jsonrpc":"2.0","result":"blocking_works","id":{}}}"#,
                req.id
            );
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(b"\n").unwrap();
            stream.flush().unwrap();
        });

        // Create client socket and set to NON-BLOCKING (simulating tokio)
        let stream = TcpStream::connect(addr).unwrap();
        stream.set_nonblocking(true).unwrap();

        // from_stream should convert to blocking mode
        let mut client = WorkerClient::from_stream(stream).unwrap();

        // This call would fail with EAGAIN if socket was still non-blocking
        let result = client.call("test.method", serde_json::json!({}));
        assert!(result.is_ok(), "Call should succeed with blocking socket");
        assert_eq!(result.unwrap(), serde_json::json!("blocking_works"));

        server_handle.join().unwrap();
    }
}
