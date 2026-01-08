# API Reference

## 🚀 API Server (Central - Rust)
Base URL: `http://localhost:3000`

### 0. Authentication & Context (New)

Trong môi trường hiện tại, hệ thống sử dụng các header sau để xác định danh tính và phạm vi dữ liệu:
- `X-User-Id`: UUID của người dùng (Global Identity).
- `X-Workspace-Id`: UUID của workspace hiện tại.

Mọi yêu cầu truy cập tài nguyên (Agents, Devices, Members) đều được kiểm tra:
1. Người dùng có tồn tại không?
2. Người dùng có là thành viên của Workspace không?
3. Người dùng có đủ quyền (Role) để thực hiện thao tác không?

---

### 1. User Profile (New)

Quản lý thông tin định danh toàn cầu.

#### GET /me
Lấy thông tin hồ sơ của người dùng hiện tại.
- **Headers**: `X-User-Id`
- **Response (200)**: `User` object (`id`, `email`, `full_name`, `avatar_url`, `bio`, `created_at`)

#### PATCH /me
Cập nhật thông tin hồ sơ.
- **Headers**: `X-User-Id`
- **Request**: `{ "full_name": "...", "avatar_url": "...", "bio": "..." }`
- **Response (200 OK)**

#### GET /users (Dev Only)
Lấy danh sách tất cả người dùng trong hệ thống (phục vụ giả lập login).
- **Response (200)**: `Array<User>`

---

### 2. Workspace Management

Tất cả các endpoint quản lý tài nguyên (Agents, Devices) đều yêu cầu header `X-Workspace-Id` và `X-User-Id`.

#### GET /workspaces
Lấy danh sách các workspace mà người dùng hiện tại là thành viên.
- **Headers**: `X-User-Id`
- **Response (200)**: `Array<Workspace>`

#### POST /workspaces
Tạo một workspace mới. Người tạo sẽ tự động trở thành `owner`.
- **Headers**: `X-User-Id`
- **Request**: `{ "name": "Tên Công Ty" }`
- **Response (201)**: `Workspace` object

#### GET /workspaces/:id/members
Lấy danh sách thành viên của workspace. Yêu cầu người dùng là thành viên.
- **Headers**: `X-User-Id`, `X-Workspace-Id`
- **Response (200)**: `Array<WorkspaceMember>`

#### POST /workspaces/:id/members
Mời thành viên vào workspace. 
- **Quyền hạn**: Chỉ `owner` hoặc `admin` mới có thể mời.
- **Headers**: `X-User-Id`, `X-Workspace-Id`
- **Request**: `{ "email": "user@example.com", "role": "admin|member" }`
- **Response (201)**: Created
- **Errors**: `404 User Not Found` (Người dùng phải có tài khoản global trước), `403 Forbidden` (Không đủ quyền).

---

### 3. Member Roles (Temporal RBAC) (New)

Hệ thống cho phép gán thêm các Role phụ có thời hạn cho thành viên.

#### GET /workspaces/:id/members/:user_id/roles
Lấy danh sách tất cả các role của một thành viên (bao gồm role chính và các role phụ).
- **Response (200)**: 
  ```json
  {
    "primary": "member",
    "extra": [
      {
        "id": "...",
        "role": "admin",
        "assigned_at": "...",
        "expires_at": "..."
      }
    ]
  }
  ```

#### POST /workspaces/:id/members/:user_id/roles
Gán một role phụ có thời hạn cho thành viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**:
  ```json
  {
    "role": "admin",
    "expires_at": "2026-01-10T00:00:00Z"
  }
  ```
- **Response (201)**: `MemberExtraRole` object

#### DELETE /workspaces/:id/members/:user_id/roles/:role_id
Thu hồi một role phụ trước thời hạn.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Response (204)**: No Content

---

### 4. Device Management (CRUD)

**Yêu cầu headers: `X-User-Id`, `X-Workspace-Id`**.

#### GET /devices
Lấy danh sách tất cả các thiết bị trong workspace hiện tại.
- **Response (200)**: `Array<Device>`

#### POST /devices
Thêm thiết bị mới vào workspace.
- **Request**: `{ "agent_id": "...", "name": "...", "ip": "...", "port": 4370 }`
- **Response (201)**: `{ "id": "uuid-v4" }`

---

### 4. Agent Management

**Yêu cầu headers: `X-User-Id`, `X-Workspace-Id`**.

#### GET /authorized-agents
Lấy danh sách Agent thuộc workspace hiện tại.
- **Query Params**: `?include_inactive=true`
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

