use clap::Parser;
use log::{error, info};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

mod service;
use service::ZkService;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port of the master server to connect to
    /// Port of the master server to connect to
    #[arg(default_value_t = 5000)]
    port: u16,

    /// Address of the master server to connect to
    #[arg(short, long, default_value = "127.0.0.1")]
    master_addr: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();
    let master_addr = format!("{}:{}", args.master_addr, args.port);

    // Shared service state
    let service = Arc::new(Mutex::new(ZkService::new()));

    info!("Starting agent-zk-worker-rs");

    loop {
        info!("🔗 Connecting to Master at {}...", master_addr);
        match TcpStream::connect(&master_addr).await {
            Ok(stream) => {
                info!("✅ Connected to Master on {}", master_addr);
                handle_connection(stream, service.clone()).await;
            }
            Err(e) => {
                error!("❌ Connection refused/error: {}. Retrying in 5s...", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn handle_connection(stream: TcpStream, service: Arc<Mutex<ZkService>>) {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF
                info!("Master closed connection.");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<JsonRpcRequest>(trimmed) {
                    Ok(req) => {
                        info!("📩 RPC Request: {}", req.method);
                        let id = req.id.clone();
                        let result = process_request(req, service.clone()).await;

                        let response = match result {
                            Ok(res) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: Some(res),
                                error: None,
                                id,
                            },
                            Err(e) => JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(e),
                                id,
                            },
                        };

                        if let Ok(json_resp) = serde_json::to_string(&response) {
                            if let Err(e) = writer.write_all(format!("{}\n", json_resp).as_bytes()).await {
                                error!("❌ Failed to write response: {}", e);
                                break;
                            }
                            if let Err(e) = writer.flush().await {
                                error!("❌ Failed to flush response: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to parse request: {}", e);
                        // Send parse error? Ideally yes, but sticking to simple loop for now
                    }
                }
            }
            Err(e) => {
                error!("❌ Error reading line: {}", e);
                break;
            }
        }
    }
}

async fn process_request(req: JsonRpcRequest, service: Arc<Mutex<ZkService>>) -> Result<Value, JsonRpcError> {
    let params = req.params.unwrap_or(Value::Null);
    let mut svc = service.lock().await;

    match req.method.as_str() {
        "common.ping" => Ok(serde_json::json!("pong")),
        "device.connect" => {
             // Implementation pending in service.rs
             svc.connect(params).await
        },
        "device.get_logs" => svc.get_attendance_logs().await,
        "device.get_time" => svc.get_time().await,
        "device.sync_time" => svc.sync_time().await,
        "device.get_info" => svc.get_info().await,
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method {} not found", req.method),
        }),
    }
}
