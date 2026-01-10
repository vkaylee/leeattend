# Deployment Guide

Tài liệu này hướng dẫn cách triển khai hệ thống LeeAttend bao gồm **API Server (Rust)** và **Agent (Python)**.

---

## 🚀 1. API Server (Rust) - Central Server

API Server đóng vai trò là trung tâm tiếp nhận dữ liệu từ các Agent ở các chi nhánh.

### Yêu cầu hệ thống
- Rust 1.75+
- PostgreSQL (Dự kiến)

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
  - Port thực tế được ghi vào file `apiserver/.api_port`.
- **Frontend (Web)**: Mặc định port 5173. Nếu bị chiếm, sẽ thử trong khoảng 5173-5273.
  - Port thực tế được ghi vào file `webdashboard/.web_port`.
  - Frontend tự động đọc `apiserver/.api_port` để cấu hình proxy `/api`.

**Lưu ý cho CI/CD & Testing**:
Khi chạy các bộ test automation (như Playwright), hãy đảm bảo cả Backend và Frontend đã khởi động xong để các file `.api_port` và `.web_port` được tạo ra. Playwright được cấu hình để tự động đọc các file này.

## 🧪 4. Môi trường Kiểm thử (Testing Environment)

Để đảm bảo chất lượng code, LeeAttend áp dụng quy trình kiểm thử nghiêm ngặt.

### 🚀 Nguyên tắc Vàng
- **Mandatory Passing**: 100% các bài test (Rust, Python, React) **BẮT BUỘC** phải PASS trước khi commit.
- **Isolation**: Kiểm thử tích hợp (Integration tests) chạy trên một database hoàn toàn biệt lập, không được phép chạm vào database development (`leeattzkt`).

### ⚙️ Cấu hình Test (apiserver/.env.test)
Trong môi trường phát triển, bạn cần cấu hình file `.env.test` để trỏ tới một PostgreSQL server dùng riêng cho việc tạo database tạm:

```env
# URL gốc để tạo các database test ngẫu nhiên (Yêu cầu quyền CREATEDB)
DATABASE_URL_FOR_TEST_GENERATION=postgres://user:pass@localhost:5434/postgres

# Safety net: Chốt chặn an toàn cho các công cụ mặc định
DATABASE_URL=postgres://user:pass@localhost:5434/postgres
```

### 🛠 Lệnh chạy kiểm thử
- **Backend**: `cd apiserver && cargo test`
- **Frontend**: `cd webdashboard && npm test`
- **Worker**: `cd agent-zk-worker && source venv/bin/activate && pytest` (Bắt buộc qua venv)

---

## 🔑 Security Considerations

## 🛡 Xử lý sự cố (Troubleshooting)
