# ZKTeco Multi-Device Attendance Agent (CLI)

Ứng dụng nền quản lý tập trung nhiều máy chấm công ZKTeco, tự động tải dữ liệu và đồng bộ lên Central Server thông qua WebSocket.

## 🚀 Tính năng chính

- **High-Performance CLI**: Loại bỏ HTTP Server, tối ưu tài nguyên cho các thiết bị Edge.
- **SRP Architecture**: Tách biệt hoàn toàn phần cứng (DeviceManager), lưu trữ (Repository) và kết nối (CentralClient).
- **Outbound WebSocket**: Tự động kết nối tới Cloud, làm việc tốt sau NAT/Firewall mà không cần mở port.
- **Guaranteed Delivery**: Cơ chế ACK đảm bảo dữ liệu không bị thất lạc khi mạng chập chờn.
- **Multi-Device Support**: Quản lý hàng chục máy chấm công song song qua các Async Tasks.
- **Memory Safety**: Cơ chế Bounded Queue và Rolling Cache giúp Agent chạy ổn định 24/7 chỉ với <100MB RAM.
- **Robust Error Handling**: Áp dụng **Exponential Backoff với Jitter** cho mọi nỗ lực kết nối lại.
- **Offline-first**: Tự động nạp cấu hình từ `devices.json` và lưu trữ log tạm thời vào SQLite khi mất mạng.

## 📦 Cài đặt & Cấu hình

### Yêu cầu
- Python 3.10 trở lên
- Mạng internet (để kết nối WebSocket tới Central Server)

### 1. Thiết lập Môi trường

**Trên Windows (PowerShell):**
```powershell
python -m venv venv
.\venv\Scripts\pip install -r requirements.txt
# Copy file cấu hình mẫu
copy .env.example .env
```

**Trên Linux/macOS:**
```bash
python3 -m venv venv
source venv/bin/activate
pip install -r requirements.txt
cp .env.example .env
```

### 2. Cấu hình (`.env`)
Chỉnh sửa file `.env` với thông tin kết nối của bạn:
```env
# Kết nối tới Server Trung tâm
CENTRAL_SERVER_URL=ws://localhost:3000/ws
AGENT_TOKEN=your_secure_token_here
AGENT_ID=SITE_01

# Database Local (Mặc định dùng SQLite trên Windows)
DATABASE_URL=sqlite:///./attendance.db

# Tùy chỉnh Sync
SYNC_BATCH_SIZE=100
SYNC_INTERVAL=0.5
```

## 🏃 Vận hành

### Chạy trực tiếp (Debug)
**Windows:**
```powershell
.\venv\Scripts\python.exe main.py
```

**Linux/macOS:**
```bash
./venv/bin/python main.py
```

### Chạy dưới dạng Service (Production)

#### Windows Service
1. Đảm bảo bạn đã cài đặt môi trường (`venv`) và cấu hình `.env` đầy đủ.
2. Tải `nssm.exe` từ [nssm.cc](https://nssm.cc/download) (hoặc dùng bản có sẵn trong repo nếu có) và đặt vào thư mục `agent`.
3. Chuột phải vào file `install_service.bat` và chọn **Run as Administrator**.
   - Script sẽ tự động phát hiện Python trong `venv` và cài đặt service `ZKAttendanceAgent`.
4. Để gỡ cài đặt, chạy `uninstall_service.bat` với quyền Admin.

#### Linux Systemd
(Đang cập nhật...)

## 🛠️ Kiến trúc nội bộ
- **`app_container.py`**: Trái tim của Agent, điều phối mọi hoạt động.
- **`device_manager.py`**: Quản lý kết nối vật lý và polling dữ liệu quẹt thẻ.
- **`repository.py`**: Quản lý SQLAlchemy Models và các thao tác Database local.
- **`central_client.py`**: Quản lý giao thức WebSocket và Handshake với Cloud.
- **`sync_worker.py`**: Đảm nhiệm việc đẩy dữ liệu từ DB local lên Cloud.