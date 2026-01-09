# API Reference

## 🚀 API Server (Central - Rust)
Base URL: `http://localhost:3000`

### 0. Authentication & Context (Updated)

Hệ thống sử dụng JWT (JSON Web Token) để xác thực người dùng.

- **Access Token**: Gửi trong header `Authorization: Bearer <token>`. Hết hạn sau 1 giờ.
- **Refresh Token**: Dùng để lấy Access Token mới. Hết hạn sau 7 ngày.
- **X-Workspace-Id**: Header bắt buộc cho các yêu cầu liên quan đến tài nguyên cụ thể của tổ chức.

#### POST /auth/register
Đăng ký tài khoản người dùng mới.
- **Request**: `{ "email": "...", "password": "...", "full_name": "..." }`
- **Response (200)**: `{ "access_token": "...", "refresh_token": "...", "user_id": "..." }`

#### POST /auth/login
Đăng nhập để lấy token.
- **Request**: `{ "email": "...", "password": "..." }`
- **Response (200)**: `{ "access_token": "...", "refresh_token": "...", "user_id": "..." }`

#### POST /auth/refresh
Làm mới Access Token.
- **Request**: `{ "refresh_token": "..." }`
- **Response (200)**: `{ "access_token": "...", "refresh_token": "..." }`

#### POST /auth/change-password
Đổi mật khẩu người dùng hiện tại.
- **Headers**: `Authorization: Bearer <token>`
- **Request**: `{ "old_password": "...", "new_password": "..." }`
- **Response (200 OK)**: "Password changed successfully"

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

#### PATCH /workspaces/:id/members/:user_id
Cập nhật vai trò (Primary Role) của thành viên.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`.
- **Request**: `{ "role": "admin|member" }`
- **Response (200 OK)**

#### DELETE /workspaces/:id/members/:user_id
Xóa thành viên khỏi workspace.
- **Quyền hạn**: Chỉ `owner` hoặc `admin`. Không thể tự xóa chính mình.
- **Response (204)**: No Content

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

---

### 5. Employee Management

Quản lý hồ sơ nhân viên trong tổ chức.

#### GET /workspaces/:ws_id/employees
Lấy danh sách tất cả nhân viên.
- **Response (200)**: `Array<Employee>`

#### POST /workspaces/:ws_id/employees
Tạo hồ sơ nhân viên mới.
- **Request**: 
  ```json
  {
    "user_id": "optional-uuid",
    "first_name": "Nguyen",
    "last_name": "Van A",
    "employee_id_number": "EMP001",
    "hire_date": "2026-01-01"
  }
  ```
- **Response (201)**: `Employee` object

#### PUT /workspaces/:ws_id/employees/:id
Cập nhật thông tin nhân viên (họ tên, ngày vào làm, trạng thái nghỉ việc).
- **Response (200 OK)**

---

### 6. Time Tracking

Theo dõi dữ liệu chấm công thực tế.

#### POST /workspaces/:ws_id/time/punch-in
Ghi nhận giờ vào làm.
- **Request**: `{ "employee_id": "...", "entry_type": "work|break", "is_break_overtime": true/false, "note": "..." }`
- **Response (201)**: `TimeEntry` object

#### POST /workspaces/:ws_id/time/punch-out/:employee_id
Ghi nhận giờ ra về cho lần punch-in gần nhất chưa đóng.
- **Request**: `{ "note": "..." }`
- **Response (200 OK)**

#### GET /workspaces/:ws_id/time/history/:employee_id
Lấy lịch sử chấm công trong khoảng thời gian.
- **Query Params**: `?start=ISO-8601&end=ISO-8601`
- **Response (200)**: `Array<TimeEntry>`

---

### 7. Leave Management

Quản lý đơn xin nghỉ phép.

#### POST /workspaces/:ws_id/leave/submit
Gửi đơn xin nghỉ phép.
- **Request**:
  ```json
  {
    "employee_id": "...",
    "leave_type": "vacation|sick|personal",
    "start_date": "2026-02-01",
    "end_date": "2026-02-03",
    "reason": "..."
  }
  ```
- **Response (201)**: `LeaveRequest` object

#### GET /workspaces/:ws_id/leave/requests
Lấy danh sách đơn nghỉ phép trong workspace.
- **Query Params**: `?status=pending|approved|rejected`
- **Response (200)**: `Array<LeaveRequest>`

#### POST /workspaces/:ws_id/leave/requests/:id/approve
Phê duyệt hoặc từ chối đơn nghỉ phép.
- **Request**: `{ "status": "approved|rejected" }`
- **Response (200 OK)**

---

### 8. Attendance Adjustments (New)

Quy trình xử lý các trường hợp quên chấm công hoặc cần điều chỉnh dữ liệu chấm công quá khứ.

#### POST /workspaces/:ws_id/attendance/adjustments
Gửi yêu cầu xác nhận/điều chỉnh công mới.
- **Request**:
  ```json
  {
    "employee_id": "...",
    "time_entry_id": "optional-uuid-if-editing",
    "requested_punch_in_time": "2026-01-08T08:00:00Z",
    "requested_punch_out_time": "2026-01-08T17:00:00Z",
    "is_break_overtime": true,
    "reason": "Làm thông trưa"
  }
  ```
- **Response (201)**: `AttendanceAdjustmentRequest` object

#### GET /workspaces/:ws_id/attendance/adjustments
Lấy danh sách các yêu cầu điều chỉnh trong workspace.
- **Query Params**: `?status=pending|approved|rejected`
- **Response (200)**: `Array<AttendanceAdjustmentRequest>`

#### POST /workspaces/:ws_id/attendance/adjustments/:id/approve
Phê duyệt hoặc từ chối yêu cầu điều chỉnh công. 
- **Lưu ý**: Nếu `status` là `approved`, hệ thống sẽ tự động cập nhật hoặc tạo mới bản ghi trong bảng chấm công chính thức.
- **Request**: `{ "status": "approved|rejected" }`
- **Response (200 OK)**

#### GET /employees/:id/attendance/adjustments
Lấy danh sách các yêu cầu điều chỉnh của một nhân viên cụ thể.
- **Response (200)**: `Array<AttendanceAdjustmentRequest>`

---

### 9. Payroll & Shifts (New)

Quản lý lương, ca làm việc và phiếu lương hàng tháng.

#### GET /workspaces/:ws_id/shifts
Lấy danh sách các ca làm việc đã định nghĩa.
- **Response (200)**: `Array<Shift>`

#### POST /workspaces/:ws_id/shifts

Tạo mới một loại ca làm việc.

**Body:**
```json
{
  "name": "Ca sáng",
  "start_time": "08:00:00",
  "end_time": "17:00:00",
  "break_duration_minutes": 60,
  "break_is_overtime": false,
  "early_punch_in_grace_period_minutes": 15
}
```

#### POST /workspaces/:ws_id/shifts/assign

Gán ca làm việc cho nhân viên. Hỗ trợ ghi đè theo thời gian (Temporal Overriding).

**Body:**
```json
{
  "employee_id": "uuid",
  "shift_id": "uuid",
  "start_date": "2026-01-01",
  "end_date": "2026-01-10" // null nếu áp dụng vĩnh viễn
}
```

#### GET /workspaces/:ws_id/employees/:id/salary
Lấy lịch sử lương của nhân viên (từ mới đến cũ).
- **Response (200)**: `Array<SalaryHistory>`

#### POST /workspaces/:ws_id/employees/:id/salary
Cập nhật mức lương mới. Hệ thống sẽ lưu vết lịch sử thay vì ghi đè.
- **Request**:
  ```json
  {
    "base_salary": "15000000.00",
    "salary_type": "monthly",
    "currency": "VND",
    "effective_date": "2026-02-01"
  }
  ```
- **Response (201)**: `SalaryHistory` object

---

### 9. Departments & Rostering (New)

Quản lý cơ cấu tổ chức và lịch trình làm việc linh hoạt.

#### GET /workspaces/:ws_id/departments
Lấy danh sách các phòng ban.
- **Response (200)**: `Array<Department>`

#### POST /workspaces/:ws_id/departments
Tạo phòng ban mới.
- **Request**: `{ "name": "IT", "description": "...", "manager_id": "uuid" }`
- **Response (201)**: `Department` object

#### POST /workspaces/:ws_id/employees/:id/departments
Điều chuyển phòng ban cho nhân viên.
- **Request**: `{ "department_id": "...", "effective_date": "...", "notes": "..." }`
- **Response (201)**: `EmployeeDepartmentHistory` object

#### GET /workspaces/:ws_id/employees/:id/departments
Xem lịch sử điều chuyển phòng ban của một nhân viên.
- **Response (200)**: `Array<EmployeeDepartmentHistory>`

#### GET /workspaces/:ws_id/employees/:id/rosters
Lấy lịch biểu (ca làm việc thực tế) của nhân viên trong một khoảng thời gian.
- **Query Params**: `?start_date=2026-01-01&end_date=2026-01-07`
- **Response (200)**: `Array<ShiftRoster>`

#### POST /workspaces/:ws_id/employees/:id/rosters
Cập nhật hoặc gán ca làm việc cho một ngày cụ thể (Upsert).
- **Request**:
  ```json
  {
    "employee_id": "...",
    "date": "2026-01-01",
    "shift_id": "shift-uuid-or-null",
    "notes": "..."
  }
  ```
- **Response (200 OK)**

---

### WebSocket Giao thức (Agent <-> Server)

Agent gửi dữ liệu chấm công thô và trạng thái sức khỏe lên Server qua một kết nối duy nhất.

#### 1. Handshake (Agent -> Server)
Bản tin đầu tiên sau khi kết nối để xác thực định danh.
```json
{
  "type": "handshake",
  "agent_id": "AGENT_01",
  "token": "secret_token_abc_123",
  "timestamp": "2026-01-07T10:00:00Z"
}
```

#### 2. Heartbeat (Agent -> Server)
Agent gửi định kỳ (mỗi 30s) để duy trì trạng thái "Online" trong Database.
```json
{
  "type": "heartbeat",
  "agent_id": "AGENT_01",
  "timestamp": "ISO-8601"
}
```

#### 3. Status Update (Agent -> Server)
Cập nhật chi tiết trạng thái của các máy chấm công đang được giám sát.
```json
{
  "type": "status_update",
  "agent_id": "AGENT_01",
  "devices": [
    {
      "device_id": "DEV_01",
      "status": "online",
      "last_sync": "...",
      "error": null
    }
  ]
}
```

#### 4. Attendance Log (Agent -> Server)
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

