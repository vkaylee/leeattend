# Deployment Guide

Tài liệu này hướng dẫn cách triển khai hệ thống LeeAttend bao gồm **API Server (Rust)** và **Agent (Python)**.

---

## 🚀 1. API Server (Rust) - Central Server

API Server đóng vai trò là trung tâm tiếp nhận dữ liệu từ các Agent ở các chi nhánh.

### Yêu cầu hệ thống
- Rust 1.75+
- PostgreSQL 14+

### Các bước triển khai
1. Di chuyển vào thư mục `apiserver`:
   ```bash
   cd apiserver
   ```
2. Cấu hình môi trường (`.env`):
   ```env
   API_HOST=0.0.0.0
   API_PORT=3000
   DATABASE_URL=postgres://user:pass@host:5432/db
   ALLOWED_ORIGINS=http://localhost:3001
   RUST_LOG=apiserver=debug,sqlx=info
   ```
3. Chạy Server:
   ```bash
   # Tự động migrate database khi khởi động
   cargo run --release
   ```
Server sẽ mặc định chạy tại cổng `3000`.

---

## 🤖 2. Agent (Rust Core + Python Sidecar)

Agent được thiết kế theo mô hình **Orchestrator (Rust)** quản lý **Worker (Python)** để đảm bảo hiệu năng và độ ổn định.

### Yêu cầu hệ thống
- Rust (để build Orchestrator)
- Python 3.10+ (để build Worker)
- Kết nối mạng đến API Server

### 🐧 Quy trình Đóng gói & Cài đặt

Chúng tôi cung cấp script tự động để đóng gói toàn bộ Agent thành các file binary gọn nhẹ.

#### Bước 1: Đóng gói Python ZK Worker
```bash
cd agent-zk-worker

# Linux
chmod +x build_linux.sh
./build_linux.sh

# Windows
build_exe.bat
```
Sau bước này, bạn sẽ có file `agent-zk-worker` (hoặc `.exe`) trong thư mục `agent-zk-worker/dist`.

#### Bước 2: Build Rust Orchestrator
```bash
cd agent-main
cargo build --release
```
File kết quả: `target/release/agent` (hoặc `agent.exe`).

#### Bước 3: Triển khai (Deploy)
Tạo một thư mục chứa ứng dụng (ví dụ `/opt/leeattend-agent`) và copy các file sau vào:

1.  **File thực thi chính**: `agent` (từ bước 2)
2.  **Sidecar Worker**: `agent-zk-worker` (từ bước 1)
3.  **Cấu hình**: File `.env` và `devices.json`

Cấu trúc thư mục cuối cùng sẽ trông như sau:
```text
/deploy-folder/
├── agent              # Chạy file này
├── agent-zk-worker    # Để yên đây (Agent sẽ tự gọi)
├── .env
└── devices.json
```

### ⚙️ Cấu hình (.env)

Tạo file `.env` tại thư mục deploy:
```env
# Central Server
CENTRAL_SERVER_URL=ws://your-api-server:3000/ws
AGENT_TOKEN=your_secure_token
AGENT_ID=BRANCH_NAME_01

# Database Local
DATABASE_URL=sqlite:///./attendance.db

# Sync Config
SYNC_BATCH_SIZE=100
SYNC_INTERVAL=0.5
```

### 🏃 Vận hành

Chỉ cần chạy file `agent`:
```bash
./agent
```
Nó sẽ tự động khởi động `agent-zk-worker`, kết nối database và bắt đầu đồng bộ.

### 🔧 CLI Commands

Agent provides several command-line utilities for debugging and operations:

**View Help:**
```bash
./agent --help
```

**View Failed Sync Logs:**
```bash
# Show last 50 failed logs (default limit)
./agent failed-logs

# Show last N failed logs
./agent failed-logs 100

# Alias
./agent failed 50
```

**Output Format:**
```
Failed Sync Logs (limit: 50):
- ID: 123 | Device: ZK1 | Time: 2026-01-28 10:30:45 | Status: Retry pending
- ID: 124 | Device: ZK2 | Time: 2026-01-28 10:31:20 | Status: Retry pending
```

### 🧹 Automatic Maintenance

The Agent automatically performs these maintenance tasks:

**Two-Phase Startup (Signature Cache)**
- Phase A: Load all signatures from disk (~25 bytes per row)
  - Populates DashSet<u64> for persistent dedup protection
  - 25x faster than loading full logs
  - Typically completes in <1 second for 100K signatures
- Phase B: Load unsynced/in-flight logs into sync queue
  - Same as original startup behavior
  - Only pending logs loaded into memory

**365-Day Log Cleanup (Daily)**
- Runs every 24 hours
- Deletes synced logs older than 365 days
- Reduces database file size and improves query performance
- Preserves in-flight logs (is_synced=0)
- Signatures remain in memory for continued dedup protection

**Fair Polling Scheduler**
- Ensures no device is starved of polling resources
- When `MAX_CONCURRENT_WORKERS=1`, devices take turns sequentially
- With `MAX_CONCURRENT_WORKERS>1`, enables parallel polling with semaphore-based fairness

**Duplicate Detection & Queue Traffic Reduction**
- **Worker-level HashSet**: In-memory dedup at polling stage (user_id:timestamp key)
  - Reduces queue channel traffic by up to 99.95%
  - Memory: ~5MB per 50,000 logs (bounded per session)
  - Resets on worker reconnect or configuration reload
- **Agent-level Signature Cache**: Hash-based persistent dedup (FxHash u64)
  - Formula: hash(device_id|user_id|timestamp|punch|status)
  - Memory: 8 bytes per signature (96% savings vs full log)
  - Survives log cleanup and agent restarts
  - Prevents false positives (includes punch+status)
- Combined logging: `"Fetched X logs (Y new, Z duplicates filtered)"` when duplicates detected

---

## 🪟 Triển khai Service trên Windows (NSSM)

---

## 🪟 Triển khai trên Windows (Dùng NSSM)

Đây là phương pháp ổn định nhất để chạy App 24/7 mà không cần người dùng đăng nhập.

### 1. API Server (Rust)
1. Build binary ở chế độ release: `cargo build --release` trong thư mục `apiserver`.
2. Tải **NSSM** và copy vào `apiserver/nssm.exe`.
3. Chuột phải `apiserver/install_service.bat` chọn **Run as Administrator**.
4. Service `LeeAttendAPIServer` sẽ được cài đặt và tự khởi động.

### 2. Agent (Python)
1. Cài đặt Python 3.10+ và thiết lập môi trường ảo `venv`.
2. Copy file `nssm.exe` vào thư mục gốc của agent (nếu chưa có).
3. Chuột phải vào `agent/install_service.bat` và chọn **Run as Administrator**.
4. Service `ZKAttendanceAgent` sẽ tự động được cài đặt và khởi động.

## 💻 3. Web Dashboard (React + Vite)

Giao diện quản trị tập trung, hiện có thể chạy dưới dạng ứng dụng Desktop Windows (Tauri) hoặc Web Server độc lập.

### Yêu cầu hệ thống
- Node.js 18+
- Rust (để build Tauri hoặc Standalone EXE)

### 🚀 Phát triển và Cấu hình
1. Cài đặt thư viện:
   ```bash
   cd webdashboard
   npm install
   ```
2. Cấu hình môi trường:
   Tạo file `webdashboard/.env.local`:
   ```env
   VITE_API_URL=http://your-apiserver:3000
   VITE_DEV_SERVER_HOST=0.0.0.0 # Tùy chọn: Để lắng nghe trên mọi interface
   ```

### 📦 Build ứng dụng Desktop (Tauri)
1. Build ứng dụng:
   ```bash
   npm run tauri build
   ```
2. File cài đặt (`.msi` hoặc `.exe`) sẽ nằm tại: `src-tauri/target/release/bundle/`.

### 🚀 Build ứng dụng Web Server (Standalone EXE)
Giải pháp tối ưu để triển khai mà không cần Node.js runtime. Đóng gói toàn bộ giao diện Web thành 1 file `.exe` duy nhất tích hợp sẵn server.

1. Di chuyển vào thư mục `webdashboard`.
2. Chạy script build tự động:
   ```cmd
   build_standalone.bat
   ```
3. File kết quả sẽ nằm tại `webdashboard/dist/LeeAttendDashboard.exe`.
4. Cách chạy:
   - Chạy trực tiếp (mặc định port 1420, tự mở trình duyệt).
   - Chạy với port tùy chỉnh: `LeeAttendDashboard.exe --port 8080`
   - Chạy ngầm (không mở browser): `LeeAttendDashboard.exe --no-open`

### 🪟 Cài đặt Web Dashboard Service (Windows)
Để Dashboard chạy ngầm 24/7 và tự khởi động cùng Windows:

1. Chạy `build_standalone.bat` để tạo file exe.
2. Chuột phải vào `webdashboard/install_service.bat` chọn **Run as Administrator**.
3. Service `LeeAttendDashboard` sẽ được cài đặt với cấu hình an toàn (Port 3000, No Browser).
4. Để gỡ cài đặt: Chạy `webdashboard/uninstall_service.bat` (Admin).

### Triển khai Web (Standalone)
... (giữ nguyên hướng dẫn cũ nếu cần) ...



## 🔄 Dynamic Port Discovery (Dev Mode)

Hệ thống hỗ trợ tự động tìm port khả dụng để tránh xung đột:

- **Backend (API)**: Mặc định port 3000. Nếu bị chiếm, sẽ thử trong khoảng 3000-3100.
  - Port thực tế được ghi vào file cấu hình.
  - **Biến môi trường `PORT_FILE_NAME`**: Cho phép đổi tên file tín hiệu port (mặc định là `.api_port`). Hữu ích khi chạy nhiều instance song song trên cùng một máy (ví dụ: `.api_port_worker_0`).
- **Frontend (Web)**: Mặc định port 5173. Nếu bị chiếm, sẽ thử trong dải port rộng hơn.
  - **Biến môi trường `VITE_BACKEND_PORT_FILE`**: Chỉ định file port của backend mà frontend cần đọc để cấu hình proxy.
  - **Biến môi trường `VITE_WEB_PORT_FILE`**: Chỉ định file port của chính frontend instance đó.

**Lưu ý cho Parallel Testing**:
Cơ chế này cho phép chạy song song hàng chục Playwright workers mà không lo xung đột networking, bằng cách cấp phát cho mỗi worker một suffix định danh file port riêng biệt.

---

## 🌐 WebSocket Infrastructure Requirements (NEW - 2026-02-03)

### Redis Version Compatibility
- **Minimum Version**: Redis 3.0+ (for basic Pub/Sub and TTL support)
- **Recommended Version**: Redis 6.0+ (for optimized KEEPTTL command)
- **Configuration**:
  ```
  REDIS_URL=redis://localhost:6379
  REPORT_RESULT_TTL=86400        # 24 hours
  WEBSOCKET_TIMEOUT=600          # 10 minutes
  ```

### WebSocket Proxy Configuration (Development)

**Vite Configuration (webdashboard/vite.config.ts):**
```typescript
proxy: {
  // WebSocket proxy MUST come before HTTP proxy
  '/api/ws': {
    target: `http://127.0.0.1:${apiPort}`,
    ws: true,  // Enable WebSocket upgrade
    rewrite: (path) => path.replace(/^\/api/, ''),
  },
  // HTTP proxy comes after
  '/api': {
    target: `http://127.0.0.1:${apiPort}`,
    rewrite: (path) => path.replace(/^\/api/, ''),
  },
}
```

**Order Matters:**
1. WebSocket routes (`/api/ws/*`) must be processed first
2. HTTP routes (`/api/*`) come second
3. Failure to order correctly causes WebSocket requests to be caught by HTTP proxy

### Production Deployment Checklist

- [ ] Redis server is running and accessible
- [ ] Redis version is 3.0 or higher (`redis-cli --version`)
- [ ] WebSocket port (3000 or custom) is accessible from clients
- [ ] Firewall allows WebSocket upgrades (HTTP 101 Switching Protocols)
- [ ] Load balancer (if used) supports WebSocket (sticky sessions or stateless design)
- [ ] HTTPS/WSS certificates are valid for production domain
- [ ] Environment variables are set correctly (Redis URL, timeouts)

### Monitoring & Troubleshooting

**Key Metrics to Track:**
- WebSocket connection success rate (target: >99%)
- Fallback to HTTP polling rate (should be <1%)
- Average export completion time
- Worker job queue depth
- Redis memory usage for report jobs

**Common Issues:**

| Symptom | Cause | Fix |
|---------|-------|-----|
| "Cannot reach WebSocket" | Proxy not configured | Verify vite.config.ts proxy order |
| WebSocket connects then closes | Redis not running | Check Redis service status |
| High fallback rate | Network latency or Redis slow | Check network/Redis performance |
| Out of memory | TTL not set on job metadata | Verify `REPORT_RESULT_TTL` env var |

---

## 🧪 4. Môi trường Kiểm thử (Testing Environment)

Để đảm bảo chất lượng code, LeeAttend áp dụng quy trình kiểm thử nghiêm ngặt dựa trên hạ tầng Docker.

### 🚀 Nguyên tắc Vàng
- **Mandatory Passing**: 100% các bài test (Rust, Python, React) **BẮT BUỘC** phải PASS trước khi commit.
- **Isolation**: Mọi bài test đều chạy trong container. Không được phép chạy test trực tiếp trên host nếu có tương tác Database.
- **Unified Entry Point**: Luôn sử dụng `./run-tests.sh` để thực thi.

### ⚙️ Quy trình chạy Test (Tự động hoàn toàn)
Bạn không cần cấu hình file `.env.test` thủ công. Hệ thống tự động quản lý vòng đời của hạ tầng test:

```bash
# Chạy toàn bộ (Lint + Unit + Integration + E2E)
./run-tests.sh --all

# Chạy riêng Backend (Rust + Python)
./run-tests.sh --api [pattern]

# Chạy riêng Frontend (Playwright)
./run-tests.sh --web [pattern]
```

### ⚡ Tối ưu hóa (Key Features)
- **Docker Isolation**: Tất cả tests chạy trong Docker containers để tránh I/O bottleneck trên WSL.
- **Nextest**: Sử dụng `cargo nextest` cho hiệu năng vượt trội.
- **Chef Cache**: Tối ưu hóa Docker build cache cho dependencies.

---

## 🔑 Security Considerations

## 🐳 3. Docker Compose Production Deployment (Recommended)

This is the most reliable way to deploy LeeAttend in a production environment. It packages all services, including the database and Redis, into a single, orchestrated environment.

### Prerequisites
- Docker and Docker Compose (V2) installed.
- Domain name pointed to your server IP (optional but recommended).

### 🚀 Setup Steps

1. **Clone and Prepare**:
   ```bash
   # Copy the example environment file
   cp .env.example .env
   ```

2. **Configure Environment Variables**:
   Edit the `.env` file and set secure values:
   - `DB_PASSWORD`: Set a strong database password.
   - `JWT_SECRET`: Set a secure JWT secret.

3. **Build and Launch**:
   You can use the helper script or run docker compose directly using the base file:
   ```bash
   # Option A: Using helper script
   ./scripts/docker-helper.sh prod up --build -d

   # Option B: Direct docker compose
   docker compose -f docker-compose.yml up --build -d
   ```

### 🏗 Architecture in Docker
The production setup uses the `runtime` stage of our multi-stage Dockerfiles for minimal image size and maximum security.

- **`webdashboard`**: React Frontend served by Nginx (Port 80 by default).
- **`apiserver`**: Rust API Server (Internal Port 3000).
- **`db`**: PostgreSQL 16 database.
- **`redis`**: Redis 7.
- **`agent-main`**: Edge Orchestrator.

### 🔄 Automatic Proxy & Routing
The Nginx server in the `webdashboard` container acts as a reverse proxy:
- Requests to `/` go to the React App.
- Requests to `/api` are forwarded to the `apiserver`.
- WebSocket upgrades for `/api/ws` are handled automatically.

---

## 🛡 Xử lý sự cố (Troubleshooting)
