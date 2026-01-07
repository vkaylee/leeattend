# API Reference

## 🚀 API Server (Central - Rust)
Base URL: `http://localhost:3000`

### 2. Device Management (CRUD)

Các endpoint dành cho Web Dashboard để quản lý danh sách máy chấm công.

#### GET /devices
Lấy danh sách tất cả các thiết bị.
- **Response (200)**: `Array<Device>`

#### POST /devices
Thêm thiết bị mới. Server sẽ tự động sinh UUID v4 và gửi cấu hình mới cho Agent qua WebSocket.
- **Request**:
  ```json
  {
    "agent_id": "AGENT_01",
    "name": "Máy Cổng Chính",
    "ip": "192.168.1.100",
    "port": 4370
  }
  ```
- **Response (201)**: `{ "id": "uuid-v4" }`

#### DELETE /devices/:id
Xóa thiết bị. Agent sẽ ngay lập tức ngừng giám sát thiết bị này.
- **Response (204)**: No Content

### 3. WebSocket Messages
Agent gửi dữ liệu chấm công thô lên Server.

**Message (Agent -> Server):**
```json
{
  "type": "attendance",
  "msg_id": "uuid-v4-string",
  "agent_id": "AGENT_01",
  "device_id": "M1",
  "data": {
    "user_id": "123",
    "timestamp": "2026-01-07 10:00:00",
    "status": 1,
    "punch": 0
  }
}
```

**Acknowledgment (Server -> Agent):**
Server phản hồi sau khi nhận và xử lý thành công.
```json
{
  "type": "ack",
  "msg_id": "uuid-v4-string",
  "status": "success",
  "timestamp": "2026-01-07T10:00:01Z"
}
```
*Lưu ý: Agent sẽ gửi lại tin nhắn nếu không nhận được ACK trong vòng 5 giây.*

#### Server Shutdown (Server -> Agent)
Thông báo khi Server sắp tắt để bảo trì.
```json
{
  "type": "system",
  "code": "SERVER_SHUTDOWN",
  "message": "API Server is shutting down gracefully."
}
```

---

### WebSocket Error Codes
Khi có lỗi xảy ra trong quá trình Handshake hoặc kết nối, Server sẽ gửi message `type: "error"`.

| Code | Message | Mô tả |
|------|---------|-------|
| `DUPLICATE_ID` | Agent ID ... is already active | ID này đã có một connection khác đang chạy. |

---



### POST /agent/:agent_id/command
Gửi lệnh điều khiển đến một Agent đang online.

**Request Body:**
```json
{
  "method": "sync_time",
  "device_id": "OFFICE_01",
  "data": {}
}
```

**Supported Methods:**
- `sync_time`: Đồng bộ thời gian cho máy chấm công.
- `get_users`: Lấy danh sách nhân viên từ máy.
- `ping`: Kiểm tra kết nối đến Agent.

**Response (200 OK):**
```text
Command sent
```

**Errors:**
- `404 Not Found`: Agent không online hoặc không tồn tại.

---

## 🤖 Agent API (Local - Python)
Base URL: `http://localhost:8000`

## GET /health

Kiểm tra xem service Agent có đang hoạt động hay không.

### Response (200 OK)
```json
{
  "status": "healthy",
  "timestamp": "2026-01-07T11:00:00.123456"
}
```

## GET /status

Báo cáo chi tiết trạng thái vận hành của Agent và các máy chấm công đang kết nối.

### Response (200 OK)
```json
{
  "status": "running",
  "uptime_seconds": 3600.5,
  "total_processed": 1250,
  "queue_size": 0,
  "devices": [
    {
      "device_id": "OFFICE_01",
      "status": "online",
      "last_sync": "2026-01-07T10:55:00",
      "error": null
    }
  ]
}
```

## GET /attendance/raw

Truy vấn dữ liệu chấm công thô (raw) từ cơ sở dữ liệu PostgreSQL. Dữ liệu này được lưu trữ tự động ngay khi máy chấm công đẩy về hệ thống.

### Query Parameters

| Tham số | Kiểu | Bắt buộc | Mô tả |
|---------|------|----------|-------|
| `device_id` | string | Không | Lọc theo ID của máy chấm công |
| `user_id` | string | Không | Lọc theo ID của nhân viên |
| `start_date` | datetime | Không | Lọc từ thời điểm (ISO 8601, e.g., `2026-01-01T08:00:00`) |
| `end_date` | datetime | Không | Lọc đến thời điểm |
| `page` | integer | Không | Số trang (Mặc định: 1) |
| `page_size` | integer | Không | Số bản ghi trên mỗi trang (Mặc định: 100) |

### Response (200 OK)

Trả về danh sách các bản ghi chấm công thô, sắp xếp theo thời gian mới nhất trước.

```json
[
  {
    "id": 125,
    "device_id": "OFFICE_01",
    "user_id": "1001",
    "timestamp": "2026-01-07T10:30:00",
    "status": 1,
    "punch": 0,
    "created_at": "2026-01-07T10:30:05"
  }
]
```

### cURL Example
```bash
curl -X GET "http://localhost:8000/attendance/raw?user_id=1001&page_size=10"
```

---

# 🌐 Central Server Integration (Legacy/WebSocket)
*Note: This section describes the Agent's WebSocket capability.*

Agent kết nối đến Server trung tâm qua WebSocket (Outbound) để đẩy dữ liệu và nhận cấu hình.

## Outbound Payloads (Agent -> Server)

### Attendance Log
```json
{
  "type": "attendance",
  "agent_id": "BRANCH_01",
  "device_id": "DEV_01",
  "data": {
    "user_id": "101",
    "timestamp": "2026-01-07 10:00:00",
    "status": 1,
    "punch": 0
  },
  "timestamp": "ISO-8601-Timestamp"
}
```

## Inbound Commands (Server -> Agent)

#### Remote Configuration (Server -> Agent)
Server gửi payload này để cập nhật danh sách máy chấm công Agent cần quản lý:
```json
{
  "type": "config_push",
  "data": {
    "devices": [
      { "device_id": "NEW_DEV", "ip": "192.168.1.100", "port": 4370 }
    ]
  }
}
```

## GET /devices
Liệt kê trạng thái hiện tại của tất cả các máy trong cấu hình.

### GET `/devices/{device_id}/logs`
Tải dữ liệu chấm công của một máy cụ thể.
- **Example**: `GET /devices/OFFICE_01/logs`

### GET `/devices/{device_id}/users`
Liệt kê danh sách nhân viên của một máy cụ thể.

### POST `/devices/{device_id}/sync-time`
Đồng bộ giờ cho một máy cụ thể.

---

## 🛠 Định dạng Log đầu ra (Console)

Hệ thống sẽ in log kèm theo `device_id`:
```text
[🔔 NEW LOG - OFFICE_01] - 12:00:00
 👤 User ID  : 101
 ⏰ Time     : 2026-01-06 11:59:00
 📍 Status   : 1
-----------------------------------
```
