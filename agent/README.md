# ZKTeco Multi-Device Attendance API

Ứng dụng quản lý tập trung nhiều máy chấm công ZKTeco XFace100, tự động tải dữ liệu và cung cấp API điều khiển.

## 🚀 Tính năng chính

- **Multi-Device Support**: Quản lý hàng chục máy chấm công cùng lúc qua file `devices.json`.
- **Centralized Event System**: Mọi thông báo hệ thống và log chấm công được tập trung về một Runner chính xử lý.
- **Silent Runners**: Các task kết nối máy chấm công chạy ngầm hoàn toàn, không gây nhiễu Console.
- **Async Orchestration**: Mỗi máy chạy trong một task riêng biệt, không làm nghẽn mạch nhau.
- **Real-time Console**: Giao diện in log rực rỡ, phân loại thông tin thông minh (Info, Warning, Attendance).
- **Safety First**: Thiết kế "Read-only", tuyệt đối không xóa dữ liệu trên thiết bị.
- **Robust Error Handling**: Cô lập lỗi kết nối, áp dụng thuật toán **Exponential Backoff** khi mất mạng.
- **Persistent Cache**: Ghi nhớ lịch sử đã xử lý vào file để không in trùng khi khởi động lại.

## 📦 Cài đặt & Cấu hình

1. **Cài đặt thư viện:**
   ```bash
   python3 -m venv venv
   source venv/bin/activate
   pip install pyzk2 fastapi uvicorn python-dotenv
   ```

2. **Cấu hình danh sách máy (`devices.json`):**
   ```json
   [
     {
       "id": "OFFICE_01",
       "name": "Văn Phòng Chính",
       "ip": "192.168.12.14",
       "port": 4370
     },
     {
       "id": "WAREHOUSE_01",
       "name": "Kho Tổng",
       "ip": "192.168.12.15",
       "port": 4370
     }
   ]
   ```

## 🏃 Vận hành

### Chạy trực tiếp
```bash
# Chạy ứng dụng
python main.py

# Chạy test
PYTHONPATH=. ./venv/bin/pytest -v tests/
```

### Chạy dưới dạng Windows Service (Khuyên dùng cho Prod)
1. Tải `nssm.exe` từ [nssm.cc](https://nssm.cc/download) và copy vào thư mục gốc.
2. Chuột phải vào `install_service.bat` chọn **Run as Administrator**.
3. Hệ thống sẽ tự động cài đặt và khởi chạy ứng dụng ngầm.

## 📖 Tài liệu
- [API Reference](docs/api-reference.md) (Endpoints mới có `device_id`)
- [Architecture](docs/system-architecture.md) (Cơ chế Async Manager)