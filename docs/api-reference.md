# API Reference

## 🚀 API Server (Central - Rust)
Base URL: `http://localhost:3000`

### 1. Workspace Management (New)

Tất cả các endpoint quản lý tài nguyên (Agents, Devices) đều yêu cầu header `X-Workspace-Id` để đảm bảo cách ly dữ liệu giữa các tổ chức.

#### GET /workspaces
Lấy danh sách các workspace mà người dùng hiện tại là thành viên.
- **Response (200)**: `Array<Workspace>`

#### POST /workspaces
Tạo một workspace mới.
- **Request**: `{ "name": "Tên Công Ty" }`
- **Response (201)**: `Workspace` object

#### GET /workspaces/:id/members
Lấy danh sách thành viên của workspace.
- **Response (200)**: `Array<WorkspaceMember>`

#### POST /workspaces/:id/members
Mời hoặc thêm thành viên vào workspace.
- **Request**: `{ "email": "user@example.com", "role": "admin|member" }`
- **Response (201)**: Created

### 2. Device Management (CRUD)

Các endpoint dành cho Web Dashboard để quản lý danh sách máy chấm công. **Yêu cầu header `X-Workspace-Id`**.

#### GET /devices
Lấy danh sách tất cả các thiết bị trong workspace hiện tại.
- **Response (200)**: `Array<Device>`

#### POST /devices
Thêm thiết bị mới vào workspace. Server sẽ tự động sinh UUID v4 và gửi cấu hình mới cho Agent qua WebSocket.
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

#### PATCH /devices/:id (New)
Cập nhật thông tin thiết bị (tên, IP, port, trạng thái enabled).
- **Response (200 OK)**

#### DELETE /devices/:id
Xóa thiết bị khỏi workspace. Agent sẽ ngay lập tức ngừng giám sát thiết bị này.
- **Response (204)**: No Content

### 3. Agent Management

Các endpoint dành cho Web Dashboard để kiểm soát quyền truy cập của Agent. **Yêu cầu header `X-Workspace-Id`**.

#### GET /authorized-agents
Lấy danh sách Agent thuộc workspace hiện tại.
- **Query Params**: `?include_inactive=true` (mặc định chỉ lấy agent đang active)
- **Response (200)**: `Array<AuthorizedAgent>`

#### POST /authorized-agents
Khai báo một Agent mới vào workspace.
- **Request**:
  ```json
  {
    "id": "AGENT_01",
    "name": "Chi nhánh Hà Nội",
    "token": "secret_token_123"
  }
  ```
- **Response (201)**: `{ "id": "AGENT_01" }`

#### PATCH /authorized-agents/:id (New)
Cập nhật thông tin Agent (tên, token, trạng thái active/trusted).
- **Response (200 OK)**

#### DELETE /authorized-agents/:id (Soft Delete)
Đánh dấu Agent là không hoạt động (`is_active = false`). Agent sẽ bị ngắt kết nối WebSocket và ẩn khỏi danh sách mặc định.
- **Response (204)**: No Content

#### PUT /authorized-agents/:id/trust
Phê duyệt (Trust) hoặc Thu hồi quyền của một Agent. Khi thu hồi, kết nối WebSocket hiện tại sẽ bị ngắt.
- **Request**:
  ```json
  { "is_trusted": true }
  ```
- **Response (200 OK)**

### WebSocket Messages
Agent gửi dữ liệu chấm công thô lên Server.

#### Handshake (Agent -> Server)
Bản tin đầu tiên sau khi kết nối để xác thực định danh.
```json
{
  "type": "handshake",
  "agent_id": "AGENT_01",
  "token": "secret_token_abc_123",
  "timestamp": "2026-01-07T10:00:00Z"
}
```

#### Attendance Log (Agent -> Server)
... (giữ nguyên cấu trúc cũ) ...

---

### WebSocket Error Codes
Khi có lỗi xảy ra trong quá trình Handshake hoặc kết nối, Server sẽ gửi message `type: "error"` và đóng kết nối.

| Code | Message | Mô tả |
|------|---------|-------|
| `UNAUTHORIZED_AGENT` | Agent ID không nằm trong danh sách... | ID này chưa được khai báo trong bảng `authorized_agents`. |
| `INVALID_TOKEN` | Token không chính xác. | ID đúng nhưng chuỗi bí mật `token` không khớp. |
| `AGENT_NOT_TRUSTED` | Agent chưa được phê duyệt... | Agent đã khai báo nhưng cờ `is_trusted` đang là `false`. |
| `DUPLICATE_ID` | Agent ID ... đã có kết nối hoạt động. | ID này hiện đang có một WebSocket session khác đang chạy. |

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

## 🤖 Agent Remote Control (WebSocket Tunnels)

Agent không cung cấp HTTP API local. Mọi tương tác với Agent được thực hiện gián tiếp thông qua Central Server WebSocket. 

Admin gửi lệnh tới API Server, và API Server sẽ định tuyến lệnh đó tới Agent tương ứng qua kết nối WebSocket đang hoạt động.

### POST /agent/:agent_id/command (Central Server API)
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
- `sync_time`: Đồng bộ thời gian cho máy chấm công cụ thể.
- `get_users`: Lấy danh sách nhân viên từ máy.
- `ping`: Kiểm tra kết nối và độ trễ tới Agent.

**Response (200 OK):**
```json
{
  "success": true,
  "data": { ... result of method ... },
  "error": null
}
```

---

## 🌐 Agent Outbound Giao thức (WebSocket)

Agent chủ động kết nối tới `CENTRAL_SERVER_URL` để thiết lập kênh giao tiếp 2 chiều.

### 1. Handshake (Agent -> Server)
Bản tin đầu tiên để xác thực.
```json
{
  "type": "handshake",
  "agent_id": "AGENT_01",
  "token": "secret_token_abc",
  "timestamp": "ISO-8601"
}
```

### 2. Status Update (Agent -> Server)
Agent định kỳ (mỗi 30s) gửi trạng thái chi tiết của các máy chấm công local.
```json
{
  "type": "status_update",
  "agent_id": "AGENT_01",
  "devices": [
    {
      "device_id": "DEV_01",
      "status": "online",
      "last_sync": "2026-01-07T18:00:00",
      "error": null
    }
  ]
}
```

### 3. Attendance Batch (Agent -> Server)
Gửi dữ liệu chấm công theo lô (batch) để tối ưu hiệu năng.
```json
{
  "type": "attendance",
  "msg_id": "uuid-v4",
  "agent_id": "AGENT_01",
  "device_id": "BATCH",
  "data": [
    { "user_id": "101", "timestamp": "2026-01-07 18:00:00", "status": 1, "punch": 0 },
    { "user_id": "102", "timestamp": "2026-01-07 18:01:00", "status": 1, "punch": 0 }
  ]
}
```

---

## 🛠 Định dạng Log (Agent Console Output)

Dù không có HTTP server, Agent vẫn in thông tin hoạt động ra Console:
```text
[18:00:00] [SYSTEM] ℹ️ INFO : Agent đã khởi động và đang giám sát thiết bị local.
[18:05:00] [OFFICE_01] ⚠️ WARN : Lỗi kết nối máy chấm công (ZKError). Thử lại sau 15.0s.
[18:10:00] [OFFICE_01] ℹ️ INFO : Kết nối máy chấm công đã được phục hồi.
```

