use axum::{
    body::Body,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};
use clap::Parser;
use rust_embed::RustEmbed;
use std::net::SocketAddr;

// Nhúng toàn bộ thư mục output của Next.js vào file exe
#[derive(RustEmbed)]
#[folder = "../dist"]
struct Assets;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long)]
    port: Option<u16>,

    /// Do not open the browser automatically
    #[arg(long, default_value_t = false)]
    no_open: bool,
}

#[tokio::main]
async fn main() {
    // 1. Setup logging
    tracing_subscriber::fmt::init();

    // 2. Parse CLI args
    let args = Args::parse();

    // 3. Determine port: CLI arg > ENV var > Default (3000)
    let port = args.port.unwrap_or_else(|| {
        std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000)
    });

    // 4. Define routes
    let app = Router::new()
        .fallback(static_handler);

    // 5. Listen on 0.0.0.0
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind to port {}: {}", port, e);
            std::process::exit(1);
        }
    };

    tracing::info!("================================================");
    tracing::info!("Dashboard Server is running!");
    tracing::info!("Local:        http://localhost:{}", port);
    tracing::info!("Network:      Access via your machine IP on port {}", port);
    tracing::info!("================================================");

    // 6. Open browser automatically (unless --no-open is set)
    if !args.no_open {
        let _ = opener::open(format!("http://localhost:{}", port));
    }

    // 7. Start server with graceful shutdown
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        tracing::info!("Shutdown signal received, stopping server...");
    };

    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await
    {
        tracing::error!("Server error: {}", e);
        std::process::exit(1);
    }

    tracing::info!("Server stopped gracefully.");
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    // Mặc định trả về index.html nếu path rỗng
    let path = if path.is_empty() { "index.html" } else { path };

    // 1. Thử tìm file chính xác (JS, CSS, images, fonts, etc.)
    if let Some(content) = Assets::get(path) {
        return serve_file(path, content);
    }

    // 2. Thử tìm dạng folder (VD: /agents -> /agents.html hoặc /agents/index.html)
    let html_path = format!("{}.html", path);
    if let Some(content) = Assets::get(&html_path) {
        return serve_file(&html_path, content);
    }

    let index_path = format!("{}/index.html", path);
    if let Some(content) = Assets::get(&index_path) {
        return serve_file(&index_path, content);
    }

    // 3. SPA Fallback: Nếu path không có extension (là route, không phải file tĩnh)
    //    → trả về index.html để React Router xử lý client-side routing
    //    VD: /login, /dashboard, /devices/123 → đều serve index.html
    let is_file_request = path.rsplit('/').next().map_or(false, |last| last.contains('.'));
    if !is_file_request {
        if let Some(content) = Assets::get("index.html") {
            return serve_file("index.html", content);
        }
    }

    // 4. File tĩnh thực sự không tìm thấy → 404
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

fn serve_file(path: &str, content: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    
    ([(header::CONTENT_TYPE, mime.as_ref())], Body::from(content.data)).into_response()
}
