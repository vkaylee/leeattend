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
#[folder = "../out"]
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
    
    println!("================================================");
    println!("Dashboard Server is running!");
    println!("Local:        http://localhost:{}", port);
    println!("Network:      Access via your machine IP on port {}", port);
    println!("================================================");

    // 6. Open browser automatically (unless --no-open is set)
    if !args.no_open {
        let _ = opener::open(format!("http://localhost:{}", port));
    }

    // 7. Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');
    
    // Mặc định trả về index.html nếu path rỗng
    let path = if path.is_empty() { "index.html" } else { path };

    // 1. Thử tìm file chính xác
    if let Some(content) = Assets::get(path) {
        return serve_file(path, content);
    }

    // 2. Nếu không thấy, thử tìm dạng folder (VD: /agents -> /agents.html hoặc /agents/index.html)
    // Next.js static export thường tạo agents.html cho route /agents
    let html_path = format!("{}.html", path);
    if let Some(content) = Assets::get(&html_path) {
        return serve_file(&html_path, content);
    }

    let index_path = format!("{}/index.html", path);
    if let Some(content) = Assets::get(&index_path) {
        return serve_file(&index_path, content);
    }

    // 3. Fallback về 404.html hoặc index.html (tùy logic)
    if let Some(content) = Assets::get("404.html") {
        return serve_file("404.html", content);
    }
    
    (StatusCode::NOT_FOUND, "404 Not Found").into_response()
}

fn serve_file(path: &str, content: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    
    ([(header::CONTENT_TYPE, mime.as_ref())], Body::from(content.data)).into_response()
}
