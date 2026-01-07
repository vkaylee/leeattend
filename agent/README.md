# ZKTeco Multi-Device Attendance Agent (CLI)

Ứng dụng nền quản lý tập trung nhiều máy chấm công ZKTeco, tự động tải dữ liệu và đồng bộ lên Central Server thông qua WebSocket.

## 🚀 Tính năng chính

- **High-Performance CLI**: Loại bỏ HTTP Server, tối ưu tài nguyên cho các thiết bị Edge.
- **SRP Architecture**: Tách biệt hoàn toàn phần cứng (DeviceManager), lưu trữ (Repository) và kết nối (CentralClient).
- **Outbound WebSocket**: Tự động kết nối tới Cloud, làm việc tốt sau NAT/Firewall mà không cần mở port.
- **Guaranteed Delivery**: Cơ chế ACK đảm bảo dữ liệu không bị thất lạc khi mạng chập chờn.
- **Multi-Device Support**: Quản lý hàng chục máy chấm công song song qua các Async Tasks.
- **Robust Error Handling**: Áp dụng **Exponential Backoff với Jitter** cho mọi nỗ lực kết nối lại.
- **Offline-first**: Tự động nạp cấu hình từ `devices.json` và lưu trữ log tạm thời vào SQLite khi mất mạng.

## 📦 Cài đặt & Cấu hình

1. **Cài đặt thư viện:**
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   pip install -r requirements.txt
   ```

2. **Biến môi trường (`.env`):**
   ```env
   CENTRAL_SERVER_URL=ws://your-server.com/ws
   AGENT_ID=SITE_01
   AGENT_TOKEN=your_secret_token
   DATABASE_URL=sqlite:///./attendance.db
   ```

## 🏃 Vận hành

### Chạy trực tiếp
```bash
# Khởi chạy Agent (CLI Mode)
python main.py

# Chạy bộ test toàn diện (60+ cases)
cd agent && ../venv/bin/pytest -v
```

### Chạy dưới dạng Windows Service (Khuyên dùng cho Prod)
1. Tải `nssm.exe` từ [nssm.cc](https://nssm.cc/download) và copy vào thư mục gốc của agent.
2. Chuột phải vào `install_service.bat` chọn **Run as Administrator**.
3. Hệ thống sẽ tự động cài đặt và khởi chạy `AttendanceAgent` như một service ngầm.

## 🛠️ Kiến trúc nội bộ
- **`app_container.py`**: Trái tim của Agent, điều phối mọi hoạt động.
- **`device_manager.py`**: Quản lý kết nối vật lý và polling dữ liệu quẹt thẻ.
- **`repository.py`**: Quản lý SQLAlchemy Models và các thao tác Database local.
- **`central_client.py`**: Quản lý giao thức WebSocket và Handshake với Cloud.
- **`sync_worker.py`**: Đảm nhiệm việc đẩy dữ liệu từ DB local lên Cloud.