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

## 🤖 2. Agent (Python) - Local Site Agent

Agent chạy tại máy tính nội bộ nơi kết nối với các máy chấm công ZKTeco.

### Yêu cầu hệ thống
- Python 3.10+
- Kết nối mạng đến API Server

### 🐧 Cấu hình đa máy chấm công

Chỉnh sửa file `agent/devices.json`:

```json
[
  { "id": "M1", "name": "Máy 1", "ip": "192.168.12.14", "port": 4370 },
  { "id": "M2", "name": "Máy 2", "ip": "192.168.12.15", "port": 4370 }
]
```

### 🗄️ Cấu hình Database & Central (.env)

Tạo file `agent/.env`:
```env
# Central Server (WebSocket)
CENTRAL_SERVER_URL=ws://your-api-server:3000/ws
AGENT_TOKEN=your_secure_token
AGENT_ID=BRANCH_NAME_01

# Local Database (SQLite mặc định hoặc Postgres)
DATABASE_URL=sqlite:///./attendance.db

# Cấu hình đồng bộ
SYNC_BATCH_SIZE=100
SYNC_INTERVAL=0.5
```

### 🚀 Khởi tạo và Chạy
1. Cài đặt thư viện:
   ```bash
   cd agent
   # Windows
   python -m venv venv
   .\venv\Scripts\pip install -r requirements.txt
   
   # Linux/Mac
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   ```
2. Khởi tạo Database:
   ```bash
   # Agent tự động tạo bảng khi khởi động
   # Nếu dùng Postgres, hãy đảm bảo DB đã được tạo
   ```
3. Chạy Agent:
   ```bash
   # Windows
   .\venv\Scripts\python main.py
   
   # Linux/Mac
   python main.py
   ```

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

## 🔑 Security Considerations

## 🛡 Xử lý sự cố (Troubleshooting)
