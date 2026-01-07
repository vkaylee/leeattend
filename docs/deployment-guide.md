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
# Central Database (PostgreSQL)
PG_DATABASE_ADDR=192.168.x.x
PG_DATABASE_PORT=5432
PG_DATABASE_DB=leeatt
PG_DATABASE_USER=leeatt
PG_DATABASE_PASS=@Elite@123456

# CENTRAL SERVER (WebSocket)
CENTRAL_SERVER_URL=ws://your-api-server:3000/ws
AGENT_TOKEN=your_secure_token
AGENT_ID=BRANCH_NAME_01
# Cấu hình đồng bộ
SYNC_BATCH_SIZE=100
SYNC_INTERVAL=0.1
# Đặt 0 để chọn port ngẫu nhiên (tránh xung đột local)
API_PORT=0 
```

### 🚀 Khởi tạo và Chạy
1. Cài đặt thư viện:
   ```bash
   cd agent
   python -m venv venv
   source venv/bin/activate  # Hoặc venv\Scripts\activate trên Windows
   pip install -r requirements.txt
   ```
2. Khởi tạo Database (Migration):
   ```bash
   alembic upgrade head
   ```
3. Chạy Agent:
   ```bash
   python main.py
   ```

---

## 🪟 Triển khai trên Windows (Dùng NSSM cho Agent)

Đây là phương pháp ổn định nhất để chạy App 24/7 mà không cần người dùng đăng nhập.

### 1. Chuẩn bị
1. Cài đặt Python 3.10+ và thiết lập môi trường ảo `venv`.
2. Tải **NSSM (Non-Sucking Service Manager)** từ [nssm.cc](https://nssm.cc/download).
3. Copy file `nssm.exe` (bản 64-bit) vào thư mục gốc của dự án (`zkteco-attendance-py/`).

### 2. Cài đặt Service
1. Chỉnh sửa file `devices.json` với danh sách máy chấm công của bạn.
2. Chuột phải vào file `install_service.bat` và chọn **Run as Administrator**.
3. Một cửa sổ terminal hiện ra thông báo các bước cài đặt. Khi kết thúc, service `ZKAttendanceAPI` sẽ tự động khởi động.

### 3. Quản lý Service
- **Xem Log**: Mở thư mục `logs/` để xem `stdout.log` (dữ liệu in ra màn hình) và `stderr.log` (lỗi nếu có).
- **Gỡ bỏ**: Chuột phải vào `uninstall_service.bat` và chọn **Run as Administrator**.
- **Services Manager**: Bạn có thể quản lý service thủ công qua công cụ `services.msc` của Windows.

## 💻 3. Web Dashboard (Next.js)



Giao diện quản trị tập trung cho toàn bộ hệ thống.



### Yêu cầu hệ thống

- Node.js 18.x hoặc mới hơn



### Các bước triển khai

1. Di chuyển vào thư mục `webdashboard`:

   ```bash

   cd webdashboard

   ```

2. Cài đặt dependencies:

   ```bash

   npm install

   ```

3. Cấu hình môi trường (Tạo file `.env.local` nếu cần):

   ```env

   NEXT_PUBLIC_API_URL=http://your-api-server:3000

   ```

4. Build và chạy:

   - **Development**: `npm run dev`

   - **Production (Standalone)**:

     ```bash

     npm run build

     # Sau đó chạy file server đã đóng gói

     node .next/standalone/server.js

     ```



## 🛡 Xử lý sự cố (Troubleshooting)
